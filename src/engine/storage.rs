//! Workflow persistence — SQLite-backed storage with JSON import/export.
//!
//! Stores workflows in `flowforge.db`, auto-migrates old JSON files on startup.

use std::path::PathBuf;
use std::sync::Mutex;

use crate::engine::workflow::Workflow;
use crate::error::{FlowError, FlowResult};

/// SQLite-backed workflow storage.
pub struct WorkflowStorage {
    db: Mutex<rusqlite::Connection>,
    data_dir: PathBuf,
}

impl WorkflowStorage {
    pub fn new(data_dir: impl Into<PathBuf>) -> Self {
        let dir = data_dir.into();
        let _ = std::fs::create_dir_all(&dir);
        let db_path = dir.join("flowforge.db");
        let conn = rusqlite::Connection::open(&db_path)
            .expect("Failed to open SQLite database");

        Self { db: Mutex::new(conn), data_dir: dir }
    }

    fn conn(&self) -> FlowResult<std::sync::MutexGuard<rusqlite::Connection>> {
        self.db.lock().map_err(|e| FlowError::StorageError {
            detail: format!("Lock: {}", e),
        })
    }

    /// Initialize the storage.
    pub fn init(&self) -> FlowResult<()> {
        std::fs::create_dir_all(&self.data_dir).map_err(|e| FlowError::StorageError {
            detail: format!("Failed to create data dir: {}", e),
        })?;

        let db = self.conn()?;
        db.execute_batch(
            "CREATE TABLE IF NOT EXISTS workflows (
                id TEXT PRIMARY KEY,
                name TEXT NOT NULL,
                description TEXT NOT NULL DEFAULT '',
                owner_id TEXT,
                json_data TEXT NOT NULL,
                created_at TEXT NOT NULL
            );
            CREATE INDEX IF NOT EXISTS idx_owner ON workflows(owner_id);
            CREATE INDEX IF NOT EXISTS idx_created ON workflows(created_at DESC);"
        ).map_err(|e| FlowError::StorageError {
            detail: format!("Failed to init SQLite: {}", e),
        })?;

        Ok(())
    }

    /// List all saved workflows.
    pub fn list(&self) -> FlowResult<Vec<Workflow>> {
        let db = self.conn()?;
        let mut stmt = db.prepare(
            "SELECT json_data FROM workflows ORDER BY created_at DESC"
        ).map_err(|e| FlowError::StorageError { detail: format!("SQLite: {}", e) })?;

        let results = stmt.query_map([], |row| row.get::<_, String>(0))
            .map_err(|e| FlowError::StorageError { detail: format!("SQLite: {}", e) })?;

        let mut workflows = Vec::new();
        for row in results {
            let json_str = row.map_err(|e| FlowError::StorageError { detail: format!("Row: {}", e) })?;
            if let Ok(wf) = serde_json::from_str(&json_str) {
                workflows.push(wf);
            }
        }
        Ok(workflows)
    }

    /// Load a workflow by ID.
    pub fn load(&self, id: &str) -> FlowResult<Workflow> {
        let db = self.conn()?;
        let json_str: String = db.query_row(
            "SELECT json_data FROM workflows WHERE id = ?1",
            rusqlite::params![id],
            |row| row.get(0),
        ).map_err(|e| {
            if matches!(e, rusqlite::Error::QueryReturnedNoRows) {
                FlowError::WorkflowNotFound(id.to_string())
            } else {
                FlowError::StorageError { detail: format!("SQLite: {}", e) }
            }
        })?;
        serde_json::from_str(&json_str).map_err(|e| FlowError::StorageError {
            detail: format!("Parse: {}", e),
        })
    }

    /// Save a workflow (create or update).
    pub fn save(&self, workflow: &Workflow) -> FlowResult<()> {
        self.init()?;
        let json_str = serde_json::to_string(workflow).map_err(|e| FlowError::StorageError {
            detail: format!("Serialize: {}", e),
        })?;
        let db = self.conn()?;
        db.execute(
            "INSERT INTO workflows (id, name, description, owner_id, json_data, created_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6)
             ON CONFLICT(id) DO UPDATE SET
               name = excluded.name, description = excluded.description,
               owner_id = excluded.owner_id, json_data = excluded.json_data,
               created_at = excluded.created_at",
            rusqlite::params![
                workflow.id, workflow.name, workflow.description,
                workflow.owner_id.as_deref().unwrap_or(""),
                json_str, workflow.created_at.to_rfc3339(),
            ],
        ).map_err(|e| FlowError::StorageError { detail: format!("Save: {}", e) })?;
        Ok(())
    }

    /// Delete a workflow by ID.
    pub fn delete(&self, id: &str) -> FlowResult<()> {
        let db = self.conn()?;
        let rows = db.execute("DELETE FROM workflows WHERE id = ?1", rusqlite::params![id])
            .map_err(|e| FlowError::StorageError { detail: format!("Delete: {}", e) })?;
        if rows == 0 {
            return Err(FlowError::WorkflowNotFound(id.to_string()));
        }
        Ok(())
    }

    /// Export a single workflow as JSON.
    pub fn export_json(&self, id: &str) -> FlowResult<String> {
        let wf = self.load(id)?;
        serde_json::to_string_pretty(&wf).map_err(|e| FlowError::StorageError {
            detail: format!("Export: {}", e),
        })
    }

    /// Export all workflows as a JSON array.
    pub fn export_all_json(&self) -> FlowResult<String> {
        let workflows = self.list()?;
        serde_json::to_string_pretty(&workflows).map_err(|e| FlowError::StorageError {
            detail: format!("Export all: {}", e),
        })
    }

    /// Import a workflow from JSON.
    pub fn import_json(&self, json_str: &str) -> FlowResult<Workflow> {
        let wf: Workflow = serde_json::from_str(json_str).map_err(|e| FlowError::StorageError {
            detail: format!("Import: {}", e),
        })?;
        self.save(&wf)?;
        Ok(wf)
    }

    /// Migrate old JSON files from data_dir/workflows/*.json into SQLite.
    pub fn migrate_from_files(&self) -> Result<usize, String> {
        let dir = self.data_dir.join("workflows");
        if !dir.exists() { return Ok(0); }

        let mut migrated = 0;
        let entries = std::fs::read_dir(&dir).map_err(|e| format!("read dir: {}", e))?;
        for entry in entries {
            let entry = entry.map_err(|e| format!("entry: {}", e))?;
            let path = entry.path();
            if path.extension().and_then(|e| e.to_str()) != Some("json") { continue; }
            match std::fs::read_to_string(&path) {
                Ok(content) => {
                    if let Ok(wf) = serde_json::from_str::<Workflow>(&content) {
                        if self.save(&wf).is_ok() { migrated += 1; }
                    }
                }
                Err(_) => {}
            }
        }
        Ok(migrated)
    }
}
