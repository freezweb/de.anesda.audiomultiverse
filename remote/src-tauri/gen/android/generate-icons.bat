@echo off
REM ================================================================
REM Icon-Generator für Android
REM Benötigt: ImageMagick (convert.exe im PATH)
REM ================================================================

set SOURCE=..\..\..\..\icons\128x128.png
set RES_DIR=app\src\main\res

echo Generiere Android Icons aus %SOURCE%...

REM mdpi (48x48)
magick convert "%SOURCE%" -resize 48x48 "%RES_DIR%\mipmap-mdpi\ic_launcher.png"
magick convert "%SOURCE%" -resize 48x48 "%RES_DIR%\mipmap-mdpi\ic_launcher_round.png"

REM hdpi (72x72)
magick convert "%SOURCE%" -resize 72x72 "%RES_DIR%\mipmap-hdpi\ic_launcher.png"
magick convert "%SOURCE%" -resize 72x72 "%RES_DIR%\mipmap-hdpi\ic_launcher_round.png"

REM xhdpi (96x96)
magick convert "%SOURCE%" -resize 96x96 "%RES_DIR%\mipmap-xhdpi\ic_launcher.png"
magick convert "%SOURCE%" -resize 96x96 "%RES_DIR%\mipmap-xhdpi\ic_launcher_round.png"

REM xxhdpi (144x144)
magick convert "%SOURCE%" -resize 144x144 "%RES_DIR%\mipmap-xxhdpi\ic_launcher.png"
magick convert "%SOURCE%" -resize 144x144 "%RES_DIR%\mipmap-xxhdpi\ic_launcher_round.png"

REM xxxhdpi (192x192)
magick convert "%SOURCE%" -resize 192x192 "%RES_DIR%\mipmap-xxxhdpi\ic_launcher.png"
magick convert "%SOURCE%" -resize 192x192 "%RES_DIR%\mipmap-xxxhdpi\ic_launcher_round.png"

echo Fertig! Icons wurden generiert.
pause
