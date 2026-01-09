# 🎛️ AudioMultiverse - Virtuelles DANTE Mischpult

## Projektübersicht

Ein virtuelles, softwarebasiertes Mischpult mit nativer DANTE-Unterstützung, MIDI-Steuerung und moderner Web-Fernsteuerung. Entwickelt für professionelle Audioanwendungen mit Fokus auf Stabilität und Flexibilität.

### Kernziele
- **32x32 Audio-Matrix** (Eingänge × Ausgänge, erweiterbar)
- **Native DANTE-Integration** via Dante API / AES67
- **MIDI-Steuerung** mit Bi-direktionalem Feedback
- **Touchscreen-optimierte UI**
- **Multi-Client Fernsteuerung**
- **REST/WebSocket API** für Hausautomatisierung
- **Linux-Server** (Debian/Ubuntu, Raspberry Pi 4 kompatibel)
- **Windows/macOS/Linux Clients**

---

## 📋 TODO-Liste

### Phase 1: Grundlagen & Architektur

#### 1.1 Projektstruktur
- [x] Monorepo-Struktur aufsetzen (pnpm workspaces)
- [x] Server-Projekt initialisieren (Rust mit Axum, tokio)
- [x] App-Projekt initialisieren (Tauri + Svelte für lokale UI)
- [x] Remote-Projekt initialisieren (Tauri + Svelte für Fernsteuerung)
- [x] Shared Types/Protokolle definieren (shared/protocol)
- [x] Jenkins Pipeline einrichten (Jenkinsfile)
- [x] Build-Agents Dokumentation (docs/jenkins-agent-setup.md)
- [ ] Docker-Container für Server vorbereiten
- [x] Dokumentationsstruktur angelegt

#### 1.1.1 CI/CD Pipeline (Jenkins)
- [x] Jenkinsfile konfigurieren
- [x] Windows Agent Dokumentation
- [x] Linux Agent Dokumentation
- [x] Android Build-Umgebung (SDK, NDK) - dokumentiert
- [ ] Raspberry Pi ARM64 Cross-Compile - dokumentiert
- [x] Inno Setup Script (deploy/windows/installer.iss)
- [x] cargo-deb für Linux .deb Pakete
- [x] APK Signing konfigurieren
- [x] Artifact-Archivierung im Jenkinsfile
- [x] Remote-App auf Tauri 2.x aktualisiert (Android-Support)
- [x] Android-Stage im Jenkinsfile
- [ ] Optional: macOS Agent für .dmg

#### 1.2 Audio-Engine Kern
- [ ] Audio-Processing-Framework auswählen (JUCE, PortAudio, oder eigene Implementierung)
- [ ] Ringbuffer für Audio-Streams implementieren
- [ ] Lock-free Audio-Thread-Architektur
- [ ] Sample-Rate-Konvertierung (44.1kHz, 48kHz, 96kHz)
- [ ] Latenz-Monitoring und -Optimierung
- [ ] Audio-Metering (Peak, RMS, LUFS)

#### 1.3 AES67 Integration (DANTE-kompatibel)
> **Strategie:** AES67 als primäres Protokoll (Open Source), DANTE-Geräte im AES67-Modus.
> Architektur erlaubt spätere native DANTE SDK Integration.

##### Phase 1: AES67 (Primär)
- [ ] AES67-Stack auswählen:
  - [ ] Option A: **Ravenna ALSA Driver** (Open Source, Linux)
  - [ ] Option B: **PipeWire AES67 Module** (modern, empfohlen)
  - [ ] Option C: **Merging ALSA Driver** (Merging Technologies)
- [ ] SAP/SDP Discovery implementieren
- [ ] PTP (IEEE 1588) Clock-Synchronisation
- [ ] Multicast Stream Empfang (4x 8-Kanal Streams = 32 Kanäle)
- [ ] Multicast Stream Senden (4x 8-Kanal Streams = 32 Kanäle)
- [ ] Stream-Konfiguration (48kHz, 24-bit)
- [ ] Latenz-Messung und -Kompensation
- [ ] Netzwerk-Redundanz (falls verfügbar)

##### Phase 2: DANTE SDK (Optional, später)
- [ ] Audinate Lizenzierung evaluieren
- [ ] Abstraktionsschicht für Audio-Backend
  - [ ] Interface: `AudioNetworkBackend`
  - [ ] Implementierung: `Aes67Backend`
  - [ ] Implementierung: `DanteBackend` (später)
