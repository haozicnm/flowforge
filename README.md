# FlowForge

Visual workflow automation engine. Rust backend + Flutter desktop UI.

## Architecture

```
Flutter Desktop Window
├── Dart UI (Material 3)
│   ├── Dashboard
│   ├── Workflow Editor
│   └── Execution Log
├── HTTP / WebSocket
└── Rust Backend (axum)
    ├── /api/workflows
    ├── /api/nodes/types
    ├── /api/execute
    └── ws://logs
```

## Features

- **Visual Editor** — Drag-and-drop workflow design
- **33 Node Types** — HTTP, file, shell, data transform, flow control, scheduling
- **Real-time Execution** — WebSocket-based live updates
- **Variable System** — Automatic variable resolution between nodes
- **Error Handling** — Unified error types with context
- **Cross-platform** — Windows, Linux, macOS

## Installation

### Pre-built Binaries

Download from [GitHub Releases](https://github.com/haozicnm/flowforge/releases).

### Docker (Recommended)

```bash
# One-command start
docker compose up -d

# Or build manually
docker build -t flowforge .
docker run -d -p 19529:19529 -v flowforge-data:/opt/flowforge/data flowforge
```

Open http://localhost:19529 in your browser.

### Build from Source

```bash
# Backend (Rust)
cargo build --release

# Frontend (Flutter)
cd flutter_app
flutter pub get
flutter build linux --release  # or windows/macos
```

## Quick Start

1. **Start the server**
   ```bash
   ./target/release/flowforge --port 19529 --data-dir ./data
   # Or with defaults (127.0.0.1:19529)
   ./target/release/flowforge
   ```

2. **Open the UI**
   - Launch the Flutter app
   - Or open http://localhost:19529 in your browser

3. **Create a workflow**
   - Click "New Workflow" in the dashboard
   - Drag nodes from the palette
   - Connect nodes by clicking ports
   - Configure node properties

4. **Execute**
   - Click the "Run" button
   - Watch real-time execution in the log panel

## Node Types

### Basic
| Node | Description |
|------|-------------|
| `log` | Output data to execution log |
| `http` | Make HTTP requests |
| `delay` | Pause execution |
| `shell` | Execute shell commands (emergency only) |
| `script` | Run JavaScript/Rhai scripts |
| `webhook` | Receive external HTTP triggers |

### Flow Control
| Node | Description |
|------|-------------|
| `condition` | Branch based on expressions |
| `loop` | Iterate over collections or repeat N times |
| `try_catch` | Error handling wrapper |

### Data Operations
| Node | Description |
|------|-------------|
| `variable` | Set/cast variables |
| `json` | Parse/stringify/extract JSON |
| `regex` | Pattern matching and replacement |
| `template` | String templating with `{{var}}` |

### File Operations
| Node | Description |
|------|-------------|
| `file` | Read/write/delete/list files |
| `excel_read` | Read Excel spreadsheets |
| `excel_write` | Write Excel spreadsheets |
| `docx_read` | Read Word documents |
| `docx_create` | Create Word documents |

### Web Automation
| Node | Description |
|------|-------------|
| `web_navigate` | Open URLs in browser |
| `web_click` | Click elements |
| `web_input` | Fill form fields |
| `web_extract` | Extract data from pages |
| `web_screenshot` | Capture screenshots |
| `web_wait` | Wait for elements |

## API Reference

### Workflows

```bash
# List all workflows
GET /api/workflows

# Get workflow
GET /api/workflows/:id

# Create workflow
POST /api/workflows
{
  "name": "My Workflow",
  "yaml": "nodes:\n  - id: log_1\n    type: log\n    config:\n      message: Hello"
}

# Update workflow
PUT /api/workflows/:id

# Delete workflow
DELETE /api/workflows/:id

# Execute workflow
POST /api/workflows/:id/execute

# WebSocket execution
ws://localhost:19529/ws/execute/:id
```

### Node Types

```bash
# List all node types
GET /api/nodes/types
```

## Configuration

### Server

| Option | Default | Description |
|--------|---------|-------------|
| `--host` | `127.0.0.1` | Bind address |
| `--port` | `19529` | Server port |
| `--db` | `flowforge.db` | SQLite database path |

### Environment Variables

| Variable | Description |
|----------|-------------|
| `FLOWFORGE_HOST` | Server host |
| `FLOWFORGE_PORT` | Server port |
| `FLOWFORGE_DB` | Database path |

## Development

### Prerequisites

- Rust 1.75+
- Flutter 3.16+

### Running in Development

```bash
# Terminal 1: Rust backend
cargo run

# Terminal 2: Flutter frontend
cd flutter_app
flutter run -d linux  # or windows/macos
```

### Testing

```bash
# Run all tests
cargo test

# Run specific test
cargo test --lib nodes::log_node::tests

# Run with output
cargo test -- --nocapture
```

### Code Quality

```bash
# Lint
cargo clippy -- -D warnings

# Format
cargo fmt

# Check
cargo check
```

## Key Design Decisions

1. **Single-layer variable resolution** — no container bypass, no multi-layer resolve
2. **HTTP bridge, not FFI** — Flutter talks to Rust via REST/WebSocket
3. **Shell is last resort** — dedicated nodes (HTTP, file, DB) are preferred
4. **No unwrap()** — all errors flow through `FlowError`
5. **Default bind 127.0.0.1** — not 0.0.0.0

## Contributing

1. Fork the repository
2. Create a feature branch
3. Make your changes
4. Add tests
5. Submit a pull request

### Adding a New Node

1. Create `src/nodes/my_node.rs`
2. Implement `NodeExecutor` trait
3. Register in `src/nodes/registry.rs`
4. Add tests
5. Update documentation

## License

MIT
