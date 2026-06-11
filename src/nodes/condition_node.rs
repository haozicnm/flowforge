//! Condition node — branches execution based on an expression.
use async_trait::async_trait;
use std::collections::HashMap;
use crate::error::{FlowError, FlowResult};
use crate::engine::workflow::Node;
use crate::nodes::traits::{NodeExecutor, NodeTypeDef, PortDef};

#[derive(Default)]
pub struct ConditionNode;

#[async_trait]
impl NodeExecutor for ConditionNode {
    fn type_def(&self) -> NodeTypeDef {
        NodeTypeDef {
            version: "1.0".to_string(),
            type_name: "condition".to_string(),
            display_name: "条件判断".to_string(),
            description: "根据条件表达式分支执行".to_string(),
            category: "流程控制".to_string(),
            inputs: vec![PortDef {
                label: "value".to_string(),
                data_type: "any".to_string(),
                required: true,
            }],
            outputs: vec![
                PortDef { label: "true".to_string(), data_type: "any".to_string(), required: false },
                PortDef { label: "false".to_string(), data_type: "any".to_string(), required: false },
                PortDef { label: "result".to_string(), data_type: "boolean".to_string(), required: false },
            ],
            config_schema: serde_json::json!({
                "type": "object",
                "properties": {
                    "operator": {
                        "type": "string",
                        "enum": ["equals", "not_equals", "contains", "gt", "lt", "gte", "lte", "is_empty", "is_not_empty", "regex_match", "starts_with", "ends_with"],
                        "default": "equals"
                    },
                    "compare_value": { "type": "string" },
                    "expression": { "type": "string", "description": "Complex expression (e.g. 'value > 10 AND value < 100')" }
                },
                "required": ["operator"]
            }),
        }
    }

    fn validate_config(
        &self,
        config: &serde_json::Value,
    ) -> Result<(), Vec<crate::nodes::traits::ValidationError>> {
        let mut errors = Vec::new();

        // If expression is set, validate it can be parsed
        if let Some(expr) = config.get("expression").and_then(|v| v.as_str()) {
            if expr.trim().is_empty() {
                errors.push(crate::nodes::traits::ValidationError {
                    field: "expression".to_string(),
                    message: "expression cannot be empty".to_string(),
                });
            }
        }

        if errors.is_empty() { Ok(()) } else { Err(errors) }
    }

    async fn execute(
        &self,
        _node: &Node,
        _ctx: &crate::nodes::traits::NodeContext,
        config: serde_json::Value,
        inputs: HashMap<String, serde_json::Value>,
    ) -> FlowResult<HashMap<String, serde_json::Value>> {
        let value = inputs.get("value").cloned().unwrap_or(serde_json::Value::Null);

        // If expression is provided, use the expression engine
        if let Some(expr) = config.get("expression").and_then(|v| v.as_str()) {
            let result = eval_expression(expr, &value, &inputs)?;
            tracing::info!("Condition expression '{}' = {}", expr, result);

            let mut outputs = HashMap::new();
            outputs.insert("result".to_string(), serde_json::json!(result));
            if result {
                outputs.insert("true".to_string(), value);
            } else {
                outputs.insert("false".to_string(), value);
            }
            return Ok(outputs);
        }

        // Otherwise use the operator/compare_value approach
        let operator = config["operator"].as_str().unwrap_or("equals");
        let compare = &config["compare_value"];

        let result = match operator {
            "equals" => value == *compare,
            "not_equals" => value != *compare,
            "contains" => {
                let s = value.as_str().unwrap_or("");
                let needle = compare.as_str().unwrap_or("");
                s.contains(needle)
            }
            "gt" => {
                let a = value.as_f64().unwrap_or(0.0);
                let b = compare.as_f64().unwrap_or(0.0);
                a > b
            }
            "lt" => {
                let a = value.as_f64().unwrap_or(0.0);
                let b = compare.as_f64().unwrap_or(0.0);
                a < b
            }
            "gte" => {
                let a = value.as_f64().unwrap_or(0.0);
                let b = compare.as_f64().unwrap_or(0.0);
                a >= b
            }
            "lte" => {
                let a = value.as_f64().unwrap_or(0.0);
                let b = compare.as_f64().unwrap_or(0.0);
                a <= b
            }
            "is_empty" => {
                value.is_null()
                    || value.as_str().map_or(false, |s| s.is_empty())
                    || value.as_array().map_or(false, |a| a.is_empty())
                    || value.as_object().map_or(false, |o| o.is_empty())
            }
            "is_not_empty" => {
                !value.is_null()
                    && !value.as_str().map_or(false, |s| s.is_empty())
                    && !value.as_array().map_or(false, |a| a.is_empty())
                    && !value.as_object().map_or(false, |o| o.is_empty())
            }
            "regex_match" => {
                let s = value.as_str().unwrap_or("");
                let pattern = compare.as_str().unwrap_or("");
                regex::Regex::new(pattern).map_or(false, |re| re.is_match(s))
            }
            "starts_with" => {
                let s = value.as_str().unwrap_or("");
                let prefix = compare.as_str().unwrap_or("");
                s.starts_with(prefix)
            }
            "ends_with" => {
                let s = value.as_str().unwrap_or("");
                let suffix = compare.as_str().unwrap_or("");
                s.ends_with(suffix)
            }
            _ => {
                return Err(FlowError::InvalidNodeConfig {
                    node_id: "condition".to_string(),
                    detail: format!("unknown operator: {}", operator),
                });
            }
        };

        tracing::info!("Condition: {:?} {} {:?} = {}", value, operator, compare, result);

        let mut outputs = HashMap::new();
        outputs.insert("result".to_string(), serde_json::json!(result));
        if result {
            outputs.insert("true".to_string(), value);
        } else {
            outputs.insert("false".to_string(), value);
        }
        Ok(outputs)
    }
}