- [ ] DANTE-spezifische Features:
  - [ ] Dante Browse Discovery
  - [ ] Niedrigere Latenz-Modi
  - [ ] Mehr Kanäle pro Stream
  - [ ] 44.1kHz / 96kHz Support

##### Gemeinsam (beide Backends)
- [ ] Geräte-Discovery UI
- [ ] Routing-Matrix Synchronisation
- [ ] Audio-Stream Empfang abstrahiert
- [ ] Audio-Stream Senden abstrahiert
- [ ] Fallback bei Netzwerkproblemen
- [ ] Dante Controller Kompatibilität testen

---

### Phase 2: Mixer-Kernfunktionen

#### 2.1 Kanal-Strip Implementierung
- [x] Input Gain (-∞ bis +20dB) - Grundstruktur
- [ ] Phase Invert (Ø)
- [ ] Phantom Power Status (nur Anzeige, DANTE-seitig)
- [x] High-Pass Filter (schaltbar, 80Hz/120Hz)
- [x] 3-Band EQ (später erweiterbar) - 4-Band parametrischer EQ
  - [x] Low Shelf (80Hz)
  - [x] Mid Parametric (250Hz - 5kHz)
  - [x] High Shelf (12kHz)
- [x] Pan/Balance (-100L bis +100R)
- [x] Fader (-∞ bis +10dB, Logarithmisch)
- [x] Mute-Button
- [x] Solo-Button (PFL/AFL umschaltbar)
- [x] Kanal-Benennung (frei editierbar)
- [x] Kanal-Farbe (zur visuellen Gruppierung)

#### 2.2 Routing-Matrix
- [x] Vollständige NxM Matrix (32x32 Standard)
- [x] Matrix-Ansicht (Kreuzschienen-Stil) - RoutingMatrix.svelte
- [ ] Direkt-Routing pro Kanal
- [ ] Bus-System
  - [ ] Stereo Master Bus
  - [ ] 8x Stereo Aux Sends (erweiterbar)
  - [ ] 4x Stereo Gruppen/Subgruppen
  - [ ] Matrix Outputs
- [ ] Routing-Presets speichern/laden
- [ ] "Follow Main" Option für Outputs

#### 2.3 Master-Sektion
- [x] Stereo Master Fader
- [x] Master-Limiter (Schutz)
- [x] Talkback-Funktion (mit externem Mic)
- [x] DIM-Funktion (-20dB)
- [x] Mono-Summen-Check
- [x] Oscillator (1kHz Testton)

#### 2.4 Metering & Monitoring
- [ ] Peak-Meter pro Kanal (12-Segment LED-Stil)
- [ ] Master-Meter (größer, detaillierter)
- [ ] Clipping-Anzeige mit Hold
- [ ] LUFS-Meter für Master
- [ ] Correlation-Meter (Stereo)
- [ ] Spektrum-Analysator (optional, später)

---

### Phase 3: MIDI-Integration

#### 3.1 MIDI-Grundlagen
- [x] MIDI-Backend auswählen (RtMidi, JUCE MIDI, oder PortMidi) - midir
- [x] MIDI-Geräte-Erkennung (Hot-Plug)
- [x] MIDI-Learn-Funktion
- [x] MIDI-Mapping speichern/laden
- [x] Multi-Device Support

#### 3.2 MIDI-Controller Support
- [x] Generic CC-Mapping
- [x] Mackie Control Universal (MCU) Protokoll
- [ ] HUI Protokoll (optional)
- [ ] Behringer X-Touch Unterstützung
- [ ] Korg nanoKONTROL Support
- [ ] AKAI APC Serie
- [ ] Custom Controller Profile erstellen

#### 3.3 MIDI-Feedback
- [x] Fader-Position senden (motorisierte Fader)
- [x] LED-Ring Feedback (für Encoder)
- [x] Mute/Solo LED Status
- [x] Meter-Daten über MIDI (für Controller mit Displays)
- [x] Kanal-Namen über SysEx (MCU-kompatibel)

#### 3.4 MIDI-Erweiterungen
- [ ] OSC-to-MIDI Bridge
- [ ] Virtual MIDI Ports (für DAW-Integration)
- [ ] MIDI Clock/MTC Sync (für Show-Steuerung)

