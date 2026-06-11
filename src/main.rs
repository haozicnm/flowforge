//! FlowForge — Visual Workflow Automation Engine
//!
//! Architecture: Rust HTTP server + Flutter desktop UI.
//! The server runs on localhost, Flutter connects via HTTP/WebSocket.

mod api;
mod api_gateway;
#[allow(dead_code)]
mod auth;
mod engine;
mod error;
mod nodes;
mod plugin;
mod scheduler;
mod state;
mod webbridge;

use axum::{
    routing::{delete, get, post, put},
    Router,
};
use tower_http::cors::CorsLayer;
use tracing_subscriber::prelude::*;

use clap::Parser;
use state::{AppState, ServerConfig};

/// FlowForge — Visual Workflow Automation Engine
#[derive(Parser, Debug)]
#[command(name = "flowforge", version, about)]
struct Cli {
    /// Port to listen on
    #[arg(short, long, default_value = "19529")]
    port: u16,

    /// Data directory for persistence
    #[arg(short, long, default_value = "data")]
    data_dir: String,

    /// Static files directory (Flutter web build)
    #[arg(short, long, default_value = "dist")]
    static_dir: String,

    /// Bind address
    #[arg(long, default_value = "127.0.0.1")]
    bind: String,
}

#[tokio::main]
async fn main() {
    let cli = Cli::parse();

    // Setup logging
    tracing_subscriber::registry()
        .with(tracing_subscriber::EnvFilter::new(
            std::env::var("RUST_LOG").unwrap_or_else(|_| "flowforge=info,tower_http=info".into()),
        ))
        .with(tracing_subscriber::fmt::layer())
        .init();

    let config = ServerConfig {
        bind_addr: format!("{}:{}", cli.bind, cli.port),
        static_dir: cli.static_dir,
        data_dir: cli.data_dir,
    };
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

    // Clone scheduler before state is moved into router
    let scheduler = state.scheduler.clone();

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

        // Schedule routes
        .route("/api/schedules", get(api::list_schedules))
        .route("/api/schedules", post(api::create_schedule))
        .route("/api/schedules/:id", get(api::get_schedule))
        .route("/api/schedules/:id", put(api::update_schedule))
        .route("/api/schedules/:id", delete(api::delete_schedule))
        .route("/api/schedules/:id/trigger", post(api::trigger_schedule))

        // API Gateway routes
        .route("/api/openapi.json", get(api::openapi::openapi_spec))
        .route("/api/docs", get(api::openapi::swagger_ui))
        .route("/api/gateway", get(api::gateway_list))
        .route("/api/gateway/publish", post(api::gateway_publish))
        .route("/api/gateway/unpublish/:path", delete(api::gateway_unpublish))
        .route("/api/run/:path", get(api::gateway_run))
        .route("/api/run/:path", post(api::gateway_run))
        .route("/api/run/:path", put(api::gateway_run))
        .route("/api/run/:path", delete(api::gateway_run))
        .layer(CorsLayer::permissive())
        .with_state(state)
        .fallback_service(tower_http::services::ServeDir::new(&static_dir));

    // Start scheduler background tick
    tokio::spawn(async move {
        loop {
            tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;
            scheduler.tick().await;
        }
    });
    tracing::info!("📅 Scheduler started");

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
