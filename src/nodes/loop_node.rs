//! Loop node — iterates over a collection or repeats N times.
use async_trait::async_trait;
use std::collections::HashMap;
use crate::error::{FlowError, FlowResult};
use crate::engine::workflow::Node;
use crate::nodes::traits::{NodeExecutor, NodeTypeDef, PortDef};

#[derive(Default)]
pub struct LoopNode;

#[async_trait]
impl NodeExecutor for LoopNode {
    fn type_def(&self) -> NodeTypeDef {
        NodeTypeDef {
            type_name: "loop".to_string(),
            display_name: "循环".to_string(),
            description: "遍历集合或重复执行 N 次".to_string(),
            category: "流程控制".to_string(),
            inputs: vec![
                PortDef { label: "collection".to_string(), data_type: "any".to_string(), required: false },
            ],
            outputs: vec![
                PortDef { label: "item".to_string(), data_type: "any".to_string(), required: false },
                PortDef { label: "index".to_string(), data_type: "number".to_string(), required: false },
                PortDef { label: "items".to_string(), data_type: "array".to_string(), required: false },
            ],
            config_schema: serde_json::json!({
                "type": "object",
                "properties": {
                    "mode": {
                        "type": "string",
                        "enum": ["count", "collection"],
                        "default": "count"
                    },
                    "count": { "type": "number", "default": 1 }
                }
            }),
        }
    }

    async fn execute(
        &self,
        _node: &Node,
        _ctx: &crate::nodes::traits::NodeContext,
        config: serde_json::Value,
        inputs: HashMap<String, serde_json::Value>,
    ) -> FlowResult<HashMap<String, serde_json::Value>> {
        let mode = config["mode"].as_str().unwrap_or("count");
        let mut outputs = HashMap::new();

        match mode {
            "count" => {
                let count = config["count"].as_u64().unwrap_or(1);
                tracing::info!("Loop: repeat {} times", count);
                outputs.insert("index".to_string(), serde_json::json!(count));
                outputs.insert("item".to_string(), serde_json::json!(count));
            }
            "collection" => {
                let collection = inputs.get("collection").cloned().unwrap_or(serde_json::Value::Array(vec![]));
                if let Some(arr) = collection.as_array() {
                    tracing::info!("Loop: {} items in collection", arr.len());
                    outputs.insert("items".to_string(), serde_json::json!(arr));
                    outputs.insert("index".to_string(), serde_json::json!(arr.len()));
                    if let Some(first) = arr.first() {
                        outputs.insert("item".to_string(), first.clone());
                    }
                } else {
                    return Err(FlowError::InvalidNodeConfig {
                        node_id: "loop".to_string(),
                        detail: "collection input must be an array".to_string(),
                    });
                }
            }
            _ => {
                return Err(FlowError::InvalidNodeConfig {
                    node_id: "loop".to_string(),
                    detail: format!("unknown mode: {}", mode),
                });
            }
        }
        Ok(outputs)
    }
}
