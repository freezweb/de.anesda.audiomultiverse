# Android Icon Setup

Die Icons müssen manuell erstellt werden. Empfohlene Größen:

| Ordner | Größe (px) |
|--------|------------|
| mipmap-mdpi | 48x48 |
| mipmap-hdpi | 72x72 |
| mipmap-xhdpi | 96x96 |
| mipmap-xxhdpi | 144x144 |
| mipmap-xxxhdpi | 192x192 |

## Benötigte Dateien pro Ordner:
- `ic_launcher.png` - Standard-Icon
- `ic_launcher_round.png` - Rundes Icon

## Schnell-Setup:
Nutze [Android Asset Studio](https://romannurik.github.io/AndroidAssetStudio/icons-launcher.html) oder Android Studio's Image Asset Tool.

## Platzhalter:
Für den ersten Build werden die Desktop-Icons verwendet. Du kannst die Icons aus `remote/src-tauri/icons/` konvertieren.
