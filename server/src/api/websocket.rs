//! WebSocket Handler
//! 
//! Echtzeit-Kommunikation mit Clients

use axum::extract::ws::{Message, WebSocket};
use futures::{StreamExt, SinkExt};
use std::sync::Arc;
use tokio::sync::broadcast;
use tracing::{info, warn, error};

use audiomultiverse_protocol::{
    ClientMessage, ServerMessage, ClientInfo, ServerInfo, MeterData,
    Aes67Status, Aes67StreamInfo,
};
use super::routes::AppState;

/// WebSocket Handler
pub struct WebSocketHandler {
    // TODO: Client-Management
}

/// WebSocket Verbindung handhaben
pub async fn handle_websocket(socket: WebSocket, state: AppState) {
    let (mut sender, mut receiver) = socket.split();
    
    let client_id = uuid::Uuid::new_v4().to_string();
    info!("🔌 Neuer WebSocket Client: {}", client_id);
    
    // Willkommensnachricht senden
    let welcome = ServerMessage::Welcome {
        server_info: ServerInfo {
            name: "AudioMultiverse".to_string(),
            version: env!("CARGO_PKG_VERSION").to_string(),
            input_count: state.mixer.input_count as u32,
            output_count: state.mixer.output_count as u32,
            sample_rate: 48000,
            client_count: 1,
            audio_backend: "aes67".to_string(),
        },
        state: state.mixer.get_state(),
    };
    
    if let Ok(json) = serde_json::to_string(&welcome) {
        if sender.send(Message::Text(json.into())).await.is_err() {
            return;
        }
    }
    
    // Meter-Task (sendet regelmäßig Meter-Updates)
    let mixer = state.mixer.clone();
    let meter_sender = Arc::new(tokio::sync::Mutex::new(sender));
    let meter_sender_clone = meter_sender.clone();
    
    let meter_task = tokio::spawn(async move {
        let mut interval = tokio::time::interval(tokio::time::Duration::from_millis(50));
        
        loop {
            interval.tick().await;
            
            let meters = mixer.get_meters();
            let msg = ServerMessage::Meters(MeterData {
                peaks: meters,
                timestamp: std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap()
                    .as_millis() as u64,
            });
            
            if let Ok(json) = serde_json::to_string(&msg) {
                let mut sender = meter_sender_clone.lock().await;
                if sender.send(Message::Text(json.into())).await.is_err() {
                    break;
                }
            }
        }
    });
    
    // Nachrichten empfangen
    while let Some(result) = receiver.next().await {
        match result {
            Ok(Message::Text(text)) => {
                match serde_json::from_str::<ClientMessage>(&text) {
                    Ok(msg) => {
                        if let Some(response) = handle_client_message(msg, &state).await {
                            if let Ok(json) = serde_json::to_string(&response) {
                                let mut sender = meter_sender.lock().await;
                                if sender.send(Message::Text(json.into())).await.is_err() {
                                    break;
                                }
                            }
                        }
                    }
                    Err(e) => {
                        warn!("Ungültige Nachricht: {}", e);
                        let error = ServerMessage::Error {
                            code: "PARSE_ERROR".to_string(),
                            message: e.to_string(),
                        };
                        if let Ok(json) = serde_json::to_string(&error) {
                            let mut sender = meter_sender.lock().await;
                            let _ = sender.send(Message::Text(json.into())).await;
                        }
                    }
                }
            }
            Ok(Message::Close(_)) => {
                info!("🔌 WebSocket Client getrennt: {}", client_id);
                break;
            }
            Err(e) => {
                error!("WebSocket Fehler: {}", e);
                break;
            }
            _ => {}
        }
    }
    
    // Meter-Task beenden
    meter_task.abort();
    info!("🔌 Client {} Verbindung beendet", client_id);
}

