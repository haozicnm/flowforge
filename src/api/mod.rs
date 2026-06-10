//! HTTP API handlers.
//!
//! Endpoints:
//! - GET /api/health — health check
//! - POST /api/auth/register — register a new user
//! - POST /api/auth/login — login, returns JWT
//! - GET /api/nodes/types — list all node types
//! - GET /api/workflows — list all workflows
//! - POST /api/workflows — create a new workflow
//! - GET /api/workflows/:id — get a workflow
//! - PUT /api/workflows/:id — update a workflow
//! - DELETE /api/workflows/:id — delete a workflow
//! - POST /api/workflows/:id/execute — execute a workflow
//! - POST /api/webhook/:workflow_id/:node_id — webhook trigger

use axum::{
    extract::{Path, State},
    http::StatusCode,
    Extension, Json,
};
use serde::{Deserialize, Serialize};

use crate::auth::middleware::UserId;
use crate::engine::workflow::{Edge, Node, Workflow};
use crate::state::AppState;

/// GET /api/browser/status — check if Chrome extension is connected.
pub async fn browser_status(
    State(state): State<AppState>,
) -> Json<serde_json::Value> {
    let connected = state.webbridge.is_connected().await;
    Json(serde_json::json!({
        "connected": connected,
        "message": if connected {
            "Chrome extension connected"
        } else {
            "Chrome extension not connected. Install the WebBridge extension and navigate to a page."
        }
    }))
}

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

// ── Auth endpoints ──────────────────────────────────────────────

/// POST /api/auth/register
#[derive(Deserialize)]
pub struct RegisterRequest {
    pub username: String,
    pub password: String,
}

#[derive(Serialize)]
pub struct AuthResponse {
    pub token: String,
    pub user: crate::auth::UserInfo,
}

pub async fn register(
    State(state): State<AppState>,
    Json(req): Json<RegisterRequest>,
) -> Result<(StatusCode, Json<AuthResponse>), (StatusCode, Json<serde_json::Value>)> {
    if req.username.len() < 3 || req.password.len() < 6 {
        return Err((
            StatusCode::BAD_REQUEST,
            Json(serde_json::json!({"error": "username must be ≥3 chars, password ≥6 chars"})),
        ));
    }

    let user = state.auth_db.create_user(&req.username, &req.password).map_err(|e| {
        (
            StatusCode::CONFLICT,
            Json(serde_json::json!({"error": e})),
        )
    })?;

    let token = crate::auth::create_jwt(&user).map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({"error": e})),
        )
    })?;

    Ok((
        StatusCode::CREATED,
        Json(AuthResponse {
            token,
            user: crate::auth::UserInfo::from(&user),
        }),
    ))
}

/// POST /api/auth/login
#[derive(Deserialize)]
pub struct LoginRequest {
    pub username: String,
    pub password: String,
}

pub async fn login(
    State(state): State<AppState>,
    Json(req): Json<LoginRequest>,
) -> Result<Json<AuthResponse>, (StatusCode, Json<serde_json::Value>)> {
    let user = state.auth_db.verify_password(&req.username, &req.password).map_err(|e| {
        (
            StatusCode::UNAUTHORIZED,
            Json(serde_json::json!({"error": e})),
        )
    })?;

    let token = crate::auth::create_jwt(&user).map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({"error": e})),
        )
    })?;

    Ok(Json(AuthResponse {
        token,
        user: crate::auth::UserInfo::from(&user),
    }))
}

