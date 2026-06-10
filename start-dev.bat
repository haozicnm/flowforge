@echo off
title FlowForge - Full Dev Environment
echo.
echo  FlowForge Development Environment
echo  ════════════════════════════════════
echo  Terminal 1: Rust backend (auto-restart on src/ changes)
echo  Terminal 2: Flutter frontend (hot reload on lib/ changes)
echo.

:: Kill old instances
taskkill /IM flowforge.exe /F >nul 2>&1

:: Start Rust with cargo-watch in a new window
echo [1/2] Starting Rust backend with auto-restart...
start "FlowForge-Rust" cmd /c "cd /d %~dp0 && set RUST_LOG=flowforge=info && cargo watch -x run"
timeout /t 8 /nobreak >nul

:: Verify backend
curl -s http://127.0.0.1:19529/api/health >nul 2>&1
if %ERRORLEVEL% NEQ 0 (
    echo  Waiting for backend...
    timeout /t 5 /nobreak >nul
)
echo  Backend ready

:: Flutter env
echo [2/2] Starting Flutter desktop...
echo.
set PUB_HOSTED_URL=https://pub.flutter-io.cn
set FLUTTER_STORAGE_BASE_URL=https://storage.flutter-io.cn
set SERVER_URL=http://127.0.0.1:19529
cd /d "%~dp0flutter_app"
echo  Commands: r=hot reload  R=restart  q=quit
echo.
flutter run -d windows --dart-define=SERVER_URL=http://127.0.0.1:19529