/// Client-Nachricht verarbeiten
async fn handle_client_message(
    msg: ClientMessage,
    state: &AppState,
) -> Option<ServerMessage> {
    match msg {
        ClientMessage::Hello(info) => {
            info!("👋 Client Hello: {} ({})", info.name, info.client_type);
            None // Bereits mit Welcome beantwortet
        }
        
        ClientMessage::Ping { timestamp } => {
            Some(ServerMessage::Pong {
                timestamp,
                server_time: std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap()
                    .as_millis() as u64,
            })
        }
        
        ClientMessage::GetState => {
            Some(ServerMessage::State(state.mixer.get_state()))
        }
        
        ClientMessage::GetServerInfo => {
            Some(ServerMessage::ServerInfo(ServerInfo {
                name: "AudioMultiverse".to_string(),
                version: env!("CARGO_PKG_VERSION").to_string(),
                input_count: state.mixer.input_count as u32,
                output_count: state.mixer.output_count as u32,
                sample_rate: 48000,
                client_count: 1,
                audio_backend: "aes67".to_string(),
            }))
        }
        
        ClientMessage::SetFader { channel, value } => {
            state.mixer.set_fader(channel, value)
                .map(ServerMessage::ChannelUpdated)
        }
        
        ClientMessage::SetMute { channel, muted } => {
            state.mixer.set_mute(channel, muted)
                .map(ServerMessage::ChannelUpdated)
        }
        
        ClientMessage::SetSolo { channel, solo } => {
            state.mixer.set_solo(channel, solo)
                .map(ServerMessage::ChannelUpdated)
        }
        
        ClientMessage::SetPan { channel, value } => {
            state.mixer.set_pan(channel, value)
                .map(ServerMessage::ChannelUpdated)
        }
        
        ClientMessage::SetGain { channel, value: _ } => {
            // TODO: set_gain implementieren
            state.mixer.get_channel(channel)
                .map(ServerMessage::ChannelUpdated)
        }
        
        ClientMessage::SetChannelName { channel, name } => {
            state.mixer.set_channel_name(channel, name)
                .map(ServerMessage::ChannelUpdated)
        }
        
        ClientMessage::SetRouting { input, output, gain } => {
            let success = state.mixer.set_routing(input as usize, output as usize, gain);
            if success {
                Some(ServerMessage::RoutingUpdated { input, output, gain })
            } else {
                Some(ServerMessage::Error {
                    code: "INVALID_ROUTING".to_string(),
                    message: format!("Ungültiges Routing: {}x{}", input, output),
                })
            }
        }
        
        ClientMessage::SubscribeMeters { enabled: _, interval_ms: _ } => {
            // TODO: Meter-Subscription pro Client verwalten
            None
        }
        
        // === AES67 Network Audio ===
        
        ClientMessage::GetAes67Status => {
            let (ptp_sync, offset) = if let Some(ref ptp) = state.ptp_clock {
                (ptp.is_synchronized(), ptp.offset_ns())
            } else {
                (false, 0)
            };
            
            Some(ServerMessage::Aes67Status(Aes67Status {
                enabled: state.sap_discovery.is_some(),
                ptp_synchronized: ptp_sync,
                ptp_offset_ns: offset,
                our_stream: None,
                subscribed_streams: vec![],
            }))
        }
        
        ClientMessage::GetAes67Streams => {
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
            
            Some(ServerMessage::Aes67Streams(streams))
        }
        
        ClientMessage::SubscribeAes67Stream { stream_id, start_channel } => {
            let audio_cmd = match &state.audio_cmd {
                Some(cmd) => cmd.clone(),
                None => return Some(ServerMessage::Error {
                    code: "NO_ENGINE".to_string(),
                    message: "AudioEngine not available".to_string(),
                }),
            };
            
            // Subscribe async
            match audio_cmd.subscribe_stream(stream_id.clone(), start_channel).await {
                Ok(result) => {
                    info!("🔊 WS: Subscribed to '{}' ({} ch) -> Kanal {}", 
                          result.stream_name, result.channels, result.start_channel);
                    Some(ServerMessage::Aes67Subscribed {
                        stream_id: result.stream_id,
                        stream_name: result.stream_name,
                        channels: result.channels,
                        start_channel: result.start_channel,
                    })
                }
                Err(e) => Some(ServerMessage::Error {
                    code: "SUBSCRIBE_FAILED".to_string(),
                    message: e,
                }),
            }
        }
        
        ClientMessage::UnsubscribeAes67Stream { stream_id } => {
            let audio_cmd = match &state.audio_cmd {
                Some(cmd) => cmd.clone(),
                None => return Some(ServerMessage::Error {
                    code: "NO_ENGINE".to_string(),
                    message: "AudioEngine not available".to_string(),
                }),
            };
            
            match audio_cmd.unsubscribe_stream(stream_id.clone()).await {
                Ok(_) => {
                    info!("🔇 WS: Unsubscribed from '{}'", stream_id);
                    Some(ServerMessage::Aes67Unsubscribed { stream_id })
                }
                Err(e) => Some(ServerMessage::Error {
                    code: "UNSUBSCRIBE_FAILED".to_string(),
                    message: e,
                }),
            }
        }
        
        ClientMessage::RefreshAes67 => {
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
            
            Some(ServerMessage::Aes67Streams(streams))
        }
        
        _ => {
            warn!("Unbehandelte Nachricht: {:?}", msg);
            Some(ServerMessage::Error {
                code: "NOT_IMPLEMENTED".to_string(),
                message: "Diese Funktion ist noch nicht implementiert".to_string(),
            })
        }
    }
}
