//! Application state — single source of truth.
//!
//! Rule: All shared state lives here. Passed via axum::extract::State.
//! No global singletons. No static mutable state.

use std::sync::Arc;

use crate::auth::AuthState;
use crate::engine::storage::WorkflowStorage;
use crate::nodes::registry::NodeRegistry;
use crate::scheduler::Scheduler;
use crate::webbridge::WebBridgeState;

/// Shared application state.
#[derive(Clone)]
pub struct AppState {
    /// Node type registry.
    pub node_registry: Arc<NodeRegistry>,

    /// Workflow persistence.
    pub storage: Arc<WorkflowStorage>,

    /// WebBridge — Chrome extension relay.
    pub webbridge: WebBridgeState,

    /// Pending webhook payloads (keyed by "workflow_id:node_id").
    /// Each value is a JSON object: { "body", "headers", "method", "received_at" }.
    pub webhook_store: Arc<std::sync::Mutex<std::collections::HashMap<String, Vec<serde_json::Value>>>>,

    /// 调度器 — Cron 定时执行
    pub scheduler: Arc<Scheduler>,

    /// API 网关 — 工作流暴露为 API
    pub api_gateway: Arc<crate::api_gateway::ApiGateway>,

    /// Authentication database.
    pub auth_db: AuthState,

    /// Server configuration.
    pub _config: ServerConfig,
}

/// Server configuration.
#[derive(Debug, Clone)]
pub struct ServerConfig {
    pub bind_addr: String,
    pub static_dir: String,
    pub data_dir: String,
}

impl Default for ServerConfig {
    fn default() -> Self {
        Self {
            bind_addr: "127.0.0.1:19529".to_string(),
            static_dir: "dist".to_string(),
            data_dir: "data".to_string(),
        }
    }
}

impl AppState {
    pub fn new(config: ServerConfig) -> Self {
        let storage = WorkflowStorage::new(&config.data_dir);
        storage.init().expect("Failed to initialize storage");

        // Migrate old JSON files into SQLite
        match storage.migrate_from_files() {
            Ok(0) => {}
            Ok(n) => tracing::info!("Migrated {} workflows from JSON files to SQLite", n),
            Err(e) => tracing::warn!("Migration from files failed (non-fatal): {}", e),
        }

        let auth_db = Arc::new(
            crate::auth::AuthDb::open(&format!("{}/users.db", config.data_dir))
                .expect("Failed to initialize auth database"),
        );

        let node_registry = Arc::new(NodeRegistry::new());
        let storage = Arc::new(storage);
        let webbridge = WebBridgeState::new();
        let webhook_store = Arc::new(std::sync::Mutex::new(std::collections::HashMap::new()));

        let scheduler = Arc::new(Scheduler::new(
            storage.clone(),
            node_registry.clone(),
            webbridge.clone(),
            webhook_store.clone(),
            &config.data_dir,
        ));

        let api_gateway = Arc::new(crate::api_gateway::ApiGateway::new(
            storage.clone(),
            node_registry.clone(),
            webbridge.clone(),
            webhook_store.clone(),
            &config.data_dir,
        ));

        Self {
            node_registry,
            storage,
            webbridge,
            webhook_store,
            scheduler,
            api_gateway,
            auth_db,
            _config: config,
        }
    }
}
