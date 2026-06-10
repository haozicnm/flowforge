#!/usr/bin/env bash
SCRIPT_DIR=""
echo "FlowForge v1.0.0 — Starting backend..."
"/backend/flowforge" &
BACKEND_PID=
sleep 1
echo "Backend: http://127.0.0.1:19529"
echo "Press Ctrl+C to stop"
trap "kill  2>/dev/null; exit" INT TERM
wait 
