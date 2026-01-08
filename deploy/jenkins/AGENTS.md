# AudioMultiverse - Jenkins Agent Anforderungen

## Übersicht der benötigten Agents

Da Jenkins auf einem Windows Server läuft, werden für Cross-Platform-Builds
verschiedene Agents (Nodes) benötigt.

## Agent: `windows` (auf dem Jenkins Server selbst)

### Software-Anforderungen
- Windows 10/11 oder Windows Server 2019+
- Node.js 20 LTS
- pnpm 8.x
- Rust 1.75+ (mit MSVC toolchain)
- Visual Studio Build Tools 2022
- Inno Setup 6

### Installation

```powershell
# Chocolatey (Package Manager)
Set-ExecutionPolicy Bypass -Scope Process -Force
[System.Net.ServicePointManager]::SecurityProtocol = [System.Net.ServicePointManager]::SecurityProtocol -bor 3072
iex ((New-Object System.Net.WebClient).DownloadString('https://community.chocolatey.org/install.ps1'))

# Build Tools
choco install -y nodejs-lts python visualstudio2022buildtools git

# Rust
choco install -y rustup.install
rustup default stable-msvc

# pnpm
npm install -g pnpm@8

# Inno Setup
choco install -y innosetup
```

---

## Agent: `linux` (VM oder separater Server)

### Empfohlen
- Ubuntu 22.04 LTS oder Debian 12
- Docker alternativ möglich

### Software-Anforderungen
- Node.js 20 LTS
- pnpm 8.x
- Rust 1.75+
- Tauri Build Dependencies
- cargo-deb

### Installation

```bash
# System-Updates
sudo apt update && sudo apt upgrade -y

# Node.js 20
curl -fsSL https://deb.nodesource.com/setup_20.x | sudo -E bash -
sudo apt install -y nodejs

# pnpm
npm install -g pnpm@8

# Rust
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
source "$HOME/.cargo/env"

# Tauri Dependencies
sudo apt install -y \
    build-essential \
    libwebkit2gtk-4.1-dev \
    libappindicator3-dev \
    librsvg2-dev \
    patchelf \
    libssl-dev \
    libasound2-dev \
    libjack-jackd2-dev \
    pkg-config

# cargo-deb für .deb Pakete
cargo install cargo-deb

# Jenkins Agent
sudo apt install -y openjdk-17-jre-headless
```

### Jenkins Agent verbinden

```bash
# Als Service einrichten
sudo mkdir -p /opt/jenkins-agent
cd /opt/jenkins-agent

# Agent JAR herunterladen (URL aus Jenkins UI)
wget http://jenkins-server:8080/jnlpJars/agent.jar

# Service erstellen
sudo tee /etc/systemd/system/jenkins-agent.service > /dev/null << 'EOF'
[Unit]
Description=Jenkins Agent
After=network.target

[Service]
Type=simple
User=jenkins
WorkingDirectory=/opt/jenkins-agent
ExecStart=/usr/bin/java -jar agent.jar -jnlpUrl http://jenkins-server:8080/computer/linux/jenkins-agent.jnlp -secret YOUR_SECRET -workDir /opt/jenkins-agent
Restart=always

[Install]
WantedBy=multi-user.target
EOF

sudo systemctl enable jenkins-agent
sudo systemctl start jenkins-agent
```

---

## Agent: `linux-arm64` (Raspberry Pi oder ARM Server)

### Empfohlen
- Raspberry Pi 4 (8GB) oder ARM64 VM
- Ubuntu 22.04 ARM64 oder Raspberry Pi OS 64-bit

### Installation

```bash
# Wie Linux-Agent, aber mit ARM-spezifischen Anpassungen

# Rust mit ARM Target
rustup target add aarch64-unknown-linux-gnu

# Für Cross-Compile von x64 nach ARM:
sudo apt install -y gcc-aarch64-linux-gnu
```

---

## Agent: `android` (Linux mit Android SDK)

### Empfohlen
- Ubuntu 22.04 LTS
- Kann der gleiche wie `linux` Agent sein

### Software-Anforderungen
- Alles von `linux` Agent
- Android SDK (Command Line Tools)
- Android NDK 25.x
- Java 17

### Installation

