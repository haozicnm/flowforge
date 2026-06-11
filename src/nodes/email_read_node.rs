//! Email Read node — reads emails via IMAP.
//!
//! Connects to IMAP server and fetches unread messages.

use async_trait::async_trait;
use std::collections::HashMap;

use crate::engine::workflow::Node;
use crate::error::{FlowError, FlowResult};
use crate::nodes::traits::{NodeExecutor, NodeTypeDef, PortDef};

#[derive(Default)]
pub struct EmailReadNode;

#[async_trait]
impl NodeExecutor for EmailReadNode {
    fn type_def(&self) -> NodeTypeDef {
        NodeTypeDef {
            version: "1.0".to_string(),
            type_name: "email_read".to_string(),
            display_name: "读取邮件".to_string(),
            description: "通过 IMAP 读取邮件".to_string(),
            category: "通信".to_string(),
            inputs: vec![],
            outputs: vec![
                PortDef { label: "messages".to_string(), data_type: "array".to_string(), required: false },
                PortDef { label: "count".to_string(), data_type: "number".to_string(), required: false },
                PortDef { label: "success".to_string(), data_type: "boolean".to_string(), required: false },
            ],
            config_schema: serde_json::json!({
                "type": "object",
                "properties": {
                    "imap_host": { "type": "string", "description": "IMAP server host" },
                    "imap_port": { "type": "number", "default": 993, "description": "IMAP server port" },
                    "username": { "type": "string", "description": "IMAP username" },
                    "password": { "type": "string", "description": "IMAP password" },
                    "folder": { "type": "string", "default": "INBOX", "description": "Mailbox folder" },
                    "limit": { "type": "number", "default": 10, "description": "Max messages to fetch" },
                    "unreadOnly": { "type": "boolean", "default": true, "description": "Only fetch unread messages" }
                },
                "required": ["imap_host", "username", "password"]
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
        let imap_host = config["imap_host"].as_str().ok_or_else(|| FlowError::InvalidNodeConfig {
            node_id: node.id.clone(),
            detail: "imap_host is required".to_string(),
        })?;

        let imap_port = config["imap_port"].as_u64().unwrap_or(993) as u16;
        let username = config["username"].as_str().ok_or_else(|| FlowError::InvalidNodeConfig {
            node_id: node.id.clone(),
            detail: "username is required".to_string(),
        })?;
        let password = config["password"].as_str().ok_or_else(|| FlowError::InvalidNodeConfig {
            node_id: node.id.clone(),
            detail: "password is required".to_string(),
        })?;
        let folder = config["folder"].as_str().unwrap_or("INBOX");
        let limit = config["limit"].as_u64().unwrap_or(10);
        let unread_only = config["unreadOnly"].as_bool().unwrap_or(true);

        tracing::info!("Reading emails from {}:{}/{}", imap_host, imap_port, folder);

        // IMAP connection
        let tls = native_tls::TlsConnector::builder()
            .build()
            .map_err(|e| FlowError::NodeExecutionFailed {
                node_id: node.id.clone(),
                detail: format!("TLS error: {}", e),
            })?;

        let client = imap::connect(
            (imap_host, imap_port),
            imap_host,
            &tls,
        ).map_err(|e| FlowError::NodeExecutionFailed {
            node_id: node.id.clone(),
            detail: format!("IMAP connect error: {}", e),
        })?;

        let mut session = client
            .login(username, password)
            .map_err(|e| FlowError::NodeExecutionFailed {
                node_id: node.id.clone(),
                detail: format!("IMAP login error: {}", e.0),
            })?;

        session.select(folder).map_err(|e| FlowError::NodeExecutionFailed {
            node_id: node.id.clone(),
            detail: format!("IMAP select error: {}", e),
        })?;

        // Search for messages
        let search_query = if unread_only { "UNSEEN" } else { "ALL" };
        let message_ids = session.search(search_query).map_err(|e| FlowError::NodeExecutionFailed {
            node_id: node.id.clone(),
            detail: format!("IMAP search error: {}", e),
        })?;

        let mut ids: Vec<u32> = message_ids.into_iter().collect();
        ids.sort_unstable();
        ids.reverse(); // newest first

        let fetch_ids: Vec<String> = ids.iter().take(limit as usize).map(|id| id.to_string()).collect();
        let fetch_range = fetch_ids.join(",");

        if fetch_range.is_empty() {
            session.logout().ok();
            let mut outputs = HashMap::new();
            outputs.insert("messages".to_string(), serde_json::json!([]));
            outputs.insert("count".to_string(), serde_json::json!(0));
            outputs.insert("success".to_string(), serde_json::json!(true));
            return Ok(outputs);
        }

        let messages = session.fetch(&fetch_range, "(ENVELOPE BODY[HEADER.FIELDS (FROM TO SUBJECT DATE)])")
            .map_err(|e| FlowError::NodeExecutionFailed {
                node_id: node.id.clone(),
                detail: format!("IMAP fetch error: {}", e),
            })?;

        let mut result_messages = Vec::new();
        for message in messages.iter() {
            let envelope = message.envelope();
            let headers = message.header();

            let subject = envelope
                .and_then(|e| e.subject.as_ref())
                .map(|s| String::from_utf8_lossy(s).to_string())
                .unwrap_or_default();

            let from = envelope
                .and_then(|e| e.from.as_ref())
                .and_then(|f| f.first())
                .map(|f| {
                    let name = f.name.as_ref().map(|n| String::from_utf8_lossy(n).to_string()).unwrap_or_default();
                    let mailbox = f.mailbox.as_ref().map(|m| String::from_utf8_lossy(m).to_string()).unwrap_or_default();
                    let host = f.host.as_ref().map(|h| String::from_utf8_lossy(h).to_string()).unwrap_or_default();
                    if name.is_empty() {
                        format!("{}@{}", mailbox, host)
                    } else {
                        format!("{} <{}@{}>", name, mailbox, host)
                    }
                })
                .unwrap_or_default();

            let date = envelope
                .and_then(|e| e.date.as_ref())
                .map(|d| String::from_utf8_lossy(d).to_string())
                .unwrap_or_default();

            let body_preview = headers
                .map(|h| String::from_utf8_lossy(h).to_string())
                .unwrap_or_default();

            result_messages.push(serde_json::json!({
                "subject": subject,
                "from": from,
                "date": date,
                "preview": body_preview,
                "id": message.message,
            }));
        }

        session.logout().ok();

        let mut outputs = HashMap::new();
        outputs.insert("messages".to_string(), serde_json::json!(result_messages));
        outputs.insert("count".to_string(), serde_json::json!(result_messages.len()));
        outputs.insert("success".to_string(), serde_json::json!(true));
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
            node_type: "email_read".to_string(),
            label: "Test Email Read".to_string(),
            config: serde_json::json!({}),
            position: Default::default(),
        }
    }

    #[tokio::test]
    async fn test_email_read_no_host() {
        let node = make_node("email_1");
        let ctx = NodeContext::empty();
        let config = serde_json::json!({});
        let inputs = HashMap::new();
        let result = EmailReadNode.execute(&node, &ctx, config, inputs).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_email_read_type_def() {
        let def = EmailReadNode.type_def();
        assert_eq!(def.type_name, "email_read");
        assert_eq!(def.outputs.len(), 3);
    }
}
