# FlowForge — Visual Workflow Automation Engine
# Multi-stage build: Rust backend + Flutter frontend

# ── Stage 1: Build Rust backend ──
FROM rust:1.82-bookworm AS backend-builder

WORKDIR /app
COPY Cargo.toml Cargo.lock ./
# Cache dependencies
RUN mkdir src && echo 'fn main(){}' > src/main.rs && \
    echo '' > src/lib.rs && \
    cargo build --release 2>/dev/null || true && \
    rm -rf src

COPY src/ src/
COPY tests/ tests/
RUN touch src/main.rs src/lib.rs && cargo build --release

# ── Stage 2: Build Flutter frontend ──
FROM ghcr.io/cirruslabs/flutter:stable AS frontend-builder

WORKDIR /app
COPY flutter_app/pubspec.yaml flutter_app/pubspec.lock ./
RUN flutter pub get

COPY flutter_app/ .
RUN flutter build web --release

# ── Stage 3: Runtime ──
FROM debian:bookworm-slim

RUN apt-get update && apt-get install -y --no-install-recommends \
    ca-certificates \
    libssl3 \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /opt/flowforge

# Copy backend binary
COPY --from=backend-builder /app/target/release/flowforge /opt/flowforge/flowforge

# Copy Flutter web build
COPY --from=frontend-builder /app/build/web/ /opt/flowforge/dist/

# Create data directory
RUN mkdir -p /opt/flowforge/data

# Expose port
EXPOSE 19529

# Health check
HEALTHCHECK --interval=30s --timeout=3s --start-period=5s --retries=3 \
    CMD curl -f http://localhost:19529/api/health || exit 1

# Run
CMD ["/opt/flowforge/flowforge", "--port", "19529", "--data-dir", "/opt/flowforge/data"]
