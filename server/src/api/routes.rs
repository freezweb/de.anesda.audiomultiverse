//! HTTP/REST Routes
//! 
//! REST API Endpoints für Hausautomatisierung und einfache Abfragen

use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};
use axum::{
    routing::{get, patch, post, delete},
    Router,
    Json,
    extract::{Path, State, WebSocketUpgrade},
    response::IntoResponse,
};
use tower_http::cors::{CorsLayer, Any};
use tracing::info;
use tokio::sync::{RwLock, broadcast};
use audiomultiverse_protocol::ServerMessage;

use crate::config::ApiConfig;
use crate::mixer::{Mixer, SceneManager, SceneMetadata, MasterSection, MasterState};
use crate::network_audio::{NetworkDevice, SapDiscovery, PtpClock};
use crate::audio::AudioCommandSender;
use audiomultiverse_protocol::{ApiResponse, ChannelState, MixerState, ServerInfo};

use super::websocket::handle_websocket;

/// App State für alle Handlers
#[derive(Clone)]
pub struct AppState {
    pub mixer: Arc<Mixer>,
    pub config: ApiConfig,
    pub scene_manager: Arc<RwLock<SceneManager>>,
    pub master: Arc<MasterSection>,
    /// SAP Discovery for AES67 stream discovery (thread-safe)
    pub sap_discovery: Option<Arc<SapDiscovery>>,
    /// PTP Clock for AES67 synchronization (thread-safe)
    pub ptp_clock: Option<Arc<PtpClock>>,
    /// Command sender for AudioEngine control (thread-safe)
    pub audio_cmd: Option<AudioCommandSender>,
    /// Broadcast-Channel für Multi-Client-Synchronisation
    /// Alle Änderungen werden an alle verbundenen Clients gesendet
    pub broadcast_tx: broadcast::Sender<ServerMessage>,
    /// Anzahl der verbundenen Clients (atomic für thread-safety)
    pub client_count: Arc<AtomicUsize>,
}

/// API Server starten
pub async fn start_api_server(
    config: ApiConfig,
    mixer: Arc<Mixer>,
    scene_manager: Arc<RwLock<SceneManager>>,
    master: Arc<MasterSection>,
    sap_discovery: Option<Arc<SapDiscovery>>,
    ptp_clock: Option<Arc<PtpClock>>,
    audio_cmd: Option<AudioCommandSender>,
) -> anyhow::Result<()> {
    // Broadcast-Channel für Multi-Client-Sync (Kapazität für bis zu 256 gepufferte Nachrichten)
    let (broadcast_tx, _) = broadcast::channel::<ServerMessage>(256);
    
    let state = AppState {
        mixer,
        config: config.clone(),
        scene_manager,
        master,
        sap_discovery,
        ptp_clock,
        audio_cmd,
        broadcast_tx,
        client_count: Arc::new(AtomicUsize::new(0)),
    };

    // CORS konfigurieren
    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods(Any)
        .allow_headers(Any);

    // Router aufbauen
    let app = Router::new()
        // WebSocket
        .route("/api/ws", get(ws_handler))
        
        // Server Info
        .route("/api/info", get(get_server_info))
        
        // Mixer State
        .route("/api/state", get(get_state))
        
        // Kanäle
        .route("/api/channels", get(get_channels))
        .route("/api/channels/:id", get(get_channel))
        .route("/api/channels/:id", patch(update_channel))
        .route("/api/channels/:id/fader", post(set_fader))
        .route("/api/channels/:id/mute", post(set_mute))
        .route("/api/channels/:id/solo", post(set_solo))
        
        // Routing
        .route("/api/routing", get(get_routing))
        .route("/api/routing", post(set_routing))
        
        // Szenen
        .route("/api/scenes", get(get_scenes))
        .route("/api/scenes", post(save_scene))
        .route("/api/scenes/:id", get(get_scene))
        .route("/api/scenes/:id", delete(delete_scene))
        .route("/api/scenes/:id/recall", post(recall_scene))
        
        // Master-Sektion
        .route("/api/master", get(get_master_state))
        .route("/api/master/fader", post(set_master_fader))
        .route("/api/master/mute", post(set_master_mute))
        .route("/api/master/dim", post(set_master_dim))
        .route("/api/master/mono", post(set_master_mono))
        .route("/api/master/talkback", post(set_master_talkback))
        .route("/api/master/oscillator", post(set_master_oscillator))
        
        // AES67 Network Audio
        .route("/api/aes67/status", get(get_aes67_status))
        .route("/api/aes67/streams", get(get_aes67_streams))
        .route("/api/aes67/streams/:id/subscribe", post(subscribe_aes67_stream))
        .route("/api/aes67/streams/:id/unsubscribe", post(unsubscribe_aes67_stream))
        .route("/api/aes67/refresh", post(refresh_aes67_discovery))
        
        // Health Check
        .route("/health", get(health_check))
        
        .layer(cors)
        .with_state(state);

    // Server binden
    let addr = format!("{}:{}", config.host, config.port);
    let listener = tokio::net::TcpListener::bind(&addr).await?;
    
    info!("🌐 API Server läuft auf http://{}", addr);
    info!("   WebSocket: ws://{}/api/ws", addr);
    
    axum::serve(listener, app).await?;
    
    Ok(())
}

