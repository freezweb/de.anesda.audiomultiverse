//! HTTP/REST Routes
//! 
//! REST API Endpoints für Hausautomatisierung und einfache Abfragen

use std::sync::Arc;
use axum::{
    routing::{get, patch, post, delete},
    Router,
    Json,
    extract::{Path, State, WebSocketUpgrade},
    response::IntoResponse,
};
use tower_http::cors::{CorsLayer, Any};
use tracing::info;
use tokio::sync::RwLock;

use crate::config::ApiConfig;
use crate::mixer::{Mixer, SceneManager, SceneMetadata, MasterSection, MasterState};
use audiomultiverse_protocol::{ApiResponse, ChannelState, MixerState, ServerInfo};

use super::websocket::handle_websocket;

/// App State für alle Handlers
#[derive(Clone)]
pub struct AppState {
    pub mixer: Arc<Mixer>,
    pub config: ApiConfig,
    pub scene_manager: Arc<RwLock<SceneManager>>,
    pub master: Arc<MasterSection>,
}

/// API Server starten
pub async fn start_api_server(
    config: ApiConfig,
    mixer: Arc<Mixer>,
    scene_manager: Arc<RwLock<SceneManager>>,
    master: Arc<MasterSection>,
) -> anyhow::Result<()> {
    let state = AppState {
        mixer,
        config: config.clone(),
        scene_manager,
        master,
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
        client_count: 0, // TODO: Clients zählen
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
