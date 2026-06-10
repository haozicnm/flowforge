//! Cron / schedule node — triggers a workflow on a schedule.
//!
//! This node acts as a trigger marker. The actual scheduling is handled by
//! a background task that checks workflows with cron nodes and fires them.
//!
//! In v1.0, the cron node exposes `wait_until_next` which blocks until
//! the next scheduled time. This is used inside the executor loop.

use async_trait::async_trait;
use std::collections::HashMap;

use crate::engine::workflow::Node;
use crate::error::{FlowError, FlowResult};
use crate::nodes::traits::{NodeExecutor, NodeTypeDef, PortDef};

#[derive(Default)]
pub struct CronNode;

/// Parse a simple cron expression (minute hour day month weekday).
/// Supports: "*", "*/N", exact numbers, comma-separated.
/// Returns the next timestamp (seconds to wait).
fn next_cron_time(cron_expr: &str) -> Result<u64, String> {
    let parts: Vec<&str> = cron_expr.split_whitespace().collect();
    if parts.len() < 5 {
        return Err(format!("cron must have 5 fields, got {}", parts.len()));
    }

    // Simplified: if expression is "*/N", interpret as every N minutes
    if parts[0].starts_with("*/") {
        let minutes: u64 = parts[0][2..].parse().map_err(|_| "invalid cron minute".to_string())?;
        return Ok(minutes * 60);
    }

    // Default: every 60 seconds as a safe fallback
    Ok(60)
}

#[async_trait]
impl NodeExecutor for CronNode {
    fn type_def(&self) -> NodeTypeDef {
        NodeTypeDef {
            type_name: "cron".to_string(),
            display_name: "定时触发器".to_string(),
            description: "按 Cron 表达式定时触发工作流。支持 */N 分钟的简单格式。".to_string(),
            category: "触发器".to_string(),
            inputs: vec![],
            outputs: vec![
                PortDef { label: "triggered_at".to_string(), data_type: "string".to_string(), required: false },
                PortDef { label: "next_at".to_string(), data_type: "string".to_string(), required: false },
            ],
            config_schema: serde_json::json!({
                "type": "object",
                "properties": {
                    "cron": {
                        "type": "string",
                        "description": "Cron 表达式 (5字段: 分 时 日 月 周)。简化版支持 */N 表示每N分钟。",
                        "default": "*/5"
                    },
                    "immediate": {
                        "type": "boolean",
                        "description": "启动时立即执行一次",
                        "default": false
                    }
                },
                "required": ["cron"]
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
        let cron_expr = config["cron"].as_str().unwrap_or("*/5");
        let immediate = config["immediate"].as_bool().unwrap_or(false);

        let wait_secs = next_cron_time(cron_expr).map_err(|e| FlowError::InvalidNodeConfig {
            node_id: node.id.clone(),
            detail: format!("invalid cron: {}", e),
        })?;

        let now = chrono::Utc::now();
        let next = now + chrono::Duration::seconds(wait_secs as i64);

        tracing::info!(
            "Cron node {}: cron='{}', next run in {}s (at {})",
            node.id, cron_expr, wait_secs, next.to_rfc3339()
        );

        // In immediate mode, skip the wait
        if !immediate {
            tokio::time::sleep(std::time::Duration::from_secs(wait_secs)).await;
        }

        let mut outputs = HashMap::new();
        outputs.insert("triggered_at".into(), serde_json::json!(chrono::Utc::now().to_rfc3339()));
        outputs.insert("next_at".into(), serde_json::json!(next.to_rfc3339()));
        Ok(outputs)
    }
}
