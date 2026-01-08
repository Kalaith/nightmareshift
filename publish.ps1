# Nightmare Shift - Game Publishing Script
# Builds WebGL package and deploys to preview/production servers

param(
    [switch]$SkipBuild = $false,
    [switch]$WindowsOnly = $false,
    [switch]$WebGLOnly = $false,
    [switch]$DeployOnly = $false,
    [Alias('p')] [switch]$Production = $false,
    [switch]$DryRun = $false
)

$ErrorActionPreference = "Stop"
$ProjectRoot = $PSScriptRoot
$DistDir = Join-Path $ProjectRoot "dist"
$CargoToml = Join-Path $ProjectRoot "Cargo.toml"

# Deployment paths
$PreviewRoot = "H:\xampp\htdocs"
$ProductionRoot = "F:\WebHatchery"

# Parse project name from Cargo.toml
if (-not (Test-Path $CargoToml)) {
    Write-Error "Cargo.toml not found at: $CargoToml"
    exit 1
}

$CargoContent = Get-Content $CargoToml -Raw
if ($CargoContent -match 'name\s*=\s*"([^"]+)"') {
    $ProjectName = $matches[1]
} else {
    Write-Error "Could not parse project name from Cargo.toml"
    exit 1
}

$ProjectTitle = ($ProjectName -replace '_', ' ').ToUpper()

Write-Host "=== $ProjectTitle Publisher ===" -ForegroundColor Cyan
Write-Host "Project: $ProjectName"
Write-Host ""

# Determine deployment target
$DeployRoot = $PreviewRoot
$Environment = "Preview"
if ($Production) {
    $DeployRoot = $ProductionRoot
    $Environment = "Production"
}
$DeployDir = Join-Path $DeployRoot "games\$ProjectName"

Write-Host "Target: $Environment ($DeployDir)" -ForegroundColor Magenta
Write-Host ""

# Skip build and just deploy?
if ($DeployOnly) {
    Write-Host "Deploy-only mode: Skipping build, deploying existing files..." -ForegroundColor Yellow
    $SkipBuild = $true
}

# Calculate steps
$buildWindows = -not $WebGLOnly -and -not $DeployOnly
$buildWebGL = -not $WindowsOnly
$totalSteps = 3  # Clean + Deploy + Summary
if ($buildWindows) { $totalSteps += 2 }
if ($buildWebGL) { $totalSteps += 2 }
$currentStep = 0

# Step: Prepare dist folder
if (-not $DeployOnly) {
    $currentStep++
    Write-Host "[$currentStep/$totalSteps] Preparing dist folder..." -ForegroundColor Yellow
    if (Test-Path $DistDir) {
        Remove-Item $DistDir -Recurse -Force
    }
    New-Item -ItemType Directory -Path $DistDir -Force | Out-Null
}

# === WINDOWS BUILD ===
if ($buildWindows) {
    $currentStep++
    if (-not $SkipBuild) {
        Write-Host "[$currentStep/$totalSteps] Building Windows release..." -ForegroundColor Yellow
        cargo build --release
        if ($LASTEXITCODE -ne 0) {
            Write-Error "Windows build failed!"
            exit 1
        }
        Write-Host "Windows build complete!" -ForegroundColor Green
    } else {
        Write-Host "[$currentStep/$totalSteps] Skipping Windows build (using existing)" -ForegroundColor Gray
    }

    $currentStep++
    Write-Host "[$currentStep/$totalSteps] Packaging Windows build..." -ForegroundColor Yellow
    $WindowsPackageDir = Join-Path $DistDir "windows"
    New-Item -ItemType Directory -Path $WindowsPackageDir -Force | Out-Null

    $ExePath = Join-Path $ProjectRoot "..\target\release\$ProjectName.exe"
    if (-not (Test-Path $ExePath)) {
        Write-Error "Executable not found at: $ExePath"
        exit 1
    }
    Copy-Item $ExePath $WindowsPackageDir

    $AssetsPath = Join-Path $ProjectRoot "assets"
    if (Test-Path $AssetsPath) {
        Copy-Item $AssetsPath -Destination $WindowsPackageDir -Recurse
    }

    $WindowsZipPath = Join-Path $DistDir "${ProjectName}_windows.zip"
    Compress-Archive -Path "$WindowsPackageDir\*" -DestinationPath $WindowsZipPath -CompressionLevel Optimal
    Write-Host "Windows package created!" -ForegroundColor Green
}

