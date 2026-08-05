<#
.SYNOPSIS
    Run the fixed-seed campaign balance matrix from the review resolution plan.

.DESCRIPTION
    Builds once, then plays each seed as one campaign in six progression tiers.
    The bot emits one JSON record per reached night. This script writes raw
    shift rows and a compact report containing campaign reach, outcome causes,
    route use, earnings/time distributions, and highest-fare share.

.EXAMPLE
    .\scripts\run-campaign-matrix.ps1
    .\scripts\run-campaign-matrix.ps1 -Seeds 1,2,3 -SkipBuild
#>
param(
    [int[]]$Seeds = (1..15),
    [string]$OutputDir = "docs\verification",
    [switch]$SkipBuild
)

$ErrorActionPreference = "Stop"
$projectRoot = Split-Path -Parent $PSScriptRoot
$outputRoot = Join-Path $projectRoot $OutputDir
$metadata = (& cargo metadata --no-deps --format-version 1 | ConvertFrom-Json)
if ($LASTEXITCODE -ne 0) { throw "Could not resolve Cargo target directory." }
$exe = Join-Path $metadata.target_directory "release\nightmare_shift.exe"

$tiers = @(
    @{ Name = "baseline"; Args = @("--bot-almanac-level=0") },
    @{ Name = "comfort"; Args = @("--bot-almanac-level=0", "--bot-skills=stereo_1,stereo_2,climate_1,climate_2,upholstery_1,upholstery_2") },
    @{ Name = "almanac-1"; Args = @("--bot-almanac-level=1") },
    @{ Name = "almanac-2"; Args = @("--bot-almanac-level=2") },
    @{ Name = "almanac-3"; Args = @("--bot-almanac-level=3") },
    @{ Name = "almanac-3-all-skills"; Args = @("--bot-almanac-level=3", "--bot-all-skills") }
)

function Get-Percentile([double[]]$Values, [double]$Fraction) {
    if ($Values.Count -eq 0) { return 0 }
    $ordered = @($Values | Sort-Object)
    $index = [Math]::Ceiling($Fraction * $ordered.Count) - 1
    $index = [Math]::Max(0, [Math]::Min($ordered.Count - 1, $index))
    return [Math]::Round($ordered[$index], 1)
}

function Format-Distribution([double[]]$Values) {
    if ($Values.Count -eq 0) { return "n/a" }
    $average = [Math]::Round(($Values | Measure-Object -Average).Average, 1)
    $median = Get-Percentile $Values 0.5
    $p10 = Get-Percentile $Values 0.1
    $p90 = Get-Percentile $Values 0.9
    return "avg $average; median $median; p10/p90 $p10/$p90"
}