---

### Phase 4: Benutzeroberfläche

#### 4.1 UI-Framework & Design
- [ ] Tech-Stack finalisieren
  - [ ] Option A: Electron + React + TailwindCSS
  - [ ] Option B: Tauri + Svelte (leichtgewichtiger)
  - [ ] Option C: Qt/QML (native Performance)
- [ ] Design-System erstellen (Komponenten-Bibliothek)
- [ ] Dark Theme (Standard für Audio)
- [ ] High-Contrast Theme (für helle Umgebungen)
- [ ] Responsive Layout (Desktop + Tablet)

#### 4.2 Hauptansichten
- [x] **Mixer-View**: Traditionelle Kanalzug-Ansicht
- [x] **Matrix-View**: Routing-Kreuzschiene
- [x] **Meters-View**: Großes Meter-Display
- [x] **Settings-View**: Konfiguration
- [x] **Scenes-View**: Preset-Verwaltung
- [x] View-Tabs oder Split-Screen

#### 4.3 Mixer-View Details
- [ ] Skalierbare Kanalbreite
- [ ] Kanalzüge horizontal scrollbar
- [ ] Layer/Bank-System (8 Kanäle pro Bank)
- [ ] Fader-Flip Modus
- [ ] Selected Channel Detail-Ansicht
- [ ] Quick-Access Toolbar

#### 4.4 Touch-Optimierung
- [ ] Große Touch-Targets (min. 44px)
- [ ] Swipe-Gesten für Navigation
- [ ] Long-Press Kontextmenüs
- [ ] Pinch-to-Zoom für Matrix
- [ ] Multi-Touch Fader (mehrere gleichzeitig)
- [ ] Haptic Feedback (wo verfügbar)

#### 4.5 Spezielle UI-Elemente
- [ ] Motorisierte Fader-Animation
- [ ] Smooth Fader-Bewegung (Interpolation)
- [ ] VU-Meter Animation (Ballistics)
- [ ] Kanal-Drag & Drop (Reihenfolge ändern)
- [ ] Keyboard Shortcuts
- [ ] Undo/Redo System

---

### Phase 5: Server & Netzwerk

#### 5.1 Server-Architektur
- [ ] Rust-basierter Audio-Server (oder C++ mit guter Abstraktion)
- [ ] Systemd Service-Integration (Linux)
- [ ] Automatischer Start beim Boot
- [ ] Graceful Shutdown
- [ ] Crash-Recovery
- [ ] Logging-System (strukturierte Logs)
- [ ] Konfigurations-Management (YAML/TOML)

#### 5.2 Client-Server Kommunikation
- [ ] WebSocket-Verbindung (primär)
- [ ] Binäres Protokoll für Audio-Meter (Performance)
- [ ] JSON-RPC für Steuerung
- [ ] Heartbeat/Reconnect-Logik
- [ ] State-Synchronisation bei Reconnect
- [ ] Optimistic Updates mit Rollback

#### 5.3 Multi-Client Support
- [ ] Mehrere Clients gleichzeitig
- [ ] Client-Priorisierung (optional)
- [ ] "Follow" Modus (Client folgt anderem)
- [ ] Konflikterkennung bei gleichzeitiger Änderung
- [ ] Client-Disconnect Handling
- [ ] Maximale Client-Anzahl konfigurierbar

#### 5.4 API für Hausautomatisierung
- [ ] RESTful HTTP API
  - [ ] GET /api/channels - Alle Kanäle
  - [ ] GET /api/channels/{id} - Einzelner Kanal
  - [ ] PATCH /api/channels/{id} - Kanal ändern
  - [ ] GET /api/routing - Routing-Matrix
  - [ ] POST /api/scenes/recall - Scene abrufen
  - [ ] WebSocket /api/ws - Live-Updates
- [ ] API-Authentifizierung (Token-basiert)
- [ ] Rate-Limiting
- [ ] OpenAPI/Swagger Dokumentation
- [ ] Home Assistant Integration (Custom Component)
- [ ] MQTT Support (optional)
- [ ] Node-RED Nodes (optional)

#### 5.5 Sicherheit
- [ ] TLS/HTTPS für alle Verbindungen
- [ ] Client-Zertifikate (optional)
- [ ] Benutzer-Authentifizierung
- [ ] Rollen-System (Admin, Operator, Viewer)
- [ ] Audit-Log
- [ ] Firewall-Empfehlungen dokumentieren

