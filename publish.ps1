# Nightmare Shift Publish Script
# Builds and deploys the game for Windows and Web (WASM)

param(
    [switch]$Production,
    [switch]$SkipBuild
)

$ErrorActionPreference = "Stop"
$GameName = "nightmare_shift"

Write-Host "=== Nightmare Shift Publish Script ===" -ForegroundColor Cyan

# Paths
$ProjectRoot = $PSScriptRoot
$AssetsDir = Join-Path $ProjectRoot "assets"
$TargetDir = Join-Path $ProjectRoot "target"

# XAMPP preview paths (local testing)
$PreviewRoot = "C:\APPS\xampp\htdocs\webhatchery\games\$GameName"

# Production paths (if deploying to production)
$ProductionRoot = "\\server\webhatchery\games\$GameName"

if (-not $SkipBuild) {
    Write-Host "`n[1/4] Building Windows Release..." -ForegroundColor Yellow
    Push-Location $ProjectRoot
    cargo build --release
    if ($LASTEXITCODE -ne 0) {
        Write-Error "Windows build failed!"
        Pop-Location
        exit 1
    }
    Pop-Location
    Write-Host "Windows build complete!" -ForegroundColor Green

    Write-Host "`n[2/4] Building WASM Release..." -ForegroundColor Yellow
    Push-Location $ProjectRoot
    cargo build --release --target wasm32-unknown-unknown
    if ($LASTEXITCODE -ne 0) {
        Write-Error "WASM build failed!"
        Pop-Location
        exit 1
    }
    Pop-Location
    Write-Host "WASM build complete!" -ForegroundColor Green
} else {
    Write-Host "`n[1-2/4] Skipping builds..." -ForegroundColor Yellow
}

Write-Host "`n[3/4] Preparing deployment..." -ForegroundColor Yellow

# Determine target directory
if ($Production) {
    $DeployDir = $ProductionRoot
    Write-Host "Deploying to PRODUCTION: $DeployDir" -ForegroundColor Red
} else {
    $DeployDir = $PreviewRoot
    Write-Host "Deploying to Preview: $DeployDir" -ForegroundColor Cyan
}

# Create deploy directory if needed
if (-not (Test-Path $DeployDir)) {
    New-Item -ItemType Directory -Path $DeployDir -Force | Out-Null
}

# Copy WASM file
$WasmSource = Join-Path $TargetDir "wasm32-unknown-unknown\release\$GameName.wasm"
if (Test-Path $WasmSource) {
    Copy-Item $WasmSource -Destination $DeployDir -Force
    Write-Host "  Copied: $GameName.wasm" -ForegroundColor Gray
} else {
    Write-Warning "WASM file not found: $WasmSource"
}

# Copy index.html
$IndexSource = Join-Path $ProjectRoot "index.html"
if (Test-Path $IndexSource) {
    Copy-Item $IndexSource -Destination $DeployDir -Force
    Write-Host "  Copied: index.html" -ForegroundColor Gray
} else {
    Write-Warning "index.html not found"
}

# Copy mq_js_bundle.js (Miniquad loader)
$MqBundleSource = Join-Path $ProjectRoot "mq_js_bundle.js"
if (Test-Path $MqBundleSource) {
    Copy-Item $MqBundleSource -Destination $DeployDir -Force
    Write-Host "  Copied: mq_js_bundle.js" -ForegroundColor Gray
} else {
    # Try to find it in shared location
    $SharedMqBundle = "C:\APPS\xampp\htdocs\webhatchery\shared\mq_js_bundle.js"
    if (Test-Path $SharedMqBundle) {
        Copy-Item $SharedMqBundle -Destination $DeployDir -Force
        Write-Host "  Copied: mq_js_bundle.js (from shared)" -ForegroundColor Gray
    } else {
        Write-Warning "mq_js_bundle.js not found - WebGL loading may fail!"
    }
}

# Copy assets directory
if (Test-Path $AssetsDir) {
    $AssetsDestDir = Join-Path $DeployDir "assets"
    if (Test-Path $AssetsDestDir) {
        Remove-Item $AssetsDestDir -Recurse -Force
    }
    Copy-Item $AssetsDir -Destination $DeployDir -Recurse -Force
    Write-Host "  Copied: assets/" -ForegroundColor Gray
}

Write-Host "`n[4/4] Deployment complete!" -ForegroundColor Green

# Summary
Write-Host "`n=== Summary ===" -ForegroundColor Cyan
Write-Host "Game: $GameName"
Write-Host "Target: $DeployDir"

if (-not $Production) {
    Write-Host "`nPreview URL: http://localhost/webhatchery/games/$GameName/" -ForegroundColor Yellow
}

Write-Host "`nDone!" -ForegroundColor Green
