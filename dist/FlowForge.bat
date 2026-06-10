@echo off
title FlowForge v0.1.0
echo.
echo FlowForge v0.1.0
echo.
start /B "" "%~dp0backend\flowforge.exe"
timeout /t 2 /nobreak >nul
set SERVER_URL=http://127.0.0.1:19529
start "" "%~dp0flowforge.exe"
exit