---

### Phase 6: Szenen & Presets

#### 6.1 Scene-System
- [ ] Vollständige Mixer-Snapshots
- [ ] Selektives Scene-Recall (nur bestimmte Parameter)
- [ ] Scene-Crossfade (zeitbasiert)
- [ ] Scene-Safe (Kanäle von Recall ausschließen)
- [ ] 100+ Scene-Speicherplätze
- [ ] Scene-Benennung und Notizen

#### 6.2 Cue-Listen
- [ ] Geordnete Cue-Liste
- [ ] GO-Button Trigger
- [ ] Auto-Follow (zeitbasiert)
- [ ] Cue-Vorschau
- [ ] MIDI/OSC Trigger

#### 6.3 Projektmanagement
- [ ] Projekt-Dateien (enthält alles)
- [ ] Import/Export
- [ ] Cloud-Sync (optional, später)
- [ ] Vorlagen-System
- [ ] Auto-Save

---

### Phase 7: Raspberry Pi Optimierung

#### 7.1 Performance-Analyse
- [ ] CPU-Profiling auf RPi4
- [ ] Memory-Footprint optimieren
- [ ] Audio-Latenz messen
- [ ] Maximale Kanal-Anzahl ermitteln
- [ ] Thermal Throttling vermeiden

#### 7.2 Lite-Version
- [ ] Konfigurierbare Kanal-Anzahl (8/16/24/32)
- [ ] Deaktivierbare Features
- [ ] Headless-Modus (nur Server, kein Display)
- [ ] GPIO-Integration für Hardware-Buttons
- [ ] I2C Display Support (Status)

#### 7.3 Deployment
- [ ] Raspberry Pi OS Image vorbereiten
- [ ] One-Click Installer
- [ ] Automatische Updates
- [ ] Readonly Filesystem Option
- [ ] Backup/Restore über USB

---

### Phase 8: Qualitätssicherung

#### 8.1 Testing
- [ ] Unit Tests (Audio-Engine)
- [ ] Integration Tests (API)
- [ ] End-to-End Tests (UI)
- [ ] Performance-Benchmarks
- [ ] Audio-Qualität Tests (THD+N, Frequenzgang)
- [ ] Stress-Tests (72h Dauerbetrieb)

#### 8.2 Dokumentation
- [ ] Benutzerhandbuch
- [ ] Installations-Anleitung
- [ ] API-Dokumentation
- [ ] Entwickler-Guide
- [ ] Troubleshooting-Guide
- [ ] Video-Tutorials

#### 8.3 Community & Support
- [ ] GitHub Issues Templates
- [ ] Discussion Forum
- [ ] Discord/Matrix Server
- [ ] FAQ
- [ ] Changelog

---

## 🛠️ Technologie-Stack (Vorschlag)

### Server (Linux/Raspberry Pi)
| Komponente | Technologie | Begründung |
|------------|-------------|------------|
| Audio-Engine | **Rust** mit CPAL/Jack | Memory-Safety, Performance, keine GC |
| Audio-Framework | **JACK Audio** oder **PipeWire** | Low-Latency, Linux-Standard, AES67-fähig |
| AES67 | **PipeWire AES67** oder **Ravenna** | Open Source, DANTE-kompatibel |
| DANTE (später) | **Dante SDK** (optional) | Native Integration wenn benötigt |
| MIDI | **RtMidi** (Rust Bindings) | Cross-Platform, bewährt |
| API Server | **Axum** (Rust) oder **Actix** | Async, WebSocket Support |
| Database | **SQLite** | Embedded, kein Setup |
| Config | **TOML** | Lesbar, Rust-nativ |

### Client (Haupt-App am Client - Linux)
| Komponente | Technologie | Begründung |
|------------|-------------|------------|
| Framework | **Tauri** | Rust-Backend, WebView UI, klein |
| UI | **Svelte** + **TypeScript** | Reaktiv, performant, einfach |
| Styling | **TailwindCSS** | Utility-first, Touch-optimiert |
| State | **Svelte Stores** | Built-in, ausreichend |
| Charts/Meter | **Canvas 2D** oder **WebGL** | Performance für Echtzeit |

