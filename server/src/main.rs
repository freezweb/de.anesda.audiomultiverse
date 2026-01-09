//! AudioMultiverse Server
//! 
//! Virtuelles DANTE/AES67 Mischpult - Audio Server
//! 
//! Funktionen:
//! - 32x32 Audio Routing Matrix
//! - AES67 Netzwerk-Audio (DANTE-kompatibel)
//! - MIDI Controller Support mit Feedback
//! - WebSocket API für Remote-Clients
//! - REST API für Hausautomatisierung

mod audio;
mod midi;
mod network_audio;
mod api;
mod config;
mod mixer;
mod discovery;

use std::sync::Arc;
use anyhow::Result;
use tokio::sync::RwLock;
use tracing::{info, Level};
use tracing_subscriber::FmtSubscriber;

use crate::config::ServerConfig;
use crate::mixer::{Mixer, SceneManager, MasterSection};
use crate::audio::AudioEngine;
use crate::midi::MidiController;
use crate::api::start_api_server;

#[tokio::main]
async fn main() -> Result<()> {
    // Logging initialisieren
    let subscriber = FmtSubscriber::builder()
        .with_max_level(Level::INFO)
        .with_target(true)
        .with_thread_ids(true)
        .init();

    info!("🎛️ AudioMultiverse Server v{}", env!("CARGO_PKG_VERSION"));
    info!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");

    // Konfiguration laden
    let config = ServerConfig::load()?;
    info!("Konfiguration geladen: {} Eingänge, {} Ausgänge", 
          config.audio.input_channels, 
          config.audio.output_channels);

    // Mixer erstellen
    let mixer = Arc::new(Mixer::new(
        config.audio.input_channels,
        config.audio.output_channels,
    ));
    info!("Mixer initialisiert: {}x{} Matrix", 
          config.audio.input_channels, 
          config.audio.output_channels);

    // Master-Sektion erstellen
    let master = Arc::new(MasterSection::new());
    info!("Master-Sektion initialisiert");

    // Szenen-Manager erstellen
    let scenes_path = shellexpand::tilde("~/.audiomultiverse/scenes").to_string();
    let scene_manager = Arc::new(RwLock::new(SceneManager::new(&scenes_path)));
    info!("Szenen-Manager initialisiert: {}", scenes_path);

    // Audio-Engine initialisieren
    let mut audio_engine = AudioEngine::new(
        config.audio.sample_rate,
        config.audio.buffer_size,
    );
    audio_engine.set_mixer(mixer.clone());
    audio_engine.set_master(master.clone());
    
    // Audio-Geräte auflisten
    info!("Verfügbare Audio-Geräte:");
    for device in audio_engine.list_devices() {
        let flags = format!(
            "{}{}{}",
            if device.is_input { "I" } else { "-" },
            if device.is_output { "O" } else { "-" },
            if device.is_default { "*" } else { "" }
        );
        info!("   [{}] {}", flags, device.name);
    }

    // Audio-Engine starten
    if config.audio.enabled {
        match audio_engine.start() {
            Ok(_) => info!("Audio-Engine gestartet"),
            Err(e) => info!("Audio-Engine konnte nicht gestartet werden: {} (kein Audio-Passthrough)", e),
        }
    }

    // AES67 Network Audio initialisieren
    if config.audio.aes67_enabled.unwrap_or(true) {
        match audio_engine.init_aes67(None) {
            Ok(_) => {
                info!("🌐 AES67 Network Audio initialisiert");
                
                // Warte kurz und zeige entdeckte Geräte
                tokio::time::sleep(tokio::time::Duration::from_secs(2)).await;
                match audio_engine.discover_aes67_devices() {
                    Ok(devices) => {
                        if devices.is_empty() {
                            info!("   Keine AES67 Geräte im Netzwerk gefunden (noch)");
                        } else {
                            info!("   {} AES67 Gerät(e) gefunden:", devices.len());
                            for device in &devices {
                                info!("     - {} ({} channels)", device.name, device.channels);
                            }
                        }
                    }
                    Err(e) => info!("   Fehler bei AES67 Discovery: {}", e),
                }
            }
            Err(e) => info!("AES67 konnte nicht initialisiert werden: {} (nur lokales Audio)", e),
        }
    }

    // AES67-Referenzen für API Server extrahieren
    let (sap_discovery, ptp_clock) = (
        audio_engine.sap_discovery(),
        audio_engine.ptp_clock(),
    );
    
    // Command-Sender für Echtzeit-Steuerung
    // Hinweis: AudioEngine ist nicht Send wegen cpal::Stream, daher können wir
    // keine Commands asynchron verarbeiten. Für echte Echtzeit-Subscription
    // müsste das Audio-System komplett überarbeitet werden.
    // Aktuell: SAP Discovery und PTP Status funktionieren, Subscribe ist TODO.
    let audio_cmd: Option<crate::audio::AudioCommandSender> = None;

    // MIDI initialisieren
    let mut midi_controller = MidiController::new();
    midi_controller.set_mixer(mixer.clone());
    
    if config.midi.enabled {
        match midi_controller.init() {
            Ok(_) => info!("MIDI Controller initialisiert"),
            Err(e) => info!("MIDI konnte nicht initialisiert werden: {}", e),
        }
    }

    // mDNS Discovery Service starten
    let mut discovery_service = match discovery::DiscoveryService::new() {
        Ok(svc) => {
            info!("mDNS/DNS-SD Discovery Service gestartet");
            Some(svc)
        }
        Err(e) => {
            info!("mDNS Discovery nicht verfügbar: {}", e);
            None
        }
    };
    
    if let Some(ref mut svc) = discovery_service {
        if let Err(e) = svc.register_server(
            "AudioMultiverse",
            config.api.port,
            env!("CARGO_PKG_VERSION"),
        ) {
            info!("Konnte Server nicht im Netzwerk registrieren: {}", e);
        }
    }

    // API Server starten (blockiert)
    info!("API Server startet auf {}:{}", config.api.host, config.api.port);
    start_api_server(
        config.api.clone(),
        mixer.clone(),
        scene_manager.clone(),
        master.clone(),
        sap_discovery,
        ptp_clock,
        audio_cmd,
    ).await?;

    Ok(())
}