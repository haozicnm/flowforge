//! HTTP API handlers.
//!
//! Endpoints:
//! - GET /api/health — health check
//! - GET /api/nodes/types — list all node types
//! - GET /api/workflows — list all workflows
//! - POST /api/workflows — create a new workflow
//! - GET /api/workflows/:id — get a workflow
//! - PUT /api/workflows/:id — update a workflow
//! - DELETE /api/workflows/:id — delete a workflow
//! - POST /api/workflows/:id/execute — execute a workflow

use axum::{
    extract::{Path, State},
    http::StatusCode,
    Json,
};
use serde::{Deserialize, Serialize};

use crate::engine::workflow::{Edge, Node, Workflow};
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

/// GET /api/workflows — list all workflows.
pub async fn list_workflows(
    State(state): State<AppState>,
) -> Result<Json<Vec<Workflow>>, StatusCode> {
    state
        .storage
        .list()
        .map(Json)
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)
}

/// Request body for creating a workflow.
#[derive(Deserialize)]
pub struct CreateWorkflowRequest {
    pub name: String,
    pub description: Option<String>,
}

/// POST /api/workflows — create a new workflow.
pub async fn create_workflow(
    State(state): State<AppState>,
    Json(req): Json<CreateWorkflowRequest>,
) -> Result<(StatusCode, Json<Workflow>), StatusCode> {
    let workflow = Workflow::new(req.name, req.description);
    state
        .storage
        .save(&workflow)
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    Ok((StatusCode::CREATED, Json(workflow)))
}

/// GET /api/workflows/:id — get a workflow.
pub async fn get_workflow(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<Json<Workflow>, StatusCode> {
    state
        .storage
        .load(&id)
        .map(Json)
        .map_err(|_| StatusCode::NOT_FOUND)
}

/// Request body for updating a workflow.
#[derive(Deserialize)]
pub struct UpdateWorkflowRequest {
    pub name: Option<String>,
    pub description: Option<String>,
    pub nodes: Option<Vec<Node>>,
    pub edges: Option<Vec<Edge>>,
}

/// PUT /api/workflows/:id — update a workflow.
pub async fn update_workflow(
    State(state): State<AppState>,
    Path(id): Path<String>,
    Json(req): Json<UpdateWorkflowRequest>,
) -> Result<Json<Workflow>, StatusCode> {
    let mut workflow = state.storage.load(&id).map_err(|_| StatusCode::NOT_FOUND)?;

    if let Some(name) = req.name {
        workflow.name = name;
    }
    if let Some(desc) = req.description {
        workflow.description = desc;
    }
    if let Some(nodes) = req.nodes {
        workflow.nodes = nodes;
    }
    if let Some(edges) = req.edges {
        workflow.edges = edges;
    }

    state
        .storage
        .save(&workflow)
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(Json(workflow))
}

/// DELETE /api/workflows/:id — delete a workflow.
pub async fn delete_workflow(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<StatusCode, StatusCode> {
    state
        .storage
        .delete(&id)
        .map(|_| StatusCode::NO_CONTENT)
        .map_err(|_| StatusCode::NOT_FOUND)
}

/// POST /api/workflows/:id/execute — execute a workflow.
pub async fn execute_workflow(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let workflow = state.storage.load(&id).map_err(|_| StatusCode::NOT_FOUND)?;

    let executor = crate::engine::executor::Executor::new(state.node_registry.clone());
    let result = executor.execute(&workflow, None).await;

    match result {
        Ok(exec_state) => Ok(Json(serde_json::json!({
            "status": "completed",
            "node_outputs": exec_state.node_outputs,
            "completed": exec_state.completed,
        }))),
        Err(e) => Ok(Json(serde_json::json!({
            "status": "failed",
            "error": e.to_string(),
        }))),
    }
}
