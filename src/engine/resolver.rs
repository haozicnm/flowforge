//! Unified variable resolver — THE SINGLE LAYER.
//!
//! This is the most critical module in FlowForge. The old project had three
//! layers of variable resolution (global resolve, container-internal resolve,
//! Rhai engine), each with different rules. That caused variables to not expand
//! in containers, action.id ≠ action.label data断流, and countless bugs.
//!
//! Rule: ALL nodes (including containers) go through this ONE resolver.
//! No exceptions. No "skip resolve for containers" hacks.
//!
//! ## Variable Reference Format
//!
//! - Storage: `{{stepId.portLabel}}` — step.id as root, port label as key
//! - Display: `步骤名 › 端口名` — human-friendly
//! - These are SEPARATE. Never mix them.
//!
//! ## Two-Phase Resolution
//!
//! Phase 1: Scan config for `{{...}}` patterns, replace with safe placeholders.
//!          This prevents type corruption during deserialization.
//!
//! Phase 2: After deserialization, replace placeholders with actual values.
//!
//! This two-phase approach solves the "template variable becomes a number/object
//! and breaks deserialization" problem that forced the old project to skip
//! global resolve for containers.

use once_cell::sync::Lazy;
use regex::Regex;
use std::collections::HashMap;

use crate::engine::workflow::Node;
use crate::error::{FlowError, FlowResult};

/// A resolved variable reference.
#[derive(Debug, Clone)]
pub struct ResolvedRef {
    /// The step ID (root key).
    pub step_id: String,
    /// The port label (sub key).
    pub port_label: String,
    /// The placeholder that replaced this reference in the config.
    pub placeholder: String,
}

/// The result of Phase 1 (placeholder insertion).
#[derive(Debug)]
pub struct PlaceholderMap {
    /// placeholder → (step_id, port_label)
    pub map: HashMap<String, (String, String)>,
}

/// Regex to match `{{stepId.portLabel}}` patterns outside of code fences.
static VAR_REF_RE: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"\{\{([a-zA-Z_][a-zA-Z0-9_]*\.[a-zA-Z0-9_]+)\}\}").expect("invalid regex")
});

/// Phase 1: Scan a JSON value for variable references and replace them with placeholders.
///
/// Returns the modified value and a map of placeholders to their (step_id, port_label).
///
/// # Example
/// ```json
/// {"url": "{{http1.response}}", "timeout": 30}
/// ```
/// becomes:
/// ```json
/// {"url": "__FF_VAR_0__", "timeout": 30}
/// ```
/// with map: {"__FF_VAR_0__" → ("http1", "response")}
pub fn phase1_insert_placeholders(
    value: &serde_json::Value,
) -> (serde_json::Value, PlaceholderMap) {
    let mut map = HashMap::new();
    let mut counter = 0;
    let modified = replace_refs_recursive(value, &mut map, &mut counter);
    (modified, PlaceholderMap { map })
}

/// Phase 2: Replace placeholders with actual values from step outputs.
///
/// This runs AFTER deserialization, so types are preserved.
pub fn phase2_resolve_placeholders(
    value: &serde_json::Value,
    placeholders: &PlaceholderMap,
    step_outputs: &HashMap<String, serde_json::Value>,
) -> FlowResult<serde_json::Value> {
    resolve_recursive(value, placeholders, step_outputs)
}

/// High-level: resolve all variables in a node's config.
///
/// This is the ONLY function that nodes should call for variable resolution.
/// Do NOT implement your own resolve logic.
pub fn resolve_node_config(
    node: &Node,
    step_outputs: &HashMap<String, serde_json::Value>,
) -> FlowResult<serde_json::Value> {
    // Phase 1: insert placeholders
    let (intermediate, placeholders) = phase1_insert_placeholders(&node.config);

    // Phase 2: resolve placeholders with actual values
    phase2_resolve_placeholders(&intermediate, &placeholders, step_outputs)
}

// ── Internal helpers ──

fn replace_refs_recursive(
    value: &serde_json::Value,
    map: &mut HashMap<String, (String, String)>,
    counter: &mut usize,
) -> serde_json::Value {
    match value {
        serde_json::Value::String(s) => {
            // Replace all {{stepId.portLabel}} in this string
            let mut result = s.clone();
            for cap in VAR_REF_RE.captures_iter(s) {
                let full_match = &cap[0]; // {{stepId.portLabel}}
                let ref_expr = &cap[1]; // stepId.portLabel
                let parts: Vec<&str> = ref_expr.splitn(2, '.').collect();
                if parts.len() == 2 {
                    let placeholder = format!("__FF_VAR_{}__", counter);
                    result = result.replace(full_match, &placeholder);
                    map.insert(placeholder, (parts[0].to_string(), parts[1].to_string()));
                    *counter += 1;
                }
            }
            serde_json::Value::String(result)
        }
        serde_json::Value::Array(arr) => {
            let new_arr: Vec<_> = arr
                .iter()
                .map(|v| replace_refs_recursive(v, map, counter))
                .collect();
            serde_json::Value::Array(new_arr)
        }
        serde_json::Value::Object(obj) => {
            let new_obj: serde_json::Map<_, _> = obj
                .iter()
                .map(|(k, v)| (k.clone(), replace_refs_recursive(v, map, counter)))
                .collect();
            serde_json::Value::Object(new_obj)
        }
        // Numbers, bools, null — pass through unchanged
        other => other.clone(),
    }
}

