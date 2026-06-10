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
            inputs: vec![PortDef { label: "in".to_string(), data_type: "any".to_string(), required: false }],
            outputs: vec![PortDef { label: "out".to_string(), data_type: "any".to_string(), required: false }],
            config_schema: serde_json::json!({
                "type": "object",
                "properties": {
                    "message": {"type": "string", "default": ""},
                    "level": {"type": "string", "default": "info"}
                }
            }),
        }
    }

    async fn execute(&self, node: &Node, _ctx: &crate::nodes::traits::NodeContext, config: serde_json::Value, inputs: HashMap<String, serde_json::Value>) -> FlowResult<HashMap<String, serde_json::Value>> {
        let level = config["level"].as_str().unwrap_or("info");
        // Read message from config, fall back to input
        let msg = config["message"].as_str()
            .map(|s| s.to_string())
            .or_else(|| inputs.get("in").map(|v| v.to_string()))
            .unwrap_or_default();

        match level {
            "warn" => tracing::warn!("[{}] {}", node.id, msg),
            "error" => tracing::error!("[{}] {}", node.id, msg),
            _ => tracing::info!("[{}] {}", node.id, msg),
        }

        // Pass through: output = input (or config message)
        let mut outputs = HashMap::new();
        outputs.insert("out".to_string(), serde_json::Value::String(msg));
        Ok(outputs)
    }
}
