param(
    [int]$Shifts = 3,
    [ValidateSet("coverage", "conservative", "learned")]
    [string]$Strategy = "coverage",
    [ValidateRange(0, 3)]
    [int]$AlmanacLevel = 0,
    [int]$DelayMs = 150,
    [switch]$Release,
    # The bot plays itself and reports on stdout, so the game window is noise —
    # it is hidden by default. Pass -Visible to watch the bot play.
    [switch]$Visible
)

$ErrorActionPreference = "Stop"
$ProjectRoot = Split-Path -Parent $PSScriptRoot

Push-Location $ProjectRoot
try {
    $env:NIGHTMARE_SHIFT_HEADLESS = if ($Visible) { "0" } else { "1" }

    $cargoArgs = @("run")
    if ($Release) {
        $cargoArgs += "--release"
    }

    $cargoArgs += @(
        "--",
        "--bot",
        "--bot-shifts=$Shifts",
        "--bot-strategy=$Strategy",
        "--bot-almanac-level=$AlmanacLevel",
        "--bot-delay-ms=$DelayMs"
    )

    cargo @cargoArgs
    exit $LASTEXITCODE
}
finally {
    Remove-Item Env:NIGHTMARE_SHIFT_HEADLESS -ErrorAction SilentlyContinue
    Pop-Location
}
