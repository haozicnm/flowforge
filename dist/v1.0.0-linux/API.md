# FlowForge API 文档

所有端点均以 `http://127.0.0.1:19529` 为基础地址。

---

## 通用

### `GET /api/health`

检查服务健康状态。

**响应**
```json
{
  "version": "1.0.0",
  "status": "ok"
}
```

---

## 认证

### `POST /api/auth/register`

注册新用户。

**请求体**
```json
{
  "username": "alice",
  "password": "secret123"
}
```

**响应** `201`
```json
{
  "token": "eyJhbGciOi...",
  "user": {
    "id": "a1b2c3d4-...",
    "username": "alice",
    "created_at": "2025-06-10T12:00:00Z"
  }
}
```

### `POST /api/auth/login`

登录，返回 JWT 令牌（72h 有效期）。

**请求体**
```json
{
  "username": "alice",
  "password": "secret123"
}
```

**响应**
```json
{
  "token": "eyJhbGciOi...",
  "user": { "id": "...", "username": "alice" }
}
```

### `GET /api/auth/me`

返回当前用户信息（需要 Authorization header）。

**响应**
```json
{
  "id": "a1b2c3d4-...",
  "username": "alice",
  "created_at": "2025-06-10T12:00:00Z"
}
```

---

## 节点类型

### `GET /api/nodes/types`

列出所有已注册的节点类型（24 种）。

**响应**
```json
[
  {
    "type_name": "log",
    "display_name": "日志输出",
    "description": "将数据输出到执行日志",
    "category": "调试",
    "inputs": [...],
    "outputs": [...],
    "config_schema": {...}
  }
]
```

---

## 工作流 CRUD

### `GET /api/workflows`

列出所有工作流。

### `POST /api/workflows`

创建工作流。

**请求体**
```json
{
  "name": "我的工作流",
  "description": "可选描述"
}
```

**响应** `201`

### `GET /api/workflows/:id`

获取工作流详情。

### `PUT /api/workflows/:id`

更新工作流（名称、描述、节点、连接）。

**请求体**
```json
{
  "name": "新名称",
  "nodes": [...],
  "edges": [...]
}
```

### `DELETE /api/workflows/:id`

删除工作流。

---

## 执行

### `POST /api/workflows/:id/execute`

执行一个工作流（同步，返回最终结果）。

**响应**
```json
{
  "status": "completed",
  "node_outputs": { "node1": { "out": "hello" } },
  "completed": ["node1", "node2"]
}
```

---

## WebSocket 实时执行

### `ws://127.0.0.1:19529/ws/execute/:id`

WebSocket 实时事件流。连接到工作流执行时，逐节点推送事件：

```json
{"type":"node_started","node_id":"log1"}
{"type":"node_completed","node_id":"log1","outputs":{"out":"hello"}}
{"type":"node_failed","node_id":"shell1","error":"..."}
{"type":"done","completed":["log1"],"failed":[]}
```

---

## Webhook 触发器

### `POST /api/webhook/:workflow_id/:node_id`

### `GET /api/webhook/:workflow_id/:node_id`

接收外部 HTTP 请求作为工作流触发器。接收到请求后自动执行对应工作流。

**响应**
```json
{
  "status": "triggered",
  "node_id": "wh1",
  "completed": ["wh1", "log1"]
}
```

---

## JSON 导入/导出

### `GET /api/workflows/:id/export`

导出单个工作流为 JSON 字符串。

### `GET /api/workflows/export-all`

导出所有工作流为 JSON 数组。

### `POST /api/workflows/import`

从 JSON 导入工作流。

---

## WebBridge 浏览器自动化

### `GET /api/browser/status`

检查 Chrome 扩展连接状态。

### `POST /api/browser/command`

发送浏览器命令（通过 WebBridge）。

### `ws://127.0.0.1:19529/ws/browser`

WebBridge WebSocket 连接点。

---

## 插件

### `GET /api/plugins/list`

列出所有已注册的节点类型（内置 + 插件）。
