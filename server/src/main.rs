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
use crate::audio::{AudioEngine, AudioCommandSender};
use crate::midi::MidiController;
use crate::api::start_api_server;
use crate::network_audio::{SapDiscovery, PtpClock};

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

    // Audio-Engine Konfiguration für den Thread vorbereiten
    let audio_sample_rate = config.audio.sample_rate;
    let audio_buffer_size = config.audio.buffer_size;
    let audio_enabled = config.audio.enabled;
    let aes67_enabled = config.audio.aes67_enabled.unwrap_or(true);
    let mixer_for_audio = mixer.clone();
    let master_for_audio = master.clone();
    
    // Command-Channel erstellen (Sender bleibt hier, Receiver geht in den Thread)
    let (cmd_tx, cmd_rx) = tokio::sync::mpsc::channel::<crate::audio::AudioCommand>(32);
    let audio_cmd = AudioCommandSender::from_sender(cmd_tx);
    
    // Channels für SAP/PTP Referenzen aus dem Audio-Thread
    let (sap_tx, sap_rx) = std::sync::mpsc::channel::<Option<Arc<SapDiscovery>>>();
    let (ptp_tx, ptp_rx) = std::sync::mpsc::channel::<Option<Arc<PtpClock>>>();
    
    // Audio Engine komplett im eigenen Thread erstellen und starten
    // Damit vermeiden wir das Send-Problem mit cpal::Stream
    let audio_running = Arc::new(std::sync::atomic::AtomicBool::new(true));
    let audio_running_clone = audio_running.clone();
    
    let audio_thread = std::thread::spawn(move || {
        info!("🔊 Audio-Thread gestartet");
        
        // AudioEngine HIER erstellen (nicht im Hauptthread)
        let mut audio_engine = AudioEngine::new(audio_sample_rate, audio_buffer_size);
        audio_engine.set_mixer(mixer_for_audio);
        audio_engine.set_master(master_for_audio);
        audio_engine.set_command_receiver(cmd_rx);
        
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
        if audio_enabled {
            match audio_engine.start() {
                Ok(_) => info!("Audio-Engine gestartet"),
                Err(e) => info!("Audio-Engine konnte nicht gestartet werden: {} (kein Audio-Passthrough)", e),
            }
        }

        // AES67 Network Audio initialisieren
        if aes67_enabled {
            match audio_engine.init_aes67(None) {
                Ok(_) => {
                    info!("🌐 AES67 Network Audio initialisiert");
                }
                Err(e) => info!("AES67 konnte nicht initialisiert werden: {} (nur lokales Audio)", e),
            }
        }
        
        // SAP/PTP Referenzen an Hauptthread senden
        let _ = sap_tx.send(audio_engine.sap_discovery());
        let _ = ptp_tx.send(audio_engine.ptp_clock());
        
        // Command-Loop
        while audio_running_clone.load(std::sync::atomic::Ordering::Relaxed) {
            audio_engine.process_commands();
            std::thread::sleep(std::time::Duration::from_millis(10));
        }
        
        info!("🔊 Audio-Thread beendet");
    });
    
    // Warte auf SAP/PTP Referenzen vom Audio-Thread
    let sap_discovery = sap_rx.recv().ok().flatten();
    let ptp_clock = ptp_rx.recv().ok().flatten();
    
    // Kurz warten für AES67 Discovery
    if sap_discovery.is_some() {
        tokio::time::sleep(tokio::time::Duration::from_secs(2)).await;
        if let Some(ref sap) = sap_discovery {
            let streams = sap.streams();
            if streams.is_empty() {
                info!("   Keine AES67 Geräte im Netzwerk gefunden (noch)");
            } else {
                info!("   {} AES67 Gerät(e) gefunden:", streams.len());
                for stream in &streams {
                    info!("     - {} ({} channels)", stream.name, stream.channels);
                }
            }
        }
    }

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
        Some(audio_cmd),
    ).await?;
    
    // Cleanup: Audio-Thread stoppen
    audio_running.store(false, std::sync::atomic::Ordering::Relaxed);
    let _ = audio_thread.join();

    Ok(())
}