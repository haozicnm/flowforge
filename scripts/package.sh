#!/usr/bin/env bash
# FlowForge Cross-Platform Packaging Script
#
# Usage:
#   ./scripts/package.sh           # auto-detect platform, build+package
#   ./scripts/package.sh linux     # Linux AppImage
#   ./scripts/package.sh macos     # macOS .dmg
#   ./scripts/package.sh windows   # Windows zip (cross-compile from Linux)
#
# Requires: cargo, flutter, zip, tar

set -euo pipefail

ROOT="$(cd "$(dirname "$0")/.." && pwd)"
VERSION="1.0.0"
OUT_DIR="$ROOT/dist/$VERSION"
PLATFORM="${1:-$(uname -s | tr '[:upper:]' '[:lower:]')}"

echo ""
echo "  FlowForge Cross-Platform Packager v$VERSION"
echo "  Platform: $PLATFORM"
echo "  ════════════════════════════════════════"
echo ""

# Step 1: Build Rust backend
echo "[1/4] Building Rust backend..."
cd "$ROOT"
cargo build --release --quiet
echo "  OK"

# Step 2: Build Flutter frontend
echo "[2/4] Building Flutter frontend..."
cd "$ROOT/flutter_app"
case "$PLATFORM" in
  linux)
    flutter build linux --release --quiet
    FLUTTER_BUILD_DIR="$ROOT/flutter_app/build/linux/x64/release/bundle"
    ;;
  macos|darwin)
    PLATFORM="macos"
    flutter build macos --release --quiet
    FLUTTER_BUILD_DIR="$ROOT/flutter_app/build/macos/Build/Products/Release/flowforge.app"
    ;;
  windows)
    flutter build windows --release --quiet
    FLUTTER_BUILD_DIR="$ROOT/flutter_app/build/windows/x64/runner/Release"
    ;;
  *)
    echo "Unknown platform: $PLATFORM"
    exit 1
    ;;
esac
echo "  OK"

# Step 3: Assemble distribution
echo "[3/4] Assembling distribution..."

rm -rf "$OUT_DIR"
mkdir -p "$OUT_DIR/backend"

# Copy Rust backend
cp "$ROOT/target/release/flowforge" "$OUT_DIR/backend/" 2>/dev/null || \
  cp "$ROOT/target/release/flowforge.exe" "$OUT_DIR/backend/" 2>/dev/null || \
  echo "  Warning: Rust binary not found"

case "$PLATFORM" in
  linux)
    # Copy Flutter bundle
    cp -r "$FLUTTER_BUILD_DIR"/* "$OUT_DIR/"
    # Create data dir
    mkdir -p "$OUT_DIR/data"

    # Create launcher script
    cat > "$OUT_DIR/flowforge.sh" << 'LAUNCHER'
#!/usr/bin/env bash
SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
echo "Starting FlowForge backend..."
"$SCRIPT_DIR/backend/flowforge" &
BACKEND_PID=$!
sleep 1
export SERVER_URL=http://127.0.0.1:19529
echo "Starting FlowForge..."
"$SCRIPT_DIR/flowforge"
kill $BACKEND_PID 2>/dev/null
LAUNCHER
    chmod +x "$OUT_DIR/flowforge.sh"

    # Try to create AppImage if linuxdeploy is available
    if command -v linuxdeploy &>/dev/null; then
      echo "  Creating AppImage..."
      mkdir -p "$OUT_DIR/AppDir/usr/bin"
      cp "$ROOT/target/release/flowforge" "$OUT_DIR/AppDir/usr/bin/"
      linuxdeploy --appdir "$OUT_DIR/AppDir" --output appimage \
        --desktop-file "$ROOT/scripts/flowforge.desktop" 2>/dev/null || true
      mv FlowForge-*.AppImage "$ROOT/dist/" 2>/dev/null || true
    fi
    ;;

  macos)
    # Copy Flutter .app bundle
    cp -r "$FLUTTER_BUILD_DIR" "$OUT_DIR/"
    cp "$ROOT/target/release/flowforge" "$OUT_DIR/FlowForge.app/Contents/MacOS/backend"
    mkdir -p "$OUT_DIR/FlowForge.app/Contents/Resources/data"

    # Create .dmg if hdiutil is available
    if command -v hdiutil &>/dev/null; then
      echo "  Creating .dmg..."
      hdiutil create -volname "FlowForge" -srcfolder "$OUT_DIR" \
        -ov -format UDZO "$ROOT/dist/flowforge-v$VERSION-macos.dmg" 2>/dev/null || true
    fi
    ;;

  windows)
    cp -r "$FLUTTER_BUILD_DIR"/* "$OUT_DIR/"
    mkdir -p "$OUT_DIR/data"

    # Create launcher batch
    cat > "$OUT_DIR/FlowForge.bat" << 'LAUNCHER'
@echo off
echo Starting FlowForge backend...
start /B "" "%~dp0backend\flowforge.exe"
timeout /t 2 /nobreak >nul
echo Starting FlowForge UI...
set SERVER_URL=http://127.0.0.1:19529
start "" "%~dp0flowforge.exe"
LAUNCHER
    ;;
esac

echo "  OK"

# Step 4: Archive
echo "[4/4] Creating archive..."
cd "$ROOT/dist"
case "$PLATFORM" in
  linux)
    tar -czf "flowforge-v$VERSION-linux-x64.tar.gz" "$VERSION/"
    echo "  Archive: $ROOT/dist/flowforge-v$VERSION-linux-x64.tar.gz"
    ;;
  macos)
    tar -czf "flowforge-v$VERSION-macos.tar.gz" "$VERSION/"
    echo "  Archive: $ROOT/dist/flowforge-v$VERSION-macos.tar.gz"
    ;;
  windows)
    zip -rq "flowforge-v$VERSION-windows-x64.zip" "$VERSION/"
    echo "  Archive: $ROOT/dist/flowforge-v$VERSION-windows-x64.zip"
    ;;
esac

echo ""
echo "  ╔══════════════════════════════════════╗"
echo "  ║  Package created successfully!       ║"
echo "  ╚══════════════════════════════════════╝"
echo ""
