//! Node registry — type_name → NodeExecutor implementation.
//!
//! Rule: All node types must be registered here at startup.
//! Supports both built-in (Arc) and dynamic plugins (Box→Arc).

use std::collections::HashMap;
use std::sync::{Arc, RwLock};

use crate::error::{FlowError, FlowResult};
use crate::nodes::traits::{NodeExecutor, NodeTypeDef};

/// Thread-safe registry of all node types.
pub struct NodeRegistry {
    executors: RwLock<HashMap<String, Arc<dyn NodeExecutor>>>,
}

impl NodeRegistry {
    pub fn new() -> Self {
        let registry = Self {
            executors: RwLock::new(HashMap::new()),
        };

        // === Original 6 ===
        registry.register_builtin::<super::http_node::HttpNode>();
        registry.register_builtin::<super::shell_node::ShellNode>();
        registry.register_builtin::<super::delay_node::DelayNode>();
        registry.register_builtin::<super::script_node::ScriptNode>();
        registry.register_builtin::<super::webhook_node::WebhookNode>();
        registry.register_builtin::<super::log_node::LogNode>();

        // === Flow control ===
        registry.register_builtin::<super::condition_node::ConditionNode>();
        registry.register_builtin::<super::loop_node::LoopNode>();
        registry.register_builtin::<super::try_catch_node::TryCatchNode>();

        // === Data operations ===
        registry.register_builtin::<super::variable_node::VariableNode>();
        registry.register_builtin::<super::json_node::JsonNode>();
        registry.register_builtin::<super::regex_node::RegexNode>();
        registry.register_builtin::<super::template_node::TemplateNode>();

        // === Web automation (WebBridge) ===
        registry.register_builtin::<super::web_navigate_node::WebNavigateNode>();
        registry.register_builtin::<super::web_click_node::WebClickNode>();
        registry.register_builtin::<super::web_input_node::WebInputNode>();
        registry.register_builtin::<super::web_extract_node::WebExtractNode>();
        registry.register_builtin::<super::web_screenshot_node::WebScreenshotNode>();
        registry.register_builtin::<super::web_wait_node::WebWaitNode>();

        // === Excel ===
        registry.register_builtin::<super::excel_read_node::ExcelReadNode>();
        registry.register_builtin::<super::excel_write_node::ExcelWriteNode>();

        // === Word (.docx) ===
        registry.register_builtin::<super::docx_read_node::DocxReadNode>();
        registry.register_builtin::<super::docx_create_node::DocxCreateNode>();

        // === Database ===
        registry.register_builtin::<super::database_node::DatabaseNode>();

        // === Notification ===
        registry.register_builtin::<super::notification_node::NotificationNode>();

        // === File ===
        registry.register_builtin::<super::file_node::FileNode>();

        // === Cron / schedule ===
        registry.register_builtin::<super::cron_node::CronNode>();

        registry
    }

    /// Register a node type (built-in).
    pub fn register<E: NodeExecutor + 'static>(&self, executor: E) {
        let type_name = executor.type_def().type_name.clone();
        self.executors
            .write()
            .expect("registry lock poisoned")
            .insert(type_name, Arc::new(executor));
    }

    /// Register a node type from a Box (for dynamic plugins).
    pub fn register_boxed(&self, executor: Box<dyn NodeExecutor>) {
        let type_name = executor.type_def().type_name.clone();
        self.executors
            .write()
            .expect("registry lock poisoned")
            .insert(type_name, Arc::from(executor));
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
    pub fn get_executor(&self, type_name: &str) -> FlowResult<Arc<dyn NodeExecutor>> {
        self.executors
            .read()
            .expect("registry lock poisoned")
            .get(type_name)
            .cloned()
            .ok_or_else(|| FlowError::NodeTypeNotFound(type_name.to_string()))
    }

    /// Check if a node type is registered.
    #[allow(dead_code)]
    pub fn has(&self, type_name: &str) -> bool {
        self.executors
            .read()
            .expect("registry lock poisoned")
            .contains_key(type_name)
    }
}
