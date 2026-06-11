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
            .or_else(|| inputs.get("in").and_then(|v| v.as_str()).map(|s| s.to_string()))
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::nodes::traits::NodeContext;

    fn make_node(id: &str) -> Node {
        Node {
            id: id.to_string(),
            node_type: "log".to_string(),
            label: "Test Log".to_string(),
            config: serde_json::json!({}),
            position: Default::default(),
        }
    }

    fn make_ctx() -> NodeContext {
        NodeContext::empty()
    }

    #[tokio::test]
    async fn test_log_with_config_message() {
        let node = make_node("test_log");
        let ctx = make_ctx();
        let config = serde_json::json!({"message": "Hello World", "level": "info"});
        let inputs = HashMap::new();

        let result = LogNode.execute(&node, &ctx, config, inputs).await.unwrap();
        assert_eq!(result["out"], "Hello World");
    }

    #[tokio::test]
    async fn test_log_with_input() {
        let node = make_node("test_log");
        let ctx = make_ctx();
        let config = serde_json::json!({"level": "info"});
        let mut inputs = HashMap::new();
        inputs.insert("in".to_string(), serde_json::json!("Input data"));

        let result = LogNode.execute(&node, &ctx, config, inputs).await.unwrap();
        assert_eq!(result["out"], "Input data");
    }

    #[tokio::test]
    async fn test_log_empty() {
        let node = make_node("test_log");
        let ctx = make_ctx();
        let config = serde_json::json!({});
        let inputs = HashMap::new();

        let result = LogNode.execute(&node, &ctx, config, inputs).await.unwrap();
        assert_eq!(result["out"], "");
    }

    #[tokio::test]
    async fn test_log_type_def() {
        let def = LogNode.type_def();
        assert_eq!(def.type_name, "log");
        assert_eq!(def.display_name, "日志输出");
        assert_eq!(def.inputs.len(), 1);
        assert_eq!(def.outputs.len(), 1);
    }
}
