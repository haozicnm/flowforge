//! Workflow executor — runs workflows topologically.
//!
//! Key rules from lessons-learned:
//! 1. All variable resolution goes through resolver.rs (single layer)
//! 2. Nodes receive already-resolved config
//! 3. Execution state is explicit (not global singletons)
//! 4. Each node gets its own execution context
//!
//! v2 improvements:
//! - Config validation before execution (validate_config)
//! - Parallel execution of independent nodes (same in-degree batch)
/// Type alias for webhook store (complex Arc<Mutex<HashMap<...>>>)
type WebhookStore = Arc<std::sync::Mutex<std::collections::HashMap<String, Vec<serde_json::Value>>>>;

use std::collections::{HashMap, HashSet};
use std::sync::Arc;

use tokio::sync::RwLock;

use crate::engine::resolver;
use crate::engine::workflow::{Edge, Node, Variable, Workflow};
use crate::error::{FlowError, FlowResult};
use crate::nodes::registry::NodeRegistry;
use crate::nodes::traits::NodeContext;
use crate::webbridge::WebBridgeState;

/// Execution state for a single workflow run.
#[derive(Debug, Clone)]
pub struct ExecutionState {
    /// Outputs from each node (node_id → port_label → value).
    pub node_outputs: HashMap<String, HashMap<String, serde_json::Value>>,

    /// Which nodes have completed.
    pub completed: Vec<String>,

    /// Which nodes failed.
    pub failed: Vec<String>,

    /// Currently executing nodes.
    pub running: Vec<String>,
}

impl Default for ExecutionState {
    fn default() -> Self {
        Self::new()
    }
}

impl ExecutionState {
    pub fn new() -> Self {
        Self {
            node_outputs: HashMap::new(),
            completed: Vec::new(),
            failed: Vec::new(),
            running: Vec::new(),
        }
    }

    /// Get all available outputs as a flat map (for the resolver).
    /// Key format: "nodeId.portLabel" → value.
    pub fn flat_outputs(&self) -> HashMap<String, serde_json::Value> {
        let mut flat = HashMap::new();
        for (node_id, ports) in &self.node_outputs {
            for (port_label, value) in ports {
                flat.insert(format!("{}.{}", node_id, port_label), value.clone());
            }
        }
        flat
    }
}

/// Execution event for real-time UI updates.
#[derive(Debug, Clone)]
pub enum ExecutionEvent {
    NodeStarted { _node_id: String },
    NodeCompleted { _node_id: String, _outputs: HashMap<String, serde_json::Value>, _duration_ms: u64 },
    NodeFailed { _node_id: String, _error: String, _duration_ms: u64 },
    WorkflowCompleted,
    _WorkflowFailed { _error: String },
}

/// Workflow executor.
pub struct Executor {
    registry: Arc<NodeRegistry>,
    webbridge: Option<WebBridgeState>,
    webhook_store: Option<WebhookStore>,
}

impl Executor {
    pub fn new(registry: Arc<NodeRegistry>) -> Self {
        Self { registry, webbridge: None, webhook_store: None }
    }

    pub fn with_webbridge(mut self, webbridge: WebBridgeState) -> Self {
        self.webbridge = Some(webbridge);
        self
    }

    pub fn with_webhook_store(mut self, store: Arc<std::sync::Mutex<std::collections::HashMap<String, Vec<serde_json::Value>>>>) -> Self {
        self.webhook_store = Some(store);
        self
    }

    /// Execute a workflow.
    ///
    /// Nodes are grouped by topological level (same in-degree batch) and
    /// independent nodes in the same level are executed in parallel.
    pub async fn execute(
        &self,
        workflow: &Workflow,
        event_tx: Option<tokio::sync::mpsc::Sender<ExecutionEvent>>,
    ) -> FlowResult<ExecutionState> {
        let nodes = workflow.nodes();
        let edges = workflow.edges();

        // Build levels for parallel execution
        let levels = self.topological_levels(nodes, edges)?;

        // Validate: all variable references point to valid nodes
        self.validate_references(nodes)?;

        let state = Arc::new(RwLock::new(ExecutionState::new()));

        // Execute level by level — nodes in the same level run in parallel
        for level in &levels {
            if level.len() == 1 {
                // Single node — no need for parallelism overhead
                self.execute_single_node(&level[0], nodes, edges, &workflow.variables, &state, &event_tx).await?;
            } else {
                // Multiple independent nodes — run in parallel
                self.execute_parallel_nodes(level, nodes, edges, &workflow.variables, &state, &event_tx).await?;
            }
        }

        self.send_event(&event_tx, ExecutionEvent::WorkflowCompleted).await;
        Arc::try_unwrap(state)
            .map(|s| s.into_inner())
            .map_err(|_| FlowError::ExecutionError(
                "execution state Arc still has outstanding references".into()
            ))
    }

