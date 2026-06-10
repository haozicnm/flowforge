# Changelog

## [1.0.0] — 2025-06-10

### 🚀 Platform-grade v1.0 Release

FlowForge 从 70% 原型升级为完整的平台化 v1.0 工作流自动化引擎。

### ✨ 新增功能

#### 后端 (Rust)
- **28 种节点类型**：新增 Database（SQLite SQL 查询）、Notification（Slack/Email/Webhook 通知）、File（读/写/删除/移动/列表）、Cron（定时触发器）
- **SQLite 持久化**：工作流从 JSON 文件迁移到 SQLite（rusqlite+bundled），自动迁移旧数据
- **JWT 认证体系**：注册/登录 API，bcrypt 密码哈希，72h 令牌，工作流 owner 隔离
- **动态插件系统**：`libloading` 扫描 `plugins/*.{so,dll,dylib}`，`export_plugin!` 宏，运行时注册
- **WebSocket 实时执行事件流**：`/ws/execute/:id` 逐节点推送 start/completed/failed/done
- **Webhook 触发器**：`POST /api/webhook/:wid/:nid` 接收 HTTP 请求并自动执行工作流
- **Loop 子图迭代**：`loopBody { nodes, edges }` 配置，count/collection 两种模式
- **Shell 节点沙箱**：env 隔离、30s 超时、1MiB 输出截断
- **JSON 导入/导出 API**：单文件 + 全量导出
- **3 处 `unwrap()` 修复**：全部改用 `FlowError` 传播
- **32 个测试**：14 单元 + 8 集成 + 10 节点烟雾

#### 前端 (Flutter)
- **多级设计 Token**：bg/border/icon 三级色阶，Poppins 字体，FontSizes/FontWeights 常量
- **14 个可复用组件**：FfButton（primary/outlined/text + sm/md/lg）、FfTextField、FfDropdown、FfDialog、FfToggle、FfToast、FfTooltip、SidebarResizer、FfSvg（26 图标）、CommandPalette
- **属性面板双模式**：config_schema 驱动表单 ↔ JSON 源码编辑器
- **设置页重构**：左菜单 204px + 分割线 + 右内容区，通用/快捷键/关于/插件 4 个 tab
- **Command Palette**：Ctrl+K 模糊搜索工作流/节点/命令
- **i18n 国际化**：easy_localization + zh.json / en.json（~80 keys）

#### 工程基础设施
- **GitHub Actions CI**：Rust × 3 OS（build/test/clippy）+ Flutter × 3 OS（pub get/analyze）
- **跨平台打包脚本**：`scripts/package.sh`（linux/macos/windows）
- **MIT LICENSE**

#### 文档
- `docs/API.md` — 16 个端点完整文档
- `docs/USER_GUIDE.md` — 安装/导航/28 节点/快捷键/FAQ
- `docs/DEVELOPING_NODES.md` — trait 教程/动态插件导出

### 🔧 修复
- `executor.rs:170` unwrap → map_err
- `webbridge/mod.rs:133` unwrap → map_err  
- `excel_write_node.rs:95` unwrap → ok_or_else
- Loop 节点从伪实现改为真实子图迭代
- Webhook 节点从占位符改为真实 HTTP 监听
