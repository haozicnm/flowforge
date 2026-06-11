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

use std::collections::HashMap;

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
            Json(serde_json::json!({"error": e.to_string()})),
        )
    })?;

    let token = crate::auth::create_jwt(&user).map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({"error": e.to_string()})),
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
            Json(serde_json::json!({"error": e.to_string()})),
        )
    })?;

    let token = crate::auth::create_jwt(&user).map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({"error": e.to_string()})),
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
        let mut store = state.webhook_store.lock().map_err(|_| {
            StatusCode::INTERNAL_SERVER_ERROR
        })?;
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
            "failed": exec_state.failed,
        }))),
        Err(e) => Ok(Json(serde_json::json!({
            "status": "failed",
            "error": e.to_string(),
            "failed": Vec::<String>::new(),
        }))),
    }
}

/// POST /api/workflows/:id/execute-step — single-step execution.
/// Body: { "completed": [...], "node_outputs": {...}, "failed": [...] }
/// Returns: { "executed": [...], "has_more": bool, "completed": [...], "node_outputs": {...}, "failed": [...] }
pub async fn execute_step(
    State(state): State<AppState>,
    Path(id): Path<String>,
    Json(body): Json<serde_json::Value>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let workflow = state.storage.load(&id).map_err(|_| StatusCode::NOT_FOUND)?;

    // Rebuild execution state from request body
    let mut exec_state = crate::engine::executor::ExecutionState::new();
    if let Some(completed) = body.get("completed").and_then(|v| v.as_array()) {
        exec_state.completed = completed.iter().filter_map(|v| v.as_str().map(String::from)).collect();
    }
    if let Some(failed) = body.get("failed").and_then(|v| v.as_array()) {
        exec_state.failed = failed.iter().filter_map(|v| v.as_str().map(String::from)).collect();
    }
    if let Some(outputs) = body.get("node_outputs").and_then(|v| v.as_object()) {
        for (node_id, ports) in outputs {
            if let Some(ports_obj) = ports.as_object() {
                let port_map: HashMap<String, serde_json::Value> = ports_obj
                    .iter()
                    .map(|(k, v)| (k.clone(), v.clone()))
                    .collect();
                exec_state.node_outputs.insert(node_id.clone(), port_map);
            }
        }
    }

    let executor = crate::engine::executor::Executor::new(state.node_registry.clone())
        .with_webbridge(state.webbridge.clone())
        .with_webhook_store(state.webhook_store.clone());

    match executor.execute_step(&workflow, exec_state).await {
        Ok((new_state, executed, has_more)) => Ok(Json(serde_json::json!({
            "status": if executed.is_empty() { "already_done" } else { "stepped" },
            "executed": executed,
            "has_more": has_more,
            "node_outputs": new_state.node_outputs,
            "completed": new_state.completed,
            "failed": new_state.failed,
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

// ============================================================
// Schedule API
// ============================================================

/// GET /api/schedules — list all schedules.
pub async fn list_schedules(State(state): State<AppState>) -> Json<serde_json::Value> {
    let schedules = state.scheduler.list_schedules();
    Json(serde_json::json!({
        "schedules": schedules,
        "total": schedules.len()
    }))
}

/// POST /api/schedules — create a new schedule.
pub async fn create_schedule(
    State(state): State<AppState>,
    Json(body): Json<serde_json::Value>,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<serde_json::Value>)> {
    let name = body.get("name").and_then(|v| v.as_str()).unwrap_or("Unnamed");
    let workflow_id = match body.get("workflow_id").and_then(|v| v.as_str()) {
        Some(id) => id.to_string(),
        None => return Err((StatusCode::BAD_REQUEST, Json(serde_json::json!({"error": "workflow_id required"})))),
    };
    let cron_expr = match body.get("cron_expr").and_then(|v| v.as_str()) {
        Some(expr) => expr.to_string(),
        None => return Err((StatusCode::BAD_REQUEST, Json(serde_json::json!({"error": "cron_expr required"})))),
    };

    match state.scheduler.create_schedule(name.to_string(), workflow_id, cron_expr) {
        Ok(schedule) => Ok(Json(serde_json::json!(schedule))),
        Err(e) => { let msg = e.to_string(); Err((StatusCode::BAD_REQUEST, Json(serde_json::json!({"error": msg})))) },
    }
}

/// GET /api/schedules/:id — get a schedule.
pub async fn get_schedule(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<serde_json::Value>)> {
    match state.scheduler.get_schedule(&id) {
        Ok(schedule) => Ok(Json(serde_json::json!(schedule))),
        Err(e) => Err((StatusCode::NOT_FOUND, Json(serde_json::json!({"error": e.to_string()})))),
    }
}

/// PUT /api/schedules/:id — update a schedule.
pub async fn update_schedule(
    State(state): State<AppState>,
    Path(id): Path<String>,
    Json(body): Json<serde_json::Value>,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<serde_json::Value>)> {
    match state.scheduler.update_schedule(&id, body) {
        Ok(schedule) => Ok(Json(serde_json::json!(schedule))),
        Err(e) => { let msg = e.to_string(); Err((StatusCode::BAD_REQUEST, Json(serde_json::json!({"error": msg})))) },
    }
}

/// DELETE /api/schedules/:id — delete a schedule.
pub async fn delete_schedule(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<serde_json::Value>)> {
    match state.scheduler.delete_schedule(&id) {
        Ok(()) => Ok(Json(serde_json::json!({"deleted": true}))),
        Err(e) => Err((StatusCode::NOT_FOUND, Json(serde_json::json!({"error": e.to_string()})))),
    }
}

/// POST /api/schedules/:id/trigger — trigger a schedule immediately.
pub async fn trigger_schedule(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<serde_json::Value>)> {
    match state.scheduler.trigger_schedule(&id).await {
        Ok(result) => Ok(Json(result)),
        Err(e) => Err((StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({"error": e.to_string()})))),
    }
}

fn event_to_json(event: crate::engine::executor::ExecutionEvent) -> serde_json::Value {
    use crate::engine::executor::ExecutionEvent;
    match event {
        ExecutionEvent::NodeStarted { _node_id } => serde_json::json!({
            "type": "node_started", "node_id": _node_id
        }),
        ExecutionEvent::NodeCompleted { _node_id, _outputs, _duration_ms } => serde_json::json!({
            "type": "node_completed", "node_id": _node_id, "outputs": _outputs, "duration_ms": _duration_ms
        }),
        ExecutionEvent::NodeFailed { _node_id, _error, _duration_ms } => serde_json::json!({
            "type": "node_failed", "node_id": _node_id, "error": _error, "duration_ms": _duration_ms
        }),
        ExecutionEvent::WorkflowCompleted => serde_json::json!({
            "type": "workflow_completed"
        }),
        ExecutionEvent::_WorkflowFailed { _error } => serde_json::json!({
            "type": "workflow_failed", "error": _error
        }),
    }
}

// ============================================================
// API Gateway
// ============================================================

/// GET /api/gateway — list published APIs.
pub async fn gateway_list(State(state): State<AppState>) -> Json<serde_json::Value> {
    let apis = state.api_gateway.list();
    Json(serde_json::json!({
        "apis": apis,
        "total": apis.len()
    }))
}

/// POST /api/gateway/publish — publish a workflow as API.
pub async fn gateway_publish(
    State(state): State<AppState>,
    Json(body): Json<serde_json::Value>,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<serde_json::Value>)> {
    let workflow_id = match body.get("workflow_id").and_then(|v| v.as_str()) {
        Some(id) => id.to_string(),
        None => return Err((StatusCode::BAD_REQUEST, Json(serde_json::json!({"error": "workflow_id required"})))),
    };
    let path = match body.get("path").and_then(|v| v.as_str()) {
        Some(p) => p.to_string(),
        None => return Err((StatusCode::BAD_REQUEST, Json(serde_json::json!({"error": "path required"})))),
    };
    let api_key = body.get("api_key").and_then(|v| v.as_str()).map(|s| s.to_string());
    let rate_limit = body.get("rate_limit").and_then(|v| v.as_u64()).unwrap_or(0);

    match state.api_gateway.publish(workflow_id, path, api_key, rate_limit) {
        Ok(api) => Ok(Json(serde_json::json!(api))),
        Err(e) => {
            let err = serde_json::Value::String(e.to_string());
            let body = serde_json::json!({  "error": err });
            Err((StatusCode::BAD_REQUEST, Json(body)))
        },
    }
}

/// DELETE /api/gateway/unpublish/:path — unpublish an API.
pub async fn gateway_unpublish(
    State(state): State<AppState>,
    Path(path): Path<String>,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<serde_json::Value>)> {
    match state.api_gateway.unpublish(&path) {
        Ok(()) => Ok(Json(serde_json::json!({"unpublished": true}))),
        Err(e) => {
            let err = serde_json::Value::String(e);
            let body = serde_json::json!({  "error": err });
            Err((StatusCode::NOT_FOUND, Json(body)))
        },
    }
}

/// ANY /api/run/:path — dynamic API gateway endpoint.
pub async fn gateway_run(
    State(state): State<AppState>,
    Path(path): Path<String>,
    method: axum::http::Method,
    headers: axum::http::HeaderMap,
    uri: axum::http::Uri,
    body: String,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<serde_json::Value>)> {
    // Parse body as JSON (fallback to empty object)
    let body_json: serde_json::Value = serde_json::from_str(&body)
        .unwrap_or_else(|_| serde_json::json!({}));

    // Extract query params
    let query_params: HashMap<String, String> = uri.query()
        .map(|q| url::form_urlencoded::parse(q.as_bytes())
            .map(|(k, v)| (k.into_owned(), v.into_owned()))
            .collect())
        .unwrap_or_default();

    // Extract API key from header
    let api_key = headers.get("X-API-Key")
        .and_then(|v| v.to_str().ok());

    let result = state.api_gateway.handle_request(
        &path,
        method.as_str(),
        api_key,
        body_json,
        query_params,
    ).await;

    match result {
        Ok(val) => Ok(Json(val)),
        Err((code, msg)) => {
            let status = StatusCode::from_u16(code).unwrap_or(StatusCode::INTERNAL_SERVER_ERROR);
            Err((status, Json(serde_json::json!({"error": msg}))))
        }
    }
}