fn resolve_recursive(
    value: &serde_json::Value,
    placeholders: &PlaceholderMap,
    step_outputs: &HashMap<String, serde_json::Value>,
) -> FlowResult<serde_json::Value> {
    match value {
        serde_json::Value::String(s) => {
            let mut result = s.clone();
            for (placeholder, (step_id, port_label)) in &placeholders.map {
                if result.contains(placeholder) {
                    let output = step_outputs
                        .get(step_id)
                        .and_then(|o| o.get(port_label))
                        .ok_or_else(|| FlowError::UndefinedVariable {
                            ref_expr: format!("{}.{}", step_id, port_label),
                        })?;
                    // If the string is EXACTLY the placeholder, replace with the raw value
                    // (preserves type: number, object, etc.)
                    if result == *placeholder {
                        return Ok(output.clone());
                    }
                    // Otherwise, stringify and embed (partial replacement in a string)
                    let display_value = match output {
                        serde_json::Value::String(s) => s.clone(),
                        other => other.to_string(),
                    };
                    result = result.replace(placeholder, &display_value);
                }
            }
            Ok(serde_json::Value::String(result))
        }
        serde_json::Value::Array(arr) => {
            let new_arr: Vec<_> = arr
                .iter()
                .map(|v| resolve_recursive(v, placeholders, step_outputs))
                .collect::<FlowResult<_>>()?;
            Ok(serde_json::Value::Array(new_arr))
        }
        serde_json::Value::Object(obj) => {
            let new_obj: serde_json::Map<_, _> = obj
                .iter()
                .map(|(k, v)| {
                    resolve_recursive(v, placeholders, step_outputs)
                        .map(|resolved| (k.clone(), resolved))
                })
                .collect::<FlowResult<_>>()?;
            Ok(serde_json::Value::Object(new_obj))
        }
        other => Ok(other.clone()),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_phase1_simple_string() {
        let config = json!({"url": "{{http1.response}}"});
        let (modified, map) = phase1_insert_placeholders(&config);

        assert_eq!(modified["url"], "__FF_VAR_0__");
        assert_eq!(map.map.len(), 1);
        assert_eq!(
            map.map["__FF_VAR_0__"],
            ("http1".to_string(), "response".to_string())
        );
    }

    #[test]
    fn test_phase1_multiple_refs() {
        let config = json!({
            "method": "POST",
            "url": "{{api.endpoint}}",
            "body": {"user": "{{auth.username}}"}
        });
        let (modified, map) = phase1_insert_placeholders(&config);

        assert_eq!(map.map.len(), 2);
        assert_eq!(modified["method"], "POST"); // unchanged
        assert!(modified["url"].as_str().unwrap().starts_with("__FF_VAR_"));
    }

    #[test]
    fn test_phase2_resolve_string() {
        let config = json!({"url": "{{http1.response}}"});
        let (modified, placeholders) = phase1_insert_placeholders(&config);

        let mut step_outputs = HashMap::new();
        step_outputs.insert(
            "http1".to_string(),
            json!({"response": "https://example.com"}),
        );

        let resolved =
            phase2_resolve_placeholders(&modified, &placeholders, &step_outputs).unwrap();
        assert_eq!(resolved["url"], "https://example.com");
    }

    #[test]
    fn test_phase2_preserves_type() {
        // When a string is EXACTLY one placeholder, the raw value type is preserved
        let config = json!({"count": "{{step1.num}}"});
        let (modified, placeholders) = phase1_insert_placeholders(&config);

        let mut step_outputs = HashMap::new();
        step_outputs.insert("step1".to_string(), json!({"num": 42}));

        let resolved =
            phase2_resolve_placeholders(&modified, &placeholders, &step_outputs).unwrap();
        // Should be number 42, not string "42"
        assert_eq!(resolved["count"], 42);
    }

    #[test]
    fn test_phase2_partial_string_replacement() {
        // When a placeholder is embedded in a larger string, it becomes a string
        let config = json!({"message": "User {{user.name}} logged in"});
        let (modified, placeholders) = phase1_insert_placeholders(&config);

        let mut step_outputs = HashMap::new();
        step_outputs.insert("user".to_string(), json!({"name": "Alice"}));

        let resolved =
            phase2_resolve_placeholders(&modified, &placeholders, &step_outputs).unwrap();
        assert_eq!(resolved["message"], "User Alice logged in");
    }

    #[test]
    fn test_phase2_undefined_variable_error() {
        let config = json!({"url": "{{missing.step}}"});
        let (modified, placeholders) = phase1_insert_placeholders(&config);

        let step_outputs = HashMap::new(); // empty
        let result = phase2_resolve_placeholders(&modified, &placeholders, &step_outputs);
        assert!(result.is_err());
    }

    #[test]
    fn test_no_variables_passes_through() {
        let config = json!({"timeout": 30, "retries": 3});
        let (modified, map) = phase1_insert_placeholders(&config);
        assert!(map.map.is_empty());
        assert_eq!(modified, config);
    }
}
