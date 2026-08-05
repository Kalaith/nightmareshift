<#
.SYNOPSIS
    Generate Nightmare Shift's original procedural WAV library.

.DESCRIPTION
    Writes deterministic 16-bit mono WAVs. The short library is deliberately
    synthetic: dashboard hum, rain hiss, tension pulse, and supernatural
    stingers. Re-running this script produces byte-identical assets.
#>
param([string]$OutputDir = "assets\sounds")

$ErrorActionPreference = "Stop"
$projectRoot = Split-Path -Parent $PSScriptRoot
$out = Join-Path $projectRoot $OutputDir
New-Item -ItemType Directory -Force -Path $out | Out-Null
$sampleRate = 22050

function Write-Wave([string]$Name, [double]$Seconds, [scriptblock]$Sample) {
    $count = [int]($sampleRate * $Seconds)
    $path = Join-Path $out $Name
    $stream = [System.IO.File]::Create($path)
    $writer = [System.IO.BinaryWriter]::new($stream)
    try {
        $dataBytes = $count * 2
        $writer.Write([Text.Encoding]::ASCII.GetBytes("RIFF"))
        $writer.Write([int](36 + $dataBytes))
        $writer.Write([Text.Encoding]::ASCII.GetBytes("WAVEfmt "))
        $writer.Write([int]16); $writer.Write([int16]1); $writer.Write([int16]1)
        $writer.Write([int]$sampleRate); $writer.Write([int]($sampleRate * 2))
        $writer.Write([int16]2); $writer.Write([int16]16)
        $writer.Write([Text.Encoding]::ASCII.GetBytes("data")); $writer.Write([int]$dataBytes)
        for ($i = 0; $i -lt $count; $i++) {
            $t = $i / [double]$sampleRate
            $value = & $Sample $t $i $count
            $writer.Write([int16]([Math]::Round([Math]::Max(-1, [Math]::Min(1, $value)) * 32767)))
        }
    } finally { $writer.Dispose(); $stream.Dispose() }
    Write-Output "[AUDIO] $path"
}

$tau = 2 * [Math]::PI
Write-Wave "engine_ambience.wav" 4.0 { param($t,$i,$n) 0.13*[Math]::Sin($tau*43*$t) + 0.04*[Math]::Sin($tau*86*$t) }
Write-Wave "rain_ambience.wav" 4.0 { param($t,$i,$n) $x=(($i*1103515245+12345)-band 0x7fffffff)/1073741824.0-1; 0.08*$x + 0.025*[Math]::Sin($tau*3100*$t) }
Write-Wave "tension_pulse.wav" 4.0 { param($t,$i,$n) $pulse=[Math]::Pow([Math]::Max(0,[Math]::Sin($tau*1.5*$t)),8); $pulse*(0.18*[Math]::Sin($tau*58*$t)+0.08*[Math]::Sin($tau*61*$t)) }
Write-Wave "warning.wav" 0.7 { param($t,$i,$n) $e=[Math]::Exp(-4*$t); $e*(0.34*[Math]::Sin($tau*(240-90*$t)*$t)+0.16*[Math]::Sin($tau*121*$t)) }
Write-Wave "violation.wav" 0.85 { param($t,$i,$n) $e=[Math]::Exp(-3*$t); $e*(0.42*[Math]::Sin($tau*(95+310*$t)*$t)+0.2*[Math]::Sin($tau*47*$t)) }
Write-Wave "ward.wav" 0.9 { param($t,$i,$n) $e=[Math]::Exp(-2.5*$t); $e*(0.28*[Math]::Sin($tau*(420+480*$t)*$t)+0.18*[Math]::Sin($tau*840*$t)) }
Write-Wave "brink.wav" 1.1 { param($t,$i,$n) $e=[Math]::Exp(-2*$t); $beat=[Math]::Pow([Math]::Max(0,[Math]::Sin($tau*2.2*$t)),6); $e*$beat*0.55*[Math]::Sin($tau*52*$t) }
Write-Wave "meltdown.wav" 1.5 { param($t,$i,$n) $e=1-$t/1.5; $x=(($i*1664525+1013904223)-band 0x7fffffff)/1073741824.0-1; $e*(0.25*$x+0.42*[Math]::Sin($tau*(70-25*$t)*$t)) }
Write-Wave "success.wav" 1.2 { param($t,$i,$n) $e=[Math]::Min(1,$t*8)*[Math]::Exp(-1.2*$t); $e*(0.22*[Math]::Sin($tau*330*$t)+0.18*[Math]::Sin($tau*495*$t)+0.12*[Math]::Sin($tau*660*$t)) }
