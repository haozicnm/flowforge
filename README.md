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

## Build

```bash
# Backend
cargo build --release

# Frontend
cd flutter_app && flutter build linux --release
```

## Test

```bash
cargo test
```

## Key Design Decisions

1. **Single-layer variable resolution** — no container bypass, no multi-layer resolve
2. **HTTP bridge, not FFI** — Flutter talks to Rust via REST/WebSocket
3. **Shell is last resort** — dedicated nodes (HTTP, file, DB) are preferred
4. **No unwrap()** — all errors flow through `FlowError`
5. **Default bind 127.0.0.1** — not 0.0.0.0

## License

MIT
