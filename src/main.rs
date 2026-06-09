//! FlowForge — Visual Workflow Automation Engine
//!
//! Architecture: Rust HTTP server + Flutter desktop UI.
//! The server runs on localhost, Flutter connects via HTTP/WebSocket.

mod api;
mod engine;
mod error;
mod nodes;
mod state;

use axum::{routing::get, Router};
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

    tracing::info!("FlowForge v{} starting...", env!("CARGO_PKG_VERSION"));

    // Build router
    let app = Router::new()
        .route("/api/health", get(api::health))
        .route("/api/nodes/types", get(api::node_types))
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
