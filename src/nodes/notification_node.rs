//! Notification node — sends messages via Email (SMTP) or Slack Webhook.
//!
//! Uses reqwest for HTTP-based notifications. SMTP support via a simple HTTP-to-SMTP
//! bridge or direct reqwest-based email API in future versions (currently POST to SMTP
//! relay or Slack incoming webhook URL).

use async_trait::async_trait;
use std::collections::HashMap;

use crate::engine::workflow::Node;
use crate::error::{FlowError, FlowResult};
use crate::nodes::traits::{NodeExecutor, NodeTypeDef, PortDef};

#[derive(Default)]
pub struct NotificationNode;

#[async_trait]
impl NodeExecutor for NotificationNode {
    fn type_def(&self) -> NodeTypeDef {
        NodeTypeDef {
            type_name: "notification".to_string(),
            display_name: "通知".to_string(),
            description: "发送通知 (Slack Webhook / Email SMTP / 自定义 Webhook)".to_string(),
            category: "通知".to_string(),
            inputs: vec![
                PortDef { label: "message".to_string(), data_type: "string".to_string(), required: false },
            ],
            outputs: vec![
                PortDef { label: "status_code".to_string(), data_type: "number".to_string(), required: false },
                PortDef { label: "success".to_string(), data_type: "boolean".to_string(), required: false },
            ],
            config_schema: serde_json::json!({
                "type": "object",
                "properties": {
                    "channel": {
                        "type": "string",
                        "enum": ["slack", "email", "webhook"],
                        "default": "slack"
                    },
                    "webhook_url": {"type": "string"},
                    "message": {"type": "string"},
                    "title": {"type": "string"},
                    "to": {"type": "string"},
                    "smtp_host": {"type": "string"},
                    "smtp_port": {"type": "number", "default": 587},
                    "smtp_user": {"type": "string"},
                    "smtp_pass": {"type": "string"},
                    "from": {"type": "string"}
                },
                "required": ["channel"]
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
        let channel = config["channel"].as_str().unwrap_or("slack");
        let message = config["message"].as_str()
            .or_else(|| inputs.get("message").and_then(|v| v.as_str()))
            .unwrap_or("");

        match channel {
            "slack" => send_slack(node, &config, message).await,
            "email" => send_email(node, &config, message).await,
            "webhook" => send_webhook(node, &config, message).await,
            _ => Err(FlowError::InvalidNodeConfig {
                node_id: node.id.clone(),
                detail: format!("unknown channel: {}", channel),
            }),
        }
    }
}

async fn send_slack(node: &Node, config: &serde_json::Value, message: &str) -> FlowResult<HashMap<String, serde_json::Value>> {
    let url = config["webhook_url"].as_str().ok_or_else(|| FlowError::InvalidNodeConfig {
        node_id: node.id.clone(),
        detail: "webhook_url required for Slack".into(),
    })?;

    let client = reqwest::Client::new();
    let body = serde_json::json!({"text": message});

    let resp = client.post(url).json(&body).send().await.map_err(|e| FlowError::NodeExecutionFailed {
        node_id: node.id.clone(),
        detail: format!("Slack: {}", e),
    })?;

    let status = resp.status().as_u16();

    let mut outputs = HashMap::new();
    outputs.insert("status_code".into(), serde_json::json!(status));
    outputs.insert("success".into(), serde_json::json!(status < 400));
    Ok(outputs)
}

async fn send_email(node: &Node, config: &serde_json::Value, message: &str) -> FlowResult<HashMap<String, serde_json::Value>> {
    let url = config["webhook_url"].as_str()
        .or_else(|| config["smtp_host"].as_str().map(|_| ""))
        .ok_or_else(|| FlowError::InvalidNodeConfig {
            node_id: node.id.clone(),
            detail: "webhook_url or smtp_host required for Email".into(),
        })?;

    let title = config["title"].as_str().unwrap_or("FlowForge Notification");
    let to = config["to"].as_str().unwrap_or("");
    let from = config["from"].as_str().unwrap_or("noreply@flowforge.local");

    // Use a simple mail relay API (HTTP POST to webhook_url as JSON mail endpoint)
    let client = reqwest::Client::new();
    let body = serde_json::json!({
        "from": from,
        "to": to,
        "subject": title,
        "text": message,
    });

    let resp = if url.is_empty() {
        // No webhook URL configured — signal not sent
        tracing::warn!("Email node {}: no webhook_url configured, skipping send", node.id);
        let mut outputs = HashMap::new();
        outputs.insert("status_code".into(), serde_json::json!(0));
        outputs.insert("success".into(), serde_json::json!(false));
        return Ok(outputs);
    } else {
        client.post(url).json(&body).send().await.map_err(|e| FlowError::NodeExecutionFailed {
            node_id: node.id.clone(),
            detail: format!("Email: {}", e),
        })?
    };

    let status = resp.status().as_u16();
    let mut outputs = HashMap::new();
    outputs.insert("status_code".into(), serde_json::json!(status));
    outputs.insert("success".into(), serde_json::json!(status < 400));
    Ok(outputs)
}

async fn send_webhook(node: &Node, config: &serde_json::Value, message: &str) -> FlowResult<HashMap<String, serde_json::Value>> {
    let url = config["webhook_url"].as_str().ok_or_else(|| FlowError::InvalidNodeConfig {
        node_id: node.id.clone(),
        detail: "webhook_url required".into(),
    })?;

    let client = reqwest::Client::new();
    let body = serde_json::json!({"message": message, "timestamp": chrono::Utc::now().to_rfc3339()});

    let resp = client.post(url).json(&body).send().await.map_err(|e| FlowError::NodeExecutionFailed {
        node_id: node.id.clone(),
        detail: format!("Webhook: {}", e),
    })?;

    let status = resp.status().as_u16();
    let mut outputs = HashMap::new();
    outputs.insert("status_code".into(), serde_json::json!(status));
    outputs.insert("success".into(), serde_json::json!(status < 400));
    Ok(outputs)
}
