# AudioMultiverse Server Dockerfile
# Multi-Stage Build für optimale Image-Größe

# Build Stage
FROM rust:1.75-bookworm AS builder

# Build-Dependencies installieren
RUN apt-get update && apt-get install -y \
    libasound2-dev \
    libjack-jackd2-dev \
    pkg-config \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /app

# Cargo.toml und Cargo.lock zuerst kopieren (für Layer-Caching)
COPY server/Cargo.toml server/Cargo.lock* ./server/
COPY shared/protocol/Cargo.toml ./shared/protocol/

# Dummy src erstellen für Dependency-Caching
RUN mkdir -p server/src shared/protocol/src && \
    echo "fn main() {}" > server/src/main.rs && \
    echo "pub fn dummy() {}" > shared/protocol/src/lib.rs

# Dependencies vorab kompilieren
WORKDIR /app/server
RUN cargo build --release || true

# Echten Source-Code kopieren
WORKDIR /app
COPY server/src ./server/src
COPY shared/protocol/src ./shared/protocol/src

# Release-Build
WORKDIR /app/server
RUN touch src/main.rs && cargo build --release

# Runtime Stage
FROM debian:bookworm-slim

# Runtime-Dependencies installieren
RUN apt-get update && apt-get install -y \
    libasound2 \
    libjack-jackd2-0 \
    ca-certificates \
    && rm -rf /var/lib/apt/lists/*

# Benutzer erstellen (nicht als root laufen)
RUN useradd -m -s /bin/bash audiomultiverse
USER audiomultiverse

WORKDIR /app

# Binary kopieren
COPY --from=builder /app/server/target/release/audiomultiverse-server ./

# Konfiguration
COPY --chown=audiomultiverse:audiomultiverse server/config.toml.example ./config.toml

# Ports
# 8080 - HTTP/WebSocket API
# 5004 - AES67 RTP (Audio)
# 319/320 - PTP (Zeit-Synchronisation)
EXPOSE 8080
EXPOSE 5004/udp
EXPOSE 319/udp
EXPOSE 320/udp

# Health Check
HEALTHCHECK --interval=30s --timeout=3s --start-period=5s \
    CMD curl -f http://localhost:8080/health || exit 1

# Starten
CMD ["./audiomultiverse-server"]
