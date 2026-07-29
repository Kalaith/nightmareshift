<#
.SYNOPSIS
    Headless screenshot harness for Nightmare Shift.

.DESCRIPTION
    Thin wrapper around the shared macroquad-toolkit capture script. Builds the
    debug exe and drives it through the env-var capture hook
    (NIGHTMARE_SHIFT_CAPTURE_*) provided by macroquad_toolkit::capture in
    src/main.rs. Scenes are seeded via Game::begin_capture_scene:
    "mainmenu" (default boot state), "briefing" (post-start briefing screen),
    "gameplay" (in a shift, waiting for a passenger), "ride_request" (a ride
    offer with the almanac fully studied, so the passenger dossier shows),
    "skill_tree" (currency in hand, so purchase buttons and the lore exchange
    are live), "trade" (a trade offer with a mixed inventory), and
    "rules_panel" (the mid-ride rules list, showing each rule's reason).

    Capture scenes seed bank balance, lore, almanac levels and inventory
    directly into player stats. Game::capture_mode suppresses saving for the
    whole process, so a capture can never write those values to a real save.

.EXAMPLE
    ./scripts/capture_ui.ps1
    ./scripts/capture_ui.ps1 -Frames 60 -SkipBuild
#>
param(
    [string[]]$Scenes = @("mainmenu", "briefing", "gameplay"),
    [int]$Frames = 150,
    [string]$OutputDir = "docs\verification",
    [switch]$SkipBuild
)

$ErrorActionPreference = "Stop"
$gameDir = Split-Path -Parent $PSScriptRoot
$shared = Join-Path (Split-Path -Parent $gameDir) "macroquad-toolkit\scripts\capture_ui.ps1"

& $shared -GameDir $gameDir -Scenes $Scenes -Frames $Frames -OutputDir $OutputDir -SkipBuild:$SkipBuild
