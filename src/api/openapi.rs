//! OpenAPI documentation generator.
//!
//! Auto-generates OpenAPI 3.0 spec from the registered routes and node types.
//! Serves Swagger UI at /api/docs and raw spec at /api/openapi.json.

use crate::state::AppState;
use axum::extract::State;
use axum::response::Html;

/// GET /api/openapi.json — returns the auto-generated OpenAPI 3.0 spec.
pub async fn openapi_spec(State(state): State<AppState>) -> axum::Json<serde_json::Value> {
    let spec = generate_spec(&state);
    axum::Json(spec)
}

/// GET /api/docs — serves Swagger UI.
pub async fn swagger_ui() -> Html<String> {
    Html(SWAGGER_HTML.to_string())
}

fn generate_spec(state: &AppState) -> serde_json::Value {
    let node_types: Vec<serde_json::Value> = state.node_registry
        .all_type_defs()
        .iter()
        .map(|nt| {
            serde_json::json!({
                "type": nt.type_name,
                "displayName": nt.display_name,
                "description": nt.description,
                "category": nt.category,
                "version": nt.version,
            })
        })
        .collect();

    let published_count = state.api_gateway.list().len();

    let info = serde_json::json!({
        "title": "FlowForge API",
        "description": "Visual Workflow Automation Engine",
        "version": env!("CARGO_PKG_VERSION"),
    });

    let tags = serde_json::json!([
        {"name": "Health", "description": "Health check"},
        {"name": "Auth", "description": "Authentication"},
        {"name": "Workflows", "description": "Workflow CRUD + execution"},
        {"name": "Schedules", "description": "Cron scheduling"},
        {"name": "Gateway", "description": "API Gateway"},
        {"name": "Nodes", "description": "Node type registry"},
    ]);

    let paths = serde_json::json!({
        "/api/health": {
            "get": {"tags": ["Health"], "summary": "Health check", "responses": {"200": {"description": "OK"}}}
        },
        "/api/auth/register": {
            "post": {"tags": ["Auth"], "summary": "Register", "responses": {"200": {"description": "OK"}}}
        },
        "/api/auth/login": {
            "post": {"tags": ["Auth"], "summary": "Login", "responses": {"200": {"description": "JWT"}}}
        },
        "/api/workflows": {
            "get": {"tags": ["Workflows"], "summary": "List", "responses": {"200": {"description": "List"}}},
            "post": {"tags": ["Workflows"], "summary": "Create", "responses": {"200": {"description": "Created"}}}
        },
        "/api/workflows/{id}": {
            "get": {"tags": ["Workflows"], "summary": "Get", "responses": {"200": {"description": "OK"}}},
            "put": {"tags": ["Workflows"], "summary": "Update", "responses": {"200": {"description": "OK"}}},
            "delete": {"tags": ["Workflows"], "summary": "Delete", "responses": {"200": {"description": "OK"}}}
        },
        "/api/workflows/{id}/execute": {
            "post": {"tags": ["Workflows"], "summary": "Execute", "responses": {"200": {"description": "Result"}}}
        },
        "/api/workflows/{id}/execute-step": {
            "post": {"tags": ["Workflows"], "summary": "Step execute", "responses": {"200": {"description": "Result"}}}
        },
        "/api/schedules": {
            "get": {"tags": ["Schedules"], "summary": "List", "responses": {"200": {"description": "List"}}},
            "post": {"tags": ["Schedules"], "summary": "Create", "responses": {"200": {"description": "Created"}}}
        },
        "/api/schedules/{id}": {
            "get": {"tags": ["Schedules"], "summary": "Get", "responses": {"200": {"description": "OK"}}},
            "put": {"tags": ["Schedules"], "summary": "Update", "responses": {"200": {"description": "OK"}}},
            "delete": {"tags": ["Schedules"], "summary": "Delete", "responses": {"200": {"description": "OK"}}}
        },
        "/api/schedules/{id}/trigger": {
            "post": {"tags": ["Schedules"], "summary": "Trigger", "responses": {"200": {"description": "Result"}}}
        },
        "/api/gateway": {
            "get": {"tags": ["Gateway"], "summary": "List APIs", "responses": {"200": {"description": "List"}}}
        },
        "/api/gateway/publish": {
            "post": {"tags": ["Gateway"], "summary": "Publish API", "responses": {"200": {"description": "Published"}}}
        },
        "/api/gateway/unpublish/{path}": {
            "delete": {"tags": ["Gateway"], "summary": "Unpublish", "responses": {"200": {"description": "OK"}}}
        },
        "/api/run/{path}": {
            "get": {"tags": ["Gateway"], "summary": "Run (GET)", "responses": {"200": {"description": "Result"}}},
            "post": {"tags": ["Gateway"], "summary": "Run (POST)", "responses": {"200": {"description": "Result"}}}
        },
        "/api/nodes/types": {
            "get": {"tags": ["Nodes"], "summary": "List types", "responses": {"200": {"description": "List"}}}
        },
        "/api/browser/status": {
            "get": {"tags": ["Browser"], "summary": "Status", "responses": {"200": {"description": "Status"}}}
        }
    });

    serde_json::json!({
        "openapi": "3.0.3",
        "info": info,
        "servers": [{"url": "http://localhost:19529"}],
        "tags": tags,
        "paths": paths,
        "components": {
            "securitySchemes": {
                "Bearer": {"type": "http", "scheme": "bearer"},
                "ApiKey": {"type": "apiKey", "in": "header", "name": "X-API-Key"}
            }
        },
        "x-flowforge": {
            "nodeTypes": node_types.len(),
            "publishedApis": published_count
        }
    })
}


const SWAGGER_HTML: &str = r#"<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="UTF-8">
    <title>FlowForge API Docs</title>
    <link rel="stylesheet" href="https://unpkg.com/swagger-ui-dist@5/swagger-ui.css">
    <style>
        html { box-sizing: border-box; overflow-y: scroll; }
        *, *:before, *:after { box-sizing: inherit; }
        body { margin: 0; background: #fafafa; }
        .topbar { display: none !important; }
    </style>
</head>
<body>
    <div id="swagger-ui"></div>
    <script src="https://unpkg.com/swagger-ui-dist@5/swagger-ui-bundle.js"></script>
    <script>
        SwaggerUIBundle({
            url: '/api/openapi.json',
            dom_id: '#swagger-ui',
            deepLinking: true,
            presets: [
                SwaggerUIBundle.presets.apis,
                SwaggerUIBundle.SwaggerUIStandalonePreset
            ],
            layout: "BaseLayout"
        });
    </script>
</body>
</html>"#;