    /// Execute one topological level (single-step mode).
    ///
    /// Takes the workflow and a partially-filled ExecutionState from previous steps.
    /// Returns the updated state, which nodes were executed this step, and whether
    /// there are more levels to execute.
    pub async fn execute_step(
        &self,
        workflow: &Workflow,
        state: ExecutionState,
    ) -> FlowResult<(ExecutionState, Vec<String>, bool)> {
        let nodes = workflow.nodes();
        let edges = workflow.edges();

        let levels = self.topological_levels(nodes, edges)?;
        self.validate_references(nodes)?;

        // Find the next level that hasn't been fully executed
        for level in &levels {
            let all_done = level.iter().all(|id| state.completed.contains(id) || state.failed.contains(id));
            if all_done {
                continue; // this level is already done
            }

            // Execute this level
            let state_arc = Arc::new(RwLock::new(state));
            if level.len() == 1 {
                self.execute_single_node(&level[0], nodes, edges, &workflow.variables, &state_arc, &None).await?;
            } else {
                self.execute_parallel_nodes(level, nodes, edges, &workflow.variables, &state_arc, &None).await?;
            }

            let state = Arc::try_unwrap(state_arc)
                .map(|s| s.into_inner())
                .map_err(|_| FlowError::ExecutionError("state Arc still referenced".into()))?;

            let executed = level.clone();
            // Check if there are more levels
            let mut has_more = false;
            for future_level in &levels {
                let all_done = future_level.iter().all(|id| state.completed.contains(id) || state.failed.contains(id));
                if !all_done {
                    has_more = true;
                    break;
                }
            }

            return Ok((state, executed, has_more));
        }

        // All levels already executed
        Ok((state, vec![], false))
    }

    /// Get the execution plan (topological levels) without executing.
    #[allow(dead_code)]
    pub fn get_execution_plan(&self, workflow: &Workflow) -> FlowResult<Vec<Vec<String>>> {
        self.topological_levels(workflow.nodes(), workflow.edges())
    }

    /// Execute a single node: resolve → validate → execute.
    async fn execute_single_node(
        &self,
        node_id: &str,
        nodes: &[Node],
        edges: &[Edge],
        workflow_vars: &[Variable],
        state: &Arc<RwLock<ExecutionState>>,
        event_tx: &Option<tokio::sync::mpsc::Sender<ExecutionEvent>>,
    ) -> FlowResult<()> {
        let node = nodes
            .iter()
            .find(|n| n.id == node_id)
            .ok_or_else(|| FlowError::NodeNotFound(node_id.to_string()))?;

        // Mark as running
        {
            let mut s = state.write().await;
            s.running.push(node_id.to_string());
        }
        self.send_event(event_tx, ExecutionEvent::NodeStarted { _node_id: node_id.to_string() }).await;

        // Resolve variables in config
        let step_outputs = state.read().await.flat_outputs();
        let resolved_config = resolver::resolve_node_config_with_context(
            node, &step_outputs, workflow_vars, true,
        )?;

        // Validate config
        let executor = self.registry.get_executor(&node.node_type)?;
        if let Err(errors) = executor.validate_config(&resolved_config) {
            let msg = errors.iter()
                .map(|e| format!("{}: {}", e.field, e.message))
                .collect::<Vec<_>>()
                .join("; ");

            let mut s = state.write().await;
            s.failed.push(node_id.to_string());
            s.running.retain(|id| id != node_id);

            self.send_event(event_tx, ExecutionEvent::NodeFailed {
                _node_id: node_id.to_string(),
                _error: format!("validation failed: {}", msg),
                _duration_ms: 0,
            }).await;

            return Err(FlowError::ExecutionError(format!(
                "Node '{}' config validation failed: {}", node_id, msg
            )));
        }

        // Collect inputs from upstream nodes, then drop the read lock
        let inputs = {
            let state_guard = state.read().await;
            self.collect_inputs(node_id, edges, &state_guard)
        };

        // Execute the node with timing
        let start = std::time::Instant::now();
        let mut ctx = match &self.webbridge {
            Some(wb) => NodeContext::with_webbridge(wb.clone()),
            None => NodeContext::empty(),
        };
        ctx.node_registry = Some(self.registry.clone());
        ctx.webhook_store = self.webhook_store.clone();

        match executor.execute(node, &ctx, resolved_config, inputs).await {
            Ok(outputs) => {
                let duration_ms = start.elapsed().as_millis() as u64;
                let mut s = state.write().await;
                s.node_outputs.insert(node_id.to_string(), outputs.clone());
                s.completed.push(node_id.to_string());
                s.running.retain(|id| id != node_id);

                self.send_event(event_tx, ExecutionEvent::NodeCompleted {
                    _node_id: node_id.to_string(),
                    _outputs: outputs,
                    _duration_ms: duration_ms,
                }).await;
            }
            Err(e) => {
                let duration_ms = start.elapsed().as_millis() as u64;
                let mut s = state.write().await;
                s.failed.push(node_id.to_string());
                s.running.retain(|id| id != node_id);

                self.send_event(event_tx, ExecutionEvent::NodeFailed {
                    _node_id: node_id.to_string(),
                    _error: e.to_string(),
                    _duration_ms: duration_ms,
                }).await;

                return Err(e);
            }
        }

        Ok(())
    }

