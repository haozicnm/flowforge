//! Loop node — iterates over a collection or repeats N times.
//!
//! Two modes:
//! - "count": repeats loopBody N times (config.count)
//! - "collection": iterates over config.collection (or input collection)
//!
//! loopBody is a config block containing { "nodes": [...], "edges": [...] }
//! that forms the sub-workflow executed per iteration.

use async_trait::async_trait;
use std::collections::HashMap;
use std::sync::Arc;

use crate::engine::resolver;
use crate::engine::workflow::Node;
use crate::error::{FlowError, FlowResult};
use crate::nodes::registry::NodeRegistry;
use crate::nodes::traits::{NodeContext, NodeExecutor, NodeTypeDef, PortDef};

#[derive(Default)]
pub struct LoopNode;

/// Internal helper: execute a sub-workflow (body) with the given registry.
async fn execute_body(
    body_nodes: &[Node],
    body_edges: &[crate::engine::workflow::Edge],
    registry: &Arc<NodeRegistry>,
    ctx: &NodeContext,
    iteration_vars: &HashMap<String, serde_json::Value>,
) -> FlowResult<HashMap<String, serde_json::Value>> {
    // Simple topological sort via in-degree
    let mut in_degree: HashMap<String, usize> = HashMap::new();
    let mut adj: HashMap<String, Vec<String>> = HashMap::new();
    for n in body_nodes {
        in_degree.entry(n.id.clone()).or_insert(0);
        adj.entry(n.id.clone()).or_default();
    }
    for e in body_edges {
        *in_degree.entry(e.to.clone()).or_insert(0) += 1;
        adj.entry(e.from.clone()).or_default().push(e.to.clone());
    }

    let mut queue: Vec<String> = in_degree
        .iter()
        .filter(|(_, &d)| d == 0)
        .map(|(id, _)| id.clone())
        .collect();
    let mut sorted = Vec::new();
    while let Some(current) = queue.pop() {
        sorted.push(current.clone());
        if let Some(nbrs) = adj.get(&current) {
            for nb in nbrs {
                if let Some(d) = in_degree.get_mut(nb) {
                    *d -= 1;
                    if *d == 0 {
                        queue.push(nb.clone());
                    }
                }
            }
        }
    }

    // Execute in topological order
    let mut outputs: HashMap<String, HashMap<String, serde_json::Value>> = HashMap::new();

    for node_id in &sorted {
        let node = body_nodes
            .iter()
            .find(|n| &n.id == node_id)
            .ok_or_else(|| FlowError::NodeNotFound(node_id.clone()))?;

        // Resolve config using iteration vars + accumulated outputs
        let mut flat = iteration_vars.clone();
        for (nid, ports) in &outputs {
            for (port, val) in ports {
                flat.insert(format!("{}.{}", nid, port), val.clone());
            }
        }
        let resolved_config = resolver::resolve_node_config(node, &flat)?;

        // Collect inputs from upstream body nodes
        let mut inputs = HashMap::new();
        for e in body_edges {
            if e.to == *node_id {
                if let Some(src_outs) = outputs.get(&e.from) {
                    if let Some(v) = src_outs.get(&e.from_port) {
                        inputs.insert(e.to_port.clone(), v.clone());
                    }
                }
            }
        }

        let exec = registry.get_executor(&node.node_type)?;
        let result = exec.execute(node, ctx, resolved_config, inputs).await?;
        outputs.insert(node_id.clone(), result);
    }

    // Return the last body node's outputs as the iteration result
    Ok(sorted.last().and_then(|k| outputs.get(k)).cloned().unwrap_or_default())
}

#[async_trait]
impl NodeExecutor for LoopNode {
    fn type_def(&self) -> NodeTypeDef {
        NodeTypeDef {
            type_name: "loop".to_string(),
            display_name: "循环".to_string(),
            description: "遍历集合或重复执行 N 次。body 内的节点可引用 {{item}} / {{index}}".to_string(),
            category: "流程控制".to_string(),
            inputs: vec![
                PortDef { label: "collection".to_string(), data_type: "array".to_string(), required: false },
            ],
            outputs: vec![
                PortDef { label: "results".to_string(), data_type: "array".to_string(), required: false },
                PortDef { label: "item".to_string(), data_type: "any".to_string(), required: false },
                PortDef { label: "index".to_string(), data_type: "number".to_string(), required: false },
            ],
            config_schema: serde_json::json!({
                "type": "object",
                "properties": {
                    "mode": {
                        "type": "string",
                        "enum": ["count", "collection"],
                        "default": "count"
                    },
                    "count": { "type": "number", "default": 3 },
                    "loopBody": {
                        "type": "object",
                        "properties": {
                            "nodes": { "type": "array" },
                            "edges": { "type": "array" }
                        }
                    }
                },
                "required": ["mode"]
            }),
        }
    }

