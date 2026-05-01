param(
    [int]$Shifts = 3,
    [ValidateSet("coverage", "conservative", "learned")]
    [string]$Strategy = "coverage",
    [ValidateRange(0, 3)]
    [int]$AlmanacLevel = 0,
    [int]$DelayMs = 150,
    [switch]$Release
)

$ErrorActionPreference = "Stop"
$ProjectRoot = Split-Path -Parent $PSScriptRoot

Push-Location $ProjectRoot
try {
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
    Pop-Location
}