    /// Execute multiple independent nodes in parallel.
    async fn execute_parallel_nodes(
        &self,
        node_ids: &[String],
        nodes: &[Node],
        edges: &[Edge],
        workflow_vars: &[Variable],
        state: &Arc<RwLock<ExecutionState>>,
        event_tx: &Option<tokio::sync::mpsc::Sender<ExecutionEvent>>,
    ) -> FlowResult<()> {
        // Collect inputs for all nodes first (while state is readable)
        #[allow(clippy::type_complexity)]
        let mut node_tasks: Vec<(Node, HashMap<String, serde_json::Value>, Arc<dyn crate::nodes::traits::NodeExecutor>, serde_json::Value)> = Vec::new();

        {
            let state_guard = state.read().await;
            let step_outputs = state_guard.flat_outputs();

            for node_id in node_ids {
                let node = nodes.iter().find(|n| &n.id == node_id)
                    .ok_or_else(|| FlowError::NodeNotFound(node_id.clone()))?;

                let resolved_config = resolver::resolve_node_config_with_context(node, &step_outputs, workflow_vars, true)?;
                let executor = self.registry.get_executor(&node.node_type)?;

                // Validate config before queuing
                if let Err(errors) = executor.validate_config(&resolved_config) {
                    let msg = errors.iter()
                        .map(|e| format!("{}: {}", e.field, e.message))
                        .collect::<Vec<_>>()
                        .join("; ");
                    return Err(FlowError::ExecutionError(format!(
                        "Node '{}' config validation failed: {}", node_id, msg
                    )));
                }

                let inputs = self.collect_inputs(node_id, edges, &state_guard);
                node_tasks.push((node.clone(), inputs, executor, resolved_config));
            }
        }

        // Mark all as running
        {
            let mut s = state.write().await;
            for node_id in node_ids {
                s.running.push(node_id.clone());
            }
        }
        for node_id in node_ids {
            self.send_event(event_tx, ExecutionEvent::NodeStarted { _node_id: node_id.clone() }).await;
        }

        // Build context
        let mut ctx = match &self.webbridge {
            Some(wb) => NodeContext::with_webbridge(wb.clone()),
            None => NodeContext::empty(),
        };
        ctx.node_registry = Some(self.registry.clone());
        ctx.webhook_store = self.webhook_store.clone();

        // Execute all in parallel using JoinSet
        let mut join_set = tokio::task::JoinSet::new();

        for (node, inputs, executor, resolved_config) in node_tasks {
            let ctx_clone = ctx.clone();
            join_set.spawn(async move {
                let node_id = node.id.clone();
                let start = std::time::Instant::now();
                let result = executor.execute(&node, &ctx_clone, resolved_config, inputs).await;
                let duration_ms = start.elapsed().as_millis() as u64;
                (node_id, result, duration_ms)
            });
        }

        // Collect results
        let mut any_error: Option<(String, FlowError)> = None;
        while let Some(result) = join_set.join_next().await {
            match result {
                Ok((node_id, Ok(outputs), duration_ms)) => {
                    let mut s = state.write().await;
                    s.node_outputs.insert(node_id.clone(), outputs.clone());
                    s.completed.push(node_id.clone());
                    s.running.retain(|id| *id != node_id);
                    self.send_event(event_tx, ExecutionEvent::NodeCompleted {
                        _node_id: node_id,
                        _outputs: outputs,
                        _duration_ms: duration_ms,
                    }).await;
                }
                Ok((node_id, Err(e), duration_ms)) => {
                    let mut s = state.write().await;
                    s.failed.push(node_id.clone());
                    s.running.retain(|id| *id != node_id);
                    self.send_event(event_tx, ExecutionEvent::NodeFailed {
                        _node_id: node_id.clone(),
                        _error: e.to_string(),
                        _duration_ms: duration_ms,
                    }).await;
                    any_error = Some((node_id, e));
                }
                Err(join_err) => {
                    return Err(FlowError::ExecutionError(format!(
                        "Parallel task panicked: {}", join_err
                    )));
                }
            }
        }

        if let Some((node_id, e)) = any_error {
            return Err(FlowError::ExecutionError(format!(
                "Node '{}' failed in parallel batch: {}", node_id, e
            )));
        }

        Ok(())
    }

