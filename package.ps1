# FlowForge Windows Packaging Script
# Builds a self-contained distribution folder and zip package.

param(
    [string]$OutputDir = "C:\Users\haozi\dev\flowforge\dist",
    [switch]$SkipBuild
)

$ErrorActionPreference = "Stop"
$base = "C:\Users\haozi\dev\flowforge"
$version = "0.1.0"

Write-Host ""
Write-Host "  FlowForge Packager v$version" -ForegroundColor Cyan
Write-Host "  ════════════════════════════" -ForegroundColor Cyan
Write-Host ""

# Step 1: Build Rust backend (release)
if (-not $SkipBuild) {
    Write-Host "[1/4] Building Rust backend..." -ForegroundColor Yellow
    Set-Location $base
    cargo build --release 2>&1 | Out-Null
    if ($LASTEXITCODE -ne 0) { Write-Error "Rust build failed"; exit 1 }
    Write-Host "  OK" -ForegroundColor Green

    # Step 2: Build Flutter frontend (release)
    Write-Host "[2/4] Building Flutter frontend..." -ForegroundColor Yellow
    Set-Location "$base\flutter_app"
    $env:PUB_HOSTED_URL = "https://pub.flutter-io.cn"
    $env:FLUTTER_STORAGE_BASE_URL = "https://storage.flutter-io.cn"
    flutter build windows --release 2>&1 | Out-Null
    if ($LASTEXITCODE -ne 0) { Write-Error "Flutter build failed"; exit 1 }
    Write-Host "  OK" -ForegroundColor Green
} else {
    Write-Host "[1/4] Skipping builds (--SkipBuild)" -ForegroundColor DarkGray
    Write-Host "[2/4] Skipping builds (--SkipBuild)" -ForegroundColor DarkGray
}

# Step 3: Assemble distribution folder
Write-Host "[3/4] Assembling distribution..." -ForegroundColor Yellow

# Clean and create dist folder
if (Test-Path $OutputDir) { Remove-Item $OutputDir -Recurse -Force }
New-Item -ItemType Directory -Path $OutputDir -Force | Out-Null

# Copy Flutter app (all files)
$flutterRelease = "$base\flutter_app\build\windows\x64\runner\Release"
Copy-Item "$flutterRelease\*" -Destination $OutputDir -Recurse -Force

# Copy Rust backend into a subfolder
New-Item -ItemType Directory -Path "$OutputDir\backend" -Force | Out-Null
Copy-Item "$base\target\release\flowforge.exe" -Destination "$OutputDir\backend\" -Force

# Create launcher script
$launcher = @"
@echo off
title FlowForge v$version
echo.
echo  FlowForge v$version
echo  ══════════════════
echo.

:: Start backend in background
echo Starting backend...
start /B "" "%~dp0backend\flowforge.exe"
timeout /t 2 /nobreak >nul

:: Verify backend
curl -s http://127.0.0.1:19529/api/health >nul 2>&1
if %ERRORLEVEL% NEQ 0 (
    echo Waiting for backend...
    timeout /t 3 /nobreak >nul
)

:: Launch frontend
echo Starting FlowForge...
set SERVER_URL=http://127.0.0.1:19529
start "" "%~dp0flowforge.exe"

:: Exit launcher (backend stays running)
exit
"@
Set-Content -Path "$OutputDir\FlowForge.bat" -Value $launcher -Encoding ASCII

# Create README
$readme = @"
FlowForge v$version
═══════════════════

Visual Workflow Automation Engine

Quick Start:
  1. Double-click FlowForge.bat
  2. The app opens automatically

Manual Start:
  1. Run backend\flowforge.exe to start the server
  2. Run flowforge.exe to open the UI

Backend API: http://127.0.0.1:19529
Keyboard Shortcuts:
  Ctrl+S      Save workflow
  Ctrl+Enter  Execute workflow

Architecture:
  - Rust backend (axum) handles workflow storage and execution
  - Flutter frontend provides the visual editor
  - Communication via HTTP REST API
"@
Set-Content -Path "$OutputDir\README.txt" -Value $readme -Encoding ASCII

# Step 4: Create zip package
Write-Host "[4/4] Creating zip package..." -ForegroundColor Yellow
$zipPath = "$base\flowforge-v$version-windows-x64.zip"
if (Test-Path $zipPath) { Remove-Item $zipPath -Force }
Compress-Archive -Path "$OutputDir\*" -DestinationPath $zipPath -CompressionLevel Optimal

$zipSize = (Get-Item $zipPath).Length / 1MB
Write-Host "  OK" -ForegroundColor Green

# Summary
Write-Host ""
Write-Host "  ╔══════════════════════════════════════╗" -ForegroundColor Green
Write-Host "  ║  Package created successfully!        ║" -ForegroundColor Green
Write-Host "  ╠══════════════════════════════════════╣" -ForegroundColor Green
Write-Host "  ║  Folder: $OutputDir" -ForegroundColor Green
Write-Host "  ║  Zip:    $zipPath" -ForegroundColor Green
Write-Host "  ║  Size:   $([math]::Round($zipSize, 1)) MB" -ForegroundColor Green
Write-Host "  ╚══════════════════════════════════════╝" -ForegroundColor Green
Write-Host ""
