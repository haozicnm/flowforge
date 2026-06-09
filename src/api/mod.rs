//! HTTP API handlers.

use axum::{extract::State, Json};
use serde::{Deserialize, Serialize};

use crate::state::AppState;

/// Health check response.
#[derive(Serialize)]
pub struct HealthResponse {
    pub version: &'static str,
    pub status: &'static str,
}

/// GET /api/health
pub async fn health() -> Json<HealthResponse> {
    Json(HealthResponse {
        version: env!("CARGO_PKG_VERSION"),
        status: "ok",
    })
}

/// GET /api/nodes/types — list all registered node types.
pub async fn node_types(
    State(state): State<AppState>,
) -> Json<Vec<crate::nodes::traits::NodeTypeDef>> {
    Json(state.node_registry.all_type_defs())
}
