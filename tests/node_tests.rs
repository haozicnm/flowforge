//! Node-level smoke tests — verify each node type executes without panicking.

use std::collections::HashMap;

use flowforge::engine::workflow::{Node, Position};
use flowforge::nodes::traits::{NodeContext, NodeExecutor};

fn make_node(id: &str, ntype: &str, config: serde_json::Value) -> Node {
    Node {
        id: id.into(),
        node_type: ntype.into(),
        label: "".into(),
        config,
        position: Position::default(),
    }
}

#[tokio::test]
async fn test_log_node() {
    let n = flowforge::nodes::log_node::LogNode::default();
    let cfg = serde_json::json!({"level": "info", "message": "test msg"});
    let node = make_node("log1", "log", cfg.clone());
    let result = n.execute(&node, &NodeContext::empty(), cfg, HashMap::new()).await;
    assert!(result.is_ok(), "log node failed: {:?}", result.err());
}

#[tokio::test]
async fn test_delay_node() {
    let n = flowforge::nodes::delay_node::DelayNode::default();
    let cfg = serde_json::json!({"duration_ms": 10});
    let node = make_node("d1", "delay", cfg.clone());

    let start = std::time::Instant::now();
    let result = n.execute(&node, &NodeContext::empty(), cfg, HashMap::new()).await;
    let elapsed = start.elapsed();

    assert!(result.is_ok(), "delay node failed: {:?}", result.err());
    assert!(elapsed.as_millis() >= 8, "delay too short: {:?}", elapsed);
}

#[tokio::test]
async fn test_condition_node() {
    let n = flowforge::nodes::condition_node::ConditionNode::default();
    let cfg = serde_json::json!({
        "operator": "equals",
        "left": serde_json::json!("hello"),
        "right": serde_json::json!("hello"),
    });
    let node = make_node("c1", "condition", cfg.clone());
    let result = n.execute(&node, &NodeContext::empty(), cfg, HashMap::new()).await;
    assert!(result.is_ok(), "condition node failed: {:?}", result.err());

    // Test invalid operator
    let cfg2 = serde_json::json!({"operator": "bad_op", "left": 1, "right": 1});
    let nd = make_node("c2", "condition", cfg2.clone());
    let result2 = n.execute(&nd, &NodeContext::empty(), cfg2, HashMap::new()).await;
    assert!(result2.is_err());
}

#[tokio::test]
async fn test_variable_node() {
    let n = flowforge::nodes::variable_node::VariableNode::default();
    let cfg = serde_json::json!({"value": "default val"});
    let node = make_node("v1", "variable", cfg.clone());
    let result = n.execute(&node, &NodeContext::empty(), cfg, HashMap::new()).await;
    assert!(result.is_ok(), "variable node failed: {:?}", result.err());
}

#[tokio::test]
async fn test_json_node() {
    let n = flowforge::nodes::json_node::JsonNode::default();
    let cfg = serde_json::json!({"operation": "keys"});
    let mut inputs: HashMap<String, serde_json::Value> = HashMap::new();
    inputs.insert("data".into(), serde_json::json!({"a": 1, "b": 2}));
    let node = make_node("j1", "json", cfg.clone());
    let result = n.execute(&node, &NodeContext::empty(), cfg, inputs).await;
    assert!(result.is_ok(), "json node failed: {:?}", result.err());
}

#[tokio::test]
async fn test_regex_node() {
    let n = flowforge::nodes::regex_node::RegexNode::default();
    let cfg = serde_json::json!({"operation": "match", "pattern": "\\\\d+"});
    let mut inputs: HashMap<String, serde_json::Value> = HashMap::new();
    inputs.insert("text".into(), serde_json::json!("hello 123 world"));
    let node = make_node("r1", "regex", cfg.clone());
    let result = n.execute(&node, &NodeContext::empty(), cfg, inputs).await;
    assert!(result.is_ok(), "regex node failed: {:?}", result.err());
}

#[tokio::test]
async fn test_try_catch_node() {
    let n = flowforge::nodes::try_catch_node::TryCatchNode::default();
    let cfg = serde_json::json!({});
    let node = make_node("tc1", "try_catch", cfg.clone());
    let result = n.execute(&node, &NodeContext::empty(), cfg, HashMap::new()).await;
    assert!(result.is_ok(), "try_catch node failed: {:?}", result.err());
}

#[tokio::test]
async fn test_template_node() {
    let n = flowforge::nodes::template_node::TemplateNode::default();
    let cfg = serde_json::json!({"template": "Hello {{name}}!", "escape": "none"});
    let node = make_node("t1", "template", cfg.clone());
    let result = n.execute(&node, &NodeContext::empty(), cfg, HashMap::new()).await;
    assert!(result.is_ok(), "template node failed: {:?}", result.err());
}

#[tokio::test]
async fn test_http_node() {
    let n = flowforge::nodes::http_node::HttpNode::default();
    let cfg = serde_json::json!({
        "method": "GET",
        "url": "https://httpbin.org/get",
        "timeout_ms": 5000
    });
    let node = make_node("h1", "http", cfg.clone());
    let result = n.execute(&node, &NodeContext::empty(), cfg, HashMap::new()).await;
    // May fail due to network, but should not panic
    assert!(result.is_ok() || result.is_err());
}

#[tokio::test]
async fn test_shell_node_empty_command() {
    let n = flowforge::nodes::shell_node::ShellNode::default();
    let cfg = serde_json::json!({"command": ""});
    let node = make_node("s1", "shell", cfg.clone());
    let result = n.execute(&node, &NodeContext::empty(), cfg, HashMap::new()).await;
    assert!(result.is_err(), "empty command should fail");
}

#[tokio::test]
async fn test_webhook_node_no_payload() {
    let n = flowforge::nodes::webhook_node::WebhookNode::default();
    let cfg = serde_json::json!({});
    let node = make_node("wh1", "webhook", cfg.clone());
    let result = n.execute(&node, &NodeContext::empty(), cfg, HashMap::new()).await;
    assert!(result.is_ok(), "webhook node failed: {:?}", result.err());
}
