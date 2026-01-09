Add-Type -AssemblyName System.Drawing

$sourcePath = "c:\Users\Danie\Documents\GitHub\de.anesda.audiomultiverse\remote\src-tauri\icons\128x128.png"
$resDir = "c:\Users\Danie\Documents\GitHub\de.anesda.audiomultiverse\remote\src-tauri\gen\android\app\src\main\res"

$sizes = @{
    "mipmap-mdpi" = 48
    "mipmap-hdpi" = 72
    "mipmap-xhdpi" = 96
    "mipmap-xxhdpi" = 144
    "mipmap-xxxhdpi" = 192
}

foreach ($folder in $sizes.Keys) {
    $size = $sizes[$folder]
    $destFolder = Join-Path $resDir $folder
    
    if (-not (Test-Path $destFolder)) {
        New-Item -ItemType Directory -Path $destFolder -Force | Out-Null
    }
    
    $img = [System.Drawing.Image]::FromFile($sourcePath)
    $newImg = New-Object System.Drawing.Bitmap($size, $size)
    $graphics = [System.Drawing.Graphics]::FromImage($newImg)
    $graphics.InterpolationMode = [System.Drawing.Drawing2D.InterpolationMode]::HighQualityBicubic
    $graphics.DrawImage($img, 0, 0, $size, $size)
    
    $launcherPath = Join-Path $destFolder "ic_launcher.png"
    $roundPath = Join-Path $destFolder "ic_launcher_round.png"
    
    $newImg.Save($launcherPath, [System.Drawing.Imaging.ImageFormat]::Png)
    $newImg.Save($roundPath, [System.Drawing.Imaging.ImageFormat]::Png)
    
    $graphics.Dispose()
    $newImg.Dispose()
    $img.Dispose()
    
    Write-Host "Icon ${size}x${size} erstellt: $folder"
}

Write-Host "`nAlle Android-Icons wurden generiert!"
