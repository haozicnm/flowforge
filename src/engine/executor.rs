//! Workflow executor — runs workflows topologically.
//!
//! Key rules from lessons-learned:
//! 1. All variable resolution goes through resolver.rs (single layer)
//! 2. Nodes receive already-resolved config
//! 3. Execution state is explicit (not global singletons)
//! 4. Each node gets its own execution context

use std::collections::HashMap;
use std::sync::Arc;

use tokio::sync::RwLock;

use crate::engine::resolver;
use crate::engine::workflow::{Edge, Node, Workflow};
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
    NodeCompleted { _node_id: String, _outputs: HashMap<String, serde_json::Value> },
    NodeFailed { _node_id: String, _error: String },
    WorkflowCompleted,
    _WorkflowFailed { _error: String },
}

/// Workflow executor.
pub struct Executor {
    registry: Arc<NodeRegistry>,
    webbridge: Option<WebBridgeState>,
}

impl Executor {
    pub fn new(registry: Arc<NodeRegistry>) -> Self {
        Self { registry, webbridge: None }
    }

    pub fn with_webbridge(mut self, webbridge: WebBridgeState) -> Self {
        self.webbridge = Some(webbridge);
        self
    }

    /// Execute a workflow.
    pub async fn execute(
        &self,
        workflow: &Workflow,
        event_tx: Option<tokio::sync::mpsc::Sender<ExecutionEvent>>,
    ) -> FlowResult<ExecutionState> {
        let nodes = workflow.nodes();
        let edges = workflow.edges();

        // Build adjacency list for topological sort
        let sorted = self.topological_sort(nodes, edges)?;

        // Validate: all variable references point to valid nodes
        self.validate_references(nodes)?;

        let state = Arc::new(RwLock::new(ExecutionState::new()));

        // Execute in topological order
        for node_id in &sorted {
            let node = nodes
                .iter()
                .find(|n| &n.id == node_id)
                .ok_or_else(|| FlowError::NodeNotFound(node_id.clone()))?;

            // Mark as running
            {
                let mut s = state.write().await;
                s.running.push(node_id.clone());
            }
            self.send_event(&event_tx, ExecutionEvent::NodeStarted { _node_id: node_id.clone() }).await;

            // Resolve variables in config
            let step_outputs = state.read().await.flat_outputs();
            let resolved_config = resolver::resolve_node_config(node, &step_outputs)?;

            // Collect inputs from upstream nodes, then drop the read lock
            // BEFORE executing the node (node execution needs write lock for state update)
            let inputs = {
                let state_guard = state.read().await;
                self.collect_inputs(node_id, edges, &state_guard)
            };

            // Execute the node
            let executor = self.registry.get_executor(&node.node_type)?;
            let ctx = match &self.webbridge {
                Some(wb) => NodeContext::with_webbridge(wb.clone()),
                None => NodeContext::empty(),
            };
            match executor.execute(node, &ctx, resolved_config, inputs).await {
                Ok(outputs) => {
                    let mut s = state.write().await;
                    s.node_outputs.insert(node_id.clone(), outputs.clone());
                    s.completed.push(node_id.clone());
                    s.running.retain(|id| id != node_id);

                    self.send_event(
                        &event_tx,
                        ExecutionEvent::NodeCompleted {
                            _node_id: node_id.clone(),
                            _outputs: outputs,
                        },
                    )
                    .await;
                }
                Err(e) => {
                    let mut s = state.write().await;
                    s.failed.push(node_id.clone());
                    s.running.retain(|id| id != node_id);

                    self.send_event(
                        &event_tx,
                        ExecutionEvent::NodeFailed {
                            _node_id: node_id.clone(),
                            _error: e.to_string(),
                        },
                    )
                    .await;

                    return Err(e);
                }
            }
        }

        self.send_event(&event_tx, ExecutionEvent::WorkflowCompleted).await;
        Ok(Arc::try_unwrap(state).unwrap().into_inner())
    }

    /// Topological sort of nodes based on edges.
    fn topological_sort(
        &self,
        nodes: &[Node],
        edges: &[Edge],
    ) -> FlowResult<Vec<String>> {
        let mut in_degree: HashMap<String, usize> = HashMap::new();
        let mut adj: HashMap<String, Vec<String>> = HashMap::new();

        // Initialize
        for node in nodes {
            in_degree.entry(node.id.clone()).or_insert(0);
            adj.entry(node.id.clone()).or_default();
        }

        // Build graph
        for edge in edges {
            *in_degree.entry(edge.to.clone()).or_insert(0) += 1;
            adj.entry(edge.from.clone())
                .or_default()
                .push(edge.to.clone());
        }

        // Kahn's algorithm
        let mut queue: Vec<String> = in_degree
            .iter()
            .filter(|(_, &deg)| deg == 0)
            .map(|(id, _)| id.clone())
            .collect();

        let mut sorted = Vec::new();

        while let Some(current) = queue.pop() {
            sorted.push(current.clone());
            if let Some(neighbors) = adj.get(&current) {
                for neighbor in neighbors {
                    let deg = in_degree.get_mut(neighbor).unwrap();
                    *deg -= 1;
                    if *deg == 0 {
                        queue.push(neighbor.clone());
                    }
                }
            }
        }

        if sorted.len() != nodes.len() {
            return Err(FlowError::ExecutionError(
                "Workflow contains a cycle".to_string(),
            ));
        }

        Ok(sorted)
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