/// Simple expression evaluator for condition node.
/// Supports: comparisons (>, <, >=, <=, ==, !=), logical (AND, OR, NOT), parentheses.
pub fn eval_expression(
    expr: &str,
    value: &serde_json::Value,
    inputs: &HashMap<String, serde_json::Value>,
) -> FlowResult<bool> {
    let mut parser = ExprParser::new(expr, value, inputs);
    parser.parse_or()
}

struct ExprParser<'a> {
    tokens: Vec<String>,
    pos: usize,
    value: &'a serde_json::Value,
    inputs: &'a HashMap<String, serde_json::Value>,
}

impl<'a> ExprParser<'a> {
    fn new(expr: &str, value: &'a serde_json::Value, inputs: &'a HashMap<String, serde_json::Value>) -> Self {
        Self {
            tokens: tokenize(expr),
            pos: 0,
            value,
            inputs,
        }
    }

    fn peek(&self) -> Option<&str> {
        self.tokens.get(self.pos).map(|s| s.as_str())
    }

    fn advance(&mut self) -> Option<String> {
        if self.pos < self.tokens.len() {
            let tok = self.tokens[self.pos].clone();
            self.pos += 1;
            Some(tok)
        } else {
            None
        }
    }

    fn expect(&mut self, expected: &str) -> FlowResult<()> {
        match self.advance() {
            Some(tok) if tok == expected => Ok(()),
            Some(tok) => Err(FlowError::ExecutionError(format!(
                "expected '{}', got '{}'", expected, tok
            ))),
            None => Err(FlowError::ExecutionError(format!(
                "expected '{}', got end of expression", expected
            ))),
        }
    }

    // or_expr = and_expr ("OR" and_expr)*
    fn parse_or(&mut self) -> FlowResult<bool> {
        let mut left = self.parse_and()?;
        while self.peek() == Some("OR") {
            self.advance();
            let right = self.parse_and()?;
            left = left || right;
        }
        Ok(left)
    }

    // and_expr = not_expr ("AND" not_expr)*
    fn parse_and(&mut self) -> FlowResult<bool> {
        let mut left = self.parse_not()?;
        while self.peek() == Some("AND") {
            self.advance();
            let right = self.parse_not()?;
            left = left && right;
        }
        Ok(left)
    }

