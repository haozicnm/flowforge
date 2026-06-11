//! Variable node — sets a variable value with optional type conversion.
use async_trait::async_trait;
use std::collections::HashMap;
use crate::error::{FlowError, FlowResult};
use crate::engine::workflow::Node;
use crate::nodes::traits::{NodeExecutor, NodeTypeDef, PortDef};

#[derive(Default)]
pub struct VariableNode;

#[async_trait]
impl NodeExecutor for VariableNode {
    fn type_def(&self) -> NodeTypeDef {
        NodeTypeDef {
            type_name: "variable".to_string(),
            display_name: "变量赋值".to_string(),
            description: "设置变量值，支持类型转换".to_string(),
            category: "数据操作".to_string(),
            inputs: vec![
                PortDef { label: "in".to_string(), data_type: "any".to_string(), required: false },
            ],
            outputs: vec![
                PortDef { label: "out".to_string(), data_type: "any".to_string(), required: false },
            ],
            config_schema: serde_json::json!({
                "type": "object",
                "properties": {
                    "value": { "description": "Static value (used if no input connected)" },
                    "cast_to": {
                        "type": "string",
                        "enum": ["auto", "string", "number", "boolean", "array", "object"],
                        "default": "auto"
                    },
                    "default": { "description": "Default value when input is null" }
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
        let cast_to = config["cast_to"].as_str().unwrap_or("auto");
        let default_val = config.get("default").cloned().unwrap_or(serde_json::Value::Null);

        // Priority: input > config.value > default
        let raw = inputs.get("in").cloned()
            .filter(|v| !v.is_null())
            .or_else(|| config.get("value").cloned().filter(|v| !v.is_null()))
            .unwrap_or(default_val);

        let raw_debug = raw.clone();
        let result = match cast_to {
            "auto" => raw,
            "string" => serde_json::json!(match &raw {
                serde_json::Value::String(s) => s.clone(),
                other => other.to_string(),
            }),
            "number" => {
                let n = raw.as_f64().ok_or_else(|| FlowError::InvalidNodeConfig {
                    node_id: "variable".to_string(),
                    detail: format!("cannot convert {:?} to number", raw),
                })?;
                serde_json::json!(n)
            }
            "boolean" => serde_json::json!(!raw.is_null() && raw != serde_json::json!(false) && raw != serde_json::json!("")),
            "array" => match raw {
                serde_json::Value::Array(_) => raw,
                _ => serde_json::json!([raw]),
            },
            "object" => match raw {
                serde_json::Value::Object(_) => raw,
                _ => serde_json::json!({"value": raw}),
            },
            _ => raw,
        };

        tracing::info!("Variable: {:?} -> {:?} (cast: {})", raw_debug, result, cast_to);
        let mut outputs = HashMap::new();
        outputs.insert("out".to_string(), result);
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
            node_type: "variable".to_string(),
            label: "Test Var".to_string(),
            config: serde_json::json!({}),
            position: Default::default(),
        }
    }

    #[tokio::test]
    async fn test_variable_passthrough() {
        let node = make_node("var_1");
        let ctx = NodeContext::empty();
        let config = serde_json::json!({"value": "hello"});
        let inputs = HashMap::new();
        let result = VariableNode.execute(&node, &ctx, config, inputs).await.unwrap();
        assert_eq!(result["out"], "hello");
    }

    #[tokio::test]
    async fn test_variable_cast_number() {
        let node = make_node("var_1");
        let ctx = NodeContext::empty();
        let config = serde_json::json!({"value": 42, "cast_to": "number"});
        let inputs = HashMap::new();
        let result = VariableNode.execute(&node, &ctx, config, inputs).await.unwrap();
        assert_eq!(result["out"].as_f64().unwrap(), 42.0);
    }

    #[tokio::test]
    async fn test_variable_cast_boolean() {
        let node = make_node("var_1");
        let ctx = NodeContext::empty();
        let config = serde_json::json!({"value": "true", "cast_to": "boolean"});
        let inputs = HashMap::new();
        let result = VariableNode.execute(&node, &ctx, config, inputs).await.unwrap();
        assert_eq!(result["out"], true);
    }

    #[tokio::test]
    async fn test_variable_from_input() {
        let node = make_node("var_1");
        let ctx = NodeContext::empty();
        let config = serde_json::json!({});
        let mut inputs = HashMap::new();
        inputs.insert("in".to_string(), serde_json::json!("from_input"));
        let result = VariableNode.execute(&node, &ctx, config, inputs).await.unwrap();
        assert_eq!(result["out"], "from_input");
    }
}
