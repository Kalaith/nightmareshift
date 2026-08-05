<#
.SYNOPSIS
    Capture every review scene at all supported desktop-browser viewports.

.DESCRIPTION
    Builds once, then runs the deterministic capture harness for the complete
    interaction/state list at 1920x1080, 1600x900, 1366x768, and the narrow
    900x720 browser tier. Output is grouped by viewport under
    docs/verification/review-matrix.
#>
param(
    [switch]$SkipBuild,
    [int]$Frames = 150
)

$ErrorActionPreference = "Stop"
$gameDir = Split-Path -Parent $PSScriptRoot
$shared = Join-Path (Split-Path -Parent $gameDir) "macroquad-toolkit\scripts\capture_ui.ps1"
$scenes = @(
    "ui_core",
    "ui_status",
    "ui_passenger",
    "ui_completion",
    "mainmenu",
    "seed_entry",
    "briefing",
    "briefing_hazards",
    "gameplay",
    "refuelling",
    "ride_request",
    "driving",
    "driving_broke",
    "driving_blocked",
    "guideline",
    "event",
    "rules_panel",
    "inventory",
    "trade",
    "trade_done",
    "warded",
    "reaction_violation",
    "reaction_ward",
    "reaction_brink",
    "reaction_meltdown",
    "paused",
    "skill_tree",
    "almanac",
    "leaderboard",
    "help_options",
    "help_options_accessible",
    "delete_armed",
    "game_over",
    "night_complete",
    "run_complete"
)
$viewports = @(
    @{ Name = "1920x1080"; Width = 1920; Height = 1080 },
    @{ Name = "1600x900"; Width = 1600; Height = 900 },
    @{ Name = "1366x768"; Width = 1366; Height = 768 },
    @{ Name = "900x720"; Width = 900; Height = 720 }
)

$first = $true
foreach ($viewport in $viewports) {
    $output = "docs\verification\review-matrix\$($viewport.Name)"
    & $shared `
        -GameDir $gameDir `
        -Scenes $scenes `
        -Frames $Frames `
        -WindowWidth $viewport.Width `
        -WindowHeight $viewport.Height `
        -OutputDir $output `
        -SkipBuild:($SkipBuild -or -not $first)
    if ($LASTEXITCODE -ne 0) {
        throw "Capture failed for $($viewport.Name)."
    }
    $first = $false
}

$files = Get-ChildItem -LiteralPath (Join-Path $gameDir "docs\verification\review-matrix") -Recurse -Filter *.png
if ($files.Count -ne $scenes.Count * $viewports.Count) {
    throw "Expected $($scenes.Count * $viewports.Count) captures, found $($files.Count)."
}
Write-Host "Captured $($files.Count) review images across $($viewports.Count) viewports."
& (Join-Path $PSScriptRoot "build-review-contact-sheets.ps1")
if ($LASTEXITCODE -ne 0) {
    throw "Contact-sheet generation failed."
}
