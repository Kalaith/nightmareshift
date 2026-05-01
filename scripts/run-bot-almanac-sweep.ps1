param(
    [ValidateSet("coverage", "conservative", "learned")]
    [string]$Strategy = "learned",
    [int]$RunsPerLevel = 1,
    [int]$DelayMs = 10,
    [switch]$FreshStats
)

$ErrorActionPreference = "Stop"
$ProjectRoot = Split-Path -Parent $PSScriptRoot
$SavePath = Join-Path $env:LOCALAPPDATA "nightmare_shift\nightmare_shift_save.json"
$BackupPath = Join-Path $env:TEMP ("nightmare_shift_save_before_sweep_{0}.json" -f (Get-Date -Format "yyyyMMddHHmmss"))
$HadSave = Test-Path -LiteralPath $SavePath

if ($HadSave) {
    Copy-Item -LiteralPath $SavePath -Destination $BackupPath -Force
    Write-Output "[SWEEP] Backed up save to $BackupPath"
} else {
    Write-Output "[SWEEP] No existing save found"
}

Push-Location $ProjectRoot
try {
    $foundSuccess = $false

    foreach ($level in 0..3) {
        foreach ($run in 1..$RunsPerLevel) {
            if ($FreshStats) {
                if (Test-Path -LiteralPath $SavePath) {
                    Remove-Item -LiteralPath $SavePath -Force
                }
            } elseif ($HadSave) {
                Copy-Item -LiteralPath $BackupPath -Destination $SavePath -Force
            }

            Write-Output "=== ALMANAC LEVEL $level / RUN $run / STRATEGY $Strategy ==="
            $output = & cargo run -- --bot --bot-shifts=1 --bot-strategy=$Strategy --bot-almanac-level=$level --bot-delay-ms=$DelayMs 2>&1
            $exitCode = $LASTEXITCODE
            $lines = $output | ForEach-Object { $_.ToString() }
            $botLines = $lines | Where-Object { $_ -match '^\[BOT\]' }
            $botLines | ForEach-Object { Write-Output $_ }
            Write-Output "[SWEEP] level=$level run=$run exit=$exitCode"

            if ($exitCode -ne 0) {
                throw "Bot sweep failed at almanac level $level run $run with exit code $exitCode"
            }

            $summary = $botLines | Where-Object { $_ -match 'Shift 1 ended on' } | Select-Object -Last 1
            if ($summary -match 'ended on Success') {
                Write-Output "[SWEEP] First success at almanac level $level run $run"
                $foundSuccess = $true
                break
            }
        }

        if ($foundSuccess) {
            break
        }
    }

    if (-not $foundSuccess) {
        Write-Output "[SWEEP] No success found through almanac level 3"
    }
}
finally {
    Pop-Location
    if ($HadSave) {
        Copy-Item -LiteralPath $BackupPath -Destination $SavePath -Force
        Write-Output "[SWEEP] Restored save from backup"
    } elseif (Test-Path -LiteralPath $SavePath) {
        Remove-Item -LiteralPath $SavePath -Force
        Write-Output "[SWEEP] Removed bot-created save"
    }
}
