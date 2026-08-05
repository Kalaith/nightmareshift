<#
.SYNOPSIS
    Check a published WebGL deployment is wired correctly.

.DESCRIPTION
    Verifies the four things that can be wrong about a deploy without the
    build being wrong, all of which are silent failures in a browser:

      1. Every file the page needs is served, and the wasm arrives as
         application/wasm - browsers refuse to stream-instantiate anything
         else.
      2. The shared runtime under games/shared-assets/runtime is reachable
         from the game's own directory.
      3. Every storage import the wasm asks for is registered by storage.js.
         mq_js_bundle stubs missing imports rather than failing, so a game
         whose bridge does not match saves into nothing and looks fine doing
         it. This catalogue has shipped that bug before.
      4. storage.js loads after sapp_jsutils.js (whose helpers it calls) and
         before the wasm (which it must register a plugin for).

    It does NOT verify the game runs. Creating a WebGL context, initialising
    macroquad and surviving the first frame need a real browser; nothing here
    executes JavaScript. A pass means the deployment is correctly assembled,
    not that it plays.

.EXAMPLE
    ./scripts/verify-deployment.ps1
    ./scripts/verify-deployment.ps1 -BaseUrl http://127.0.0.1/games -Slug nightmare_shift
#>
param(
    [string]$BaseUrl = "http://127.0.0.1/games",
    [string]$Slug = "nightmare_shift",
    [string]$DeployRoot = "D:\xampp\htdocs\games"
)

$ErrorActionPreference = "Stop"
$failures = @()

function Test-Served {
    param([string]$Url, [string]$ExpectedType)

    try {
        $response = Invoke-WebRequest -Uri $Url -Method Head -UseBasicParsing -TimeoutSec 20
    } catch {
        $script:failures += "$Url did not respond: $($_.Exception.Message)"
        return
    }

    if ($response.StatusCode -ne 200) {
        $script:failures += "$Url returned $($response.StatusCode)"
        return
    }
    if ($ExpectedType) {
        $actual = $response.Headers['Content-Type']
        if ($actual -notlike "*$ExpectedType*") {
            $script:failures += "$Url served as '$actual', expected '$ExpectedType'"
            return
        }
    }
    Write-Output "  ok  $Url"
}

Write-Output "[1/4] Game files"
Test-Served "$BaseUrl/$Slug/" "text/html"
Test-Served "$BaseUrl/$Slug/$Slug.wasm" "application/wasm"

Write-Output "[2/4] Shared runtime"
foreach ($asset in @(
    "shared.css",
    "shared-assets/runtime/mq_js_bundle.js",
    "shared-assets/runtime/sapp_jsutils.js",
    "shared-assets/runtime/storage.js"
)) {
    Test-Served "$BaseUrl/$asset" ""
}

Write-Output "[3/4] Storage bridge covers the wasm's imports"
$wasmPath = Join-Path $DeployRoot "$Slug\$Slug.wasm"
$storagePath = Join-Path $DeployRoot "shared-assets\runtime\storage.js"
if ((Test-Path $wasmPath) -and (Test-Path $storagePath)) {
    $wasmText = [System.Text.Encoding]::ASCII.GetString([System.IO.File]::ReadAllBytes($wasmPath))
    $storageText = Get-Content $storagePath -Raw
    $registered = [regex]::Matches($storageText, 'importObject\.env\.(\w+)') |
        ForEach-Object { $_.Groups[1].Value }

    foreach ($import in @("storage_set_extern", "storage_get_extern", "storage_remove_extern", "storage_exists_extern")) {
        if ($wasmText.Contains($import)) {
            if ($registered -contains $import) {
                Write-Output "  ok  $import"
            } else {
                $failures += "the wasm imports $import and storage.js does not register it - saves would be silently discarded"
            }
        }
    }
} else {
    $failures += "could not read the wasm or storage.js under $DeployRoot"
}

Write-Output "[4/4] Script load order"
$indexPath = Join-Path $DeployRoot "$Slug\index.html"
if (Test-Path $indexPath) {
    $index = Get-Content $indexPath -Raw
    # Match the script tags, not the bare filenames. The generated page carries
    # a comment naming storage.js one line above the sapp_jsutils tag, and a
    # plain IndexOf reads that comment as storage.js loading first.
    $order = @{}
    foreach ($name in @("sapp_jsutils.js", "storage.js")) {
        # Published runtime URLs carry a cache-busting query string. Match the
        # script filename plus an optional query before the closing quote.
        $pattern = '<script[^>]*src="[^"]*' + [regex]::Escape($name) + '(?:\?[^"]*)?"'
        $tag = [regex]::Match($index, $pattern)
        if ($tag.Success) { $order[$name] = $tag.Index } else { $order[$name] = -1 }
    }
    $wasmLoad = $index.IndexOf("load(""$Slug.wasm"")")

    if ($order["sapp_jsutils.js"] -lt 0 -or $order["storage.js"] -lt 0 -or $wasmLoad -lt 0) {
        $failures += "index.html does not reference sapp_jsutils.js, storage.js and the wasm load"
    } elseif ($order["storage.js"] -lt $order["sapp_jsutils.js"]) {
        $failures += "storage.js loads before sapp_jsutils.js, whose helpers it calls"
    } elseif ($wasmLoad -lt $order["storage.js"]) {
        $failures += "the wasm loads before storage.js registers its plugin"
    } else {
        Write-Output "  ok  sapp_jsutils.js -> storage.js -> wasm"
    }
} else {
    $failures += "no index.html at $indexPath"
}

Write-Output ""
if ($failures.Count -gt 0) {
    foreach ($failure in $failures) { Write-Output "FAIL: $failure" }
    exit 1
}
Write-Output "Deployment is correctly assembled. This does not prove it runs - that needs a browser."