// === Handler Funktionen ===

/// WebSocket Upgrade
async fn ws_handler(
    ws: WebSocketUpgrade,
    State(state): State<AppState>,
) -> impl IntoResponse {
    ws.on_upgrade(move |socket| handle_websocket(socket, state))
}

/// Health Check
async fn health_check() -> &'static str {
    "OK"
}

/// Server Info
async fn get_server_info(State(state): State<AppState>) -> Json<ApiResponse<ServerInfo>> {
    let info = ServerInfo {
        name: "AudioMultiverse".to_string(),
        version: env!("CARGO_PKG_VERSION").to_string(),
        input_count: state.mixer.input_count as u32,
        output_count: state.mixer.output_count as u32,
        sample_rate: 48000,
        client_count: state.client_count.load(Ordering::Relaxed) as u32,
        audio_backend: "aes67".to_string(),
    };
    
    Json(ApiResponse::ok(info))
}

/// Kompletter Mixer State
async fn get_state(State(state): State<AppState>) -> Json<ApiResponse<MixerState>> {
    let mixer_state = state.mixer.get_state();
    Json(ApiResponse::ok(mixer_state))
}

/// Alle Kanäle
async fn get_channels(State(state): State<AppState>) -> Json<ApiResponse<Vec<ChannelState>>> {
    let channels = state.mixer.get_all_channels();
    Json(ApiResponse::ok(channels))
}

/// Einzelner Kanal
async fn get_channel(
    State(state): State<AppState>,
    Path(id): Path<u32>,
) -> Json<ApiResponse<ChannelState>> {
    match state.mixer.get_channel(id) {
        Some(channel) => Json(ApiResponse::ok(channel)),
        None => Json(ApiResponse::err(format!("Kanal {} nicht gefunden", id))),
    }
}

/// Kanal aktualisieren (PATCH)
#[derive(serde::Deserialize)]
pub struct ChannelUpdate {
    pub fader: Option<f32>,
    pub mute: Option<bool>,
    pub solo: Option<bool>,
    pub pan: Option<f32>,
    pub name: Option<String>,
}

async fn update_channel(
    State(state): State<AppState>,
    Path(id): Path<u32>,
    Json(update): Json<ChannelUpdate>,
) -> Json<ApiResponse<ChannelState>> {
    // Updates anwenden
    if let Some(fader) = update.fader {
        state.mixer.set_fader(id, fader);
    }
    if let Some(mute) = update.mute {
        state.mixer.set_mute(id, mute);
    }
    if let Some(solo) = update.solo {
        state.mixer.set_solo(id, solo);
    }
    if let Some(pan) = update.pan {
        state.mixer.set_pan(id, pan);
    }
    if let Some(name) = update.name {
        state.mixer.set_channel_name(id, name);
    }
    
    // Aktuellen State zurückgeben
    match state.mixer.get_channel(id) {
        Some(channel) => Json(ApiResponse::ok(channel)),
        None => Json(ApiResponse::err(format!("Kanal {} nicht gefunden", id))),
    }
}

/// Fader setzen
#[derive(serde::Deserialize)]
pub struct FaderRequest {
    pub value: f32,
}

