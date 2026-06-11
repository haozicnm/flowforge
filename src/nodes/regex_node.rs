//! Regex node — pattern matching, extraction, and replacement.
use async_trait::async_trait;
use std::collections::HashMap;
use crate::error::{FlowError, FlowResult};
use crate::engine::workflow::Node;
use crate::nodes::traits::{NodeExecutor, NodeTypeDef, PortDef};

#[derive(Default)]
pub struct RegexNode;

#[async_trait]
impl NodeExecutor for RegexNode {
    fn type_def(&self) -> NodeTypeDef {
        NodeTypeDef {
            version: "1.0".to_string(),
            type_name: "regex".to_string(),
            display_name: "正则匹配".to_string(),
            description: "正则表达式匹配、提取、替换".to_string(),
            category: "数据操作".to_string(),
            inputs: vec![
                PortDef { label: "text".to_string(), data_type: "string".to_string(), required: true },
            ],
            outputs: vec![
                PortDef { label: "matches".to_string(), data_type: "array".to_string(), required: false },
                PortDef { label: "result".to_string(), data_type: "string".to_string(), required: false },
                PortDef { label: "count".to_string(), data_type: "number".to_string(), required: false },
            ],
            config_schema: serde_json::json!({
                "type": "object",
                "properties": {
                    "pattern": { "type": "string", "description": "Regex pattern" },
                    "operation": {
                        "type": "string",
                        "enum": ["match", "replace", "split", "find_all"],
                        "default": "match"
                    },
                    "replacement": { "type": "string", "description": "Replacement string (for replace op)" },
                    "flags": { "type": "string", "description": "Flags: i=ignorecase, m=multiline", "default": "" }
                },
                "required": ["pattern"]
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
        let text = inputs.get("text")
            .and_then(|v| v.as_str())
            .unwrap_or("");
        let pattern = config["pattern"].as_str().ok_or_else(|| FlowError::InvalidNodeConfig {
            node_id: "regex".to_string(),
            detail: "pattern is required".to_string(),
        })?;
        let operation = config["operation"].as_str().unwrap_or("match");

        let re = regex::Regex::new(pattern).map_err(|e| FlowError::InvalidNodeConfig {
            node_id: "regex".to_string(),
            detail: format!("invalid regex: {}", e),
        })?;

        let mut outputs = HashMap::new();

        match operation {
            "match" => {
                if let Some(caps) = re.captures(text) {
                    let matches: Vec<String> = caps.iter()
                        .filter_map(|m| m.map(|m| m.as_str().to_string()))
                        .collect();
                    outputs.insert("matches".to_string(), serde_json::json!(matches));
                    outputs.insert("count".to_string(), serde_json::json!(matches.len()));
                    outputs.insert("result".to_string(), serde_json::json!(caps.get(0).map(|m| m.as_str()).unwrap_or("")));
                } else {
                    outputs.insert("matches".to_string(), serde_json::json!([]));
                    outputs.insert("count".to_string(), serde_json::json!(0));
                    outputs.insert("result".to_string(), serde_json::json!(""));
                }
            }
            "replace" => {
                let replacement = config["replacement"].as_str().unwrap_or("");
                let result = re.replace_all(text, replacement);
                outputs.insert("result".to_string(), serde_json::json!(result.as_ref()));
                outputs.insert("matches".to_string(), serde_json::json!([]));
                outputs.insert("count".to_string(), serde_json::json!(re.find_iter(text).count()));
            }
            "split" => {
                let parts: Vec<&str> = re.split(text).collect();
                outputs.insert("matches".to_string(), serde_json::json!(parts));
                outputs.insert("count".to_string(), serde_json::json!(parts.len()));
                outputs.insert("result".to_string(), serde_json::json!(parts.join("")));
            }
            "find_all" => {
                let all: Vec<String> = re.find_iter(text).map(|m| m.as_str().to_string()).collect();
                outputs.insert("matches".to_string(), serde_json::json!(all));
                outputs.insert("count".to_string(), serde_json::json!(re.find_iter(text).count()));
                outputs.insert("result".to_string(), serde_json::json!(text));
            }
            _ => {
                return Err(FlowError::InvalidNodeConfig {
                    node_id: "regex".to_string(),
                    detail: format!("unknown operation: {}", operation),
                });
            }
        }
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
            node_type: "regex".to_string(),
            label: "Test Regex".to_string(),
            config: serde_json::json!({}),
            position: Default::default(),
        }
    }

    #[tokio::test]
    async fn test_regex_match() {
        let node = make_node("regex_1");
        let ctx = NodeContext::empty();
        let config = serde_json::json!({"pattern": r"(\d+)-(\d+)", "operation": "match"});
        let mut inputs = HashMap::new();
        inputs.insert("text".to_string(), serde_json::json!("2024-01-15"));
        let result = RegexNode.execute(&node, &ctx, config, inputs).await.unwrap();
        assert_eq!(result["count"], 3); // full match + 2 groups
    }

    #[tokio::test]
    async fn test_regex_replace() {
        let node = make_node("regex_1");
        let ctx = NodeContext::empty();
        let config = serde_json::json!({"pattern": r"\d+", "operation": "replace", "replacement": "X"});
        let mut inputs = HashMap::new();
        inputs.insert("text".to_string(), serde_json::json!("abc 123 def 456"));
        let result = RegexNode.execute(&node, &ctx, config, inputs).await.unwrap();
        assert_eq!(result["result"], "abc X def X");
    }

    #[tokio::test]
    async fn test_regex_find_all() {
        let node = make_node("regex_1");
        let ctx = NodeContext::empty();
        let config = serde_json::json!({"pattern": r"\d+", "operation": "find_all"});
        let mut inputs = HashMap::new();
        inputs.insert("text".to_string(), serde_json::json!("abc 123 def 456"));
        let result = RegexNode.execute(&node, &ctx, config, inputs).await.unwrap();
        assert_eq!(result["matches"], serde_json::json!(["123", "456"]));
        assert_eq!(result["count"], 2);
    }

    #[tokio::test]
    async fn test_regex_split() {
        let node = make_node("regex_1");
        let ctx = NodeContext::empty();
        let config = serde_json::json!({"pattern": r",\s*", "operation": "split"});
        let mut inputs = HashMap::new();
        inputs.insert("text".to_string(), serde_json::json!("a, b, c"));
        let result = RegexNode.execute(&node, &ctx, config, inputs).await.unwrap();
        assert_eq!(result["matches"], serde_json::json!(["a", "b", "c"]));
    }

    #[tokio::test]
    async fn test_regex_invalid_pattern() {
        let node = make_node("regex_1");
        let ctx = NodeContext::empty();
        let config = serde_json::json!({"pattern": "[invalid", "operation": "match"});
        let mut inputs = HashMap::new();
        inputs.insert("text".to_string(), serde_json::json!("test"));
        let result = RegexNode.execute(&node, &ctx, config, inputs).await;
        assert!(result.is_err());
    }
}