### Remote (Fernsteuerung - Windows, macOS, Linux)
| Komponente | Technologie | Begründung |
|------------|-------------|------------|
| Framework | **Tauri** | Cross-Platform, gleicher Stack wie App |
| UI | **Svelte** + **TypeScript** | Code-Sharing mit app/ |
| Server-Discovery | **mDNS/Bonjour** | Automatische Server-Erkennung |
| Verbindung | **WebSocket** | Echtzeit, bidirektional |

### Shared (Gemeinsamer Code)
| Komponente | Technologie | Begründung |
|------------|-------------|------------|
| UI-Komponenten | **Svelte** | Wiederverwendbar in app/ und remote/ |
| Typen | **TypeScript** | Type-Safety für Frontend |
| Protokoll | **Rust Crate** | Gemeinsame Message-Definitionen |

### Alternative: Vollständig Native
| Komponente | Technologie | Begründung |
|------------|-------------|------------|
| Alles | **JUCE** (C++) | Industrie-Standard für Audio-Software |

---

## 📊 Meilensteine

### MVP (Minimum Viable Product) - ~3 Monate
- [ ] 8-Kanal Mixer funktionsfähig
- [ ] AES67 Input funktioniert (DANTE-Hardware im AES67-Modus)
- [ ] Basis-UI (Fader, Mute, Meter)
- [ ] MIDI-Learn für Fader
- [ ] Läuft auf Windows Desktop

### Version 1.0 - ~6 Monate
- [ ] 32x32 Matrix vollständig (4x 8-Kanal AES67 Streams)
- [ ] Alle Mixer-Funktionen
- [ ] MIDI-Controller-Profile
- [ ] Multi-Client
- [ ] REST API
- [ ] Stabil auf Raspberry Pi 4

### Version 2.0 - ~12 Monate
- [ ] Effekte (EQ, Kompressor, Gate)
- [ ] Recording-Funktion
- [ ] Mobile App (iOS/Android)
- [ ] Cloud-Sync für Presets
- [ ] Plugin-System
- [ ] Optional: Native DANTE SDK Integration

---

## 💡 Notizen & Ideen

### DANTE / AES67 Strategie

**Aktueller Ansatz:** AES67-Modus (auf DANTE-Hardware aktiviert ✅)

| Aspekt | AES67 (jetzt) | DANTE SDK (später) |
|--------|---------------|-------------------|
| Lizenz | Open Source möglich | Kostenpflichtig |
| Sample Rate | 48kHz | 44.1k / 48k / 96k |
| Kanäle/Stream | 8 | bis 512 |
| Latenz | ~2ms | ~0.15ms möglich |
| Discovery | SAP/SDP | Dante Browse |

**Architektur-Prinzip:** Abstraktionsschicht `AudioNetworkBackend`
```
┌─────────────────────────────────────┐
│         Mixer-Engine                │
├─────────────────────────────────────┤
│     AudioNetworkBackend (Trait)     │
├──────────────┬──────────────────────┤
│ Aes67Backend │ DanteBackend (später)│
└──────────────┴──────────────────────┘
```

### Hardware-Empfehlungen für Raspberry Pi
- Raspberry Pi 4 (4GB oder 8GB RAM)
- Aktive Kühlung (Lüfter oder großer Kühlkörper)
- USB 3.0 Audio Interface als Fallback
- Ethernet (kein WLAN für Audio!)
- SSD statt SD-Karte (Zuverlässigkeit)

### Performance-Ziele
| Metrik | Ziel |
|--------|------|
| Audio-Latenz | < 5ms (Server-intern) |
| UI-Responsiveness | < 16ms (60fps) |
| CPU (RPi4, 16ch) | < 50% |
| RAM | < 512MB |
| Startup | < 10 Sekunden |

---

## 📁 Projektstruktur