    // not_expr = ["NOT"] comparison
    fn parse_not(&mut self) -> FlowResult<bool> {
        if self.peek() == Some("NOT") {
            self.advance();
            let val = self.parse_comparison()?;
            return Ok(!val);
        }
        self.parse_comparison()
    }

    // comparison = atom (op atom)?
    fn parse_comparison(&mut self) -> FlowResult<bool> {
        if self.peek() == Some("(") {
            self.advance();
            let val = self.parse_or()?;
            self.expect(")")?;
            return Ok(val);
        }

        let left = self.parse_atom()?;

        if let Some(op) = self.peek() {
            match op {
                ">" | "<" | ">=" | "<=" | "==" | "!=" => {
                    let op = self.advance().unwrap();
                    let right = self.parse_atom()?;
                    Ok(compare_values(&left, &op, &right)?)
                }
                _ => {
                    // No operator — truthy check
                    Ok(is_truthy(&left))
                }
            }
        } else {
            Ok(is_truthy(&left))
        }
    }

    // atom = literal | "value" | "inputs.xxx"
    fn parse_atom(&mut self) -> FlowResult<serde_json::Value> {
        let tok = self.advance().ok_or_else(|| FlowError::ExecutionError(
            "unexpected end of expression".to_string()
        ))?;

        match tok.as_str() {
            "value" => Ok(self.value.clone()),
            "true" => Ok(serde_json::json!(true)),
            "false" => Ok(serde_json::json!(false)),
            "null" => Ok(serde_json::Value::Null),
            _ if tok.starts_with('"') && tok.ends_with('"') => {
                let s = tok[1..tok.len()-1].to_string();
                Ok(serde_json::json!(s))
            }
            _ if tok.chars().next().map_or(false, |c| c.is_ascii_digit() || c == '-') => {
                if let Ok(n) = tok.parse::<f64>() {
                    Ok(serde_json::json!(n))
                } else {
                    Err(FlowError::ExecutionError(format!("invalid number: {}", tok)))
                }
            }
            _ if tok.starts_with("inputs.") => {
                let key = &tok[7..];
                Ok(self.inputs.get(key).cloned().unwrap_or(serde_json::Value::Null))
            }
            _ => Err(FlowError::ExecutionError(format!(
                "unexpected token: {}", tok
            ))),
        }
    }
}

fn tokenize(expr: &str) -> Vec<String> {
    let mut tokens = Vec::new();
    let mut chars = expr.chars().peekable();

    while let Some(&c) = chars.peek() {
        match c {
            ' ' | '\t' => { chars.next(); }
            '(' | ')' => { tokens.push(c.to_string()); chars.next(); }
            '>' | '<' | '!' => {
                let mut op = c.to_string();
                chars.next();
                if chars.peek() == Some(&'=') {
                    op.push('=');
                    chars.next();
                }
                tokens.push(op);
            }
            '=' => {
                chars.next();
                if chars.peek() == Some(&'=') {
                    chars.next();
                    tokens.push("==".to_string());
                } else {
                    tokens.push("=".to_string());
                }
            }
            '"' => {
                chars.next();
                let mut s = String::new();
                while let Some(&ch) = chars.peek() {
                    if ch == '"' {
                        chars.next();
                        break;
                    }
                    if ch == '\\' {
                        chars.next();
                        if let Some(&esc) = chars.peek() {
                            s.push(esc);
                            chars.next();
                        }
                    } else {
                        s.push(ch);
                        chars.next();
                    }
                }
                tokens.push(format!("\"{}\"", s));
            }
            '-' if chars.clone().nth(1).map_or(false, |c| c.is_ascii_digit()) => {
                let mut num = String::from('-');
                chars.next();
                while let Some(&ch) = chars.peek() {
                    if ch.is_ascii_digit() || ch == '.' {
                        num.push(ch);
                        chars.next();
                    } else {
                        break;
                    }
                }
                tokens.push(num);
            }
            _ if c.is_ascii_digit() => {
                let mut num = String::new();
                while let Some(&ch) = chars.peek() {
                    if ch.is_ascii_digit() || ch == '.' {
                        num.push(ch);
                        chars.next();
                    } else {
                        break;
                    }
                }
                tokens.push(num);
            }
            _ if c.is_ascii_alphabetic() || c == '_' => {
                let mut word = String::new();
                while let Some(&ch) = chars.peek() {
                    if ch.is_ascii_alphanumeric() || ch == '_' || ch == '.' {
                        word.push(ch);
                        chars.next();
                    } else {
                        break;
                    }
                }
                tokens.push(word);
            }
            _ => { chars.next(); }
        }
    }

    tokens
}

