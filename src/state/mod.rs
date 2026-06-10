//! Application state — single source of truth.
//!
//! Rule: All shared state lives here. Passed via axum::extract::State.
//! No global singletons. No static mutable state.

use std::sync::Arc;

use crate::engine::storage::WorkflowStorage;
use crate::nodes::registry::NodeRegistry;
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

        Self {
            node_registry: Arc::new(NodeRegistry::new()),
            storage: Arc::new(storage),
            webbridge: WebBridgeState::new(),
            _config: config,
        }
    }
}
