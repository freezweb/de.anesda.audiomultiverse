# 🎛️ AudioMultiverse

**Ein professionelles, softwarebasiertes virtuelles Mischpult mit nativer AES67/DANTE-Unterstützung, MIDI-Steuerung und Multi-Client-Fernsteuerung.**

[![Build Status](https://img.shields.io/badge/build-passing-brightgreen)]()
[![Rust](https://img.shields.io/badge/rust-1.75+-orange)]()
[![License](https://img.shields.io/badge/license-MIT-blue)]()

---

## 📋 Inhaltsverzeichnis

- [Features](#-features)
- [Systemarchitektur](#-systemarchitektur)
- [Voraussetzungen](#-voraussetzungen)
- [Installation](#-installation)
  - [Server (Linux/Raspberry Pi)](#server-linuxraspberry-pi)
  - [Desktop-App (Windows/macOS/Linux)](#desktop-app-windowsmacoslinux)
  - [Remote-App (Android)](#remote-app-android)
- [Konfiguration](#%EF%B8%8F-konfiguration)
- [Verwendung](#-verwendung)
- [API-Dokumentation](#-api-dokumentation)
- [MIDI-Steuerung](#-midi-steuerung)
- [AES67/DANTE-Integration](#-aes67dante-integration)
- [Entwicklung](#-entwicklung)
- [Fehlerbehebung](#-fehlerbehebung)
- [Lizenz](#-lizenz)

---

## ✨ Features

### Audio-Engine
- **32×32 Audio-Routing-Matrix** (erweiterbar bis 64×64)
- **Vollparametrischer 4-Band EQ** pro Kanal
- **Hochpassfilter** (80Hz / 120Hz schaltbar)
- **Phasen-Invertierung** (Ø)
- **Gain/Trim** (-∞ bis +20 dB)
- **Fader** (-∞ bis +10 dB, logarithmisch)
- **Pan** mit Constant Power Pan Law
- **8 Stereo Aux-Sends** (Pre/Post-Fader umschaltbar)
- **4 Stereo Gruppen/Subgruppen**

### Master-Sektion
- **Stereo Master-Fader** mit Limiter
- **DIM-Funktion** (-20 dB)
- **Mono-Summen-Check**
- **Talkback-Funktion**
- **Test-Oszillator** (1 kHz)

### Metering
- **Peak-Meter** mit Hold-Anzeige
- **RMS-Meter** (100 ms Integration)
- **LUFS-Metering** (EBU R128)
- **Stereo-Korrelations-Meter**
- **Clipping-Erkennung** mit Counter

### Netzwerk-Audio
- **AES67** (offen, DANTE-kompatibel)
- **SAP/SDP Discovery** (automatische Stream-Erkennung)
- **PTP Clock Sync** (IEEE 1588)
- **RTP Streaming** (L24 @ 48 kHz)
- **Multicast-Support** für bis zu 32 Kanäle

### MIDI-Steuerung
- **Mackie Control Universal (MCU)** Protokoll
- **Generic CC-Mapping** für beliebige Controller
- **MIDI-Learn-Funktion**
- **Bidirektionales Feedback** (motorisierte Fader, LEDs)
- **Multi-Device-Support**

### Szenen-Management
- **Unbegrenzte Szenen-Speicherplätze**
- **Selektiver Recall** (nur bestimmte Parameter)
- **Szenen-Crossfade** (zeitbasiert)
- **Safe-Kanäle** (von Recall ausschließen)
- **Kategorien & Farbcodes**

### Remote-Steuerung
- **Unbegrenzte Clients** gleichzeitig
- **Echtzeit-Synchronisation** via WebSocket
- **REST-API** für Hausautomatisierung
- **mDNS-Discovery** (automatische Server-Erkennung)

---

## 🏗️ Systemarchitektur

```
┌─────────────────────────────────────────────────────────────────┐
│                         NETZWERK                                 │
│   ┌─────────────┐  ┌─────────────┐  ┌─────────────────────────┐ │
│   │ DANTE/AES67 │  │ Remote-App  │  │ Smart Home / REST API   │ │
│   │   Geräte    │  │ (Win/Linux/ │  │ (Home Assistant, etc.)  │ │
│   │             │  │  Android)   │  │                         │ │
│   └──────┬──────┘  └──────┬──────┘  └────────────┬────────────┘ │
└──────────┼────────────────┼──────────────────────┼──────────────┘
           │ AES67/RTP      │ WebSocket            │ HTTP/REST
           ▼                ▼                      ▼
┌─────────────────────────────────────────────────────────────────┐
│                    AUDIOMULTIVERSE SERVER                        │
│  ┌─────────────┐  ┌─────────────┐  ┌─────────────────────────┐  │
│  │ AES67/PTP   │  │ WebSocket   │  │      REST API           │  │
│  │  Backend    │  │   Handler   │  │   (Axum Framework)      │  │
│  └──────┬──────┘  └──────┬──────┘  └────────────┬────────────┘  │
│         │                │                      │               │
│         ▼                ▼                      ▼               │
│  ┌─────────────────────────────────────────────────────────────┐│
│  │                      MIXER CORE                              ││
│  │  ┌─────────┐  ┌─────────┐  ┌─────────┐  ┌─────────────────┐ ││
│  │  │Channels │  │ Routing │  │  Buses  │  │ Scenes/Presets  │ ││
│  │  │  (32)   │  │ Matrix  │  │(8A+4G)  │  │                 │ ││
│  │  └─────────┘  └─────────┘  └─────────┘  └─────────────────┘ ││
│  └──────────────────────────────────────────────────────────────┘│
│         │                                                        │
│         ▼                                                        │
│  ┌─────────────────────────────────────────────────────────────┐│
│  │                     AUDIO ENGINE                             ││
│  │  Gain → Phase → EQ → Fader → Pan → Metering → Output        ││
│  └─────────────────────────────────────────────────────────────┘│
│         │                │                                       │
│         ▼                ▼                                       │
│  ┌─────────────┐  ┌─────────────┐                               │
│  │    MIDI     │  │   cpal/     │                               │
│  │  (midir)    │  │   JACK      │                               │
│  └─────────────┘  └─────────────┘                               │
└─────────────────────────────────────────────────────────────────┘
           │                │
           ▼                ▼
    ┌─────────────┐  ┌─────────────┐
    │ MIDI Ctrl   │  │ Audio I/O   │
    │ (X-Touch)   │  │ (Interface) │
    └─────────────┘  └─────────────┘
```

---

## 📦 Projektstruktur

```
audiomultiverse/
├── server/              # Rust Server (Audio-Engine, API, MIDI, AES67)
│   ├── src/
│   │   ├── main.rs      # Einstiegspunkt
│   │   ├── config.rs    # Konfiguration (TOML)
│   │   ├── audio/       # Audio-Engine, Metering, EQ
│   │   ├── mixer/       # Kanäle, Routing, Szenen, Busse
│   │   ├── api/         # REST & WebSocket Handler
│   │   ├── midi/        # MIDI-Controller & Feedback
│   │   └── network_audio/ # AES67, RTP, PTP, SAP
│   ├── config.toml      # Server-Konfiguration
│   └── Cargo.toml
│
├── app/                 # Desktop-App (Haupt-UI, Touchscreen)
│   ├── src/
│   │   ├── lib/
│   │   │   ├── components/  # Svelte-Komponenten
│   │   │   ├── stores/      # State Management
│   │   │   └── api/         # WebSocket-Client
│   │   └── routes/          # SvelteKit-Seiten
│   └── src-tauri/           # Tauri (Rust-Backend)
│
├── remote/              # Remote-App (Fernsteuerung)
│   ├── src/             # Gleiche Struktur wie app/
│   └── src-tauri/       # Tauri mit Android-Support
│
├── shared/protocol/     # Gemeinsame Typen (Rust Crate)
│   └── src/
│       ├── lib.rs
│       ├── messages.rs  # Client/Server-Nachrichten
│       └── types.rs     # Gemeinsame Datentypen
│
├── deploy/              # Deployment-Skripte
│   ├── innosetup/       # Windows-Installer
│   ├── systemd/         # Linux-Services
│   └── jenkins/         # CI/CD-Dokumentation
│
└── docs/                # Zusätzliche Dokumentation
```

---

## 💻 Voraussetzungen

### Server (Linux / Raspberry Pi)

| Komponente | Minimum | Empfohlen |
|------------|---------|-----------|
| **CPU** | ARM Cortex-A72 (RPi4) | x86_64 Multi-Core |
| **RAM** | 2 GB | 4 GB+ |
| **OS** | Debian 11 / Ubuntu 22.04 | Debian 12 / Ubuntu 24.04 |
| **Rust** | 1.75+ | Latest Stable |
| **Audio** | ALSA | JACK / PipeWire |
| **Netzwerk** | Gigabit Ethernet | Für AES67 erforderlich |

### Desktop-App (Windows / macOS / Linux)

| Komponente | Version |
|------------|---------|
| **Node.js** | 20+ |
| **pnpm** | 8+ |
| **Rust** | 1.75+ |
| **Tauri CLI** | 2.x |

### Remote-App (Windows / Linux / Android)

**Für Windows & Linux:** Gleiche Voraussetzungen wie Desktop-App.

**Für Android:**

| Komponente | Version |
|------------|---------|
| **Android SDK** | 34+ |
| **Android NDK** | 27+ |
| **Java JDK** | 17+ |

---

## 🚀 Installation

### Server (Linux/Raspberry Pi)

```bash
# 1. Repository klonen
git clone https://github.com/anesda/de.anesda.audiomultiverse.git
cd de.anesda.audiomultiverse

# 2. Rust installieren (falls nicht vorhanden)
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source ~/.cargo/env

# 3. Build-Abhängigkeiten installieren (Debian/Ubuntu)
sudo apt update
sudo apt install -y build-essential pkg-config libasound2-dev libssl-dev

# 4. Server kompilieren
cd server
cargo build --release

# 5. Konfiguration erstellen
cp config.toml.example config.toml
nano config.toml  # Nach Bedarf anpassen

# 6. Server starten
./target/release/audiomultiverse-server

# Optional: Als Systemd-Service installieren
sudo cp ../deploy/systemd/audiomultiverse.service /etc/systemd/system/
sudo systemctl daemon-reload
sudo systemctl enable audiomultiverse
sudo systemctl start audiomultiverse
```

### Desktop-App (Windows/macOS/Linux)

```bash
# 1. Node.js & pnpm installieren
# Windows: https://nodejs.org + npm install -g pnpm
# Linux: sudo apt install nodejs npm && npm install -g pnpm

# 2. Tauri-Voraussetzungen (nur einmalig)
# Windows: Microsoft Visual C++ Build Tools
# Linux: sudo apt install libwebkit2gtk-4.1-dev libgtk-3-dev

# 3. Dependencies installieren
cd app
pnpm install

# 4. Entwicklungsmodus starten
pnpm tauri dev

# 5. Produktions-Build erstellen
pnpm tauri build
# Ergebnis: src-tauri/target/release/bundle/
```

### Remote-App (Windows / Linux / Android)

#### Windows & Linux Build

```bash
# 1. Dependencies installieren
cd remote
pnpm install

# 2. Entwicklungsmodus starten
pnpm tauri dev

# 3. Produktions-Build erstellen
pnpm tauri build

# Ergebnis:
# Windows: src-tauri/target/release/bundle/msi/
# Linux:   src-tauri/target/release/bundle/deb/
```

#### Android Build

```bash
# 1. Android SDK & NDK installieren
# Empfohlen: Android Studio installieren

# 2. Umgebungsvariablen setzen
export ANDROID_HOME=$HOME/Android/Sdk
export NDK_HOME=$ANDROID_HOME/ndk/27.0.12077973

# 3. Rust Android-Targets hinzufügen
rustup target add aarch64-linux-android armv7-linux-androideabi

# 4. Dependencies installieren
cd remote
pnpm install

# 5. Android-Build erstellen
pnpm tauri android build --apk

# Ergebnis: src-tauri/gen/android/app/build/outputs/apk/
```

### Docker (Server)

```bash
cd server

# Mit Docker Compose starten
docker-compose up -d

# Logs anzeigen
docker-compose logs -f

# Stoppen
docker-compose down
```

> ⚠️ **Hinweis:** Für AES67/Multicast wird `network_mode: host` benötigt.

---

## ⚙️ Konfiguration

Die Server-Konfiguration erfolgt über `server/config.toml`:

```toml
[server]
port = 9000                    # API/WebSocket-Port
bind = "0.0.0.0"               # Netzwerk-Interface

[audio]
sample_rate = 48000            # 44100, 48000, 96000
buffer_size = 256              # Latenz: 256/48000 = 5.3ms
input_channels = 32            # Anzahl Eingänge
output_channels = 32           # Anzahl Ausgänge

[audio.backend]
type = "cpal"                  # "cpal", "jack", "pipewire"

[midi]
enabled = true                 # MIDI-Support aktivieren
auto_connect = true            # Automatisch verbinden

[network_audio]
backend = "aes67"              # "aes67" oder "dante"
ptp_domain = 0                 # PTP-Domain (0-127)

[logging]
level = "info"                 # "trace", "debug", "info", "warn", "error"
file = "/var/log/audiomultiverse/server.log"
```

### Umgebungsvariablen

| Variable | Beschreibung | Standard |
|----------|--------------|----------|
| `AUDIOMULTIVERSE_CONFIG` | Pfad zur config.toml | `./config.toml` |
| `AUDIOMULTIVERSE_LOG_LEVEL` | Log-Level überschreiben | `info` |
| `RUST_LOG` | Rust-spezifisches Logging | - |

---

## 📖 Verwendung

### Server starten

```bash
cd server
cargo run --release

# Oder nach Installation:
audiomultiverse-server --config /etc/audiomultiverse/config.toml
```

Der Server zeigt beim Start:
```
🎛️ AudioMultiverse Server v0.1.0
📡 API Server läuft auf http://0.0.0.0:9000
🔌 WebSocket: ws://0.0.0.0:9000/api/ws
🎹 MIDI: 2 Geräte gefunden
🔊 Audio: cpal Backend (48kHz, 256 Samples)
```

### Client verbinden

1. **Desktop-App** oder **Remote-App** starten
2. Server wird automatisch via mDNS erkannt
3. Oder manuell: `ws://SERVER-IP:9000/api/ws`

### Bedienung

| Bereich | Beschreibung |
|---------|--------------|
| **Mixer** | Kanalzüge mit Fader, Mute, Solo, Pan |
| **Matrix** | 32×32 Routing-Kreuzschiene |
| **Meter** | Große Meter-Anzeige |
| **Szenen** | Presets speichern/laden |
| **Settings** | Audio, Netzwerk, MIDI-Konfiguration |

---

## 🌐 API-Dokumentation

### REST-Endpunkte

| Methode | Endpunkt | Beschreibung |
|---------|----------|--------------|
| `GET` | `/health` | Server-Status |
| `GET` | `/api/info` | Server-Informationen |
| `GET` | `/api/state` | Kompletter Mixer-State |
| `GET` | `/api/channels` | Alle Kanäle |
| `GET` | `/api/channels/:id` | Einzelner Kanal |
| `PATCH` | `/api/channels/:id` | Kanal aktualisieren |
| `POST` | `/api/channels/:id/fader` | Fader setzen |
| `POST` | `/api/channels/:id/mute` | Mute setzen |
| `POST` | `/api/channels/:id/solo` | Solo setzen |
| `POST` | `/api/channels/:id/gain` | Gain setzen |
| `POST` | `/api/channels/:id/phase` | Phase invertieren |
| `GET` | `/api/routing` | Routing-Matrix |
| `POST` | `/api/routing` | Routing-Punkt setzen |
| `GET` | `/api/scenes` | Szenen-Liste |
| `POST` | `/api/scenes` | Szene speichern |
| `GET` | `/api/scenes/:id` | Szene abrufen |
| `DELETE` | `/api/scenes/:id` | Szene löschen |
| `POST` | `/api/scenes/:id/recall` | Szene laden |
| `GET` | `/api/master` | Master-State |
| `POST` | `/api/master/fader` | Master-Fader |
| `POST` | `/api/master/mute` | Master-Mute |
| `POST` | `/api/master/dim` | DIM-Funktion |
| `GET` | `/api/buses/aux` | Aux-Busse |
| `GET` | `/api/buses/groups` | Gruppen |
| `GET` | `/api/aes67/status` | AES67-Status |
| `GET` | `/api/aes67/streams` | Verfügbare Streams |

### WebSocket-Protokoll

**Verbindung:** `ws://SERVER:9000/api/ws`

#### Client → Server

```json
// Fader setzen
{ "type": "SetFader", "channel": 0, "value": 0.75 }

// Mute setzen
{ "type": "SetMute", "channel": 0, "muted": true }

// Solo setzen
{ "type": "SetSolo", "channel": 0, "solo": true }

// Pan setzen
{ "type": "SetPan", "channel": 0, "value": 0.0 }

// Routing setzen
{ "type": "SetRouting", "input": 0, "output": 0, "gain": 1.0 }

// State anfordern
{ "type": "GetState" }

// Ping
{ "type": "Ping", "timestamp": 1234567890 }
```

#### Server → Client

```json
// Willkommen (nach Verbindung)
{ 
  "type": "Welcome",
  "server_info": { "name": "AudioMultiverse", "version": "0.1.0", ... },
  "state": { ... }
}

// Meter-Update (50ms Intervall)
{
  "type": "Meters",
  "peaks": [0.5, 0.4, 0.3, ...],
  "timestamp": 1234567890
}

// Kanal aktualisiert
{
  "type": "ChannelUpdated",
  "id": 0,
  "fader": 0.75,
  "mute": false,
  ...
}

// Routing aktualisiert
{
  "type": "RoutingUpdated",
  "input": 0,
  "output": 0,
  "gain": 1.0
}
```

### Home Assistant Integration

```yaml
# configuration.yaml
rest_command:
  audiomultiverse_mute:
    url: "http://MIXER-IP:9000/api/channels/{{ channel }}/mute"
    method: POST
    content_type: "application/json"
    payload: '{"muted": {{ muted }}}'

  audiomultiverse_fader:
    url: "http://MIXER-IP:9000/api/channels/{{ channel }}/fader"
    method: POST
    content_type: "application/json"
    payload: '{"value": {{ value }}}'
```

---

## 🎹 MIDI-Steuerung

### Unterstützte Protokolle

| Protokoll | Controller | Status |
|-----------|------------|--------|
| **MCU** | Mackie Control, X-Touch, etc. | ✅ Vollständig |
| **Generic CC** | Beliebige MIDI-Controller | ✅ Vollständig |
| **HUI** | Pro Tools Controller | 🔄 Geplant |

### MIDI-Learn

1. In der App: Rechtsklick auf einen Parameter
2. "MIDI Learn" auswählen
3. Gewünschten MIDI-Controller bewegen
4. Zuordnung wird automatisch gespeichert

### MCU-Belegung

| Fader | Funktion | Encoder | Funktion |
|-------|----------|---------|----------|
| 1-8 | Kanal-Fader | V-Pot 1-8 | Pan |
| 9 | Master | - | - |

| Taste | Funktion |
|-------|----------|
| REC 1-8 | Phase Invert |
| SOLO 1-8 | Solo |
| MUTE 1-8 | Mute |
| SELECT 1-8 | Kanal auswählen |
| BANK ◀/▶ | 8 Kanäle wechseln |

---

## 🔊 AES67/DANTE-Integration

### Voraussetzungen

- Gigabit-Netzwerk (dediziertes Audio-VLAN empfohlen)
- DANTE-Geräte im **AES67-Modus**
- PTP-fähiger Switch (empfohlen)

### Stream-Discovery

Der Server erkennt automatisch AES67-Streams via **SAP/SDP**:

```bash
# Status abrufen
curl http://localhost:9000/api/aes67/status

# Verfügbare Streams
curl http://localhost:9000/api/aes67/streams
```

### Stream abonnieren

```bash
# Stream auf Kanal 1-8 mappen
curl -X POST http://localhost:9000/api/aes67/streams/STREAM-ID/subscribe \
  -H "Content-Type: application/json" \
  -d '{"start_channel": 0}'
```

### DANTE-Geräte konfigurieren

1. **Dante Controller** öffnen
2. Gerät auswählen → Device Config
3. **AES67 Mode** aktivieren
4. Multicast-Adresse notieren
5. Server erkennt Stream automatisch

---

## 🛠️ Entwicklung

### Entwicklungsumgebung einrichten

```bash
# Repository klonen
git clone https://github.com/anesda/de.anesda.audiomultiverse.git
cd de.anesda.audiomultiverse

# Alle Dependencies installieren
cd app && pnpm install && cd ..
cd remote && pnpm install && cd ..
```

### Server entwickeln

```bash
cd server

# Mit Hot-Reload
cargo watch -x run

# Tests ausführen
cargo test

# Alle 74 Tests
cargo test -- --nocapture
```

### Frontend entwickeln

```bash
cd app  # oder remote

# Entwicklungsserver mit Hot-Reload
pnpm tauri dev

# TypeScript-Check
pnpm check

# Linting
pnpm lint
```

### Code-Qualität

| Tool | Befehl | Beschreibung |
|------|--------|--------------|
| `cargo fmt` | Rust-Formatierung | Automatisch formatieren |
| `cargo clippy` | Rust-Linting | Warnungen prüfen |
| `cargo test` | Rust-Tests | 74 Unit-Tests |
| `pnpm check` | Svelte/TS-Check | TypeScript-Fehler |
| `pnpm lint` | ESLint | JavaScript-Linting |

---

## ❓ Fehlerbehebung

### Server startet nicht

```bash
# Logs prüfen
journalctl -u audiomultiverse -f

# Port bereits belegt?
sudo lsof -i :9000

# ALSA-Probleme?
aplay -l  # Geräte auflisten
```

### Kein Audio

1. Audio-Backend prüfen: `aplay -l`
2. Berechtigungen: Benutzer in `audio`-Gruppe?
3. JACK läuft? `jack_lsp`
4. Buffer-Größe erhöhen (config.toml)

### MIDI nicht erkannt

```bash
# MIDI-Geräte auflisten
aconnect -l

# Berechtigungen prüfen
ls -la /dev/snd/seq
```

### AES67-Streams nicht gefunden

1. Netzwerk prüfen: Multicast aktiviert?
2. Firewall: Port 5004 (RTP) + 9875 (SAP) öffnen
3. DANTE-Gerät: AES67-Modus aktiviert?
4. Gleiches Subnetz?

### WebSocket-Verbindung fehlgeschlagen

1. Server läuft? `curl http://SERVER:9000/health`
2. Firewall: Port 9000 öffnen
3. CORS: Server bindet auf `0.0.0.0`?

---

## 📜 Lizenz

MIT License - siehe [LICENSE](LICENSE)

---

## 🙏 Credits

- **Audio-Backend:** [cpal](https://github.com/RustAudio/cpal)
- **Web-Framework:** [Axum](https://github.com/tokio-rs/axum)
- **Desktop-Framework:** [Tauri](https://tauri.app)
- **UI-Framework:** [Svelte](https://svelte.dev) + [TailwindCSS](https://tailwindcss.com)
- **MIDI:** [midir](https://github.com/Boddlnagg/midir)

---

<div align="center">

**Made with ❤️ for Audio Professionals**

[Dokumentation](docs/) · [Issues](https://github.com/anesda/de.anesda.audiomultiverse/issues)

</div>
