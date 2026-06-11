//! FTP Download node — downloads files via FTP/FTPS.

use async_trait::async_trait;
use std::collections::HashMap;
use std::path::Path;

use crate::engine::workflow::Node;
use crate::error::{FlowError, FlowResult};
use crate::nodes::traits::{NodeExecutor, NodeTypeDef, PortDef};

#[derive(Default)]
pub struct FtpDownloadNode;

#[async_trait]
impl NodeExecutor for FtpDownloadNode {
    fn type_def(&self) -> NodeTypeDef {
        NodeTypeDef {
            version: "1.0".to_string(),
            type_name: "ftp_download".to_string(),
            display_name: "FTP 下载".to_string(),
            description: "通过 FTP/FTPS 下载文件".to_string(),
            category: "文件传输".to_string(),
            inputs: vec![],
            outputs: vec![
                PortDef { label: "success".to_string(), data_type: "boolean".to_string(), required: false },
                PortDef { label: "local_path".to_string(), data_type: "string".to_string(), required: false },
                PortDef { label: "error".to_string(), data_type: "string".to_string(), required: false },
            ],
            config_schema: serde_json::json!({
                "type": "object",
                "properties": {
                    "host": { "type": "string", "description": "FTP server host" },
                    "port": { "type": "number", "default": 21, "description": "FTP server port" },
                    "username": { "type": "string", "default": "anonymous", "description": "FTP username" },
                    "password": { "type": "string", "default": "", "description": "FTP password" },
                    "remote_path": { "type": "string", "description": "Remote file path" },
                    "local_path": { "type": "string", "description": "Local file path to save" },
                    "passive": { "type": "boolean", "default": true, "description": "Use passive mode" }
                },
                "required": ["host", "remote_path", "local_path"]
            }),
        }
    }

    async fn execute(
        &self,
        node: &Node,
        _ctx: &crate::nodes::traits::NodeContext,
        config: serde_json::Value,
        _inputs: HashMap<String, serde_json::Value>,
    ) -> FlowResult<HashMap<String, serde_json::Value>> {
        let host = config["host"].as_str().ok_or_else(|| FlowError::InvalidNodeConfig {
            node_id: node.id.clone(),
            detail: "host is required".to_string(),
        })?;

        let port = config["port"].as_u64().unwrap_or(21) as u16;
        let username = config["username"].as_str().unwrap_or("anonymous");
        let password = config["password"].as_str().unwrap_or("");
        let passive = config["passive"].as_bool().unwrap_or(true);

        let remote_path = config["remote_path"].as_str().ok_or_else(|| FlowError::InvalidNodeConfig {
            node_id: node.id.clone(),
            detail: "remote_path is required".to_string(),
        })?;

        let local_path = config["local_path"].as_str().ok_or_else(|| FlowError::InvalidNodeConfig {
            node_id: node.id.clone(),
            detail: "local_path is required".to_string(),
        })?;

        tracing::info!("Downloading {}:{}:{} to {}", host, port, remote_path, local_path);

        // Create local directory if needed
        if let Some(parent) = Path::new(local_path).parent() {
            if !parent.as_os_str().is_empty() {
                std::fs::create_dir_all(parent).map_err(|e| FlowError::NodeExecutionFailed {
                    node_id: node.id.clone(),
                    detail: format!("create local dir error: {}", e),
                })?;
            }
        }

        // FTP download using suppaftp
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

        // Download file
        let file_data = ftp_stream.retr_as_buffer(remote_path).map_err(|e| FlowError::NodeExecutionFailed {
            node_id: node.id.clone(),
            detail: format!("FTP download error: {}", e),
        })?;

        // Write to local file
        std::fs::write(local_path, file_data.into_inner()).map_err(|e| FlowError::NodeExecutionFailed {
            node_id: node.id.clone(),
            detail: format!("write local file error: {}", e),
        })?;

        ftp_stream.quit().ok();

        let mut outputs = HashMap::new();
        outputs.insert("success".to_string(), serde_json::json!(true));
        outputs.insert("local_path".to_string(), serde_json::json!(local_path));
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
            node_type: "ftp_download".to_string(),
            label: "Test FTP Download".to_string(),
            config: serde_json::json!({}),
            position: Default::default(),
        }
    }

    #[tokio::test]
    async fn test_ftp_download_no_host() {
        let node = make_node("ftp_1");
        let ctx = NodeContext::empty();
        let config = serde_json::json!({});
        let inputs = HashMap::new();
        let result = FtpDownloadNode.execute(&node, &ctx, config, inputs).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_ftp_download_no_remote_path() {
        let node = make_node("ftp_1");
        let ctx = NodeContext::empty();
        let config = serde_json::json!({"host": "ftp.example.com", "local_path": "/tmp/file.txt"});
        let inputs = HashMap::new();
        let result = FtpDownloadNode.execute(&node, &ctx, config, inputs).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_ftp_download_type_def() {
        let def = FtpDownloadNode.type_def();
        assert_eq!(def.type_name, "ftp_download");
        assert_eq!(def.outputs.len(), 3);
    }
}
