<#
.SYNOPSIS
    Measure whether meta-progression actually makes runs easier.

.DESCRIPTION
    Runs the playtest bot across four conditions — no progression, almanac
    only, skills only, and both — each from a cleared save so nothing carries
    between cells, and reports success rate, meltdown count and earnings.

    This answers a question the existing run-bot-almanac-sweep.ps1 does not:
    that script looks for the first almanac level that can win a single shift,
    which is a smoke test. This one compares rates across many shifts, so a
    regression that quietly flattens progression shows up as numbers rather
    than as a pass.

    The bot's `learned` strategy only consults route preferences from almanac
    level 2, so levels 0 and 1 read the same here. That is a limit of the bot,
    not of the almanac: level 1 gives a human the need type and its thresholds
    on the ride request, which the bot never looks at.

    The existing save is backed up and restored.

.EXAMPLE
    ./scripts/measure-progression.ps1
    ./scripts/measure-progression.ps1 -Shifts 40
#>
param(
    [int]$Shifts = 20,
    [ValidateSet("coverage", "conservative", "learned")]
    [string]$Strategy = "learned",
    [int]$DelayMs = 0
)

$ErrorActionPreference = "Stop"
$ProjectRoot = Split-Path -Parent $PSScriptRoot
$SavePath = Join-Path $env:LOCALAPPDATA "nightmare_shift\nightmare_shift_save.json"
$BackupPath = Join-Path $env:TEMP ("nightmare_shift_save_before_measure_{0}.json" -f (Get-Date -Format "yyyyMMddHHmmss"))
$HadSave = Test-Path -LiteralPath $SavePath

if ($HadSave) {
    Copy-Item -LiteralPath $SavePath -Destination $BackupPath -Force
    Write-Output "[MEASURE] Backed up save to $BackupPath"
}

function Measure-Cell {
    param([string]$Label, [string[]]$ExtraArgs)

    if (Test-Path -LiteralPath $SavePath) {
        Remove-Item -LiteralPath $SavePath -Force
    }

    $cargoArgs = @(
        "run", "--quiet", "--",
        "--bot",
        "--bot-shifts=$Shifts",
        "--bot-strategy=$Strategy",
        "--bot-delay-ms=$DelayMs"
    ) + $ExtraArgs

    $env:NIGHTMARE_SHIFT_BOT = "1"
    $output = & cargo @cargoArgs 2>&1 | ForEach-Object { $_.ToString() }
    if ($LASTEXITCODE -ne 0) {
        throw "Bot run failed for '$Label' with exit code $LASTEXITCODE"
    }

    $endings = $output | Where-Object { $_ -match 'ended on' }
    $successes = ($endings | Where-Object { $_ -match 'ended on Success' }).Count
    $meltdowns = ($endings | Where-Object { $_ -match 'need became uncontrollable' }).Count
    $earnings = 0
    foreach ($line in $endings) {
        if ($line -match 'earnings=\$(\d+)') { $earnings += [int]$Matches[1] }
    }
    $count = [Math]::Max($endings.Count, 1)

    [pscustomobject]@{
        Condition   = $Label
        Shifts      = $endings.Count
        Successes   = $successes
        SuccessRate = "{0:N0}%" -f (100 * $successes / $count)
        Meltdowns   = $meltdowns
        AvgEarnings = "`$$([Math]::Round($earnings / $count))"
    }
}

Push-Location $ProjectRoot
try {
    $results = @(
        Measure-Cell -Label "no progression"   -ExtraArgs @("--bot-almanac-level=0")
        Measure-Cell -Label "almanac only"     -ExtraArgs @("--bot-almanac-level=3")
        Measure-Cell -Label "skills only"      -ExtraArgs @("--bot-almanac-level=0", "--bot-all-skills")
        Measure-Cell -Label "almanac + skills" -ExtraArgs @("--bot-almanac-level=3", "--bot-all-skills")
    )
    $results | Format-Table -AutoSize

    $baseline = $results | Where-Object { $_.Condition -eq "no progression" }
    $best = $results | Where-Object { $_.Condition -ne "no progression" } |
        Sort-Object { [int]$_.Successes } -Descending | Select-Object -First 1
    if ([int]$best.Successes -le [int]$baseline.Successes) {
        Write-Output "[MEASURE] WARNING: progression did not beat the baseline. Meta-progression may be disconnected."
    } else {
        Write-Output "[MEASURE] Progression beats the baseline ($($baseline.Successes) -> $($best.Successes) successes)."
    }
}
finally {
    Pop-Location
    Remove-Item Env:\NIGHTMARE_SHIFT_BOT -ErrorAction SilentlyContinue
    if ($HadSave) {
        Copy-Item -LiteralPath $BackupPath -Destination $SavePath -Force
        Write-Output "[MEASURE] Restored save from backup"
    } elseif (Test-Path -LiteralPath $SavePath) {
        Remove-Item -LiteralPath $SavePath -Force
        Write-Output "[MEASURE] Removed save created during the run"
    }
}
