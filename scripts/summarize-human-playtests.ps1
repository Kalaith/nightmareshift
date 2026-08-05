<#
.SYNOPSIS
    Validate and summarize the human-only review release gates.

.DESCRIPTION
    Reads the first-three-runs and blind/experienced session CSVs, rejects
    malformed categorical evidence, and writes a transparent pending/pass
    report. Empty templates are valid and produce a pending report.
#>
param(
    [string]$FirstRunsCsv = "docs\verification\first_three_runs_playtest.csv",
    [string]$SessionsCsv = "docs\verification\session_playtests.csv",
    [string]$Output = "docs\verification\human_playtest_report.md"
)

$ErrorActionPreference = "Stop"
$gameDir = Split-Path -Parent $PSScriptRoot

function Read-Rows([string]$Path) {
    $resolved = Join-Path $gameDir $Path
    if (-not (Test-Path -LiteralPath $resolved)) {
        throw "Missing human-playtest file: $resolved"
    }
    return @(Import-Csv -LiteralPath $resolved | Where-Object {
        $_.PSObject.Properties.Value -join "" -ne ""
    })
}

function Assert-Rating([object[]]$Rows, [string[]]$Fields, [string]$Label) {
    $allowed = @("yes", "partly", "no")
    foreach ($row in $Rows) {
        foreach ($field in $Fields) {
            $value = "$($row.$field)".Trim().ToLowerInvariant()
            if ($value -notin $allowed) {
                throw "$Label has invalid $field rating '$value'; use yes, partly, or no."
            }
        }
    }
}

function Assert-Required([object[]]$Rows, [string[]]$Fields, [string]$Label) {
    foreach ($row in $Rows) {
        foreach ($field in $Fields) {
            if (-not "$($row.$field)".Trim()) {
                throw "$Label has an incomplete row: $field is required."
            }
        }
    }
}

function Positive-Count([object[]]$Rows, [string]$Field) {
    return @($Rows | Where-Object {
        "$($_.$Field)".Trim().ToLowerInvariant() -in @("yes", "partly")
    }).Count
}

$first = Read-Rows $FirstRunsCsv
$sessions = Read-Rows $SessionsCsv
Assert-Required $first @(
    "tester_id", "build_commit", "run", "viewport", "input_method", "outcome",
    "predicted", "understood_afterward", "avoidable"
) "First-three-runs CSV"
Assert-Rating $first @("predicted", "understood_afterward", "avoidable") "First-three-runs CSV"
foreach ($row in $first) {
    if ([int]$row.run -notin 1..3) {
        throw "First-three-runs CSV run '$($row.run)' must be 1, 2, or 3."
    }
}
Assert-Required $sessions @(
    "session_id", "build_commit", "cohort", "campaign_attempt", "viewport",
    "input_method", "completed_run", "night_reached", "route_tradeoffs_explained",
    "need_changes_explained", "post_ride_summary_understood", "fuel_changed_decision",
    "comfort_value_visible", "audio_visual_equivalent_clear"
) "Session CSV"
Assert-Rating $sessions @(
    "route_tradeoffs_explained",
    "need_changes_explained",
    "post_ride_summary_understood",
    "fuel_changed_decision",
    "comfort_value_visible",
    "audio_visual_equivalent_clear"
) "Session CSV"
foreach ($row in $sessions) {
    $cohort = "$($row.cohort)".Trim().ToLowerInvariant()
    if ($cohort -notin @("first_time", "experienced")) {
        throw "Session CSV cohort '$cohort' must be first_time or experienced."
    }
    if ("$($row.completed_run)".Trim().ToLowerInvariant() -notin @("yes", "no")) {
        throw "Session CSV completed_run must be yes or no."
    }
    if ([int]$row.night_reached -notin 1..6) {
        throw "Session CSV night_reached '$($row.night_reached)' must be from 1 through 6."
    }
}

$completeTesters = @($first | Group-Object tester_id | Where-Object {
    $_.Name -and @($_.Group.run | Sort-Object -Unique).Count -ge 3
})
$firstGate = $completeTesters.Count -ge 5 -and $first.Count -ge 15
$blind = @($sessions | Where-Object { "$($_.cohort)".Trim().ToLowerInvariant() -eq "first_time" })
$experienced = @($sessions | Where-Object { "$($_.cohort)".Trim().ToLowerInvariant() -eq "experienced" })
$experiencedFull = @($experienced | Where-Object {
    "$($_.completed_run)".Trim().ToLowerInvariant() -eq "yes" -or [int]$_.night_reached -ge 6
})
$sessionGate = $blind.Count -ge 5 -and $experienced.Count -ge 3

$lines = @(
    "# Human playtest gate report",
    "",
    "Generated: $(Get-Date -Format 'yyyy-MM-dd HH:mm zzz')",
    "",
    "## First three runs",
    "",
    "- Recorded attempts: $($first.Count) / 15",
    "- Testers with three distinct attempts: $($completeTesters.Count) / 5",
    "- Predicted ending (yes/partly): $(Positive-Count $first 'predicted') / $($first.Count)",
    "- Understood afterward (yes/partly): $(Positive-Count $first 'understood_afterward') / $($first.Count)",
    "- Considered avoidable (yes/partly): $(Positive-Count $first 'avoidable') / $($first.Count)",
    "- Evidence gate: $(if ($firstGate) { 'PASS' } else { 'PENDING HUMAN ROWS' })",
    "",
    "## Blind and experienced sessions",
    "",
    "- Blind first-session rows: $($blind.Count) / 5",
    "- Experienced campaign rows: $($experienced.Count) / 3",
    "- Experienced runs reaching/completing night 6: $($experiencedFull.Count)",
    "- Route tradeoffs explained (yes/partly): $(Positive-Count $sessions 'route_tradeoffs_explained') / $($sessions.Count)",
    "- Need changes explained (yes/partly): $(Positive-Count $sessions 'need_changes_explained') / $($sessions.Count)",
    "- Post-ride summary understood (yes/partly): $(Positive-Count $sessions 'post_ride_summary_understood') / $($sessions.Count)",
    "- Fuel changed a decision (yes/partly): $(Positive-Count $experienced 'fuel_changed_decision') / $($experienced.Count)",
    "- Comfort value visible (yes/partly): $(Positive-Count $experienced 'comfort_value_visible') / $($experienced.Count)",
    "- Audio/visual equivalence clear (yes/partly): $(Positive-Count $sessions 'audio_visual_equivalent_clear') / $($sessions.Count)",
    "- Evidence gate: $(if ($sessionGate) { 'READY FOR QUALITATIVE REVIEW' } else { 'PENDING HUMAN ROWS' })",
    "",
    "## Overall",
    "",
    $(if ($firstGate -and $sessionGate) {
        "Required rows are present. Review verbatim notes and comprehension ratios before declaring the release gates passed."
    } else {
        "Pending external participants. No bot or screenshot rows have been substituted for human observations."
    })
)

$outputPath = Join-Path $gameDir $Output
$lines | Set-Content -LiteralPath $outputPath -Encoding utf8
Write-Host "Wrote $outputPath"
