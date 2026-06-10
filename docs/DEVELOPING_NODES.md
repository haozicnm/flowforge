# FlowForge 节点开发手册

本手册介绍如何开发自定义节点。

---

## 架构概述

每个节点类型实现 `NodeExecutor` trait：

```rust
#[async_trait]
pub trait NodeExecutor: Send + Sync {
    fn type_def(&self) -> NodeTypeDef;
    async fn execute(
        &self,
        node: &Node,
        ctx: &NodeContext,
        config: serde_json::Value,
        inputs: HashMap<String, serde_json::Value>,
    ) -> FlowResult<HashMap<String, serde_json::Value>>;
}
```

- `type_def()` — 返回节点的元数据（名称、分类、端口、配置 schema）
- `execute()` — 执行节点的核心逻辑

---

## 内置节点示例

### 最小节点

```rust
use async_trait::async_trait;
use std::collections::HashMap;
use crate::engine::workflow::Node;
use crate::error::{FlowError, FlowResult};
use crate::nodes::traits::{NodeExecutor, NodeTypeDef, PortDef};

#[derive(Default)]
pub struct EchoNode;

#[async_trait]
impl NodeExecutor for EchoNode {
    fn type_def(&self) -> NodeTypeDef {
        NodeTypeDef {
            type_name: "echo".to_string(),
            display_name: "回声".to_string(),
            description: "原样返回输入".to_string(),
            category: "调试".to_string(),
            inputs: vec![
                PortDef {
                    label: "in".to_string(),
                    data_type: "any".to_string(),
                    required: false,
                },
            ],
            outputs: vec![
                PortDef {
                    label: "out".to_string(),
                    data_type: "any".to_string(),
                    required: false,
                },
            ],
            config_schema: serde_json::json!({
                "type": "object",
                "properties": {
                    "prefix": {"type": "string", "default": ""}
                }
            }),
        }
    }

    async fn execute(
        &self,
        node: &Node,
        _ctx: &NodeContext,
        config: serde_json::Value,
        inputs: HashMap<String, serde_json::Value>,
    ) -> FlowResult<HashMap<String, serde_json::Value>> {
        let prefix = config["prefix"].as_str().unwrap_or("");
        let value = inputs.get("in").cloned().unwrap_or(serde_json::Value::Null);

        let mut outputs = HashMap::new();
        outputs.insert("out".into(), serde_json::json!(format!("{}{}", prefix, value)));
        Ok(outputs)
    }
}
```

### 注册节点

在 `src/nodes/registry.rs` 的 `NodeRegistry::new()` 中添加：

```rust
registry.register_builtin::<super::echo_node::EchoNode>();
```

在 `src/nodes/mod.rs` 中声明模块：

```rust
pub mod echo_node;
```

---

## PortDef 与 config_schema

### 端口定义

```rust
PortDef {
    label: "out".to_string(),          // 端口标识（{{nodeId.out}}）
    data_type: "string".to_string(),   // 类型提示: any/string/number/object/array/boolean
    required: false,                   // 是否必需
}
```

### 配置 Schema

使用 JSON Schema 定义节点配置字段，前端属性面板自动生成表单：

```json
{
    "type": "object",
    "properties": {
        "url": {"type": "string"},
        "method": {
            "type": "string",
            "enum": ["GET", "POST", "PUT", "DELETE"],
            "default": "GET"
        },
        "timeout_ms": {"type": "number", "default": 5000}
    },
    "required": ["url"]
}
```

支持的字段类型：
- `string` — 文本输入（有 `enum` 时显示下拉框）
- `number` — 数字输入
- `boolean` — 开关（`FfToggle`）
- `object` / `array` — 多行文本编辑

---

## 变量引用

节点的 `config` 在传入 `execute()` 前已经过**两阶段变量解析**，无需手动处理。`{{nodeId.portLabel}}` 占位符已被替换为实际值。

---

## 错误处理

使用 `FlowError` 枚举返回描述性错误：

```rust
Err(FlowError::InvalidNodeConfig {
    node_id: node.id.clone(),
    detail: "url is required".into(),
})

Err(FlowError::NodeExecutionFailed {
    node_id: node.id.clone(),
    detail: format!("HTTP request failed: {}", e),
})
```

---

## NodeContext

`NodeContext` 提供共享服务访问：

```rust
pub struct NodeContext {
    pub webbridge: Option<WebBridgeState>,   // 浏览器自动化
    pub node_registry: Option<Arc<NodeRegistry>>, // 子执行（Loop 节点用）
    pub webhook_store: Option<...>,          // Webhook 数据
}
```

---

## 动态插件（第三方节点）

第三方节点以共享库形式发布（`.so` / `.dll` / `.dylib`），导出 C-ABI 入口点。

### 插件 crate 结构

```rust
// Cargo.toml
[lib]
crate-type = ["cdylib"]

// src/lib.rs
use flowforge::export_plugin;
use flowforge::nodes::traits::NodeExecutor;

#[derive(Default)]
struct MyPluginNode;

// ... impl NodeExecutor for MyPluginNode ...

flowforge::export_plugin!(MyPluginNode);
```

### 加载插件

将编译产物放入 `plugins/` 目录，重启 FlowForge 自动加载。

```
plugins/
├── my_plugin.so       (Linux)
├── my_plugin.dll      (Windows)
└── my_plugin.dylib    (macOS)
```

---

## 测试节点

```rust
#[tokio::test]
async fn test_my_node() {
    let n = MyNode::default();
    let cfg = serde_json::json!({});
    let node = Node {
        id: "test".into(),
        node_type: "my_type".into(),
        label: "".into(),
        config: cfg.clone(),
        position: Position::default(),
    };

    let result = n.execute(&node, &NodeContext::empty(), cfg, HashMap::new()).await;
    assert!(result.is_ok());
}
```

---

## 节点开发检查清单

- [ ] 实现 `NodeExecutor` trait
- [ ] `type_def()` 提供完整元数据
- [ ] `config_schema` 使用 JSON Schema 定义
- [ ] `execute()` 处理所有输入端口
- [ ] 错误使用 `FlowError` 枚举返回
- [ ] 不调用 `unwrap()` / `panic!()`
- [ ] 添加单元测试
- [ ] 在 `registry.rs` 中注册
- [ ] 在 `mod.rs` 中声明模块
