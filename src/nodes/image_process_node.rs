//! Image Process node — resize, crop, watermark, convert images.

use async_trait::async_trait;
use std::collections::HashMap;
use std::path::Path;

use crate::engine::workflow::Node;
use crate::error::{FlowError, FlowResult};
use crate::nodes::traits::{NodeExecutor, NodeTypeDef, PortDef};

#[derive(Default)]
pub struct ImageProcessNode;

#[async_trait]
impl NodeExecutor for ImageProcessNode {
    fn type_def(&self) -> NodeTypeDef {
        NodeTypeDef {
            version: "1.0".to_string(),
            type_name: "image_process".to_string(),
            display_name: "图片处理".to_string(),
            description: "图片处理：缩放、裁剪、水印、格式转换".to_string(),
            category: "媒体".to_string(),
            inputs: vec![
                PortDef { label: "input_path".to_string(), data_type: "string".to_string(), required: true },
            ],
            outputs: vec![
                PortDef { label: "success".to_string(), data_type: "boolean".to_string(), required: false },
                PortDef { label: "output_path".to_string(), data_type: "string".to_string(), required: false },
                PortDef { label: "error".to_string(), data_type: "string".to_string(), required: false },
            ],
            config_schema: serde_json::json!({
                "type": "object",
                "properties": {
                    "input_path": { "type": "string", "description": "Input image path" },
                    "output_path": { "type": "string", "description": "Output image path" },
                    "operation": {
                        "type": "string",
                        "enum": ["resize", "crop", "watermark", "convert", "thumbnail"],
                        "default": "resize"
                    },
                    "width": { "type": "number", "description": "Target width" },
                    "height": { "type": "number", "description": "Target height" },
                    "x": { "type": "number", "default": 0, "description": "Crop X offset" },
                    "y": { "type": "number", "default": 0, "description": "Crop Y offset" },
                    "text": { "type": "string", "description": "Watermark text" },
                    "format": { "type": "string", "enum": ["png", "jpg", "webp", "gif"], "default": "png" }
                },
                "required": ["input_path", "output_path", "operation"]
            }),
        }
    }

    async fn execute(
        &self,
        node: &Node,
        _ctx: &crate::nodes::traits::NodeContext,
        config: serde_json::Value,
        inputs: HashMap<String, serde_json::Value>,
    ) -> FlowResult<HashMap<String, serde_json::Value>> {
        let input_path = config["input_path"].as_str()
            .or_else(|| inputs.get("input_path").and_then(|v| v.as_str()))
            .ok_or_else(|| FlowError::InvalidNodeConfig {
                node_id: node.id.clone(),
                detail: "input_path is required".to_string(),
            })?;

        let output_path = config["output_path"].as_str().ok_or_else(|| FlowError::InvalidNodeConfig {
            node_id: node.id.clone(),
            detail: "output_path is required".to_string(),
        })?;

        let operation = config["operation"].as_str().unwrap_or("resize");

        tracing::info!("Image process: {} -> {} ({})", input_path, output_path, operation);

        // Check input file exists
        if !Path::new(input_path).exists() {
            return Err(FlowError::NodeExecutionFailed {
                node_id: node.id.clone(),
                detail: format!("input file not found: {}", input_path),
            });
        }

        // Create output directory if needed
        if let Some(parent) = Path::new(output_path).parent() {
            if !parent.as_os_str().is_empty() {
                std::fs::create_dir_all(parent).map_err(|e| FlowError::NodeExecutionFailed {
                    node_id: node.id.clone(),
                    detail: format!("create output dir error: {}", e),
                })?;
            }
        }

        // Load image
        let img = image::open(input_path).map_err(|e| FlowError::NodeExecutionFailed {
            node_id: node.id.clone(),
            detail: format!("load image error: {}", e),
        })?;

        let result = match operation {
            "resize" => {
                let width = config["width"].as_u64().unwrap_or(800) as u32;
                let height = config["height"].as_u64().unwrap_or(600) as u32;
                img.resize(width, height, image::imageops::FilterType::Lanczos3)
            }
            "crop" => {
                let x = config["x"].as_u64().unwrap_or(0) as u32;
                let y = config["y"].as_u64().unwrap_or(0) as u32;
                let width = config["width"].as_u64().unwrap_or(100) as u32;
                let height = config["height"].as_u64().unwrap_or(100) as u32;
                img.crop_imm(x, y, width, height)
            }
            "watermark" => {
                let text = config["text"].as_str().unwrap_or("FlowForge");
                let mut rgba = img.to_rgba8();
                
                let width = rgba.width();
                let height = rgba.height();
                let text_width = text.len() as u32 * 10;
                let text_height = 20;
                let x = width.saturating_sub(text_width + 10);
                let y = height.saturating_sub(text_height + 10);
                
                for dy in 0..text_height {
                    for dx in 0..text_width {
                        let px = x + dx;
                        let py = y + dy;
                        if px < width && py < height {
                            let pixel = rgba.get_pixel_mut(px, py);
                            pixel[3] = pixel[3].saturating_add(50);
                        }
                    }
                }
                
                image::DynamicImage::ImageRgba8(rgba)
            }
            "convert" => {
                // Just return the image as-is, save will handle format
                img
            }
            "thumbnail" => {
                let width = config["width"].as_u64().unwrap_or(200) as u32;
                let height = config["height"].as_u64().unwrap_or(200) as u32;
                img.thumbnail(width, height)
            }
            _ => {
                return Err(FlowError::InvalidNodeConfig {
                    node_id: node.id.clone(),
                    detail: format!("unknown operation: {}", operation),
                });
            }
        };

        // Save output
        let format = config["format"].as_str().unwrap_or("png");
        let save_result = match format {
            "jpg" | "jpeg" => result.save_with_format(output_path, image::ImageFormat::Jpeg),
            "webp" => result.save_with_format(output_path, image::ImageFormat::WebP),
            "gif" => result.save_with_format(output_path, image::ImageFormat::Gif),
            _ => result.save(output_path), // default PNG
        };

        save_result.map_err(|e| FlowError::NodeExecutionFailed {
            node_id: node.id.clone(),
            detail: format!("save image error: {}", e),
        })?;

        let mut outputs = HashMap::new();
        outputs.insert("success".to_string(), serde_json::json!(true));
        outputs.insert("output_path".to_string(), serde_json::json!(output_path));
        outputs.insert("error".to_string(), serde_json::json!(""));
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
            node_type: "image_process".to_string(),
            label: "Test Image".to_string(),
            config: serde_json::json!({}),
            position: Default::default(),
        }
    }

    #[tokio::test]
    async fn test_image_no_input() {
        let node = make_node("img_1");
        let ctx = NodeContext::empty();
        let config = serde_json::json!({});
        let inputs = HashMap::new();
        let result = ImageProcessNode.execute(&node, &ctx, config, inputs).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_image_no_output() {
        let node = make_node("img_1");
        let ctx = NodeContext::empty();
        let config = serde_json::json!({"input_path": "/tmp/test.png"});
        let inputs = HashMap::new();
        let result = ImageProcessNode.execute(&node, &ctx, config, inputs).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_image_type_def() {
        let def = ImageProcessNode.type_def();
        assert_eq!(def.type_name, "image_process");
        assert_eq!(def.outputs.len(), 3);
    }
}
