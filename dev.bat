@echo off
title FlowForge - Flutter Hot Reload
echo.
echo  ╔══════════════════════════════════════╗
echo  ║        FlowForge Dev Launcher        ║
echo  ╚══════════════════════════════════════╝
echo.

:: Check backend
echo [1/3] Checking backend at 127.0.0.1:19529...
curl -s http://127.0.0.1:19529/api/health >nul 2>&1
if %ERRORLEVEL% NEQ 0 (
    echo.
    echo  ERROR: Backend not running!
    echo  Start it first:  cd target\release ^& flowforge.exe
    echo.
    pause
    exit /b 1
)
echo  Backend OK

:: Set Flutter environment
echo [2/3] Configuring Flutter...
set PUB_HOSTED_URL=https://pub.flutter-io.cn
set FLUTTER_STORAGE_BASE_URL=https://storage.flutter-io.cn
set SERVER_URL=http://127.0.0.1:19529

:: Launch Flutter
echo [3/3] Starting Flutter desktop app with hot reload...
echo.
echo  Hot reload commands:
echo    r = Hot reload (instant)
echo    R = Hot restart (full)
echo    q = Quit
echo.
cd /d "%~dp0flutter_app"
flutter run -d windows --dart-define=SERVER_URL=http://127.0.0.1:19529
