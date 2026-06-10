//! Condition node — branches execution based on an expression.
use async_trait::async_trait;
use std::collections::HashMap;
use crate::error::{FlowError, FlowResult};
use crate::engine::workflow::Node;
use crate::nodes::traits::{NodeExecutor, NodeTypeDef, PortDef};

#[derive(Default)]
pub struct ConditionNode;

#[async_trait]
impl NodeExecutor for ConditionNode {
    fn type_def(&self) -> NodeTypeDef {
        NodeTypeDef {
            type_name: "condition".to_string(),
            display_name: "条件判断".to_string(),
            description: "根据条件表达式分支执行".to_string(),
            category: "流程控制".to_string(),
            inputs: vec![PortDef {
                label: "value".to_string(),
                data_type: "any".to_string(),
                required: true,
            }],
            outputs: vec![
                PortDef { label: "true".to_string(), data_type: "any".to_string(), required: false },
                PortDef { label: "false".to_string(), data_type: "any".to_string(), required: false },
                PortDef { label: "result".to_string(), data_type: "boolean".to_string(), required: false },
            ],
            config_schema: serde_json::json!({
                "type": "object",
                "properties": {
                    "operator": {
                        "type": "string",
                        "enum": ["equals", "not_equals", "contains", "gt", "lt", "gte", "lte", "is_empty", "is_not_empty", "regex_match", "starts_with", "ends_with"],
                        "default": "equals"
                    },
                    "compare_value": { "type": "string" }
                },
                "required": ["operator"]
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
        let value = inputs.get("value").cloned().unwrap_or(serde_json::Value::Null);
        let operator = config["operator"].as_str().unwrap_or("equals");
        let compare = &config["compare_value"];

        let result = match operator {
            "equals" => value == *compare,
            "not_equals" => value != *compare,
            "contains" => {
                let s = value.as_str().unwrap_or("");
                let needle = compare.as_str().unwrap_or("");
                s.contains(needle)
            }
            "gt" => {
                let a = value.as_f64().unwrap_or(0.0);
                let b = compare.as_f64().unwrap_or(0.0);
                a > b
            }
            "lt" => {
                let a = value.as_f64().unwrap_or(0.0);
                let b = compare.as_f64().unwrap_or(0.0);
                a < b
            }
            "gte" => {
                let a = value.as_f64().unwrap_or(0.0);
                let b = compare.as_f64().unwrap_or(0.0);
                a >= b
            }
            "lte" => {
                let a = value.as_f64().unwrap_or(0.0);
                let b = compare.as_f64().unwrap_or(0.0);
                a <= b
            }
            "is_empty" => {
                value.is_null()
                    || value.as_str().map_or(false, |s| s.is_empty())
                    || value.as_array().map_or(false, |a| a.is_empty())
                    || value.as_object().map_or(false, |o| o.is_empty())
            }
            "is_not_empty" => {
                !value.is_null()
                    && !value.as_str().map_or(false, |s| s.is_empty())
                    && !value.as_array().map_or(false, |a| a.is_empty())
                    && !value.as_object().map_or(false, |o| o.is_empty())
            }
            "regex_match" => {
                let s = value.as_str().unwrap_or("");
                let pattern = compare.as_str().unwrap_or("");
                regex::Regex::new(pattern).map_or(false, |re| re.is_match(s))
            }
            "starts_with" => {
                let s = value.as_str().unwrap_or("");
                let prefix = compare.as_str().unwrap_or("");
                s.starts_with(prefix)
            }
            "ends_with" => {
                let s = value.as_str().unwrap_or("");
                let suffix = compare.as_str().unwrap_or("");
                s.ends_with(suffix)
            }
            _ => {
                return Err(FlowError::InvalidNodeConfig {
                    node_id: "condition".to_string(),
                    detail: format!("unknown operator: {}", operator),
                });
            }
        };

        tracing::info!("Condition: {:?} {} {:?} = {}", value, operator, compare, result);

        let mut outputs = HashMap::new();
        outputs.insert("result".to_string(), serde_json::json!(result));
        if result {
            outputs.insert("true".to_string(), value);
        } else {
            outputs.insert("false".to_string(), value);
        }
        Ok(outputs)
    }
}
