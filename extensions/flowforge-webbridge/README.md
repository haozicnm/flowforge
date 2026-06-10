# FlowForge WebBridge

浏览器自动化桥接扩展，为 FlowForge 提供 CDP (Chrome DevTools Protocol) 控制能力。

## 安装

1. 打开 Chrome → `chrome://extensions/`
2. 开启「开发者模式」
3. 点击「加载已解压的扩展程序」
4. 选择此目录

## 工作原理

```
FlowForge 后端 (Rust/Axum) → ws://127.0.0.1:19529/ws/browser
    ↓ WebSocket
Chrome 扩展 (background.js)
    ↓ CDP commands
浏览器页面 (DOM)
```

## 支持的命令

| 命令 | 功能 |
|------|------|
| navigate | 打开 URL |
| click | 点击元素 (CSS selector 或 @e ref) |
| fill | 输入文本 |
| extract_text | 提取文本 |
| extract_html | 提取 HTML |
| extract_attribute | 提取属性 |
| extract_links | 提取链接 |
| extract_table | 提取表格 |
| screenshot | 截图 (支持元素级) |
| evaluate | 执行 JavaScript |
| wait_for | 等待元素/文本出现 |
| list_tabs | 列出所有标签页 |
| scroll | 滚动页面 |
| hover | 悬停 |
| send_keys | 键盘输入 |
| ... | 更多 |

## 配置

点击扩展图标可修改连接端口（默认 19529）。

## 与 workflow-engine 的关系

此扩展从 workflow-engine 的 WebBridge 扩展移植而来，协议完全兼容。
