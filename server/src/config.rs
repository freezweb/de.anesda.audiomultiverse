//! Konfigurationsmodul
//! 
//! Lädt Server-Konfiguration aus TOML-Datei

use anyhow::{Result, Context};
use serde::{Deserialize, Serialize};
use std::path::Path;
use std::fs;

/// Haupt-Konfiguration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServerConfig {
    pub audio: AudioConfig,
    pub midi: MidiConfig,
    pub api: ApiConfig,
    pub network_audio: NetworkAudioConfig,
}

/// Audio-Engine Konfiguration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AudioConfig {
    /// Audio-Engine aktiviert
    #[serde(default = "default_true")]
    pub enabled: bool,
    
    /// Anzahl der Eingangskanäle (1-64)
    #[serde(default = "default_channels")]
    pub input_channels: usize,
    
    /// Anzahl der Ausgangskanäle (1-64)
    #[serde(default = "default_channels")]
    pub output_channels: usize,
    
    /// Sample Rate (44100, 48000, 96000)
    #[serde(default = "default_sample_rate")]
    pub sample_rate: u32,
    
    /// Buffer-Größe (64, 128, 256, 512, 1024)
    #[serde(default = "default_buffer_size")]
    pub buffer_size: usize,
}

/// MIDI Konfiguration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MidiConfig {
    /// MIDI aktiviert
    #[serde(default = "default_true")]
    pub enabled: bool,
    
    /// Automatisch nach Controllern suchen
    #[serde(default = "default_true")]
    pub auto_connect: bool,
    
    /// MIDI-Mapping Datei
    pub mapping_file: Option<String>,
}

/// API Server Konfiguration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApiConfig {
    /// Host/IP Adresse
    #[serde(default = "default_host")]
    pub host: String,
    
    /// Port
    #[serde(default = "default_port")]
    pub port: u16,
    
    /// CORS erlauben
    #[serde(default = "default_true")]
    pub cors_enabled: bool,
    
    /// Maximale WebSocket Clients
    #[serde(default = "default_max_clients")]
    pub max_clients: usize,
}

/// Netzwerk-Audio Konfiguration (AES67/DANTE)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkAudioConfig {
    /// Backend: "aes67" oder "dante"
    #[serde(default = "default_backend")]
    pub backend: String,
    
    /// Netzwerk-Interface
    pub interface: Option<String>,
    
    /// Multicast-Gruppen für AES67
    #[serde(default)]
    pub multicast_groups: Vec<String>,
}

// Default-Werte
fn default_channels() -> usize { 32 }
fn default_sample_rate() -> u32 { 48000 }
fn default_buffer_size() -> usize { 256 }
fn default_true() -> bool { true }
fn default_host() -> String { "0.0.0.0".to_string() }
fn default_port() -> u16 { 8080 }
fn default_max_clients() -> usize { 10 }
fn default_backend() -> String { "aes67".to_string() }

impl Default for ServerConfig {
    fn default() -> Self {
        Self {
            audio: AudioConfig {
                enabled: true,
                input_channels: 32,
                output_channels: 32,
                sample_rate: 48000,
                buffer_size: 256,
            },
            midi: MidiConfig {
                enabled: true,
                auto_connect: true,
                mapping_file: None,
            },
            api: ApiConfig {
                host: "0.0.0.0".to_string(),
                port: 8080,
                cors_enabled: true,
                max_clients: 10,
            },
            network_audio: NetworkAudioConfig {
                backend: "aes67".to_string(),
                interface: None,
                multicast_groups: vec![],
            },
        }
    }
}

impl ServerConfig {
    /// Konfiguration laden
    pub fn load() -> Result<Self> {
        let config_paths = [
            "config.toml",
            "/etc/audiomultiverse/config.toml",
            "~/.config/audiomultiverse/config.toml",
        ];

        for path in config_paths {
            let expanded_path = shellexpand::tilde(path);
            if Path::new(expanded_path.as_ref()).exists() {
                return Self::load_from_file(expanded_path.as_ref());
            }
        }

        // Keine Konfiguration gefunden, Standard verwenden
        tracing::warn!("Keine config.toml gefunden, verwende Standardwerte");
        Ok(Self::default())
    }

    /// Konfiguration aus Datei laden
    pub fn load_from_file(path: &str) -> Result<Self> {
        let content = fs::read_to_string(path)
            .with_context(|| format!("Konnte {} nicht lesen", path))?;
        
        let config: ServerConfig = toml::from_str(&content)
            .with_context(|| format!("Fehler beim Parsen von {}", path))?;
        
        tracing::info!("Konfiguration geladen von: {}", path);
        Ok(config)
    }

    /// Konfiguration in Datei speichern
    pub fn save_to_file(&self, path: &str) -> Result<()> {
        let content = toml::to_string_pretty(self)
            .context("Fehler beim Serialisieren der Konfiguration")?;
        
        fs::write(path, content)
            .with_context(|| format!("Konnte {} nicht schreiben", path))?;
        
        Ok(())
    }
}