    async fn execute(
        &self,
        node: &Node,
        ctx: &NodeContext,
        config: serde_json::Value,
        inputs: HashMap<String, serde_json::Value>,
    ) -> FlowResult<HashMap<String, serde_json::Value>> {
        let registry = ctx
            .node_registry
            .as_ref()
            .ok_or_else(|| FlowError::NodeExecutionFailed {
                node_id: node.id.clone(),
                detail: "Loop node requires NodeRegistry in context".into(),
            })?;

        let mode = config["mode"].as_str().unwrap_or("count");

        // Parse loopBody
        let body_nodes: Vec<Node> = config["loopBody"]["nodes"]
            .as_array()
            .map(|arr| {
                arr.iter()
                    .filter_map(|v| serde_json::from_value(v.clone()).ok())
                    .collect()
            })
            .unwrap_or_default();

        let body_edges: Vec<crate::engine::workflow::Edge> = config["loopBody"]["edges"]
            .as_array()
            .map(|arr| {
                arr.iter()
                    .filter_map(|v| serde_json::from_value(v.clone()).ok())
                    .collect()
            })
            .unwrap_or_default();

        let mut all_results: Vec<serde_json::Value> = Vec::new();
        let mut last_item = serde_json::Value::Null;
        let mut last_index = 0u64;

        match mode {
            "count" => {
                let count = config["count"].as_u64().unwrap_or(3).max(0).min(1000);
                tracing::info!("Loop '{}': repeating {} times", node.id, count);

                for i in 0..count {
                    let mut iter_vars = HashMap::new();
                    iter_vars.insert("index".into(), serde_json::json!(i));
                    iter_vars.insert("item".into(), serde_json::json!(i));

                    if body_nodes.is_empty() {
                        all_results.push(serde_json::json!({"index": i}));
                        last_index = i;
                        last_item = serde_json::json!(i);
                    } else {
                        match execute_body(&body_nodes, &body_edges, registry, ctx, &iter_vars).await {
                            Ok(result) => {
                                all_results.push(serde_json::to_value(&result).unwrap_or_default());
                                last_index = i;
                                last_item = serde_json::to_value(&result).unwrap_or_default();
                            }
                            Err(e) => {
                                tracing::warn!("Loop body failed at iteration {}: {}", i, e);
                                // Continue to next iteration — don't abort the whole loop
                                all_results.push(serde_json::json!({"error": e.to_string(), "index": i}));
                            }
                        }
                    }
                }
            }
            "collection" => {
                let collection = inputs
                    .get("collection")
                    .and_then(|v| v.as_array())
                    .or_else(|| config["collection"].as_array());

                let items: Vec<serde_json::Value> = collection
                    .map(|arr| arr.to_vec())
                    .unwrap_or_default();

                tracing::info!("Loop '{}': {} items in collection", node.id, items.len());

                for (i, item) in items.iter().enumerate() {
                    if i >= 1000 { break; }
                    let mut iter_vars = HashMap::new();
                    iter_vars.insert("index".into(), serde_json::json!(i));
                    iter_vars.insert("item".into(), item.clone());

                    if body_nodes.is_empty() {
                        all_results.push(serde_json::json!({"index": i, "item": item}));
                        last_index = i as u64;
                        last_item = item.clone();
                    } else {
                        match execute_body(&body_nodes, &body_edges, registry, ctx, &iter_vars).await {
                            Ok(result) => {
                                all_results.push(serde_json::to_value(&result).unwrap_or_default());
                                last_index = i as u64;
                                last_item = serde_json::to_value(&result).unwrap_or_default();
                            }
                            Err(e) => {
                                tracing::warn!("Loop body failed at index {}: {}", i, e);
                                all_results.push(serde_json::json!({"error": e.to_string(), "index": i}));
                            }
                        }
                    }
                }
            }
            _ => {
                return Err(FlowError::InvalidNodeConfig {
                    node_id: node.id.clone(),
                    detail: format!("unknown mode: {}", mode),
                });
            }
        }

        let mut outputs = HashMap::new();
        outputs.insert("results".into(), serde_json::json!(all_results));
        outputs.insert("item".into(), last_item);
        outputs.insert("index".into(), serde_json::json!(last_index));
        Ok(outputs)
    }
}
