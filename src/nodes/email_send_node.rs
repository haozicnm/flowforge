//! Email Send node — sends emails via SMTP.
//!
//! Requires SMTP configuration in workflow or environment.

use async_trait::async_trait;
use std::collections::HashMap;

use crate::engine::workflow::Node;
use crate::error::{FlowError, FlowResult};
use crate::nodes::traits::{NodeExecutor, NodeTypeDef, PortDef};

#[derive(Default)]
pub struct EmailSendNode;

#[async_trait]
impl NodeExecutor for EmailSendNode {
    fn type_def(&self) -> NodeTypeDef {
        NodeTypeDef {
            type_name: "email_send".to_string(),
            display_name: "发送邮件".to_string(),
            description: "通过 SMTP 发送邮件".to_string(),
            category: "通信".to_string(),
            inputs: vec![
                PortDef { label: "to".to_string(), data_type: "string".to_string(), required: true },
                PortDef { label: "subject".to_string(), data_type: "string".to_string(), required: true },
                PortDef { label: "body".to_string(), data_type: "string".to_string(), required: true },
            ],
            outputs: vec![
                PortDef { label: "success".to_string(), data_type: "boolean".to_string(), required: false },
                PortDef { label: "error".to_string(), data_type: "string".to_string(), required: false },
            ],
            config_schema: serde_json::json!({
                "type": "object",
                "properties": {
                    "smtp_host": { "type": "string", "description": "SMTP server host" },
                    "smtp_port": { "type": "number", "default": 587, "description": "SMTP server port" },
                    "username": { "type": "string", "description": "SMTP username" },
                    "password": { "type": "string", "description": "SMTP password" },
                    "from": { "type": "string", "description": "Sender email address" },
                    "to": { "type": "string", "description": "Recipient email address" },
                    "subject": { "type": "string", "description": "Email subject" },
                    "body": { "type": "string", "description": "Email body" },
                    "html": { "type": "boolean", "default": false, "description": "Send as HTML" }
                },
                "required": ["smtp_host", "username", "password", "from"]
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
        let smtp_host = config["smtp_host"].as_str().ok_or_else(|| FlowError::InvalidNodeConfig {
            node_id: node.id.clone(),
            detail: "smtp_host is required".to_string(),
        })?;

        let smtp_port = config["smtp_port"].as_u64().unwrap_or(587) as u16;
        let username = config["username"].as_str().ok_or_else(|| FlowError::InvalidNodeConfig {
            node_id: node.id.clone(),
            detail: "username is required".to_string(),
        })?;
        let password = config["password"].as_str().ok_or_else(|| FlowError::InvalidNodeConfig {
            node_id: node.id.clone(),
            detail: "password is required".to_string(),
        })?;
        let from = config["from"].as_str().ok_or_else(|| FlowError::InvalidNodeConfig {
            node_id: node.id.clone(),
            detail: "from is required".to_string(),
        })?;

        let to = config["to"].as_str()
            .or_else(|| inputs.get("to").and_then(|v| v.as_str()))
            .ok_or_else(|| FlowError::InvalidNodeConfig {
                node_id: node.id.clone(),
                detail: "to is required".to_string(),
            })?;

        let subject = config["subject"].as_str()
            .or_else(|| inputs.get("subject").and_then(|v| v.as_str()))
            .unwrap_or("(no subject)");

        let body = config["body"].as_str()
            .or_else(|| inputs.get("body").and_then(|v| v.as_str()))
            .unwrap_or("");

        let is_html = config["html"].as_bool().unwrap_or(false);

        tracing::info!("Sending email to {}: {}", to, subject);

        // Build email
        let email = lettre::Message::builder()
            .from(from.parse().map_err(|e| FlowError::NodeExecutionFailed {
                node_id: node.id.clone(),
                detail: format!("invalid from address: {}", e),
            })?)
            .to(to.parse().map_err(|e| FlowError::NodeExecutionFailed {
                node_id: node.id.clone(),
                detail: format!("invalid to address: {}", e),
            })?)
            .subject(subject);

        let email = if is_html {
            email.header(lettre::message::header::ContentType::TEXT_HTML)
                .body(body.to_string())
        } else {
            email.body(body.to_string())
        }.map_err(|e| FlowError::NodeExecutionFailed {
            node_id: node.id.clone(),
            detail: format!("failed to build email: {}", e),
        })?;

        // Connect to SMTP
        let credentials = lettre::transport::smtp::authentication::Credentials::new(
            username.to_string(),
            password.to_string(),
        );

        let mailer = lettre::AsyncSmtpTransport::<lettre::Tokio1Executor>::starttls_relay(smtp_host)
            .map_err(|e| FlowError::NodeExecutionFailed {
                node_id: node.id.clone(),
                detail: format!("failed to connect to SMTP: {}", e),
            })?
            .port(smtp_port)
            .credentials(credentials)
            .build();

        // Send
        let mut outputs = HashMap::new();
        match lettre::AsyncTransport::send(&mailer, email).await {
            Ok(_) => {
                tracing::info!("Email sent to {}", to);
                outputs.insert("success".to_string(), serde_json::json!(true));
                outputs.insert("error".to_string(), serde_json::json!(""));
            }
            Err(e) => {
                tracing::error!("Failed to send email: {}", e);
                outputs.insert("success".to_string(), serde_json::json!(false));
                outputs.insert("error".to_string(), serde_json::json!(e.to_string()));
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
            node_type: "email_send".to_string(),
            label: "Test Email".to_string(),
            config: serde_json::json!({}),
            position: Default::default(),
        }
    }

    #[tokio::test]
    async fn test_email_no_smtp_host() {
        let node = make_node("email_1");
        let ctx = NodeContext::empty();
        let config = serde_json::json!({});
        let inputs = HashMap::new();
        let result = EmailSendNode.execute(&node, &ctx, config, inputs).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_email_no_to() {
        let node = make_node("email_1");
        let ctx = NodeContext::empty();
        let config = serde_json::json!({
            "smtp_host": "smtp.example.com",
            "username": "user",
            "password": "pass",
            "from": "from@example.com"
        });
        let inputs = HashMap::new();
        let result = EmailSendNode.execute(&node, &ctx, config, inputs).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_email_type_def() {
        let def = EmailSendNode.type_def();
        assert_eq!(def.type_name, "email_send");
        assert_eq!(def.outputs.len(), 2);
    }
}
