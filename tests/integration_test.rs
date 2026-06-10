//! Integration tests — full workflow execution, storage, and API handler coverage.

use std::collections::HashMap;
use std::sync::Arc;

use flowforge::engine::executor::Executor;
use flowforge::engine::resolver;
use flowforge::engine::storage::WorkflowStorage;
use flowforge::engine::workflow::{Edge, Node, Position, Workflow};
use flowforge::nodes::registry::NodeRegistry;

// ── Helper: build a minimal 2-node workflow ──────────────────────

fn make_workflow() -> Workflow {
    let mut wf = Workflow::new("test-workflow".into(), Some("test".into()));
    wf.nodes = vec![
        Node {
            id: "start".into(),
            node_type: "log".into(),
            label: "Start".into(),
            config: serde_json::json!({"level": "info", "message": "hello"}),
            position: Position::default(),
        },
        Node {
            id: "echo".into(),
            node_type: "log".into(),
            label: "Echo".into(),
            config: serde_json::json!({"level": "info", "message": "{{start.out}}"}),
            position: Position::default(),
        },
    ];
    wf.edges = vec![Edge {
        from: "start".into(),
        from_port: "out".into(),
        to: "echo".into(),
        to_port: "in".into(),
    }];
    wf
}

// ── Tests ────────────────────────────────────────────────────────

#[tokio::test]
async fn test_executor_simple_workflow() {
    let registry = Arc::new(NodeRegistry::new());
    let executor = Executor::new(registry);
    let wf = make_workflow();

    let result = executor.execute(&wf, None).await;
    assert!(result.is_ok(), "executor should succeed: {:?}", result.err());

    let state = result.unwrap();
    assert!(state.completed.contains(&"start".to_string()));
    assert!(state.completed.contains(&"echo".to_string()));
    assert!(state.failed.is_empty());

    // start node should have an out output
    let start_outputs = state.node_outputs.get("start").unwrap();
    assert_eq!(start_outputs.get("out").unwrap(), &serde_json::json!("hello"));
}

#[tokio::test]
async fn test_executor_cycle_detection() {
    let registry = Arc::new(NodeRegistry::new());
    let executor = Executor::new(registry);
    let mut wf = make_workflow();
    // Create a cycle: echo -> start
    wf.edges.push(Edge {
        from: "echo".into(),
        from_port: "out".into(),
        to: "start".into(),
        to_port: "in".into(),
    });

    let result = executor.execute(&wf, None).await;
    assert!(result.is_err());
    let err = result.err().unwrap().to_string();
    assert!(err.contains("cycle") || err.contains("Execution error"),
        "expected cycle error, got: {}", err);
}

#[tokio::test]
async fn test_executor_unknown_node_type() {
    let registry = Arc::new(NodeRegistry::new());
    let executor = Executor::new(registry);
    let mut wf = make_workflow();
    wf.nodes[0].node_type = "non_existent_type".into();

    let result = executor.execute(&wf, None).await;
    assert!(result.is_err());
    let err = result.err().unwrap().to_string();
    assert!(err.contains("Node type not found") || err.contains("non_existent"),
        "expected not-found error, got: {}", err);
}

#[tokio::test]
async fn test_resolver_simple_substitution() {
    let node = Node {
        id: "test".into(),
        node_type: "log".into(),
        label: "".into(),
        config: serde_json::json!({"message": "{{upstream.text}}"}),
        position: Position::default(),
    };

    let mut outputs = HashMap::new();
    outputs.insert("upstream.text".to_string(), serde_json::json!("hello world"));

    let resolved = resolver::resolve_node_config(&node, &outputs).unwrap();
    assert_eq!(resolved["message"], serde_json::json!("hello world"));
}

#[tokio::test]
async fn test_resolver_no_refs_passthrough() {
    let node = Node {
        id: "test".into(),
        node_type: "log".into(),
        label: "".into(),
        config: serde_json::json!({"level": "info", "timeout": 30}),
        position: Position::default(),
    };

    let outputs = HashMap::new();
    let resolved = resolver::resolve_node_config(&node, &outputs).unwrap();
    assert_eq!(resolved["level"], serde_json::json!("info"));
    assert_eq!(resolved["timeout"], serde_json::json!(30));
}

#[tokio::test]
async fn test_resolver_missing_ref() {
    let node = Node {
        id: "test".into(),
        node_type: "log".into(),
        label: "".into(),
        config: serde_json::json!({"message": "{{missing.text}}"}),
        position: Position::default(),
    };

    let outputs = HashMap::new();
    let result = resolver::resolve_node_config(&node, &outputs);
    assert!(result.is_err());
}

#[tokio::test]
async fn test_storage_save_and_load() {
    let tmp = std::env::temp_dir().join("flowforge_test_storage");
    let _ = std::fs::remove_dir_all(&tmp);
    let storage = WorkflowStorage::new(tmp.to_str().unwrap());
    storage.init().unwrap();

    let wf = make_workflow();
    storage.save(&wf).unwrap();

    let loaded = storage.load(&wf.id).unwrap();
    assert_eq!(loaded.name, wf.name);
    assert_eq!(loaded.nodes.len(), wf.nodes.len());

    let list = storage.list().unwrap();
    assert_eq!(list.len(), 1);

    storage.delete(&wf.id).unwrap();
    let list = storage.list().unwrap();
    assert_eq!(list.len(), 0);

    let _ = std::fs::remove_dir_all(&tmp);
}

#[tokio::test]
async fn test_storage_list_and_delete() {
    let tmp = std::env::temp_dir().join("flowforge_test_list");
    let _ = std::fs::remove_dir_all(&tmp);
    let storage = WorkflowStorage::new(tmp.to_str().unwrap());
    storage.init().unwrap();

    let wf1 = Workflow::new("wf1".into(), None);
    let wf2 = Workflow::new("wf2".into(), None);
    storage.save(&wf1).unwrap();
    storage.save(&wf2).unwrap();

    let list = storage.list().unwrap();
    assert_eq!(list.len(), 2);

    storage.delete(&wf1.id).unwrap();
    let list = storage.list().unwrap();
    assert_eq!(list.len(), 1);
    assert_eq!(list[0].id, wf2.id);

    let _ = std::fs::remove_dir_all(&tmp);
}
