//! Template node — renders a text template with variable interpolation.
use async_trait::async_trait;
use std::collections::HashMap;
use crate::error::FlowResult;
use crate::engine::workflow::Node;
use crate::nodes::traits::{NodeExecutor, NodeTypeDef, PortDef};

#[derive(Default)]
pub struct TemplateNode;

#[async_trait]
impl NodeExecutor for TemplateNode {
    fn type_def(&self) -> NodeTypeDef {
        NodeTypeDef {
            version: "1.0".to_string(),
            type_name: "template".to_string(),
            display_name: "文本模板".to_string(),
            description: "用 {{var}} 语法渲染文本模板".to_string(),
            category: "数据操作".to_string(),
            inputs: vec![
                PortDef { label: "vars".to_string(), data_type: "object".to_string(), required: false },
            ],
            outputs: vec![
                PortDef { label: "out".to_string(), data_type: "string".to_string(), required: false },
            ],
            config_schema: serde_json::json!({
                "type": "object",
                "properties": {
                    "template": { "type": "string", "description": "Template text with {{key}} placeholders" },
                    "escape": { "type": "string", "enum": ["none", "html", "json"], "default": "none" }
                },
                "required": ["template"]
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
        let template = config["template"].as_str().unwrap_or("");
        let escape = config["escape"].as_str().unwrap_or("none");
        let vars = inputs.get("vars").cloned().unwrap_or(serde_json::json!({}));

        let mut result = template.to_string();

        // Simple {{key}} replacement
        if let Some(obj) = vars.as_object() {
            for (key, value) in obj {
                let placeholder = format!("{{{{{}}}}}", key);
                let val_str = match escape {
                    "html" => html_escape(value),
                    "json" => serde_json::to_string(&value).unwrap_or_default(),
                    _ => value_to_string(value),
                };
                result = result.replace(&placeholder, &val_str);
            }
        }

        tracing::info!("Template: rendered {} chars", result.len());
        let mut outputs = HashMap::new();
        outputs.insert("out".to_string(), serde_json::Value::String(result));
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
            node_type: "template".to_string(),
            label: "Test Template".to_string(),
            config: serde_json::json!({}),
            position: Default::default(),
        }
    }

    #[tokio::test]
    async fn test_template_basic() {
        let node = make_node("tpl_1");
        let ctx = NodeContext::empty();
        let config = serde_json::json!({"template": "Hello {{name}}"});
        let mut inputs = HashMap::new();
        inputs.insert("vars".to_string(), serde_json::json!({"name": "World"}));
        let result = TemplateNode.execute(&node, &ctx, config, inputs).await.unwrap();
        assert_eq!(result["out"], "Hello World");
    }

    #[tokio::test]
    async fn test_template_multiple_vars() {
        let node = make_node("tpl_1");
        let ctx = NodeContext::empty();
        let config = serde_json::json!({"template": "{{greeting}} {{name}}!"});
        let mut inputs = HashMap::new();
        inputs.insert("vars".to_string(), serde_json::json!({"greeting": "Hi", "name": "Alice"}));
        let result = TemplateNode.execute(&node, &ctx, config, inputs).await.unwrap();
        assert_eq!(result["out"], "Hi Alice!");
    }

    #[tokio::test]
    async fn test_template_no_vars() {
        let node = make_node("tpl_1");
        let ctx = NodeContext::empty();
        let config = serde_json::json!({"template": "No variables here"});
        let inputs = HashMap::new();
        let result = TemplateNode.execute(&node, &ctx, config, inputs).await.unwrap();
        assert_eq!(result["out"], "No variables here");
    }
}

fn value_to_string(v: &serde_json::Value) -> String {
    match v {
        serde_json::Value::String(s) => s.clone(),
        serde_json::Value::Null => String::new(),
        other => other.to_string(),
    }
}

fn html_escape(v: &serde_json::Value) -> String {
    value_to_string(v)
        .replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&#39;")
}
