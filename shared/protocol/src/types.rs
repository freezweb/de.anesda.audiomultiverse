//! Basis-Typen für AudioMultiverse

use serde::{Deserialize, Serialize};

/// Kanal-ID (0-basiert)
pub type ChannelId = u32;

/// Fader-Wert (0.0 = -∞, 1.0 = 0dB, 1.25 = +10dB)
pub type FaderValue = f32;

/// dB-Wert
pub type Decibel = f32;

/// Pan-Wert (-1.0 = L, 0.0 = C, 1.0 = R)
pub type PanValue = f32;

/// Zustand eines einzelnen Kanals
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChannelState {
    pub id: ChannelId,
    pub name: String,
    pub fader: FaderValue,
    pub mute: bool,
    pub solo: bool,
    pub pan: PanValue,
    pub gain: Decibel,
    pub phase_invert: bool,
    pub color: String,
    pub meter: f32,
}

/// Kompletter Mixer-Zustand
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MixerState {
    pub channels: Vec<ChannelState>,
    pub routing: Vec<Vec<f32>>,
    pub input_count: u32,
    pub output_count: u32,
}

/// Meter-Daten für alle Kanäle (kompakt für häufige Updates)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MeterData {
    /// Peak-Werte pro Kanal (0.0 - 1.0)
    pub peaks: Vec<f32>,
    
    /// Timestamp in Millisekunden
    pub timestamp: u64,
}

/// MIDI-Controller Mapping
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MidiMapping {
    /// MIDI-Gerätename
    pub device: String,
    
    /// Kanal (0-15)
    pub channel: u8,
    
    /// Control Change Nummer (0-127)
    pub cc: u8,
    
    /// Ziel-Parameter
    pub target: MidiTarget,
}

/// MIDI-Mapping Ziel
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum MidiTarget {
    /// Fader eines Kanals
    #[serde(rename = "fader")]
    Fader { channel: ChannelId },
    
    /// Mute eines Kanals
    #[serde(rename = "mute")]
    Mute { channel: ChannelId },
    
    /// Solo eines Kanals
    #[serde(rename = "solo")]
    Solo { channel: ChannelId },
    
    /// Pan eines Kanals
    #[serde(rename = "pan")]
    Pan { channel: ChannelId },
    
    /// Master-Fader
    #[serde(rename = "master")]
    Master,
}

/// Scene/Preset
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Scene {
    /// Scene-ID
    pub id: u32,
    
    /// Name
    pub name: String,
    
    /// Beschreibung
    pub description: Option<String>,
    
    /// Gespeicherter Mixer-State
    pub state: MixerState,
    
    /// Erstellungszeitpunkt (Unix Timestamp)
    pub created_at: u64,
    
    /// Letztes Update
    pub updated_at: u64,
}

/// Server-Info
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServerInfo {
    /// Servername
    pub name: String,
    
    /// Version
    pub version: String,
    
    /// Anzahl Eingänge
    pub input_count: u32,
    
    /// Anzahl Ausgänge
    pub output_count: u32,
    
    /// Sample Rate
    pub sample_rate: u32,
    
    /// Aktive Clients
    pub client_count: u32,
    
    /// Audio-Backend ("aes67", "dante", "jack", etc.)
    pub audio_backend: String,
}

/// Client-Info (bei Verbindung)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClientInfo {
    /// Client-Name
    pub name: String,
    
    /// Client-Typ ("app", "remote", "api")
    pub client_type: String,
    
    /// Version
    pub version: String,
    
    /// Unterstützte Features
    pub features: Vec<String>,
}

// === AES67 Network Audio Types ===

/// AES67 Status
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Aes67Status {
    /// AES67 aktiviert
    pub enabled: bool,
    /// PTP synchronisiert
    pub ptp_synchronized: bool,
    /// PTP Offset in Nanosekunden
    pub ptp_offset_ns: i64,
    /// Unser Output-Stream (falls aktiv)
    pub our_stream: Option<Aes67StreamInfo>,
    /// Aktuell empfangene Streams
    pub subscribed_streams: Vec<String>,
}

/// AES67 Stream Information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Aes67StreamInfo {
    /// Session ID (eindeutig)
    pub id: String,
    /// Stream Name
    pub name: String,
    /// Anzahl Kanäle
    pub channels: u8,
    /// Sample Rate
    pub sample_rate: u32,
    /// Multicast-Adresse
    pub multicast_addr: String,
    /// Port
    pub port: u16,
    /// Richtung (Send, Receive, SendReceive)
    pub direction: String,
    /// Origin (Host der den Stream sendet)
    pub origin: String,
}
