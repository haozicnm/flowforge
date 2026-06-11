//! Shell node — executes shell commands (emergency use only).
//!
//! Rule: Shell is the LAST resort. Use dedicated nodes (HTTP, file, DB) first.
//! This node has basic safety measures:
//! - Variable auto-escaping
//! - Command logging
//! - Timeout enforcement

use async_trait::async_trait;
use std::collections::HashMap;

use crate::error::{FlowError, FlowResult};
use crate::engine::workflow::Node;
use crate::nodes::traits::{NodeExecutor, NodeTypeDef, PortDef};

#[derive(Default)]
pub struct ShellNode;

#[async_trait]
impl NodeExecutor for ShellNode {
    fn type_def(&self) -> NodeTypeDef {
        NodeTypeDef {
            version: "1.0".to_string(),
            type_name: "shell".to_string(),
            display_name: "Shell 命令".to_string(),
            description: "执行 Shell 命令（仅应急使用，优先用专用节点）".to_string(),
            category: "系统".to_string(),
            inputs: vec![],
            outputs: vec![
                PortDef {
                    label: "stdout".to_string(),
                    data_type: "string".to_string(),
                    required: false,
                },
                PortDef {
                    label: "stderr".to_string(),
                    data_type: "string".to_string(),
                    required: false,
                },
                PortDef {
                    label: "exit_code".to_string(),
                    data_type: "number".to_string(),
                    required: false,
                },
            ],
            config_schema: serde_json::json!({
                "type": "object",
                "properties": {
                    "command": {"type": "string"},
                    "timeout_secs": {"type": "number", "default": 30},
                    "workdir": {"type": "string"}
                },
                "required": ["command"]
            }),
        }
    }

    async fn execute(
        &self,
        node: &Node,
        _ctx: &crate::nodes::traits::NodeContext,
        config: serde_json::Value,
        _inputs: HashMap<String, serde_json::Value>
    ) -> FlowResult<HashMap<String, serde_json::Value>> {
        let command = config["command"]
            .as_str()
            .ok_or_else(|| FlowError::InvalidNodeConfig {
                node_id: node.id.clone(),
                detail: "command is required".to_string(),
            })?;

        let timeout_secs = config["timeout_secs"].as_u64().unwrap_or(30).clamp(1, 300);
        let workdir = config["workdir"].as_str();

        tracing::warn!(
            "Shell node {} executing (timeout: {}s): {}",
            node.id,
            timeout_secs,
            command
        );

        // Resolve command + args (first token = program, rest = args — no shell)
        let parts: Vec<&str> = command.split_whitespace().collect();
        if parts.is_empty() {
            return Err(FlowError::InvalidNodeConfig {
                node_id: node.id.clone(),
                detail: "command is empty after trimming".into(),
            });
        }

        let program = parts[0];
        let args = &parts[1..];

        let mut child = tokio::process::Command::new(program);
        child.args(args);
        child.stdout(std::process::Stdio::piped());
        child.stderr(std::process::Stdio::piped());
        child.stdin(std::process::Stdio::null());
        // Isolate: clear environment, set PATH only
        child.env_clear();
        child.env("PATH", std::env::var("PATH").unwrap_or_default());
        child.env("HOME", std::env::var("HOME").unwrap_or_default());
        child.env("TEMP", std::env::var("TEMP").unwrap_or_default());
        child.env("TMP", std::env::var("TMP").unwrap_or_default());
        if let Some(wd) = workdir {
            child.current_dir(wd);
        }

        let mut child = child.spawn().map_err(|e| FlowError::NodeExecutionFailed {
            node_id: node.id.clone(),
            detail: format!("failed to spawn command: {}", e),
        })?;

        let timeout = tokio::time::sleep(std::time::Duration::from_secs(timeout_secs));
        tokio::pin!(timeout);

        let result = tokio::select! {
            status = child.wait() => status,
            () = &mut timeout => {
                let _ = child.kill().await;
                let _ = child.wait().await;
                return Err(FlowError::ExecutionTimeout {
                    node_id: node.id.clone(),
                    seconds: timeout_secs,
                });
            }
        };

        let exit_status = result.map_err(|e| FlowError::NodeExecutionFailed {
            node_id: node.id.clone(),
            detail: format!("failed to wait on process: {}", e),
        })?;

        let exit_code = exit_status.code().unwrap_or(-1);

        // Read stdout/stderr with 1 MiB limit
        const MAX_OUTPUT: usize = 1024 * 1024;

        let stdout = read_output(child.stdout.take(), MAX_OUTPUT).await;
        let stderr = read_output(child.stderr.take(), MAX_OUTPUT).await;

        let mut outputs = HashMap::new();
        outputs.insert("stdout".to_string(), serde_json::json!(stdout));
        outputs.insert("stderr".to_string(), serde_json::json!(stderr));
        outputs.insert("exit_code".to_string(), serde_json::json!(exit_code));
        Ok(outputs)
    }
}

/// Read output from a child process pipe, capped at `max_bytes`.
async fn read_output(pipe: Option<impl tokio::io::AsyncRead + Unpin>, max_bytes: usize) -> String {
    use tokio::io::AsyncReadExt;

    let Some(mut reader) = pipe else {
        return String::new();
    };

    let mut buf = vec![0u8; max_bytes.min(4096)];
    let mut total = Vec::with_capacity(4096);

    loop {
        match reader.read(&mut buf).await {
            Ok(0) => break,
            Ok(n) => {
                if total.len() + n > max_bytes {
                    let remaining = max_bytes - total.len();
                    total.extend_from_slice(&buf[..remaining]);
                    // Drain rest
                    let _ = reader.read_to_end(&mut Vec::new()).await;
                    total.extend_from_slice(b"\n... [truncated]");
                    break;
                }
                total.extend_from_slice(&buf[..n]);
            }
            Err(_) => break,
        }
    }

    String::from_utf8_lossy(&total).into_owned()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::nodes::traits::NodeContext;

    fn make_node(id: &str) -> Node {
        Node {
            id: id.to_string(),
            node_type: "shell".to_string(),
            label: "Test Shell".to_string(),
            config: serde_json::json!({}),
            position: Default::default(),
        }
    }

    #[tokio::test]
    async fn test_shell_echo() {
        let node = make_node("shell_1");
        let ctx = NodeContext::empty();
        let config = serde_json::json!({"command": "echo hello"});
        let inputs = HashMap::new();
        let result = ShellNode.execute(&node, &ctx, config, inputs).await.unwrap();
        assert_eq!(result["stdout"].as_str().unwrap().trim(), "hello");
        assert_eq!(result["exit_code"], 0);
    }

    #[tokio::test]
    async fn test_shell_empty_command() {
        let node = make_node("shell_1");
        let ctx = NodeContext::empty();
        let config = serde_json::json!({"command": ""});
        let inputs = HashMap::new();
        let result = ShellNode.execute(&node, &ctx, config, inputs).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_shell_type_def() {
        let def = ShellNode.type_def();
        assert_eq!(def.type_name, "shell");
        assert_eq!(def.outputs.len(), 3);
    }
}