fn compare_values(
    left: &serde_json::Value,
    op: &str,
    right: &serde_json::Value,
) -> FlowResult<bool> {
    match op {
        "==" => Ok(left == right),
        "!=" => Ok(left != right),
        ">" | "<" | ">=" | "<=" => {
            let a = left.as_f64().unwrap_or(0.0);
            let b = right.as_f64().unwrap_or(0.0);
            match op {
                ">" => Ok(a > b),
                "<" => Ok(a < b),
                ">=" => Ok(a >= b),
                "<=" => Ok(a <= b),
                _ => unreachable!(),
            }
        }
        _ => Err(FlowError::ExecutionError(format!("unknown operator: {}", op))),
    }
}

fn is_truthy(value: &serde_json::Value) -> bool {
    match value {
        serde_json::Value::Null => false,
        serde_json::Value::Bool(b) => *b,
        serde_json::Value::Number(n) => n.as_f64().unwrap_or(0.0) != 0.0,
        serde_json::Value::String(s) => !s.is_empty(),
        serde_json::Value::Array(a) => !a.is_empty(),
        serde_json::Value::Object(o) => !o.is_empty(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::nodes::traits::NodeContext;

    fn make_node(id: &str) -> Node {
        Node {
            id: id.to_string(),
            node_type: "condition".to_string(),
            label: "Test Condition".to_string(),
            config: serde_json::json!({}),
            position: Default::default(),
        }
    }

    #[tokio::test]
    async fn test_condition_equals_true() {
        let node = make_node("cond_1");
        let ctx = NodeContext::empty();
        let config = serde_json::json!({"operator": "equals", "compare_value": "hello"});
        let mut inputs = HashMap::new();
        inputs.insert("value".to_string(), serde_json::json!("hello"));
        let result = ConditionNode.execute(&node, &ctx, config, inputs).await.unwrap();
        assert_eq!(result["result"], true);
        assert!(result.contains_key("true"));
    }

    #[tokio::test]
    async fn test_condition_equals_false() {
        let node = make_node("cond_1");
        let ctx = NodeContext::empty();
        let config = serde_json::json!({"operator": "equals", "compare_value": "world"});
        let mut inputs = HashMap::new();
        inputs.insert("value".to_string(), serde_json::json!("hello"));
        let result = ConditionNode.execute(&node, &ctx, config, inputs).await.unwrap();
        assert_eq!(result["result"], false);
        assert!(result.contains_key("false"));
    }

    #[tokio::test]
    async fn test_condition_contains() {
        let node = make_node("cond_1");
        let ctx = NodeContext::empty();
        let config = serde_json::json!({"operator": "contains", "compare_value": "ell"});
        let mut inputs = HashMap::new();
        inputs.insert("value".to_string(), serde_json::json!("hello"));
        let result = ConditionNode.execute(&node, &ctx, config, inputs).await.unwrap();
        assert_eq!(result["result"], true);
    }

    #[tokio::test]
    async fn test_condition_gt() {
        let node = make_node("cond_1");
        let ctx = NodeContext::empty();
        let config = serde_json::json!({"operator": "gt", "compare_value": "5"});
        let mut inputs = HashMap::new();
        inputs.insert("value".to_string(), serde_json::json!(10));
        let result = ConditionNode.execute(&node, &ctx, config, inputs).await.unwrap();
        assert_eq!(result["result"], true);
    }

    #[tokio::test]
    async fn test_condition_is_empty() {
        let node = make_node("cond_1");
        let ctx = NodeContext::empty();
        let config = serde_json::json!({"operator": "is_empty"});
        let mut inputs = HashMap::new();
        inputs.insert("value".to_string(), serde_json::json!(""));
        let result = ConditionNode.execute(&node, &ctx, config, inputs).await.unwrap();
        assert_eq!(result["result"], true);
    }

    #[tokio::test]
    async fn test_condition_starts_with() {
        let node = make_node("cond_1");
        let ctx = NodeContext::empty();
        let config = serde_json::json!({"operator": "starts_with", "compare_value": "hel"});
        let mut inputs = HashMap::new();
        inputs.insert("value".to_string(), serde_json::json!("hello"));
        let result = ConditionNode.execute(&node, &ctx, config, inputs).await.unwrap();
        assert_eq!(result["result"], true);
    }

    #[tokio::test]
    async fn test_expression_simple_comparison() {
        let node = make_node("cond_1");
        let ctx = NodeContext::empty();
        let config = serde_json::json!({"expression": "value > 10"});
        let mut inputs = HashMap::new();
        inputs.insert("value".to_string(), serde_json::json!(15));
        let result = ConditionNode.execute(&node, &ctx, config, inputs).await.unwrap();
        assert_eq!(result["result"], true);
    }

    #[tokio::test]
    async fn test_expression_and() {
        let node = make_node("cond_1");
        let ctx = NodeContext::empty();
        let config = serde_json::json!({"expression": "value > 10 AND value < 20"});
        let mut inputs = HashMap::new();
        inputs.insert("value".to_string(), serde_json::json!(15));
        let result = ConditionNode.execute(&node, &ctx, config, inputs).await.unwrap();
        assert_eq!(result["result"], true);
    }

    #[tokio::test]
    async fn test_expression_or() {
        let node = make_node("cond_1");
        let ctx = NodeContext::empty();
        let config = serde_json::json!({"expression": "value < 5 OR value > 10"});
        let mut inputs = HashMap::new();
        inputs.insert("value".to_string(), serde_json::json!(15));
        let result = ConditionNode.execute(&node, &ctx, config, inputs).await.unwrap();
        assert_eq!(result["result"], true);
    }

    #[tokio::test]
    async fn test_expression_not() {
        let node = make_node("cond_1");
        let ctx = NodeContext::empty();
        let config = serde_json::json!({"expression": "NOT value == 0"});
        let mut inputs = HashMap::new();
        inputs.insert("value".to_string(), serde_json::json!(5));
        let result = ConditionNode.execute(&node, &ctx, config, inputs).await.unwrap();
        assert_eq!(result["result"], true);
    }

    #[tokio::test]
    async fn test_expression_parentheses() {
        let node = make_node("cond_1");
        let ctx = NodeContext::empty();
        let config = serde_json::json!({"expression": "(value > 10) AND (value < 20)"});
        let mut inputs = HashMap::new();
        inputs.insert("value".to_string(), serde_json::json!(15));
        let result = ConditionNode.execute(&node, &ctx, config, inputs).await.unwrap();
        assert_eq!(result["result"], true);
    }

    #[tokio::test]
    async fn test_expression_string_equality() {
        let node = make_node("cond_1");
        let ctx = NodeContext::empty();
        let config = serde_json::json!({"expression": "value == \"hello\""});
        let mut inputs = HashMap::new();
        inputs.insert("value".to_string(), serde_json::json!("hello"));
        let result = ConditionNode.execute(&node, &ctx, config, inputs).await.unwrap();
        assert_eq!(result["result"], true);
    }

    #[tokio::test]
    async fn test_expression_truthy() {
        let node = make_node("cond_1");
        let ctx = NodeContext::empty();
        let config = serde_json::json!({"expression": "value"});
        let mut inputs = HashMap::new();
        inputs.insert("value".to_string(), serde_json::json!("non-empty"));
        let result = ConditionNode.execute(&node, &ctx, config, inputs).await.unwrap();
        assert_eq!(result["result"], true);
    }

    #[tokio::test]
    async fn test_expression_falsy() {
        let node = make_node("cond_1");
        let ctx = NodeContext::empty();
        let config = serde_json::json!({"expression": "value"});
        let mut inputs = HashMap::new();
        inputs.insert("value".to_string(), serde_json::json!(""));
        let result = ConditionNode.execute(&node, &ctx, config, inputs).await.unwrap();
        assert_eq!(result["result"], false);
    }
}
