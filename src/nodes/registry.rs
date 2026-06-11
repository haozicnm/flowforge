//! Node registry — type_name → NodeExecutor implementation.
//!
//! Rule: All node types must be registered here at startup.
//! Supports both built-in (Arc) and dynamic plugins (Box→Arc).
//!
//! Version support: `type@version` syntax for get_executor.
//! Without `@version`, returns the latest version of a node type.

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

        // === Email ===
        registry.register_builtin::<super::email_send_node::EmailSendNode>();
        registry.register_builtin::<super::email_read_node::EmailReadNode>();

        // === FTP ===
        registry.register_builtin::<super::ftp_upload_node::FtpUploadNode>();
        registry.register_builtin::<super::ftp_download_node::FtpDownloadNode>();

        // === Media ===
        registry.register_builtin::<super::image_process_node::ImageProcessNode>();

        // === Document ===
        registry.register_builtin::<super::pdf_extract_node::PdfExtractNode>();

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

    /// Register a node type with an explicit versioned key.
    /// The stored key is `type_name@version` (e.g., "http@2.0").
    /// Also registers the un-versioned key if no executor is registered for it.
    #[allow(dead_code)]
    pub fn register_versioned<E: NodeExecutor + 'static>(&self, executor: E) {
        let def = executor.type_def();
        let versioned_key = format!("{}@{}", def.type_name, def.version);
        let arc = Arc::new(executor);

        let mut map = self.executors.write().expect("registry lock poisoned");

        // Always register the versioned key
        map.insert(versioned_key, arc.clone());

        // Register un-versioned key only if not already present (first-come wins)
        map.entry(def.type_name).or_insert(arc);
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

    /// Get an executor by type name. Supports `type@version` syntax.
    ///
    /// - `"http"` → returns the default (un-versioned) executor
    /// - `"http@2.0"` → returns the specific version, or error if not found
    pub fn get_executor(&self, type_name: &str) -> FlowResult<Arc<dyn NodeExecutor>> {
        let map = self.executors.read().expect("registry lock poisoned");

        // Try exact match first (handles both "http" and "http@2.0")
        if let Some(exec) = map.get(type_name) {
            return Ok(exec.clone());
        }

        // If the key contains "@", it was a versioned lookup that failed
        if type_name.contains('@') {
            return Err(FlowError::NodeTypeNotFound(type_name.to_string()));
        }

        Err(FlowError::NodeTypeNotFound(type_name.to_string()))
    }

    /// Check if a node type is registered.
    #[allow(dead_code)]
    pub fn has(&self, type_name: &str) -> bool {
        self.executors
            .read()
            .expect("registry lock poisoned")
            .contains_key(type_name)
    }

    /// List all registered type names (including versioned ones).
    #[allow(dead_code)]
    pub fn list_types(&self) -> Vec<String> {
        self.executors
            .read()
            .expect("registry lock poisoned")
            .keys()
            .cloned()
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::nodes::traits::NodeTypeDef;

    #[test]
    fn test_registry_basic_lookup() {
        let registry = NodeRegistry::new();
        assert!(registry.has("log"));
        assert!(registry.has("http"));
        assert!(!registry.has("nonexistent"));
    }

    #[test]
    fn test_registry_versioned_key() {
        let registry = NodeRegistry::new();
        // All built-in nodes are registered as version "1.0"
        let exec = registry.get_executor("log").unwrap();
        let def = exec.type_def();
        assert_eq!(def.version, "1.0");
    }

    #[test]
    fn test_registry_type_at_version_not_found() {
        let registry = NodeRegistry::new();
        // "log@99.0" should fail
        assert!(registry.get_executor("log@99.0").is_err());
    }

    #[test]
    fn test_registry_list_types() {
        let registry = NodeRegistry::new();
        let types = registry.list_types();
        assert!(types.len() >= 29); // at least 29 built-in nodes
        assert!(types.contains(&"log".to_string()));
        assert!(types.contains(&"http".to_string()));
    }

    #[test]
    fn test_registry_all_type_defs() {
        let registry = NodeRegistry::new();
        let defs = registry.all_type_defs();
        assert!(defs.len() >= 29);
        // All nodes should have version "1.0"
        for def in &defs {
            assert_eq!(def.version, "1.0", "Node '{}' should have version 1.0", def.type_name);
        }
    }

    #[test]
    fn test_node_typedef_version_field() {
        // Verify the version field is serialized/deserialized correctly
        let def = NodeTypeDef {
            type_name: "test".to_string(),
            version: "2.0".to_string(),
            display_name: "Test".to_string(),
            description: "Test node".to_string(),
            category: "Test".to_string(),
            inputs: vec![],
            outputs: vec![],
            config_schema: serde_json::json!({}),
        };

        let json = serde_json::to_string(&def).unwrap();
        let parsed: NodeTypeDef = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed.version, "2.0");
    }

    #[test]
    fn test_node_typedef_version_default() {
        // Deserialize without version field — should default to "1.0"
        let json = r#"{
            "type_name": "test",
            "display_name": "Test",
            "description": "Test node",
            "category": "Test",
            "inputs": [],
            "outputs": [],
            "config_schema": {}
        }"#;
        let parsed: NodeTypeDef = serde_json::from_str(json).unwrap();
        assert_eq!(parsed.version, "1.0");
    }
}
