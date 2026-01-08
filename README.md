# 🎛️ AudioMultiverse

Ein virtuelles, softwarebasiertes Mischpult mit nativer DANTE/AES67-Unterstützung, MIDI-Steuerung und moderner Web-Fernsteuerung.

## Features

- **32x32 Audio-Matrix** (Eingänge × Ausgänge)
- **AES67 Audio-Netzwerk** (DANTE-kompatibel)
- **MIDI-Steuerung** mit bidirektionalem Feedback
- **Touch-optimierte UI**
- **Multi-Client Fernsteuerung**
- **REST/WebSocket API** für Hausautomatisierung
- **Linux-Server** (Debian/Ubuntu, Raspberry Pi 4)

## Projektstruktur

| Projekt | Beschreibung | Plattform |
|---------|--------------|-----------|
| `server/` | Audio-Engine, AES67, MIDI, API | Linux, Raspberry Pi |
| `app/` | Haupt-UI (Touchscreen) | Linux |
| `remote/` | Fernsteuerungs-Client | Windows, macOS, Linux |
| `shared/` | Gemeinsame Komponenten | - |

## Voraussetzungen

### Server (Linux)
- Rust 1.75+
- JACK Audio oder PipeWire
- AES67-fähige Netzwerkkarte

### App & Remote
- Node.js 20+
- pnpm 8+
- Rust 1.75+ (für Tauri)

## Schnellstart

```bash
# Repository klonen
git clone https://github.com/yourusername/audiomultiverse.git
cd audiomultiverse

# Dependencies installieren
pnpm install

# Server starten (auf Linux)
cd server
cargo run

# App starten (auf Linux mit Touchscreen)
pnpm dev:app

# Fernsteuerung starten (auf Windows/macOS/Linux)
pnpm dev:remote
```

## Lizenz

MIT

---

Siehe [TODO.md](TODO.md) für die vollständige Roadmap.
