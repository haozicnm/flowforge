//! Node registry — type_name → NodeExecutor implementation.
//!
//! Rule: All node types must be registered here at startup.
//! No dynamic loading, no reflection. Explicit registration.

use std::collections::HashMap;
use std::sync::RwLock;

use crate::error::{FlowError, FlowResult};
use crate::nodes::traits::{NodeExecutor, NodeTypeDef};

/// Thread-safe registry of all node types.
pub struct NodeRegistry {
    executors: RwLock<HashMap<String, Box<dyn NodeExecutor>>>,
}

impl NodeRegistry {
    pub fn new() -> Self {
        let registry = Self {
            executors: RwLock::new(HashMap::new()),
        };
        // Register built-in nodes
        registry.register_builtin::<super::http_node::HttpNode>();
        registry.register_builtin::<super::shell_node::ShellNode>();
        registry
    }

    /// Register a node type.
    pub fn register<E: NodeExecutor + 'static>(&self, executor: E) {
        let type_name = executor.type_def().type_name.clone();
        self.executors
            .write()
            .expect("registry lock poisoned") // safe: only write at startup
            .insert(type_name, Box::new(executor));
    }

    fn register_builtin<E: NodeExecutor + Default + 'static>(&self) {
        self.register(E::default());
    }

    /// Get all registered node type definitions (for the UI).
    pub fn all_type_defs(&self) -> Vec<NodeTypeDef> {
        self.executors
            .read()
            .expect("registry lock poisoned")
            .values()
            .map(|e| e.type_def())
            .collect()
    }

    /// Get an executor by type name.
    pub fn get(&self, type_name: &str) -> FlowResult<Box<dyn NodeExecutor>> {
        self.executors
            .read()
            .expect("registry lock poisoned")
            .get(type_name)
            .map(|e| {
                // Return a new instance via type_def + re-creation
                // For now, we'll use a different approach in the executor
                todo!("clone executor for execution")
            })
            .ok_or_else(|| FlowError::NodeTypeNotFound(type_name.to_string()))
    }

    /// Check if a node type is registered.
    pub fn has(&self, type_name: &str) -> bool {
        self.executors
            .read()
            .expect("registry lock poisoned")
            .contains_key(type_name)
    }
}