    /// Topological sort — returns execution order (used by tests).
    #[allow(dead_code)]
    pub fn topological_sort(
        &self,
        nodes: &[Node],
        edges: &[Edge],
    ) -> FlowResult<Vec<String>> {
        let levels = self.topological_levels(nodes, edges)?;
        Ok(levels.into_iter().flatten().collect())
    }

    /// Topological level grouping — nodes in the same level have no dependencies
    /// between them and can be executed in parallel.
    fn topological_levels(
        &self,
        nodes: &[Node],
        edges: &[Edge],
    ) -> FlowResult<Vec<Vec<String>>> {
        let mut in_degree: HashMap<String, usize> = HashMap::new();
        let mut adj: HashMap<String, Vec<String>> = HashMap::new();

        for node in nodes {
            in_degree.entry(node.id.clone()).or_insert(0);
            adj.entry(node.id.clone()).or_default();
        }

        for edge in edges {
            *in_degree.entry(edge.to.clone()).or_insert(0) += 1;
            adj.entry(edge.from.clone())
                .or_default()
                .push(edge.to.clone());
        }

        // BFS-based level grouping (Kahn's algorithm)
        let mut current_level: Vec<String> = in_degree
            .iter()
            .filter(|(_, &deg)| deg == 0)
            .map(|(id, _)| id.clone())
            .collect();

        let mut levels = Vec::new();
        let mut visited = HashSet::new();

        while !current_level.is_empty() {
            let mut next_level = Vec::new();

            for node_id in &current_level {
                visited.insert(node_id.clone());
                if let Some(neighbors) = adj.get(node_id) {
                    for neighbor in neighbors {
                        if let Some(deg) = in_degree.get_mut(neighbor) {
                            *deg -= 1;
                            if *deg == 0 {
                                next_level.push(neighbor.clone());
                            }
                        }
                    }
                }
            }

            levels.push(current_level);
            current_level = next_level;
        }

        if visited.len() != nodes.len() {
            return Err(FlowError::ExecutionError(
                "Workflow contains a cycle".to_string(),
            ));
        }

        Ok(levels)
    }

    /// Validate that all variable references point to valid nodes.
    fn validate_references(&self, nodes: &[Node]) -> FlowResult<()> {
        let node_ids: std::collections::HashSet<&str> =
            nodes.iter().map(|n| n.id.as_str()).collect();

        for node in nodes {
            let refs = resolver::extract_refs(&node.config);
            for r#ref in refs {
                if !node_ids.contains(r#ref.step_id.as_str()) {
                    return Err(FlowError::VariableNotFound {
                        node_id: node.id.clone(),
                        var_ref: format!("{}.{}", r#ref.step_id, r#ref.port_label),
                    });
                }
            }
        }

        Ok(())
    }

    /// Collect inputs from upstream nodes.
    fn collect_inputs(
        &self,
        node_id: &str,
        edges: &[Edge],
        state: &ExecutionState,
    ) -> HashMap<String, serde_json::Value> {
        let mut inputs = HashMap::new();

        for edge in edges {
            if edge.to == node_id {
                if let Some(outputs) = state.node_outputs.get(&edge.from) {
                    let value = outputs.get(&edge.from_port).cloned();
                    if let Some(v) = value {
                        inputs.insert(edge.to_port.clone(), v);
                    }
                }
            }
        }

        inputs
    }

