//! WebSocket/API Nachrichten
//! 
//! Alle Nachrichten zwischen Server und Clients

use serde::{Deserialize, Serialize};
use crate::types::*;

/// Nachricht vom Client zum Server
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", content = "payload")]
pub enum ClientMessage {
    // === Verbindung ===
    
    /// Client-Identifikation bei Verbindung
    #[serde(rename = "hello")]
    Hello(ClientInfo),
    
    /// Ping (Heartbeat)
    #[serde(rename = "ping")]
    Ping { timestamp: u64 },
    
    // === Mixer-Steuerung ===
    
    /// Fader-Wert setzen
    #[serde(rename = "set_fader")]
    SetFader { channel: ChannelId, value: FaderValue },
    
    /// Mute setzen
    #[serde(rename = "set_mute")]
    SetMute { channel: ChannelId, muted: bool },
    
    /// Solo setzen
    #[serde(rename = "set_solo")]
    SetSolo { channel: ChannelId, solo: bool },
    
    /// Pan setzen
    #[serde(rename = "set_pan")]
    SetPan { channel: ChannelId, value: PanValue },
    
    /// Gain setzen
    #[serde(rename = "set_gain")]
    SetGain { channel: ChannelId, value: Decibel },
    
    /// Kanalname setzen
    #[serde(rename = "set_channel_name")]
    SetChannelName { channel: ChannelId, name: String },
    
    /// Kanalfarbe setzen
    #[serde(rename = "set_channel_color")]
    SetChannelColor { channel: ChannelId, color: String },
    
    // === Routing ===
    
    /// Routing-Punkt setzen
    #[serde(rename = "set_routing")]
    SetRouting { input: u32, output: u32, gain: f32 },
    
    // === Szenen ===
    
    /// Scene speichern
    #[serde(rename = "save_scene")]
    SaveScene { name: String, description: Option<String> },
    
    /// Scene abrufen
    #[serde(rename = "recall_scene")]
    RecallScene { id: u32 },
    
    /// Scene löschen
    #[serde(rename = "delete_scene")]
    DeleteScene { id: u32 },
    
    // === Abfragen ===
    
    /// Kompletten State anfordern
    #[serde(rename = "get_state")]
    GetState,
    
    /// Server-Info anfordern
    #[serde(rename = "get_server_info")]
    GetServerInfo,
    
    /// Szenen-Liste anfordern
    #[serde(rename = "get_scenes")]
    GetScenes,
    
    /// Meter-Updates abonnieren/abbestellen
    #[serde(rename = "subscribe_meters")]
    SubscribeMeters { enabled: bool, interval_ms: Option<u32> },
    
    // === AES67 Network Audio ===
    
    /// AES67 Status abfragen
    #[serde(rename = "get_aes67_status")]
    GetAes67Status,
    
    /// AES67 Streams abfragen (Discovery)
    #[serde(rename = "get_aes67_streams")]
    GetAes67Streams,
    
    /// Zu AES67 Stream subscriben (Audio empfangen)
    #[serde(rename = "subscribe_aes67_stream")]
    SubscribeAes67Stream { 
        stream_id: String,
        /// Erster lokaler Kanal für Mapping
        start_channel: Option<u32>,
    },
    
    /// AES67 Stream Subscription beenden
    #[serde(rename = "unsubscribe_aes67_stream")]
    UnsubscribeAes67Stream { stream_id: String },
    
    /// Discovery aktualisieren
    #[serde(rename = "refresh_aes67")]
    RefreshAes67,
}

/// Nachricht vom Server zum Client
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", content = "payload")]
pub enum ServerMessage {
    // === Verbindung ===
    
    /// Willkommen (Antwort auf Hello)
    #[serde(rename = "welcome")]
    Welcome {
        server_info: ServerInfo,
        state: MixerState,
    },
    
    /// Pong (Antwort auf Ping)
    #[serde(rename = "pong")]
    Pong { timestamp: u64, server_time: u64 },
    
    /// Fehler
    #[serde(rename = "error")]
    Error { code: String, message: String },
    
    // === State Updates ===
    
    /// Kanal wurde geändert
    #[serde(rename = "channel_updated")]
    ChannelUpdated(ChannelState),
    
    /// Routing wurde geändert
    #[serde(rename = "routing_updated")]
    RoutingUpdated { input: u32, output: u32, gain: f32 },
    
    /// Kompletter State (Antwort auf GetState)
    #[serde(rename = "state")]
    State(MixerState),
    
    /// Server-Info (Antwort auf GetServerInfo)
    #[serde(rename = "server_info")]
    ServerInfo(ServerInfo),
    
    // === Meter ===
    
    /// Meter-Update (häufig, ~20-50ms)
    #[serde(rename = "meters")]
    Meters(MeterData),
    
    // === Szenen ===
    
    /// Szenen-Liste
    #[serde(rename = "scenes")]
    Scenes(Vec<Scene>),
    
    /// Scene wurde gespeichert
    #[serde(rename = "scene_saved")]
    SceneSaved { id: u32, name: String },
    
    /// Scene wurde geladen
    #[serde(rename = "scene_recalled")]
    SceneRecalled { id: u32, name: String },
    
    // === System ===
    
    /// Client verbunden (für andere Clients)
    #[serde(rename = "client_connected")]
    ClientConnected { name: String, client_type: String },
    
    /// Client getrennt
    #[serde(rename = "client_disconnected")]
    ClientDisconnected { name: String },
    
    // === AES67 Network Audio ===
    
    /// AES67 Status
    #[serde(rename = "aes67_status")]
    Aes67Status(Aes67Status),
    
    /// Entdeckte AES67 Streams
    #[serde(rename = "aes67_streams")]
    Aes67Streams(Vec<Aes67StreamInfo>),
    
    /// Stream Subscription erfolgreich
    #[serde(rename = "aes67_subscribed")]
    Aes67Subscribed { 
        stream_id: String,
        stream_name: String,
        channels: u8,
        start_channel: u32,
    },
    
    /// Stream Subscription beendet
    #[serde(rename = "aes67_unsubscribed")]
    Aes67Unsubscribed { stream_id: String },
    
    // === Multi-Client Sync ===
    
    /// Client-Anzahl hat sich geändert (Broadcast an alle Clients)
    #[serde(rename = "client_count_changed")]
    ClientCountChanged { count: u32 },
}

/// REST API Response Wrapper
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApiResponse<T> {
    pub success: bool,
    pub data: Option<T>,
    pub error: Option<String>,
}

impl<T> ApiResponse<T> {
    pub fn ok(data: T) -> Self {
        Self {
            success: true,
            data: Some(data),
            error: None,
        }
    }

    pub fn err(message: impl Into<String>) -> Self {
        Self {
            success: false,
            data: None,
            error: Some(message.into()),
        }
    }
}
