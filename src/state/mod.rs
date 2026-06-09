//! Unified application state.
//!
//! Rule: ALL state lives here. No global singletons, no bypassing axum State.
//! Access via `State<AppState>` in handlers.

use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

use crate::engine::workflow::Workflow;

/// The single source of truth for all application state.
#[derive(Clone)]
pub struct AppState {
    /// Saved workflows (id → Workflow).
    pub workflows: Arc<RwLock<HashMap<String, Workflow>>>,

    /// Node type registry (type_name → NodeDef).
    pub node_registry: Arc<crate::nodes::registry::NodeRegistry>,

    /// Server config.
    pub config: ServerConfig,
}

/// Server-level configuration.
#[derive(Clone, Debug)]
pub struct ServerConfig {
    /// Bind address (default: 127.0.0.1, NOT 0.0.0.0)
    pub bind_addr: String,

    /// Static files directory for the frontend.
    pub static_dir: String,

    /// Data directory for workflows, logs, etc.
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
        Self {
            workflows: Arc::new(RwLock::new(HashMap::new())),
            node_registry: Arc::new(crate::nodes::registry::NodeRegistry::new()),
            config,
        }
    }
}