    async fn send_event(
        &self,
        tx: &Option<tokio::sync::mpsc::Sender<ExecutionEvent>>,
        event: ExecutionEvent,
    ) {
        if let Some(tx) = tx {
            let _ = tx.send(event).await;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::engine::workflow::{Edge, Node, Workflow};
    use crate::nodes::registry::NodeRegistry;
    use chrono::Utc;

    fn make_node(id: &str, node_type: &str) -> Node {
        Node {
            id: id.to_string(),
            node_type: node_type.to_string(),
            label: id.to_string(),
            config: serde_json::json!({}),
            position: crate::engine::workflow::Position { x: 0.0, y: 0.0 },
        }
    }

    fn make_edge(from: &str, to: &str) -> Edge {
        Edge {
            from: from.to_string(),
            from_port: "out".to_string(),
            to: to.to_string(),
            to_port: "in".to_string(),
        }
    }

    #[tokio::test]
    async fn test_topological_levels_single_chain() {
        let registry = Arc::new(NodeRegistry::new());
        let executor = Executor::new(registry);

        let nodes = vec![
            make_node("a", "log"),
            make_node("b", "log"),
            make_node("c", "log"),
        ];
        let edges = vec![make_edge("a", "b"), make_edge("b", "c")];

        let levels = executor.topological_levels(&nodes, &edges).unwrap();
        assert_eq!(levels.len(), 3); // 3 levels for a chain
        assert_eq!(levels[0], vec!["a"]);
        assert_eq!(levels[1], vec!["b"]);
        assert_eq!(levels[2], vec!["c"]);
    }

    #[tokio::test]
    async fn test_topological_levels_parallel() {
        let registry = Arc::new(NodeRegistry::new());
        let executor = Executor::new(registry);

        // a → b, a → c (b and c are independent, should be in same level)
        let nodes = vec![
            make_node("a", "log"),
            make_node("b", "log"),
            make_node("c", "log"),
        ];
        let edges = vec![make_edge("a", "b"), make_edge("a", "c")];

        let levels = executor.topological_levels(&nodes, &edges).unwrap();
        assert_eq!(levels.len(), 2); // 2 levels
        assert_eq!(levels[0], vec!["a"]);
        // b and c should be in the same level (order may vary)
        assert_eq!(levels[1].len(), 2);
        assert!(levels[1].contains(&"b".to_string()));
        assert!(levels[1].contains(&"c".to_string()));
    }

    #[tokio::test]
    async fn test_topological_levels_diamond() {
        let registry = Arc::new(NodeRegistry::new());
        let executor = Executor::new(registry);

        // Diamond: a → b, a → c, b → d, c → d
        let nodes = vec![
            make_node("a", "log"),
            make_node("b", "log"),
            make_node("c", "log"),
            make_node("d", "log"),
        ];
        let edges = vec![
            make_edge("a", "b"),
            make_edge("a", "c"),
            make_edge("b", "d"),
            make_edge("c", "d"),
        ];

        let levels = executor.topological_levels(&nodes, &edges).unwrap();
        assert_eq!(levels.len(), 3); // a | b,c | d
        assert_eq!(levels[0], vec!["a"]);
        assert_eq!(levels[1].len(), 2);
        assert_eq!(levels[2], vec!["d"]);
    }

    #[tokio::test]
    async fn test_cycle_detection() {
        let registry = Arc::new(NodeRegistry::new());
        let executor = Executor::new(registry);

        let nodes = vec![make_node("a", "log"), make_node("b", "log")];
        let edges = vec![make_edge("a", "b"), make_edge("b", "a")];

        let result = executor.topological_levels(&nodes, &edges);
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_validation_called() {
        // Use the log node which has default validation (accepts everything)
        let registry = Arc::new(NodeRegistry::new());
        let executor = Executor::new(registry);

        let workflow = Workflow {
            id: "test".to_string(),
            name: "Test".to_string(),
            description: String::new(),
            nodes: vec![make_node("log_1", "log")],
            edges: vec![],
            variables: vec![],
            owner_id: None,
            created_at: Utc::now(),
        };

        let result = executor.execute(&workflow, None).await;
        assert!(result.is_ok());
    }
}