Push-Location $projectRoot
try {
    New-Item -ItemType Directory -Force -Path $outputRoot | Out-Null
    if (-not $SkipBuild) {
        & cargo build --release
        if ($LASTEXITCODE -ne 0) { throw "Release build failed." }
    }
    if (-not (Test-Path -LiteralPath $exe)) { throw "Bot executable not found: $exe" }

    $env:NIGHTMARE_SHIFT_HEADLESS = "1"
    $records = [System.Collections.Generic.List[object]]::new()
    foreach ($tier in $tiers) {
        foreach ($seed in $Seeds) {
            Write-Output "[MATRIX] $($tier.Name), seed $seed"
            $args = @(
                "--bot", "--bot-campaigns=1", "--bot-strategy=learned",
                "--bot-delay-ms=0", "--bot-fresh-stats", "--seed=$seed"
            ) + $tier.Args
            $output = & $exe @args 2>&1 | ForEach-Object { $_.ToString() }
            if ($LASTEXITCODE -ne 0) {
                throw "Bot failed for $($tier.Name), seed $seed (exit $LASTEXITCODE)."
            }
            foreach ($line in $output) {
                if ($line -match '^\[BOT_JSON\]\s+(.+)$') {
                    $row = $Matches[1] | ConvertFrom-Json
                    $fares = @($row.fares | ForEach-Object { [int]$_.fare })
                    $highestShare = if ([int]$row.earnings -gt 0 -and $fares.Count -gt 0) {
                        [Math]::Round((($fares | Measure-Object -Maximum).Maximum / [double]$row.earnings), 4)
                    } else { 0 }
                    $records.Add([pscustomobject]@{
                        Tier = $tier.Name; Seed = $seed; Night = [int]$row.night
                        CampaignComplete = [bool]$row.campaign_complete
                        Modifier = $row.modifier; Rides = [int]$row.rides
                        Earnings = [int]$row.earnings; Quota = [int]$row.quota
                        FuelEnd = [double]$row.fuel_end; TimeEnd = [int]$row.time_end
                        WardsEnd = [int]$row.wards_end; FailureCause = $row.failure_cause
                        Reason = $row.reason; HighestFareShare = $highestShare
                        Routes = (@($row.routes) -join ';')
                        Fares = (@($row.fares | ForEach-Object { "$($_.passenger_id):$($_.fare)" }) -join ';')
                    })
                }
            }
        }
    }

    $csvPath = Join-Path $outputRoot "campaign_matrix_shifts.csv"
    $records | Export-Csv -NoTypeInformation -Encoding UTF8 -Path $csvPath

    $report = [System.Collections.Generic.List[string]]::new()
    $report.Add("# Fixed-seed campaign matrix")
    $report.Add("")
    $report.Add("Seeds: $($Seeds -join ', '). Generated: $(Get-Date -Format 'yyyy-MM-dd HH:mm zzz').")
    $report.Add("")
    $report.Add("| Tier | Campaigns | Complete | Reach N2/N3/N4/N5/N6 | Earnings | Time left | Highest fare share |")
    $report.Add("| --- | ---: | ---: | --- | --- | --- | --- |")
    foreach ($tier in $tiers) {
        $rows = @($records | Where-Object Tier -eq $tier.Name)
        $complete = @($rows | Where-Object CampaignComplete).Count
        $reach = 2..6 | ForEach-Object { @($rows | Where-Object Night -ge $_ | Select-Object -ExpandProperty Seed -Unique).Count }
        $earnings = Format-Distribution @($rows | ForEach-Object { [double]$_.Earnings })
        $time = Format-Distribution @($rows | ForEach-Object { [double]$_.TimeEnd })
        $share = Format-Distribution @($rows | ForEach-Object { 100.0 * [double]$_.HighestFareShare })
        $report.Add("| $($tier.Name) | $($Seeds.Count) | $complete | $($reach -join '/') | $earnings | $time | $share% |")
    }

    $report.Add("")
    $report.Add("## Results by night")
    $report.Add("")
    $report.Add("| Tier | Night | Reached | Success | Failure causes |")
    $report.Add("| --- | ---: | ---: | ---: | --- |")
    foreach ($tier in $tiers) {
        foreach ($night in 1..6) {
            $rows = @($records | Where-Object { $_.Tier -eq $tier.Name -and $_.Night -eq $night })
            if ($rows.Count -eq 0) { continue }
            $success = @($rows | Where-Object FailureCause -eq 'success').Count
            $causes = $rows | Where-Object FailureCause -ne 'success' | Group-Object FailureCause |
                Sort-Object Count -Descending | ForEach-Object { "$($_.Name) $($_.Count)" }
            $report.Add("| $($tier.Name) | $night | $($rows.Count) | $success | $($causes -join ', ') |")
        }
    }

    $report.Add("")
    $report.Add("## Route selection")
    $report.Add("")
    $report.Add("| Tier | Normal | Shortcut | Scenic | Police | Largest share |")
    $report.Add("| --- | ---: | ---: | ---: | ---: | ---: |")
    foreach ($tier in $tiers) {
        $routes = @($records | Where-Object Tier -eq $tier.Name | ForEach-Object { $_.Routes -split ';' } | Where-Object { $_ })
        $counts = @{}; foreach ($name in @('Normal','Shortcut','Scenic','Police')) { $counts[$name] = @($routes | Where-Object { $_ -eq $name }).Count }
        $total = [Math]::Max(1, $routes.Count)
        $shares = @('Normal','Shortcut','Scenic','Police') | ForEach-Object { [Math]::Round(100 * $counts[$_] / $total, 1) }
        $report.Add("| $($tier.Name) | $($counts.Normal) | $($counts.Shortcut) | $($counts.Scenic) | $($counts.Police) | $(($shares | Measure-Object -Maximum).Maximum)% |")
    }

    $reportPath = Join-Path $outputRoot "campaign_matrix_report.md"
    $report | Set-Content -Encoding UTF8 -Path $reportPath
    Write-Output "[MATRIX] Wrote $csvPath"
    Write-Output "[MATRIX] Wrote $reportPath"
}
finally {
    Pop-Location
    Remove-Item Env:\NIGHTMARE_SHIFT_HEADLESS -ErrorAction SilentlyContinue
}
