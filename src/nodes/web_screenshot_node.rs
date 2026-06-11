//! Web Screenshot node — take a screenshot of the page or element.
use async_trait::async_trait;
use std::collections::HashMap;
use crate::error::{FlowError, FlowResult};
use crate::engine::workflow::Node;
use crate::nodes::traits::{NodeExecutor, NodeTypeDef, PortDef};

#[derive(Default)]
pub struct WebScreenshotNode;

#[async_trait]
impl NodeExecutor for WebScreenshotNode {
    fn type_def(&self) -> NodeTypeDef {
        NodeTypeDef {
            version: "1.0".to_string(),
            type_name: "web_screenshot".to_string(),
            display_name: "网页截图".to_string(),
            description: "对页面或指定元素截图".to_string(),
            category: "网页自动化".to_string(),
            inputs: vec![
                PortDef { label: "selector".to_string(), data_type: "string".to_string(), required: false },
            ],
            outputs: vec![
                PortDef { label: "image".to_string(), data_type: "string".to_string(), required: false },
            ],
            config_schema: serde_json::json!({
                "type": "object",
                "properties": {
                    "selector": { "type": "string", "description": "CSS selector (empty = full page)" },
                    "format": {
                        "type": "string",
                        "enum": ["png", "jpeg", "webp"],
                        "default": "png"
                    },
                    "quality": { "type": "number", "default": 80, "description": "JPEG/WebP quality (1-100)" },
                    "save_path": { "type": "string", "description": "Path to save the image file" }
                }
            }),
        }
    }

    async fn execute(
        &self,
        _node: &Node,
        ctx: &crate::nodes::traits::NodeContext,
        config: serde_json::Value,
        _inputs: HashMap<String, serde_json::Value>
    ) -> FlowResult<HashMap<String, serde_json::Value>> {
        let selector = config["selector"].as_str().unwrap_or("");
        let format = config["format"].as_str().unwrap_or("png");

        let mut params = serde_json::json!({ "format": format });
        if !selector.is_empty() {
            params["selector"] = serde_json::json!(selector);
        }

        let wb = ctx.webbridge.as_ref().ok_or_else(|| FlowError::NodeExecutionFailed {
            node_id: "web".to_string(),
            detail: "WebBridge not configured. Browser automation requires a connected Chrome extension.".to_string(),
        })?;
        let data = super::webbridge::send_browser_command(wb, "screenshot", params).await
            .map_err(|e| FlowError::NodeExecutionFailed {
                node_id: "web_screenshot".to_string(),
                detail: e.to_string(),
            })?;

        // data["dataUrl"] contains the base64 image
        let image_data = data["dataUrl"].as_str().unwrap_or("");

        // Save data URL to file if path specified
        if let Some(save_path) = config["save_path"].as_str() {
            // Save the data URL as-is (can be opened in browser)
            let _ = std::fs::write(save_path, image_data);
        }

        let mut outputs = HashMap::new();
        outputs.insert("image".to_string(), serde_json::json!(image_data));
        Ok(outputs)
    }
}