async fn set_fader(
    State(state): State<AppState>,
    Path(id): Path<u32>,
    Json(req): Json<FaderRequest>,
) -> Json<ApiResponse<ChannelState>> {
    match state.mixer.set_fader(id, req.value) {
        Some(channel) => Json(ApiResponse::ok(channel)),
        None => Json(ApiResponse::err(format!("Kanal {} nicht gefunden", id))),
    }
}

/// Mute setzen
#[derive(serde::Deserialize)]
pub struct MuteRequest {
    pub muted: bool,
}

async fn set_mute(
    State(state): State<AppState>,
    Path(id): Path<u32>,
    Json(req): Json<MuteRequest>,
) -> Json<ApiResponse<ChannelState>> {
    match state.mixer.set_mute(id, req.muted) {
        Some(channel) => Json(ApiResponse::ok(channel)),
        None => Json(ApiResponse::err(format!("Kanal {} nicht gefunden", id))),
    }
}

/// Solo setzen
#[derive(serde::Deserialize)]
pub struct SoloRequest {
    pub solo: bool,
}

async fn set_solo(
    State(state): State<AppState>,
    Path(id): Path<u32>,
    Json(req): Json<SoloRequest>,
) -> Json<ApiResponse<ChannelState>> {
    match state.mixer.set_solo(id, req.solo) {
        Some(channel) => Json(ApiResponse::ok(channel)),
        None => Json(ApiResponse::err(format!("Kanal {} nicht gefunden", id))),
    }
}

/// Routing-Matrix abrufen
async fn get_routing(State(state): State<AppState>) -> Json<ApiResponse<Vec<Vec<f32>>>> {
    let routing = state.mixer.get_routing();
    Json(ApiResponse::ok(routing))
}

/// Routing-Punkt setzen
#[derive(serde::Deserialize)]
pub struct RoutingRequest {
    pub input: usize,
    pub output: usize,
    pub gain: f32,
}

async fn set_routing(
    State(state): State<AppState>,
    Json(req): Json<RoutingRequest>,
) -> Json<ApiResponse<bool>> {
    let success = state.mixer.set_routing(req.input, req.output, req.gain);
    Json(ApiResponse::ok(success))
}

/// Szenen abrufen
async fn get_scenes(State(state): State<AppState>) -> Json<ApiResponse<Vec<SceneMetadata>>> {
    let manager = state.scene_manager.read().await;
    let scenes = manager.list_scenes();
    Json(ApiResponse::ok(scenes))
}