# === WEBGL BUILD ===
if ($buildWebGL) {
    $currentStep++
    if (-not $SkipBuild) {
        Write-Host "[$currentStep/$totalSteps] Building WebGL release..." -ForegroundColor Yellow
        
        $targets = rustup target list --installed
        if ($targets -notcontains "wasm32-unknown-unknown") {
            Write-Host "Installing wasm32-unknown-unknown target..." -ForegroundColor Yellow
            rustup target add wasm32-unknown-unknown
        }

        cargo build --release --target wasm32-unknown-unknown
        if ($LASTEXITCODE -ne 0) {
            Write-Error "WebGL build failed!"
            exit 1
        }
        Write-Host "WebGL build complete!" -ForegroundColor Green
    } else {
        Write-Host "[$currentStep/$totalSteps] Skipping WebGL build (using existing)" -ForegroundColor Gray
    }

    $currentStep++
    Write-Host "[$currentStep/$totalSteps] Packaging WebGL build..." -ForegroundColor Yellow
    $WebGLPackageDir = Join-Path $DistDir "webgl"
    New-Item -ItemType Directory -Path $WebGLPackageDir -Force | Out-Null

    # Copy WASM file
    $WasmPath = Join-Path $ProjectRoot "..\target\wasm32-unknown-unknown\release\$ProjectName.wasm"
    if (-not (Test-Path $WasmPath)) {
        Write-Error "WASM file not found at: $WasmPath"
        exit 1
    }
    Copy-Item $WasmPath $WebGLPackageDir

    # Copy assets
    $AssetsPath = Join-Path $ProjectRoot "assets"
    if (Test-Path $AssetsPath) {
        Copy-Item $AssetsPath -Destination $WebGLPackageDir -Recurse
    }

    # Download mq_js_bundle.js
    $JsBundlePath = Join-Path $WebGLPackageDir "mq_js_bundle.js"
    Write-Host "Downloading mq_js_bundle.js..." -ForegroundColor Gray
    try {
        Invoke-WebRequest -Uri "https://not-fl3.github.io/miniquad-samples/mq_js_bundle.js" -OutFile $JsBundlePath
    } catch {
        Write-Warning "Could not download mq_js_bundle.js - will use CDN reference"
    }

    # Create WebGL zip (for itch.io uploads)
    $WebGLZipPath = Join-Path $DistDir "${ProjectName}_webgl.zip"
    Compress-Archive -Path "$WebGLPackageDir\*" -DestinationPath $WebGLZipPath -CompressionLevel Optimal
    Write-Host "WebGL package created!" -ForegroundColor Green
}

# === DEPLOY ===
$currentStep++
Write-Host ""
Write-Host "[$currentStep/$totalSteps] Deploying to $Environment..." -ForegroundColor Yellow

if ($DryRun) {
    Write-Host "[DRY-RUN] Would deploy to: $DeployDir" -ForegroundColor DarkYellow
} else {
    # Ensure deploy directory exists
    if (-not (Test-Path $DeployDir)) {
        New-Item -ItemType Directory -Path $DeployDir -Force | Out-Null
    }

    # Copy index.html from project root
    $IndexPath = Join-Path $ProjectRoot "index.html"
    if (Test-Path $IndexPath) {
        Copy-Item $IndexPath $DeployDir -Force
        Write-Host "  Copied: index.html" -ForegroundColor Gray
    } else {
        Write-Warning "index.html not found in project root!"
    }

    # Copy WebGL files
    $WebGLSourceDir = Join-Path $DistDir "webgl"
    if (Test-Path $WebGLSourceDir) {
        # Copy WASM
        $wasmFile = Join-Path $WebGLSourceDir "$ProjectName.wasm"
        if (Test-Path $wasmFile) {
            Copy-Item $wasmFile $DeployDir -Force
            Write-Host "  Copied: $ProjectName.wasm" -ForegroundColor Gray
        }
        
        # Copy mq_js_bundle.js
        $jsBundle = Join-Path $WebGLSourceDir "mq_js_bundle.js"
        if (Test-Path $jsBundle) {
            Copy-Item $jsBundle $DeployDir -Force
            Write-Host "  Copied: mq_js_bundle.js" -ForegroundColor Gray
        }
        
        # Copy assets folder
        $assetsDir = Join-Path $WebGLSourceDir "assets"
        if (Test-Path $assetsDir) {
            $destAssets = Join-Path $DeployDir "assets"
            if (Test-Path $destAssets) {
                Remove-Item $destAssets -Recurse -Force
            }
            Copy-Item $assetsDir -Destination $DeployDir -Recurse
            Write-Host "  Copied: assets/" -ForegroundColor Gray
        }
    }

    Write-Host "Deployed to: $DeployDir" -ForegroundColor Green
}

# Summary
$currentStep++
Write-Host ""
Write-Host "=== Complete ===" -ForegroundColor Cyan

if ($buildWindows) {
    $WindowsZipPath = Join-Path $DistDir "${ProjectName}_windows.zip"
    if (Test-Path $WindowsZipPath) {
        $WinSize = [math]::Round((Get-Item $WindowsZipPath).Length / 1MB, 2)
        Write-Host "Windows: $WindowsZipPath (${WinSize} MB)" -ForegroundColor Green
    }
}

if ($buildWebGL) {
    $WebGLZipPath = Join-Path $DistDir "${ProjectName}_webgl.zip"
    if (Test-Path $WebGLZipPath) {
        $WebSize = [math]::Round((Get-Item $WebGLZipPath).Length / 1MB, 2)
        Write-Host "WebGL:   $WebGLZipPath (${WebSize} MB)" -ForegroundColor Green
    }
}

Write-Host ""
Write-Host "Deployed to: $DeployDir" -ForegroundColor Green
Write-Host ""
Write-Host "Options:" -ForegroundColor Yellow
Write-Host "  -SkipBuild    : Use existing builds"
Write-Host "  -WebGLOnly    : Build only WebGL version"
Write-Host "  -WindowsOnly  : Build only Windows version"
Write-Host "  -DeployOnly   : Just deploy existing builds"
Write-Host "  -Production   : Deploy to production (F:\WebHatchery)"
Write-Host "  -DryRun       : Show what would happen"
Write-Host ""
Write-Host "Examples:" -ForegroundColor Cyan
Write-Host "  .\publish.ps1                    # Build all + deploy to preview"
Write-Host "  .\publish.ps1 -WebGLOnly         # Build WebGL + deploy to preview"
Write-Host "  .\publish.ps1 -Production        # Build all + deploy to production"
Write-Host "  .\publish.ps1 -DeployOnly -p     # Deploy existing to production"
Write-Host ""

# Open dist folder if we built something
if (-not $DeployOnly -and (Test-Path $DistDir)) {
    # explorer $DistDir
}
