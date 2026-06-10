# FlowForge 用户指南

FlowForge 是一个可视化工作流自动化引擎。你可以在画布上拖拽节点、连接它们，形成一个自动化流水线。

---

## 快速开始

### 安装

1. 下载对应平台的安装包（Windows `.zip`、macOS `.dmg`、Linux `.tar.gz`）
2. 解压后双击 `FlowForge.bat`（Windows）或 `flowforge.sh`（Linux/macOS）
3. 应用自动启动，连接到本地后端

### 界面导航

- **工作流**：查看和管理所有工作流
- **编辑器**：可视化编辑工作流
- **设置**：应用配置、快捷键、插件管理

---

## 创建工作流

1. 在「工作流」页面点击「新建工作流」
2. 输入名称，点击「创建」
3. 自动进入编辑器

### 画布模式

- **画布**（默认）：拖拽节点、连接端口，实时显示
- **列表**：表格视图，适合管理大量节点
- **代码**：直接编辑 JSON 源码

### 添加节点

按 `Ctrl+K` 打开命令面板，搜索节点类型添加，或在「列表」视图中点击 + 按钮。

### 连接节点

1. 在画布上从一个节点的输出端口拖到另一个节点的输入端口
2. 或在「列表」视图中使用「添加连接」

### 变量引用

使用 `{{nodeId.portLabel}}` 格式引用上游节点的输出：

```
"message": "{{start.out}}"
```

---

## 执行工作流

1. 点击「执行」按钮或按 `Ctrl+Enter`
2. 右侧面板显示实时执行状态
3. 每个节点执行完毕后显示绿色 ✓，失败显示红色 ✗

---

## 节点类型一览

| 分类 | 节点 | 说明 |
|------|------|------|
| 触发器 | Webhook | 接收 HTTP 请求触发 |
| 触发器 | Cron | 定时触发 (*/5 = 每5分钟) |
| 流程控制 | Condition | 条件判断 (等于/大于/包含/正则…) |
| 流程控制 | Loop | 遍历集合或循环 N 次 |
| 流程控制 | Try/Catch | 错误处理分支 |
| 流程控制 | Delay | 延时等待 |
| 数据处理 | Script | Rhai 脚本执行 |
| 数据处理 | Variable | 变量定义与类型转换 |
| 数据处理 | JSON | JSON 操作 (提取/合并/解析…) |
| 数据处理 | Regex | 正则匹配/替换/分割 |
| 数据处理 | Template | 模板替换 `{{name}}` |
| 数据库 | Database | SQLite SQL 查询 |
| 通知 | Notification | Slack/Email/Webhook 通知 |
| 网络 | HTTP | HTTP 请求 (GET/POST/PUT/DELETE…) |
| 文件 | File | 文件读/写/删除/移动/列表 |
| Web 自动化 | Web Navigate | 浏览器导航 |
| Web 自动化 | Web Click | 点击元素 |
| Web 自动化 | Web Input | 输入文字 |
| Web 自动化 | Web Extract | 提取数据 |
| Web 自动化 | Web Screenshot | 截图 |
| Web 自动化 | Web Wait | 等待元素 |
| 文件 | Excel Read | 读取 Excel |
| 文件 | Excel Write | 写入 Excel |
| 文件 | DOCX Read | 读取 Word 文档 |
| 文件 | DOCX Create | 创建 Word 文档 |
| 系统 | Shell | 执行 Shell 命令 |
| 调试 | Log | 日志输出 |

---

## 属性面板

选中节点后，右侧面板显示：

- **标签**：节点的显示名称
- **表单模式**：根据 `config_schema` 自动生成的配置表单
- **代码模式**：直接编辑 JSON 配置

---

## 快捷键

| 快捷键 | 功能 |
|--------|------|
| `Ctrl + S` | 保存工作流 |
| `Ctrl + Enter` | 执行工作流 |
| `Ctrl + K` | 命令面板 |
| `Ctrl + \` | 切换侧栏 |

---

## 设置

### 通用

- 语言：简体中文 / English
- 主题：跟随系统 / 浅色 / 深色
- 数据目录：工作流存储位置
- 服务器地址：后端监听地址

### 插件

查看已安装的节点类型。将 `.so/.dll/.dylib` 插件放入 `plugins/` 目录重启即可加载。

---

## 认证

在后续版本中，可以通过 `POST /api/auth/register` 注册账户，使用 JWT 令牌进行登录认证。工作流可以绑定到特定用户（owner 隔离）。

---

## 常见问题

**Q: Shell 节点执行失败？**
A: Shell 节点有 30s 超时限制。确认命令路径正确，无网络调用。

**Q: Loop 节点不循环？**
A: 需要配置 `loopBody` 字段（包含 `nodes` 和 `edges` 子图）。`mode: "count"` 按次数循环，`mode: "collection"` 遍历数组。

**Q: Webhook 节点没有收到数据？**
A: 先创建包含 webhook 节点的工作流，然后向 `/api/webhook/:wid/:nid` 发送请求。

**Q: 如何备份工作流？**
A: 使用 `GET /api/workflows/export-all` 导出所有工作流为 JSON，或 `GET /api/workflows/:id/export` 导出单个。