/// GET /api/auth/me — return current user info (requires auth).
pub async fn whoami(
    State(state): State<AppState>,
    Extension(user_id): Extension<UserId>,
) -> Result<Json<crate::auth::UserInfo>, StatusCode> {
    let user = state.auth_db.find_by_id(&user_id.0).map_err(|_| StatusCode::NOT_FOUND)?;
    Ok(Json(crate::auth::UserInfo::from(&user)))
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
    Extension(user_id): Extension<Option<UserId>>,
    Json(req): Json<CreateWorkflowRequest>,
) -> Result<(StatusCode, Json<Workflow>), StatusCode> {
    let owner = user_id.map(|u| u.0);
    let workflow = Workflow::with_owner(req.name, req.description, owner);
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

/// GET /api/plugins/list — list all registered node types (built-in + plugins).
pub async fn list_plugins(
    State(state): State<AppState>,
) -> Json<Vec<crate::nodes::traits::NodeTypeDef>> {
    Json(state.node_registry.all_type_defs())
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

/// GET /api/workflows/:id/export — export a workflow as JSON.
pub async fn export_workflow(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<String, StatusCode> {
    state.storage.export_json(&id).map_err(|_| StatusCode::NOT_FOUND)
}

/// GET /api/workflows/export-all — export all workflows as a JSON array.
pub async fn export_all_workflows(
    State(state): State<AppState>,
) -> Result<String, StatusCode> {
    state.storage.export_all_json().map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)
}

/// POST /api/workflows/import — import a workflow from JSON.
pub async fn import_workflow(
    State(state): State<AppState>,
    Json(body): Json<serde_json::Value>,
) -> Result<(StatusCode, Json<Workflow>), StatusCode> {
    let json_str = serde_json::to_string(&body).map_err(|_| StatusCode::BAD_REQUEST)?;
    let wf = state.storage.import_json(&json_str).map_err(|_| StatusCode::BAD_REQUEST)?;
    Ok((StatusCode::CREATED, Json(wf)))
}

/// POST /api/webhook/:workflow_id/:node_id — receive a webhook trigger.
pub async fn webhook_trigger(
    State(state): State<AppState>,
    Path((workflow_id, node_id)): Path<(String, String)>,
    headers: axum::http::HeaderMap,
    body: axum::body::Bytes,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let key = format!("{}:{}", workflow_id, node_id);

    // Parse body as JSON (fall back to raw text)
    let body_value: serde_json::Value =
        serde_json::from_slice(&body).unwrap_or_else(|_| {
            serde_json::Value::String(String::from_utf8_lossy(&body).into_owned())
        });

    // Collect headers
    let mut header_map = std::collections::HashMap::new();
    for (name, value) in headers.iter() {
        if let Ok(v) = value.to_str() {
            header_map.insert(name.as_str().to_string(), v.to_string());
        }
    }

    let payload = serde_json::json!({
        "body": body_value,
        "headers": header_map,
        "method": "POST",
        "received_at": chrono::Utc::now().to_rfc3339(),
    });

    // Store the payload
    {
        let mut store = state.webhook_store.lock().unwrap();
        store.entry(key.clone()).or_default().push(payload);
    }

    tracing::info!("Webhook received for {}/{}", workflow_id, node_id);

    // Auto-execute the workflow
    let workflow = state.storage.load(&workflow_id).map_err(|_| StatusCode::NOT_FOUND)?;

    let executor = crate::engine::executor::Executor::new(state.node_registry.clone())
        .with_webbridge(state.webbridge.clone())
        .with_webhook_store(state.webhook_store.clone());
    let result = executor.execute(&workflow, None).await;

    match result {
        Ok(exec_state) => Ok(Json(serde_json::json!({
            "status": "triggered",
            "node_id": node_id,
            "completed": exec_state.completed,
        }))),
        Err(e) => Ok(Json(serde_json::json!({
            "status": "triggered",
            "node_id": node_id,
            "error": e.to_string(),
        }))),
    }
}

/// POST /api/workflows/:id/execute — execute a workflow.
pub async fn execute_workflow(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let workflow = state.storage.load(&id).map_err(|_| StatusCode::NOT_FOUND)?;

    let executor = crate::engine::executor::Executor::new(state.node_registry.clone())
        .with_webbridge(state.webbridge.clone())
        .with_webhook_store(state.webhook_store.clone());
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

/// WebSocket handler for real-time execution events.
/// GET /ws/execute/:id
pub async fn ws_execute(
    ws: axum::extract::WebSocketUpgrade,
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> impl axum::response::IntoResponse {
    ws.on_upgrade(move |mut socket| async move {
        use axum::extract::ws::Message;

        let workflow = match state.storage.load(&id) {
            Ok(wf) => wf,
            Err(e) => {
                let _ = socket.send(Message::Text(format!("{{\"type\":\"error\",\"msg\":\"{}\"}}", e))).await;
                return;
            }
        };

        let (tx, mut rx) = tokio::sync::mpsc::channel::<crate::engine::executor::ExecutionEvent>(64);

        let executor = crate::engine::executor::Executor::new(state.node_registry.clone())
            .with_webbridge(state.webbridge.clone())
            .with_webhook_store(state.webhook_store.clone());

        let mut handle = Some(tokio::spawn(async move {
            executor.execute(&workflow, Some(tx)).await
        }));

        loop {
            tokio::select! {
                msg = rx.recv() => {
                    let Some(event) = msg else { break };
                    let json = event_to_json(event);
                    if socket.send(Message::Text(serde_json::to_string(&json).unwrap_or_default())).await.is_err() {
                        break;
                    }
                }
                result = async {
                    handle.take().unwrap().await
                }, if handle.is_some() => {
                    let json = match result {
                        Ok(Ok(st)) => serde_json::json!({"type":"done","completed":st.completed,"failed":st.failed}),
                        Ok(Err(e)) => serde_json::json!({"type":"done","error":e.to_string()}),
                        Err(_) => serde_json::json!({"type":"done","error":"executor panicked"}),
                    };
                    let _ = socket.send(Message::Text(serde_json::to_string(&json).unwrap_or_default())).await;
                    break;
                }
            }
        }
    })
}

fn event_to_json(event: crate::engine::executor::ExecutionEvent) -> serde_json::Value {
    use crate::engine::executor::ExecutionEvent;
    match event {
        ExecutionEvent::NodeStarted { _node_id } => serde_json::json!({
            "type": "node_started", "node_id": _node_id
        }),
        ExecutionEvent::NodeCompleted { _node_id, _outputs } => serde_json::json!({
            "type": "node_completed", "node_id": _node_id, "outputs": _outputs
        }),
        ExecutionEvent::NodeFailed { _node_id, _error } => serde_json::json!({
            "type": "node_failed", "node_id": _node_id, "error": _error
        }),
        ExecutionEvent::WorkflowCompleted => serde_json::json!({
            "type": "workflow_completed"
        }),
        ExecutionEvent::_WorkflowFailed { _error } => serde_json::json!({
            "type": "workflow_failed", "error": _error
        }),
    }
}
