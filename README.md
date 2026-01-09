# 🎛️ AudioMultiverse

Ein virtuelles, softwarebasiertes Mischpult mit nativer DANTE/AES67-Unterstützung, MIDI-Steuerung und moderner Web-Fernsteuerung.

## Features

- **32x32 Audio-Matrix** (Eingänge × Ausgänge, erweiterbar)
- **AES67 Audio-Netzwerk** (DANTE-kompatibel, Multicast)
- **MIDI-Steuerung** mit bidirektionalem Feedback
- **Touch-optimierte UI** (für dedizierte Controller)
- **Multi-Client Fernsteuerung** (unbegrenzte Windows/Mac/Linux Clients)
- **REST/WebSocket API** für Hausautomatisierung
- **Linux-Server** (Debian/Ubuntu, Raspberry Pi 4)
- **Szenen-Management** (Speichern/Laden von Presets)

## Projektstruktur

```
audiomultiverse/
├── server/          # Rust Audio-Engine, AES67, MIDI, API
├── app/             # Lokale Mischpult-UI (Tauri + SvelteKit)
├── remote/          # Fernsteuerungs-Client (Tauri + SvelteKit)
├── shared/          # Gemeinsame Protokoll-Definitionen (Rust)
├── deploy/          # Installer, Systemd-Services, CI/CD
└── docs/            # Dokumentation
```

| Projekt | Beschreibung | Plattform |
|---------|--------------|-----------|
| `server/` | Audio-Engine, AES67, MIDI, API | Linux, Raspberry Pi |
| `app/` | Haupt-UI (Touchscreen) | Linux |
| `remote/` | Fernsteuerungs-Client | Windows, macOS, Linux, Android |
| `shared/protocol/` | Gemeinsame Typen & Nachrichten | - |

## Voraussetzungen

### Server (Linux)
- **Rust 1.75+** - `curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh`
- **ALSA** (Standard) oder **JACK/PipeWire** (optional)
- **AES67-fähige Netzwerkkarte** (für DANTE-Geräte)

### App & Remote
- **Node.js 20+** - https://nodejs.org
- **pnpm 8+** - `npm install -g pnpm`
- **Rust 1.75+** (für Tauri)

## Schnellstart

```bash
# Repository klonen
git clone https://github.com/anesda/de.anesda.audiomultiverse.git
cd de.anesda.audiomultiverse

# Server bauen & starten (Linux)
cd server
cargo run --release

# Remote-App bauen (Windows/Linux/macOS)
cd remote
pnpm install
pnpm tauri build
```

## Docker (Server)

```bash
cd server
docker-compose up -d
```

> ⚠️ Für AES67/Multicast wird `network_mode: host` benötigt.

## API

Der Server bietet eine REST- und WebSocket-API auf Port 8080:

### REST Endpoints
- `GET /health` - Server-Status
- `GET /api/channels` - Alle Kanäle abrufen
- `GET /api/channels/:id` - Einzelnen Kanal abrufen
- `POST /api/channels/:id` - Kanal ändern
- `GET /api/routing` - Routing-Matrix
- `POST /api/routing` - Routing-Punkt setzen
- `GET /api/scenes` - Szenen-Liste
- `POST /api/scenes` - Szene speichern

### WebSocket
- `ws://server:8080/ws` - Echtzeit-Updates (Meter, State-Änderungen)
pnpm dev:app

# Fernsteuerung starten (auf Windows/macOS/Linux)
pnpm dev:remote
```

## Lizenz

MIT

---

Siehe [TODO.md](TODO.md) für die vollständige Roadmap.
