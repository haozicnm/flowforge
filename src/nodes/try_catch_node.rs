//! Try/Catch node — catches errors from upstream nodes.
//!
//! In the execution graph, this node wraps error-producing nodes.
//! The executor checks if upstream nodes failed and routes accordingly.
use async_trait::async_trait;
use std::collections::HashMap;
use crate::error::FlowResult;
use crate::engine::workflow::Node;
use crate::nodes::traits::{NodeExecutor, NodeTypeDef, PortDef};

#[derive(Default)]
pub struct TryCatchNode;

#[async_trait]
impl NodeExecutor for TryCatchNode {
    fn type_def(&self) -> NodeTypeDef {
        NodeTypeDef {
            type_name: "try_catch".to_string(),
            display_name: "异常捕获".to_string(),
            description: "捕获上游错误，路由到错误处理分支".to_string(),
            category: "流程控制".to_string(),
            inputs: vec![
                PortDef { label: "in".to_string(), data_type: "any".to_string(), required: false },
                PortDef { label: "error".to_string(), data_type: "object".to_string(), required: false },
            ],
            outputs: vec![
                PortDef { label: "success".to_string(), data_type: "any".to_string(), required: false },
                PortDef { label: "error".to_string(), data_type: "object".to_string(), required: false },
                PortDef { label: "has_error".to_string(), data_type: "boolean".to_string(), required: false },
            ],
            config_schema: serde_json::json!({
                "type": "object",
                "properties": {
                    "catch_all": { "type": "boolean", "default": true }
                }
            }),
        }
    }

    async fn execute(
        &self,
        _node: &Node,
        _ctx: &crate::nodes::traits::NodeContext,
        _config: serde_json::Value,
        inputs: HashMap<String, serde_json::Value>
    ) -> FlowResult<HashMap<String, serde_json::Value>> {
        let has_error = inputs.contains_key("error");
        let mut outputs = HashMap::new();
        outputs.insert("has_error".to_string(), serde_json::json!(has_error));

        if has_error {
            let err = inputs.get("error").cloned().unwrap_or(serde_json::json!({"message": "unknown error"}));
            tracing::warn!("TryCatch: caught error: {}", err);
            outputs.insert("error".to_string(), err);
        } else {
            let value = inputs.get("in").cloned().unwrap_or(serde_json::Value::Null);
            tracing::info!("TryCatch: no error, passing through");
            outputs.insert("success".to_string(), value);
        }
        Ok(outputs)
    }
}
