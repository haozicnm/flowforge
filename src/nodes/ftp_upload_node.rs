//! FTP Upload node — uploads files via FTP/FTPS.

use async_trait::async_trait;
use std::collections::HashMap;
use std::path::Path;

use crate::engine::workflow::Node;
use crate::error::{FlowError, FlowResult};
use crate::nodes::traits::{NodeExecutor, NodeTypeDef, PortDef};

#[derive(Default)]
pub struct FtpUploadNode;

#[async_trait]
impl NodeExecutor for FtpUploadNode {
    fn type_def(&self) -> NodeTypeDef {
        NodeTypeDef {
            type_name: "ftp_upload".to_string(),
            display_name: "FTP 上传".to_string(),
            description: "通过 FTP/FTPS 上传文件".to_string(),
            category: "文件传输".to_string(),
            inputs: vec![
                PortDef { label: "local_path".to_string(), data_type: "string".to_string(), required: true },
            ],
            outputs: vec![
                PortDef { label: "success".to_string(), data_type: "boolean".to_string(), required: false },
                PortDef { label: "remote_path".to_string(), data_type: "string".to_string(), required: false },
                PortDef { label: "error".to_string(), data_type: "string".to_string(), required: false },
            ],
            config_schema: serde_json::json!({
                "type": "object",
                "properties": {
                    "host": { "type": "string", "description": "FTP server host" },
                    "port": { "type": "number", "default": 21, "description": "FTP server port" },
                    "username": { "type": "string", "default": "anonymous", "description": "FTP username" },
                    "password": { "type": "string", "default": "", "description": "FTP password" },
                    "local_path": { "type": "string", "description": "Local file path" },
                    "remote_path": { "type": "string", "description": "Remote file path" },
                    "passive": { "type": "boolean", "default": true, "description": "Use passive mode" }
                },
                "required": ["host", "local_path", "remote_path"]
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
        let host = config["host"].as_str().ok_or_else(|| FlowError::InvalidNodeConfig {
            node_id: node.id.clone(),
            detail: "host is required".to_string(),
        })?;

        let port = config["port"].as_u64().unwrap_or(21) as u16;
        let username = config["username"].as_str().unwrap_or("anonymous");
        let password = config["password"].as_str().unwrap_or("");
        let passive = config["passive"].as_bool().unwrap_or(true);

        let local_path = config["local_path"].as_str()
            .or_else(|| inputs.get("local_path").and_then(|v| v.as_str()))
            .ok_or_else(|| FlowError::InvalidNodeConfig {
                node_id: node.id.clone(),
                detail: "local_path is required".to_string(),
            })?;

        let remote_path = config["remote_path"].as_str().ok_or_else(|| FlowError::InvalidNodeConfig {
            node_id: node.id.clone(),
            detail: "remote_path is required".to_string(),
        })?;

        tracing::info!("Uploading {} to {}:{}/{}", local_path, host, port, remote_path);

        // Check local file exists
        if !Path::new(local_path).exists() {
            return Err(FlowError::NodeExecutionFailed {
                node_id: node.id.clone(),
                detail: format!("local file not found: {}", local_path),
            });
        }

        // FTP upload using suppaftp
        let mut ftp_stream = suppaftp::FtpStream::connect(format!("{}:{}", host, port))
            .map_err(|e| FlowError::NodeExecutionFailed {
                node_id: node.id.clone(),
                detail: format!("FTP connect error: {}", e),
            })?;

        ftp_stream.login(username, password).map_err(|e| FlowError::NodeExecutionFailed {
            node_id: node.id.clone(),
            detail: format!("FTP login error: {}", e),
        })?;

        if passive {
            ftp_stream.set_mode(suppaftp::Mode::Passive);
        } else {
            ftp_stream.set_mode(suppaftp::Mode::Active);
        }

        // Read local file
        let file_data = std::fs::read(local_path).map_err(|e| FlowError::NodeExecutionFailed {
            node_id: node.id.clone(),
            detail: format!("read local file error: {}", e),
        })?;

        // Create remote directory if needed
        if let Some(parent) = Path::new(remote_path).parent() {
            if !parent.as_os_str().is_empty() {
                let dir_path = parent.to_string_lossy().to_string();
                let _ = ftp_stream.mkdir(&dir_path); // Ignore if exists
            }
        }

        // Upload
        let mut reader = std::io::Cursor::new(file_data);
        ftp_stream.put_file(remote_path, &mut reader).map_err(|e| FlowError::NodeExecutionFailed {
            node_id: node.id.clone(),
            detail: format!("FTP upload error: {}", e),
        })?;

        ftp_stream.quit().ok();

        let mut outputs = HashMap::new();
        outputs.insert("success".to_string(), serde_json::json!(true));
        outputs.insert("remote_path".to_string(), serde_json::json!(remote_path));
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
            node_type: "ftp_upload".to_string(),
            label: "Test FTP Upload".to_string(),
            config: serde_json::json!({}),
            position: Default::default(),
        }
    }

    #[tokio::test]
    async fn test_ftp_upload_no_host() {
        let node = make_node("ftp_1");
        let ctx = NodeContext::empty();
        let config = serde_json::json!({});
        let inputs = HashMap::new();
        let result = FtpUploadNode.execute(&node, &ctx, config, inputs).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_ftp_upload_no_local_path() {
        let node = make_node("ftp_1");
        let ctx = NodeContext::empty();
        let config = serde_json::json!({"host": "ftp.example.com", "remote_path": "/upload/file.txt"});
        let inputs = HashMap::new();
        let result = FtpUploadNode.execute(&node, &ctx, config, inputs).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_ftp_upload_type_def() {
        let def = FtpUploadNode.type_def();
        assert_eq!(def.type_name, "ftp_upload");
        assert_eq!(def.outputs.len(), 3);
    }
}