```
audiomultiverse/
│
├── server/                    # 🐧 Audio-Server (Linux, Raspberry Pi)
│   ├── src/
│   │   ├── main.rs
│   │   ├── audio/            # Audio-Engine
│   │   │   ├── engine.rs
│   │   │   ├── channel.rs
│   │   │   ├── routing.rs
│   │   │   └── metering.rs
│   │   ├── network_audio/    # AES67/DANTE Abstraktion
│   │   │   ├── mod.rs        # AudioNetworkBackend Trait
│   │   │   ├── aes67.rs      # AES67 Implementation
│   │   │   ├── dante.rs      # DANTE SDK (später)
│   │   │   ├── discovery.rs  # SAP/SDP + Dante Browse
│   │   │   └── ptp.rs        # IEEE 1588 Clock Sync
│   │   ├── midi/             # MIDI-Handler
│   │   │   ├── controller.rs
│   │   │   └── mapping.rs
│   │   ├── api/              # REST/WebSocket API
│   │   │   ├── routes.rs
│   │   │   └── websocket.rs
│   │   └── config/           # Konfiguration
│   ├── Cargo.toml
│   └── config.toml
│
├── app/                       # 🖥️ Haupt-App (Touch-optimiert, am Server)
│   ├── src/                  # Svelte Frontend
│   │   ├── lib/
│   │   │   ├── components/   # UI Komponenten
│   │   │   │   ├── Fader.svelte
│   │   │   │   ├── Meter.svelte
│   │   │   │   ├── ChannelStrip.svelte
│   │   │   │   └── Matrix.svelte
│   │   │   ├── stores/       # State Management
│   │   │   └── api/          # Server-Kommunikation
│   │   ├── routes/           # Seiten
│   │   └── app.html
│   ├── src-tauri/            # Tauri Rust Backend
│   ├── package.json
│   └── tailwind.config.js
│
├── remote/                    # 🌐 Fernsteuerung (Windows, macOS, Linux)
│   ├── src/                  # Svelte Frontend (geteilt mit app/)
│   │   ├── lib/
│   │   │   ├── components/   # Gemeinsame + Remote-spezifische
│   │   │   ├── stores/
│   │   │   └── api/
│   │   └── routes/
│   ├── src-tauri/            # Tauri Backend
│   │   └── src/
│   │       └── main.rs       # Server-Discovery, Connection
│   ├── package.json
│   └── tauri.conf.json
│
├── shared/                    # 📦 Gemeinsamer Code
│   ├── ui-components/        # Geteilte Svelte-Komponenten
│   │   ├── Fader.svelte
│   │   ├── Meter.svelte
│   │   ├── ChannelStrip.svelte
│   │   └── Matrix.svelte
│   ├── types/                # TypeScript Typen
│   │   ├── channel.ts
│   │   ├── routing.ts
│   │   └── api.ts
│   └── protocol/             # Rust Protokoll-Definitionen
│       ├── Cargo.toml
│       └── src/
│           ├── lib.rs
│           ├── messages.rs
│           └── types.rs
│
├── docs/                      # 📚 Dokumentation
│   ├── user-manual/
│   ├── api/
│   └── development/
│
├── deploy/                    # 🚀 Deployment
│   ├── docker/
│   ├── systemd/
│   ├── raspberry-pi/
│   ├── jenkins/              # Jenkins Agent Dokumentation
│   │   └── AGENTS.md
│   └── innosetup/            # Windows Installer
│       └── remote.iss
│
├── .github/
│   └── workflows/            # GitHub Actions (optional)
│       ├── build-server.yml
│       ├── build-app.yml
│       └── build-remote.yml
│
├── Jenkinsfile               # CI/CD Pipeline (Haupt-Build)
├── pnpm-workspace.yaml       # Monorepo Config
├── package.json              # Root package.json
├── README.md
├── LICENSE
└── TODO.md
```

### Projektübersicht

| Projekt | Beschreibung | Plattform | Tech-Stack |
|---------|--------------|-----------|------------|
| **server/** | Audio-Engine, AES67, MIDI, API | Linux, Raspberry Pi | Rust |
| **app/** | Haupt-UI am Server (Touchscreen) | Linux | Tauri + Svelte |
| **remote/** | Fernsteuerungs-Client | Windows, macOS, Linux | Tauri + Svelte |
| **shared/** | Gemeinsame Komponenten & Typen | - | Svelte, TypeScript, Rust |

---

## ✅ Erste Schritte

1. [ ] Repository strukturieren (siehe oben)
2. [ ] Rust-Entwicklungsumgebung aufsetzen
3. [ ] Node.js/pnpm für Frontend
4. [ ] JACK Audio auf Linux installieren/konfigurieren
5. [ ] Erstes "Hello World" mit Audio-Passthrough
6. [ ] Basis-WebSocket Server
7. [ ] Einfaches Fader-UI

---

*Zuletzt aktualisiert: 08. Januar 2026*
