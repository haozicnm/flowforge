//! FlowForge — Visual Workflow Automation Engine
//!
//! Architecture: Rust HTTP server + Flutter desktop UI.
//! The server runs on localhost, Flutter connects via HTTP/WebSocket.

mod api;
#[allow(dead_code)]
mod auth;
mod engine;
mod error;
mod nodes;
mod plugin;
mod state;
mod webbridge;

use axum::{
    routing::{delete, get, post, put},
    Router,
};
use tower_http::cors::CorsLayer;
use tracing_subscriber::prelude::*;

use state::{AppState, ServerConfig};

#[tokio::main]
async fn main() {
    // Setup logging
    tracing_subscriber::registry()
        .with(tracing_subscriber::EnvFilter::new(
            std::env::var("RUST_LOG").unwrap_or_else(|_| "flowforge=info,tower_http=info".into()),
        ))
        .with(tracing_subscriber::fmt::layer())
        .init();

    let config = ServerConfig::default();
    let bind_addr = config.bind_addr.clone();
    let static_dir = config.static_dir.clone();

    let state = AppState::new(config);

    // Load dynamic plugins
    let plugin_mgr = plugin::PluginManager::new("plugins");
    match plugin_mgr.scan_and_load(&state.node_registry) {
        Ok(0) => {}
        Ok(n) => tracing::info!("Loaded {} plugins", n),
        Err(e) => tracing::warn!("Plugin loading error: {}", e),
    }

    tracing::info!("FlowForge v{} starting...", env!("CARGO_PKG_VERSION"));

    // Build router
    let app = Router::new()
        // Health
        .route("/api/health", get(api::health))
        // Auth
        .route("/api/auth/register", post(api::register))
        .route("/api/auth/login", post(api::login))
        .route("/api/auth/me", get(api::whoami))
        // Node types
        .route("/api/nodes/types", get(api::node_types))
        // Plugin management
        .route("/api/plugins/list", get(api::list_plugins))
        // Workflow CRUD
        .route("/api/workflows", get(api::list_workflows))
        .route("/api/workflows", post(api::create_workflow))
        .route("/api/workflows/export-all", get(api::export_all_workflows))
        .route("/api/workflows/import", post(api::import_workflow))
        .route("/api/workflows/:id", get(api::get_workflow))
        .route("/api/workflows/:id", put(api::update_workflow))
        .route("/api/workflows/:id", delete(api::delete_workflow))
        .route("/api/workflows/:id/export", get(api::export_workflow))
        // Execution
        .route("/api/workflows/:id/execute", post(api::execute_workflow))
        .route("/api/workflows/:id/execute-step", post(api::execute_step))
        // WebBridge — browser automation via Chrome extension
        .route("/api/browser/status", get(api::browser_status))
        .route("/api/browser/command", post(webbridge::browser_command))
        .route("/ws/browser", get(webbridge::ws_handler))
        // WebSocket execution streaming
        .route("/ws/execute/:id", get(api::ws_execute))
        // Webhook trigger
        .route("/api/webhook/:workflow_id/:node_id", post(api::webhook_trigger))
        .route("/api/webhook/:workflow_id/:node_id", get(api::webhook_trigger))
        .layer(CorsLayer::permissive())
        .with_state(state)
        .fallback_service(tower_http::services::ServeDir::new(&static_dir));

    // Bind and serve
    let listener = tokio::net::TcpListener::bind(&bind_addr)
        .await
        .unwrap_or_else(|e| {
            eprintln!("❌ Failed to bind {}: {}", bind_addr, e);
            std::process::exit(1);
        });

    tracing::info!("🚀 Listening on http://{}", bind_addr);

    axum::serve(listener, app).await.expect("server error");
}
