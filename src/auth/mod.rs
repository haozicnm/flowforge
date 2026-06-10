//! Authentication module — SQLite-backed user system + JWT tokens.

pub mod middleware;

use rusqlite::{params, Connection};
use serde::{Deserialize, Serialize};
use std::sync::{Arc, Mutex};

/// A registered user.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct User {
    pub id: String,
    pub username: String,
    #[serde(skip_serializing)]
    pub password_hash: String,
    pub created_at: String,
}

/// Public-facing user info (never exposes hash).
#[derive(Debug, Clone, Serialize)]
pub struct UserInfo {
    pub id: String,
    pub username: String,
    pub created_at: String,
}

impl From<&User> for UserInfo {
    fn from(u: &User) -> Self {
        Self {
            id: u.id.clone(),
            username: u.username.clone(),
            created_at: u.created_at.clone(),
        }
    }
}

/// JWT claims.
#[derive(Debug, Serialize, Deserialize)]
pub struct Claims {
    pub sub: String,       // user id
    pub username: String,
    pub exp: usize,        // expiry
    pub iat: usize,        // issued at
}

// ── JWT secret ──────────────────────────────────────────────────

/// Default JWT secret. Override with env FLOWFORGE_JWT_SECRET.
fn jwt_secret() -> String {
    std::env::var("FLOWFORGE_JWT_SECRET")
        .unwrap_or_else(|_| "flowforge-dev-secret-change-me".into())
}

// ── Database ─────────────────────────────────────────────────────

pub struct AuthDb {
    conn: Mutex<Connection>,
}

impl AuthDb {
    /// Open or create the SQLite database.
    pub fn open(path: &str) -> Result<Self, String> {
        let conn = Connection::open(path).map_err(|e| format!("auth db open: {}", e))?;

        conn.execute_batch(
            "CREATE TABLE IF NOT EXISTS users (
                id TEXT PRIMARY KEY,
                username TEXT UNIQUE NOT NULL,
                password_hash TEXT NOT NULL,
                created_at TEXT NOT NULL
            )",
        )
        .map_err(|e| format!("auth db init: {}", e))?;

        Ok(Self { conn: Mutex::new(conn) })
    }

    /// Create a new user.
    pub fn create_user(&self, username: &str, password: &str) -> Result<User, String> {
        let conn = self.conn.lock().map_err(|e| format!("lock: {}", e))?;

        // Check duplicate
        let exists: bool = conn
            .query_row(
                "SELECT COUNT(*) > 0 FROM users WHERE username = ?1",
                params![username],
                |row| row.get(0),
            )
            .unwrap_or(false);

        if exists {
            return Err(format!("username '{}' already exists", username));
        }

        let id = uuid::Uuid::new_v4().to_string();
        let now = chrono::Utc::now().to_rfc3339();

        let hash = bcrypt::hash(password, bcrypt::DEFAULT_COST)
            .map_err(|e| format!("hash: {}", e))?;

        conn.execute(
            "INSERT INTO users (id, username, password_hash, created_at) VALUES (?1, ?2, ?3, ?4)",
            params![id, username, hash, now],
        )
        .map_err(|e| format!("insert: {}", e))?;

        Ok(User {
            id,
            username: username.into(),
            password_hash: hash,
            created_at: now,
        })
    }

    /// Find user by username, verifying password.
    pub fn verify_password(&self, username: &str, password: &str) -> Result<User, String> {
        let conn = self.conn.lock().map_err(|e| format!("lock: {}", e))?;

        let (id, hash, created_at): (String, String, String) = conn
            .query_row(
                "SELECT id, password_hash, created_at FROM users WHERE username = ?1",
                params![username],
                |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?)),
            )
            .map_err(|_| "invalid username or password".to_string())?;

        let valid = bcrypt::verify(password, &hash).unwrap_or(false);

        if !valid {
            return Err("invalid username or password".into());
        }

        Ok(User {
            id,
            username: username.into(),
            password_hash: hash,
            created_at,
        })
    }

    /// Find user by id.
    pub fn find_by_id(&self, user_id: &str) -> Result<User, String> {
        let conn = self.conn.lock().map_err(|e| format!("lock: {}", e))?;

        let (username, hash, created_at): (String, String, String) = conn
            .query_row(
                "SELECT username, password_hash, created_at FROM users WHERE id = ?1",
                params![user_id],
                |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?)),
            )
            .map_err(|_| "user not found".to_string())?;

        Ok(User {
            id: user_id.into(),
            username,
            password_hash: hash,
            created_at,
        })
    }
}

// ── JWT helpers ──────────────────────────────────────────────────

pub fn create_jwt(user: &User) -> Result<String, String> {
    let now = chrono::Utc::now();
    let claims = Claims {
        sub: user.id.clone(),
        username: user.username.clone(),
        iat: now.timestamp() as usize,
        exp: (now + chrono::Duration::hours(72)).timestamp() as usize,
    };

    jsonwebtoken::encode(
        &jsonwebtoken::Header::default(),
        &claims,
        &jsonwebtoken::EncodingKey::from_secret(jwt_secret().as_bytes()),
    )
    .map_err(|e| format!("jwt encode: {}", e))
}

pub fn verify_jwt(token: &str) -> Result<Claims, String> {
    let data = jsonwebtoken::decode::<Claims>(
        token,
        &jsonwebtoken::DecodingKey::from_secret(jwt_secret().as_bytes()),
        &jsonwebtoken::Validation::default(),
    )
    .map_err(|e| format!("jwt invalid: {}", e))?;
    Ok(data.claims)
}

// ── Type alias ──────────────────────────────────────────────────

/// Type alias for shared auth database.
pub type AuthState = Arc<AuthDb>;
