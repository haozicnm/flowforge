@echo off
title FlowForge - Rust Hot Reload
echo.
echo  FlowForge Rust Backend (hot reload mode)
echo  Watching src/ for changes, auto-recompile + restart
echo  Press Ctrl+C to stop
echo.
cd /d "%~dp0"
set RUST_LOG=flowforge=info
cargo watch -x run
