<#
.SYNOPSIS
    Build compact review contact sheets from the raw viewport matrix.

.DESCRIPTION
    The raw captures are intentionally ignored because they are large and
    reproducible. This script preserves every inspected scene in four compact,
    labelled JPEG sheets suitable for review and version control.
#>
param(
    [string]$MatrixDir = "docs\verification\review-matrix"
)

$ErrorActionPreference = "Stop"
Add-Type -AssemblyName System.Drawing

$gameDir = Split-Path -Parent $PSScriptRoot
$root = Join-Path $gameDir $MatrixDir
$columns = 4
$sheetWidth = 1200
$tileWidth = [int]($sheetWidth / $columns)
$imageHeight = 220
$labelHeight = 29
$tileHeight = $imageHeight + $labelHeight

foreach ($viewportDir in Get-ChildItem -LiteralPath $root -Directory | Sort-Object Name) {
    $captures = @(Get-ChildItem -LiteralPath $viewportDir.FullName -Filter "ui_*.png" | Sort-Object Name)
    if ($captures.Count -eq 0) {
        continue
    }

    $rows = [int][Math]::Ceiling($captures.Count / [double]$columns)
    $bitmap = [System.Drawing.Bitmap]::new($sheetWidth, $rows * $tileHeight)
    $graphics = [System.Drawing.Graphics]::FromImage($bitmap)
    $graphics.Clear([System.Drawing.Color]::FromArgb(12, 14, 14))
    $graphics.InterpolationMode = [System.Drawing.Drawing2D.InterpolationMode]::HighQualityBicubic
    $graphics.SmoothingMode = [System.Drawing.Drawing2D.SmoothingMode]::HighQuality
    $labelFont = [System.Drawing.Font]::new("Arial", 9.0)
    $labelBrush = [System.Drawing.SolidBrush]::new([System.Drawing.Color]::FromArgb(220, 220, 210))

    try {
        for ($index = 0; $index -lt $captures.Count; $index++) {
            $column = $index % $columns
            $row = [int][Math]::Floor($index / $columns)
            $x = $column * $tileWidth
            $y = $row * $tileHeight
            $source = [System.Drawing.Image]::FromFile($captures[$index].FullName)
            try {
                $scale = [Math]::Min(($tileWidth - 4) / $source.Width, $imageHeight / $source.Height)
                $width = [int]($source.Width * $scale)
                $height = [int]($source.Height * $scale)
                $drawX = $x + [int](($tileWidth - $width) / 2)
                $drawY = $y + [int](($imageHeight - $height) / 2)
                $graphics.DrawImage($source, $drawX, $drawY, $width, $height)
            }
            finally {
                $source.Dispose()
            }

            $label = $captures[$index].BaseName -replace '^ui_', ''
            $graphics.DrawString($label, $labelFont, $labelBrush, $x + 4, $y + $imageHeight + 5)
        }

        $output = Join-Path $root "contact_$($viewportDir.Name).jpg"
        $codec = [System.Drawing.Imaging.ImageCodecInfo]::GetImageEncoders() |
            Where-Object MimeType -eq "image/jpeg"
        $quality = [System.Drawing.Imaging.EncoderParameters]::new(1)
        $quality.Param[0] = [System.Drawing.Imaging.EncoderParameter]::new(
            [System.Drawing.Imaging.Encoder]::Quality,
            84L
        )
        try {
            $bitmap.Save($output, $codec, $quality)
        }
        finally {
            $quality.Dispose()
        }
        Write-Host "Built $output ($($captures.Count) scenes)."
    }
    finally {
        $labelBrush.Dispose()
        $labelFont.Dispose()
        $graphics.Dispose()
        $bitmap.Dispose()
    }
}
