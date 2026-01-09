#!/bin/bash
# AudioMultiverse Server - Post-Install Script für Debian/Ubuntu
# Wird nach der .deb Installation ausgeführt

set -e

# Benutzer erstellen
if ! id "audiomultiverse" &>/dev/null; then
    useradd --system --user-group --shell /bin/false \
        --home /opt/audiomultiverse \
        --comment "AudioMultiverse Server" \
        audiomultiverse
    usermod -a -G audio audiomultiverse
fi

# Verzeichnisse erstellen
mkdir -p /opt/audiomultiverse/data
mkdir -p /opt/audiomultiverse/midi-mappings
mkdir -p /etc/audiomultiverse

# Berechtigungen setzen
chown -R audiomultiverse:audiomultiverse /opt/audiomultiverse

# Konfiguration kopieren (nur wenn nicht vorhanden)
if [ ! -f /etc/audiomultiverse/config.toml ]; then
    cp /opt/audiomultiverse/config.toml.example /etc/audiomultiverse/config.toml
    ln -sf /etc/audiomultiverse/config.toml /opt/audiomultiverse/config.toml
fi

# Systemd Service aktivieren
systemctl daemon-reload
systemctl enable audiomultiverse.service

echo ""
echo "╔════════════════════════════════════════════════════════════╗"
echo "║  AudioMultiverse Server erfolgreich installiert!          ║"
echo "╠════════════════════════════════════════════════════════════╣"
echo "║  Konfiguration: /etc/audiomultiverse/config.toml          ║"
echo "║  Daten:         /opt/audiomultiverse/data                 ║"
echo "║                                                            ║"
echo "║  Server starten:                                           ║"
echo "║    sudo systemctl start audiomultiverse                    ║"
echo "║                                                            ║"
echo "║  Status prüfen:                                            ║"
echo "║    sudo systemctl status audiomultiverse                   ║"
echo "║                                                            ║"
echo "║  Web-Interface: http://localhost:8080                      ║"
echo "╚════════════════════════════════════════════════════════════╝"
echo ""
