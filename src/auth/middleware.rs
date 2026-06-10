//! Axum middleware for JWT authentication.
//!
//! Extracts Bearer token from Authorization header, verifies JWT,
//! and injects `UserId` into request extensions.
//!
//! Usage:
//!   Router::new().route_layer(from_fn_with_state(state, auth_middleware))

use axum::{
    extract::State,
    http::{Request, StatusCode},
    middleware::Next,
    response::Response,
};

use crate::auth::{verify_jwt, AuthState};

/// Wrapper type for user identity in request extensions.
#[derive(Debug, Clone)]
pub struct UserId(pub String);

/// Middleware that requires a valid JWT. Injects `UserId` extension.
pub async fn auth_middleware(
    State(_auth_db): State<AuthState>,
    mut request: Request<axum::body::Body>,
    next: Next,
) -> Result<Response, StatusCode> {
    let header = request
        .headers()
        .get("Authorization")
        .and_then(|v| v.to_str().ok())
        .and_then(|v| v.strip_prefix("Bearer "));

    match header {
        Some(token) => {
            match verify_jwt(token) {
                Ok(claims) => {
                    request.extensions_mut().insert(UserId(claims.sub));
                    Ok(next.run(request).await)
                }
                Err(_) => Err(StatusCode::UNAUTHORIZED),
            }
        }
        None => Err(StatusCode::UNAUTHORIZED),
    }
}

/// Optional middleware — injects `UserId` if token present, otherwise `None`.
pub async fn optional_auth_middleware(
    State(_auth_db): State<AuthState>,
    mut request: Request<axum::body::Body>,
    next: Next,
) -> Result<Response, StatusCode> {
    if let Some(header) = request
        .headers()
        .get("Authorization")
        .and_then(|v| v.to_str().ok())
        .and_then(|v| v.strip_prefix("Bearer "))
    {
        if let Ok(claims) = verify_jwt(header) {
            request.extensions_mut().insert(UserId(claims.sub));
        }
    }
    Ok(next.run(request).await)
}