```bash
# Java 17
sudo apt install -y openjdk-17-jdk

# Android SDK
export ANDROID_HOME=/opt/android-sdk
sudo mkdir -p $ANDROID_HOME
cd $ANDROID_HOME

# Command Line Tools herunterladen
wget https://dl.google.com/android/repository/commandlinetools-linux-10406996_latest.zip
unzip commandlinetools-linux-*.zip
mkdir -p cmdline-tools/latest
mv cmdline-tools/* cmdline-tools/latest/ 2>/dev/null || true

# SDK Manager konfigurieren
export PATH=$PATH:$ANDROID_HOME/cmdline-tools/latest/bin

# Lizenzen akzeptieren
yes | sdkmanager --licenses

# SDK Komponenten installieren
sdkmanager "platform-tools"
sdkmanager "platforms;android-34"
sdkmanager "build-tools;34.0.0"
sdkmanager "ndk;25.2.9519653"

# Rust Android Targets
source "$HOME/.cargo/env"
rustup target add aarch64-linux-android
rustup target add armv7-linux-androideabi
rustup target add x86_64-linux-android

# Environment permanent setzen
echo 'export ANDROID_HOME=/opt/android-sdk' >> ~/.bashrc
echo 'export ANDROID_NDK_HOME=$ANDROID_HOME/ndk/25.2.9519653' >> ~/.bashrc
echo 'export PATH=$PATH:$ANDROID_HOME/cmdline-tools/latest/bin:$ANDROID_HOME/platform-tools' >> ~/.bashrc
```

### Android Keystore erstellen

```bash
keytool -genkey -v \
    -keystore audiomultiverse.keystore \
    -alias audiomultiverse \
    -keyalg RSA \
    -keysize 2048 \
    -validity 10000
```

---

## Agent: `macos` (Optional)

### Anforderungen
- macOS 12+ (Monterey oder neuer)
- Xcode Command Line Tools
- Apple Developer Account für Signierung

### Installation

```bash
# Xcode CLI
xcode-select --install

# Homebrew
/bin/bash -c "$(curl -fsSL https://raw.githubusercontent.com/Homebrew/install/HEAD/install.sh)"

# Tools
brew install node pnpm rust

# Jenkins Agent
brew install openjdk@17
```

---

## Jenkins Credentials einrichten

Folgende Credentials müssen in Jenkins konfiguriert werden:

| ID | Typ | Beschreibung |
|----|-----|--------------|
| `tauri-private-key` | Secret Text | Tauri Updater Private Key |
| `android-keystore` | Secret File | Android Signing Keystore (.keystore) |
| `android-keystore-password` | Secret Text | Keystore Passwort |
| `apple-developer-cert` | Secret File | Apple Developer Certificate (.p12) |
| `apple-cert-password` | Secret Text | Certificate Passwort |
| `apple-id` | Secret Text | Apple Developer Account Email |
| `apple-team-id` | Secret Text | Apple Team ID |

### Credentials hinzufügen

1. Jenkins → Manage Jenkins → Credentials
2. (global) → Add Credentials
3. Typ auswählen und Werte eintragen

---

## Docker Alternative

Für Linux und Android Builds kann auch Docker verwendet werden:

```yaml
# docker-compose.yml für Build-Agents
version: '3.8'

services:
  linux-builder:
    image: rust:1.75-bookworm
    volumes:
      - ./workspace:/workspace
    command: sleep infinity
    
  android-builder:
    build:
      context: ./deploy/docker
      dockerfile: Dockerfile.android
    volumes:
      - ./workspace:/workspace
```

Dockerfile für Android:

```dockerfile
# deploy/docker/Dockerfile.android
FROM rust:1.75-bookworm

# Android SDK
ENV ANDROID_HOME=/opt/android-sdk
ENV ANDROID_NDK_HOME=$ANDROID_HOME/ndk/25.2.9519653

RUN apt-get update && apt-get install -y \
    openjdk-17-jdk \
    wget \
    unzip \
    && rm -rf /var/lib/apt/lists/*

# SDK installieren...
# (wie oben beschrieben)
```

---

## Fehlerbehebung

### Windows: "MSVC not found"
```powershell
# Visual Studio Build Tools mit C++ installieren
choco install visualstudio2022buildtools --package-parameters "--add Microsoft.VisualStudio.Workload.VCTools"
```

### Linux: "webkit2gtk not found"
```bash
# Für Ubuntu 22.04
sudo apt install libwebkit2gtk-4.1-dev

# Für ältere Versionen
sudo apt install libwebkit2gtk-4.0-dev
```

### Android: "NDK not found"
```bash
sdkmanager "ndk;25.2.9519653"
export ANDROID_NDK_HOME=$ANDROID_HOME/ndk/25.2.9519653
```