/// Einzelne Szene abrufen
async fn get_scene(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Json<ApiResponse<Option<crate::mixer::Scene>>> {
    let manager = state.scene_manager.read().await;
    let scene = manager.get_scene(&id).cloned();
    Json(ApiResponse::ok(scene))
}

/// Szene speichern
#[derive(serde::Deserialize)]
pub struct SaveSceneRequest {
    pub name: String,
    pub description: Option<String>,
    pub category: Option<String>,
    pub color: Option<String>,
}

async fn save_scene(
    State(state): State<AppState>,
    Json(req): Json<SaveSceneRequest>,
) -> Json<ApiResponse<SceneMetadata>> {
    let mixer_state = state.mixer.get_state();
    let mut manager = state.scene_manager.write().await;
    let scene = manager.create_scene(&req.name, &mixer_state, req.description);
    Json(ApiResponse::ok(scene.metadata))
}

/// Szene löschen
async fn delete_scene(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Json<ApiResponse<bool>> {
    let mut manager = state.scene_manager.write().await;
    match manager.delete_scene(&id) {
        Ok(()) => Json(ApiResponse::ok(true)),
        Err(e) => Json(ApiResponse::err(e.to_string())),
    }
}

/// Szene abrufen/laden
#[derive(serde::Deserialize)]
pub struct RecallSceneRequest {
    pub faders: Option<bool>,
    pub mutes: Option<bool>,
    pub solos: Option<bool>,
    pub pans: Option<bool>,
    pub eq: Option<bool>,
    pub routing: Option<bool>,
    pub names: Option<bool>,
}

async fn recall_scene(
    State(state): State<AppState>,
    Path(id): Path<String>,
    Json(_req): Json<RecallSceneRequest>,
) -> Json<ApiResponse<bool>> {
    let manager = state.scene_manager.read().await;
    
    if let Some(scene) = manager.get_scene(&id) {
        // Szene auf Mixer anwenden
        for ch_state in &scene.channels {
            let id = ch_state.base.id;
            state.mixer.set_fader(id, ch_state.base.fader);
            state.mixer.set_mute(id, ch_state.base.mute);
            state.mixer.set_solo(id, ch_state.base.solo);
            state.mixer.set_pan(id, ch_state.base.pan);
        }
        
        // Routing anwenden
        for (input, row) in scene.routing.iter().enumerate() {
            for (output, &gain) in row.iter().enumerate() {
                state.mixer.set_routing(input, output, gain);
            }
        }
        
        Json(ApiResponse::ok(true))
    } else {
        Json(ApiResponse::err(format!("Szene {} nicht gefunden", id)))
    }
}

// === Master-Sektion Handlers ===

/// Master-State abrufen
async fn get_master_state(State(state): State<AppState>) -> Json<ApiResponse<MasterState>> {
    let master_state = state.master.get_state();
    Json(ApiResponse::ok(master_state))
}

/// Master-Fader setzen
#[derive(serde::Deserialize)]
pub struct MasterFaderRequest {
    pub value: f32,
}

async fn set_master_fader(
    State(state): State<AppState>,
    Json(req): Json<MasterFaderRequest>,
) -> Json<ApiResponse<MasterState>> {
    let new_state = state.master.set_fader(req.value);
    Json(ApiResponse::ok(new_state))
}

/// Master-Mute setzen
#[derive(serde::Deserialize)]
pub struct MasterMuteRequest {
    pub muted: bool,
}

async fn set_master_mute(
    State(state): State<AppState>,
    Json(req): Json<MasterMuteRequest>,
) -> Json<ApiResponse<MasterState>> {
    let new_state = state.master.set_mute(req.muted);
    Json(ApiResponse::ok(new_state))
}

/// Master-DIM setzen
#[derive(serde::Deserialize)]
pub struct MasterDimRequest {
    pub dim: bool,
    pub level: Option<f32>,
}

async fn set_master_dim(
    State(state): State<AppState>,
    Json(req): Json<MasterDimRequest>,
) -> Json<ApiResponse<MasterState>> {
    if let Some(level) = req.level {
        state.master.set_dim_level(level);
    }
    let new_state = state.master.set_dim(req.dim);
    Json(ApiResponse::ok(new_state))
}

/// Master-Mono setzen
#[derive(serde::Deserialize)]
pub struct MasterMonoRequest {
    pub mono: bool,
}

async fn set_master_mono(
    State(state): State<AppState>,
    Json(req): Json<MasterMonoRequest>,
) -> Json<ApiResponse<MasterState>> {
    let new_state = state.master.set_mono(req.mono);
    Json(ApiResponse::ok(new_state))
}

/// Master-Talkback setzen
#[derive(serde::Deserialize)]
pub struct MasterTalkbackRequest {
    pub talkback: bool,
}

async fn set_master_talkback(
    State(state): State<AppState>,
    Json(req): Json<MasterTalkbackRequest>,
) -> Json<ApiResponse<MasterState>> {
    let new_state = state.master.set_talkback(req.talkback);
    Json(ApiResponse::ok(new_state))
}

/// Master-Oszillator setzen
#[derive(serde::Deserialize)]
pub struct MasterOscillatorRequest {
    pub enabled: bool,
    pub frequency: Option<f32>,
    pub level: Option<f32>,
}

async fn set_master_oscillator(
    State(state): State<AppState>,
    Json(req): Json<MasterOscillatorRequest>,
) -> Json<ApiResponse<MasterState>> {
    if let Some(freq) = req.frequency {
        state.master.set_oscillator_freq(freq);
    }
    if let Some(level) = req.level {
        state.master.set_oscillator_level(level);
    }
    let new_state = state.master.set_oscillator(req.enabled);
    Json(ApiResponse::ok(new_state))
}

// === AES67 Network Audio API ===

/// AES67 Status Response
#[derive(serde::Serialize)]
pub struct Aes67Status {
    pub enabled: bool,
    pub ptp_synchronized: bool,
    pub ptp_offset_ns: i64,
    pub our_stream: Option<Aes67StreamInfo>,
    pub subscribed_streams: Vec<String>,
}

/// AES67 Stream Info für API
#[derive(serde::Serialize, Clone)]
pub struct Aes67StreamInfo {
    pub id: String,
    pub name: String,
    pub channels: u8,
    pub sample_rate: u32,
    pub multicast_addr: String,
    pub port: u16,
    pub direction: String,
    pub origin: String,
}

/// Get AES67 status
async fn get_aes67_status(
    State(state): State<AppState>,
) -> Json<ApiResponse<Aes67Status>> {
    let (ptp_sync, offset) = if let Some(ref ptp) = state.ptp_clock {
        (ptp.is_synchronized(), ptp.offset_ns())
    } else {
        (false, 0)
    };
    
    let enabled = state.sap_discovery.is_some();
    
    let status = Aes67Status {
        enabled,
        ptp_synchronized: ptp_sync,
        ptp_offset_ns: offset,
        our_stream: None, // Our output stream info - would need separate tracking
        subscribed_streams: vec![], // TODO: Track subscribed streams via channel
    };
    
    Json(ApiResponse::ok(status))
}

/// Get discovered AES67 streams
async fn get_aes67_streams(
    State(state): State<AppState>,
) -> Json<ApiResponse<Vec<Aes67StreamInfo>>> {
    let streams = if let Some(ref sap) = state.sap_discovery {
        sap.streams()
            .into_iter()
            .map(|s| Aes67StreamInfo {
                id: s.session_id.clone(),
                name: s.name.clone(),
                channels: s.channels,
                sample_rate: s.sample_rate,
                multicast_addr: s.multicast_addr.to_string(),
                port: s.port,
                direction: format!("{:?}", s.direction),
                origin: s.origin.clone(),
            })
            .collect()
    } else {
        vec![]
    };
    
    Json(ApiResponse::ok(streams))
}

/// Subscribe to an AES67 stream (receive audio from it)
#[derive(serde::Deserialize)]
pub struct SubscribeRequest {
    /// Which local input channels to map this stream to (starting channel)
    pub start_channel: Option<u32>,
}

async fn subscribe_aes67_stream(
    State(state): State<AppState>,
    Path(stream_id): Path<String>,
    Json(req): Json<SubscribeRequest>,
) -> Json<ApiResponse<String>> {
    // Check if we have a command sender
    let audio_cmd = match &state.audio_cmd {
        Some(cmd) => cmd.clone(),
        None => return Json(ApiResponse::err("AudioEngine not available".to_string())),
    };
    
    // Subscribe via command channel
    match audio_cmd.subscribe_stream(stream_id.clone(), req.start_channel).await {
        Ok(result) => {
            let msg = format!(
                "✅ Subscribed to '{}' ({} channels) -> Kanal {}",
                result.stream_name, result.channels, result.start_channel
            );
            tracing::info!("🔊 {}", msg);
            Json(ApiResponse::ok(msg))
        }
        Err(e) => Json(ApiResponse::err(e)),
    }
}

/// Unsubscribe from an AES67 stream
async fn unsubscribe_aes67_stream(
    State(state): State<AppState>,
    Path(stream_id): Path<String>,
) -> Json<ApiResponse<String>> {
    // Check if we have a command sender
    let audio_cmd = match &state.audio_cmd {
        Some(cmd) => cmd.clone(),
        None => return Json(ApiResponse::err("AudioEngine not available".to_string())),
    };
    
    // Unsubscribe via command channel
    match audio_cmd.unsubscribe_stream(stream_id.clone()).await {
        Ok(_) => {
            let msg = format!("✅ Unsubscribed from stream '{}'", stream_id);
            tracing::info!("🔇 {}", msg);
            Json(ApiResponse::ok(msg))
        }
        Err(e) => Json(ApiResponse::err(e)),
    }
}

/// Refresh AES67 discovery (re-scan network)
async fn refresh_aes67_discovery(
    State(state): State<AppState>,
) -> Json<ApiResponse<Vec<Aes67StreamInfo>>> {
    // SAP discovery runs continuously, return current streams
    let streams = if let Some(ref sap) = state.sap_discovery {
        sap.streams()
            .into_iter()
            .map(|s| Aes67StreamInfo {
                id: s.session_id.clone(),
                name: s.name.clone(),
                channels: s.channels,
                sample_rate: s.sample_rate,
                multicast_addr: s.multicast_addr.to_string(),
                port: s.port,
                direction: format!("{:?}", s.direction),
                origin: s.origin.clone(),
            })
            .collect()
    } else {
        vec![]
    };
    
    Json(ApiResponse::ok(streams))
}
