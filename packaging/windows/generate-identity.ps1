# Reproduce the original Xiri wordmark as vector outlines and PNG, without runtime fonts.
$ErrorActionPreference = 'Stop'
Add-Type -AssemblyName System.Drawing
$identityRoot = (Resolve-Path (Join-Path $PSScriptRoot '../../assets/xiri')).Path
$identityFont = [Drawing.FontFamily]::new('Arial')
$identityPath = [Drawing.Drawing2D.GraphicsPath]::new()
try {
    $identityPath.AddString('Xiri Launcher', $identityFont, 1, 24, [Drawing.PointF]::new(36, -1), [Drawing.StringFormat]::GenericDefault)
    $identityPoints = $identityPath.PathPoints
    $identityTypes = $identityPath.PathTypes
    $identitySvg = [Text.StringBuilder]::new()
    function Point-Text($point) {
        $point.X.ToString('0.###', [Globalization.CultureInfo]::InvariantCulture) + ',' + $point.Y.ToString('0.###', [Globalization.CultureInfo]::InvariantCulture)
    }
    for ($i=0; $i -lt $identityPoints.Count; $i++) {
        $kind = $identityTypes[$i] -band 7
        switch ($kind) {
            0 { [void]$identitySvg.Append('M' + (Point-Text $identityPoints[$i])) }
            1 { [void]$identitySvg.Append('L' + (Point-Text $identityPoints[$i])) }
            3 {
                [void]$identitySvg.Append('C' + (Point-Text $identityPoints[$i]) + ' ' + (Point-Text $identityPoints[$i+1]) + ' ' + (Point-Text $identityPoints[$i+2]))
                $i += 2
            }
        }
        if (($identityTypes[$i] -band 128) -ne 0) { [void]$identitySvg.Append('Z') }
    }
    foreach ($spec in @(@('logo', '#151518', [Drawing.Color]::FromArgb(21,21,24)), @('logo-white', '#ffffff', [Drawing.Color]::White))) {
        $svg = '<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 250 28"><path stroke="' + $spec[1] + '" stroke-width="5" d="m3 4 20 20m0-20L3 24"/><path fill="' + $spec[1] + '" d="' + $identitySvg.ToString() + '"/></svg>'
        [IO.File]::WriteAllText((Join-Path $identityRoot ($spec[0]+'.svg')), $svg, [Text.UTF8Encoding]::new($false))
        $bitmap = [Drawing.Bitmap]::new(1000,112)
        $graphics = [Drawing.Graphics]::FromImage($bitmap)
        $brush = [Drawing.SolidBrush]::new($spec[2])
        $pen = [Drawing.Pen]::new($spec[2],5)
        try {
            $graphics.Clear([Drawing.Color]::Transparent)
            $graphics.SmoothingMode = [Drawing.Drawing2D.SmoothingMode]::AntiAlias
            $graphics.ScaleTransform(4,4)
            $graphics.DrawLine($pen,3,4,23,24)
            $graphics.DrawLine($pen,23,4,3,24)
            $graphics.FillPath($brush,$identityPath)
            $bitmap.Save((Join-Path $identityRoot ($spec[0]+'.png')),[Drawing.Imaging.ImageFormat]::Png)
        } finally { $pen.Dispose();$brush.Dispose();$graphics.Dispose();$bitmap.Dispose() }
    }
} finally { $identityPath.Dispose();$identityFont.Dispose() }
