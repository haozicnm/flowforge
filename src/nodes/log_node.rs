//! Log node — outputs data to the execution log.
use async_trait::async_trait;
use std::collections::HashMap;
use crate::error::FlowResult;
use crate::engine::workflow::Node;
use crate::nodes::traits::{NodeExecutor, NodeTypeDef, PortDef};

#[derive(Default)]
pub struct LogNode;

#[async_trait]
impl NodeExecutor for LogNode {
    fn type_def(&self) -> NodeTypeDef {
        NodeTypeDef {
            type_name: "log".to_string(),
            display_name: "日志输出".to_string(),
            description: "将数据输出到执行日志".to_string(),
            category: "调试".to_string(),
            inputs: vec![PortDef { label: "message".to_string(), data_type: "any".to_string(), required: true }],
            outputs: vec![],
            config_schema: serde_json::json!({"type": "object", "properties": {"level": {"type": "string", "default": "info"}}}),
        }
    }

    async fn execute(&self, node: &Node, config: serde_json::Value, inputs: HashMap<String, serde_json::Value>) -> FlowResult<HashMap<String, serde_json::Value>> {
        let level = config["level"].as_str().unwrap_or("info");
        let msg = inputs.get("message").map(|v| v.to_string()).unwrap_or_default();
        match level {
            "warn" => tracing::warn!("[{}] {}", node.id, msg),
            "error" => tracing::error!("[{}] {}", node.id, msg),
            _ => tracing::info!("[{}] {}", node.id, msg),
        }
        Ok(HashMap::new())
    }
}
