//! WebSocket Handler
//! 
//! Echtzeit-Kommunikation mit Clients
//! Unterstützt Multi-Client-Synchronisation - Änderungen werden an alle verbundenen Clients gebroadcastet

use axum::extract::ws::{Message, WebSocket};
use futures::{StreamExt, SinkExt};
use std::sync::Arc;
use std::sync::atomic::Ordering;
use tokio::sync::broadcast;
use tracing::{info, warn, error, debug};

use audiomultiverse_protocol::{
    ClientMessage, ServerMessage, ClientInfo, ServerInfo, MeterData,
    Aes67Status, Aes67StreamInfo,
};
use super::routes::AppState;

/// WebSocket Verbindung handhaben mit Multi-Client-Support
pub async fn handle_websocket(socket: WebSocket, state: AppState) {
    let (mut sender, mut receiver) = socket.split();
    
    let client_id = uuid::Uuid::new_v4().to_string();
    
    // Client-Zähler erhöhen
    let client_num = state.client_count.fetch_add(1, Ordering::SeqCst) + 1;
    info!("🔌 Neuer WebSocket Client #{}: {} ({} verbunden)", client_num, &client_id[..8], client_num);
    
    // Broadcast-Receiver für diesen Client erstellen
    let mut broadcast_rx = state.broadcast_tx.subscribe();
    
    // Willkommensnachricht senden
    let welcome = ServerMessage::Welcome {
        server_info: ServerInfo {
            name: "AudioMultiverse".to_string(),
            version: env!("CARGO_PKG_VERSION").to_string(),
            input_count: state.mixer.input_count as u32,
            output_count: state.mixer.output_count as u32,
            sample_rate: 48000,
            client_count: client_num as u32,
            audio_backend: "aes67".to_string(),
        },
        state: state.mixer.get_state(),
    };
    
    if let Ok(json) = serde_json::to_string(&welcome) {
        if sender.send(Message::Text(json.into())).await.is_err() {
            state.client_count.fetch_sub(1, Ordering::SeqCst);
            return;
        }
    }
    
    // Sender in Arc<Mutex> für shared access
    let sender = Arc::new(tokio::sync::Mutex::new(sender));
    
    // Task 1: Meter-Updates (50ms Intervall)
    let meter_sender = sender.clone();
    let meter_mixer = state.mixer.clone();
    let meter_task = tokio::spawn(async move {
        let mut interval = tokio::time::interval(tokio::time::Duration::from_millis(50));
        
        loop {
            interval.tick().await;
            
            let meters = meter_mixer.get_meters();
            let msg = ServerMessage::Meters(MeterData {
                peaks: meters,
                timestamp: std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap()
                    .as_millis() as u64,
            });
            
            if let Ok(json) = serde_json::to_string(&msg) {
                let mut sender = meter_sender.lock().await;
                if sender.send(Message::Text(json.into())).await.is_err() {
                    break;
                }
            }
        }
    });
    
    // Task 2: Broadcast-Empfänger (Änderungen von anderen Clients)
    let broadcast_sender = sender.clone();
    let broadcast_client_id = client_id.clone();
    let broadcast_task = tokio::spawn(async move {
        loop {
            match broadcast_rx.recv().await {
                Ok(msg) => {
                    // Broadcast an diesen Client weiterleiten
                    if let Ok(json) = serde_json::to_string(&msg) {
                        let mut sender = broadcast_sender.lock().await;
                        if sender.send(Message::Text(json.into())).await.is_err() {
                            break;
                        }
                    }
                }
                Err(broadcast::error::RecvError::Lagged(n)) => {
                    warn!("Client {} hat {} Broadcast-Nachrichten verpasst", &broadcast_client_id[..8], n);
                }
                Err(broadcast::error::RecvError::Closed) => {
                    break;
                }
            }
        }
    });
    
    // Task 3: Client-Nachrichten empfangen und verarbeiten
    let msg_sender = sender.clone();
    let msg_state = state.clone();
    let msg_client_id = client_id.clone();
    
    while let Some(result) = receiver.next().await {
        match result {
            Ok(Message::Text(text)) => {
                match serde_json::from_str::<ClientMessage>(&text) {
                    Ok(msg) => {
                        // Nachricht verarbeiten
                        let (response, should_broadcast) = handle_client_message(msg, &msg_state, &msg_client_id).await;
                        
                        // Antwort an diesen Client
                        if let Some(resp) = &response {
                            if let Ok(json) = serde_json::to_string(resp) {
                                let mut sender = msg_sender.lock().await;
                                if sender.send(Message::Text(json.into())).await.is_err() {
                                    break;
                                }
                            }
                        }
                        
                        // Broadcast an alle anderen Clients
                        if should_broadcast {
                            if let Some(broadcast_msg) = response {
                                // Sende an alle Clients (inklusive uns selbst, da wir schon geantwortet haben ignorieren wir Fehler)
                                let _ = msg_state.broadcast_tx.send(broadcast_msg);
                            }
                        }
                    }
                    Err(e) => {
                        warn!("Ungültige Nachricht von {}: {}", &msg_client_id[..8], e);
                        let error = ServerMessage::Error {
                            code: "PARSE_ERROR".to_string(),
                            message: e.to_string(),
                        };
                        if let Ok(json) = serde_json::to_string(&error) {
                            let mut sender = msg_sender.lock().await;
                            let _ = sender.send(Message::Text(json.into())).await;
                        }
                    }
                }
            }
            Ok(Message::Ping(data)) => {
                let mut sender = msg_sender.lock().await;
                let _ = sender.send(Message::Pong(data)).await;
            }
            Ok(Message::Close(_)) => {
                debug!("🔌 Client {} hat Verbindung geschlossen", &msg_client_id[..8]);
                break;
            }
            Err(e) => {
                error!("WebSocket Fehler für {}: {}", &msg_client_id[..8], e);
                break;
            }
            _ => {}
        }
    }
    
    // Cleanup: Tasks beenden und Client-Zähler verringern
    meter_task.abort();
    broadcast_task.abort();
    
    let remaining = state.client_count.fetch_sub(1, Ordering::SeqCst) - 1;
    info!("🔌 Client {} getrennt ({} verbleibend)", &client_id[..8], remaining);
    
    // Broadcast: Client-Count-Update an alle anderen
    let _ = state.broadcast_tx.send(ServerMessage::ClientCountChanged { count: remaining as u32 });
}

/// Client-Nachricht verarbeiten
/// Gibt (Option<Antwort>, soll_broadcasten) zurück
/// soll_broadcasten = true für Änderungen die alle Clients sehen müssen (Fader, Mute, etc.)
async fn handle_client_message(
    msg: ClientMessage,
    state: &AppState,
    client_id: &str,
) -> (Option<ServerMessage>, bool) {
    match msg {
        ClientMessage::Hello(info) => {
            info!("👋 Client {} Hello: {} ({})", &client_id[..8], info.name, info.client_type);
            (None, false) // Bereits mit Welcome beantwortet
        }
        
        ClientMessage::Ping { timestamp } => {
            (Some(ServerMessage::Pong {
                timestamp,
                server_time: std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap()
                    .as_millis() as u64,
            }), false)
        }
        
        ClientMessage::GetState => {
            (Some(ServerMessage::State(state.mixer.get_state())), false)
        }
        
        ClientMessage::GetServerInfo => {
            (Some(ServerMessage::ServerInfo(ServerInfo {
                name: "AudioMultiverse".to_string(),
                version: env!("CARGO_PKG_VERSION").to_string(),
                input_count: state.mixer.input_count as u32,
                output_count: state.mixer.output_count as u32,
                sample_rate: 48000,
                client_count: state.client_count.load(Ordering::Relaxed) as u32,
                audio_backend: "aes67".to_string(),
            })), false)
        }
        
        // === Fader/Mixer-Änderungen - MÜSSEN gebroadcastet werden ===
        
        ClientMessage::SetFader { channel, value } => {
            debug!("Client {} setzt Fader {} auf {:.2}", &client_id[..8], channel, value);
            (state.mixer.set_fader(channel, value)
                .map(ServerMessage::ChannelUpdated), true) // BROADCAST!
        }
        
        ClientMessage::SetMute { channel, muted } => {
            debug!("Client {} setzt Mute {} auf {}", &client_id[..8], channel, muted);
            (state.mixer.set_mute(channel, muted)
                .map(ServerMessage::ChannelUpdated), true) // BROADCAST!
        }
        
        ClientMessage::SetSolo { channel, solo } => {
            debug!("Client {} setzt Solo {} auf {}", &client_id[..8], channel, solo);
            (state.mixer.set_solo(channel, solo)
                .map(ServerMessage::ChannelUpdated), true) // BROADCAST!
        }
        
        ClientMessage::SetPan { channel, value } => {
            debug!("Client {} setzt Pan {} auf {:.2}", &client_id[..8], channel, value);
            (state.mixer.set_pan(channel, value)
                .map(ServerMessage::ChannelUpdated), true) // BROADCAST!
        }
        
        ClientMessage::SetGain { channel, value: _ } => {
            // TODO: set_gain implementieren
            (state.mixer.get_channel(channel)
                .map(ServerMessage::ChannelUpdated), true) // BROADCAST!
        }
        
        ClientMessage::SetChannelName { channel, name } => {
            debug!("Client {} benennt Kanal {} um zu '{}'", &client_id[..8], channel, name);
            (state.mixer.set_channel_name(channel, name)
                .map(ServerMessage::ChannelUpdated), true) // BROADCAST!
        }
        
        ClientMessage::SetRouting { input, output, gain } => {
            let success = state.mixer.set_routing(input as usize, output as usize, gain);
            if success {
                debug!("Client {} setzt Routing {}x{} auf {:.2}", &client_id[..8], input, output, gain);
                (Some(ServerMessage::RoutingUpdated { input, output, gain }), true) // BROADCAST!
            } else {
                (Some(ServerMessage::Error {
                    code: "INVALID_ROUTING".to_string(),
                    message: format!("Ungültiges Routing: {}x{}", input, output),
                }), false)
            }
        }
        
        ClientMessage::SubscribeMeters { enabled: _, interval_ms: _ } => {
            // TODO: Meter-Subscription pro Client verwalten
            (None, false)
        }
        
        // === AES67 Network Audio ===
        
        ClientMessage::GetAes67Status => {
            let (ptp_sync, offset) = if let Some(ref ptp) = state.ptp_clock {
                (ptp.is_synchronized(), ptp.offset_ns())
            } else {
                (false, 0)
            };
            
            (Some(ServerMessage::Aes67Status(Aes67Status {
                enabled: state.sap_discovery.is_some(),
                ptp_synchronized: ptp_sync,
                ptp_offset_ns: offset,
                our_stream: None,
                subscribed_streams: vec![],
            })), false)
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
            
            (Some(ServerMessage::Aes67Streams(streams)), false)
        }
        
        ClientMessage::SubscribeAes67Stream { stream_id, start_channel } => {
            let audio_cmd = match &state.audio_cmd {
                Some(cmd) => cmd.clone(),
                None => return (Some(ServerMessage::Error {
                    code: "NO_ENGINE".to_string(),
                    message: "AudioEngine not available".to_string(),
                }), false),
            };
            
            // Subscribe async - AES67 Subscriptions MÜSSEN gebroadcastet werden
            match audio_cmd.subscribe_stream(stream_id.clone(), start_channel).await {
                Ok(result) => {
                    info!("🔊 Client {} subscribed to '{}' ({} ch) -> Kanal {}", 
                          &client_id[..8], result.stream_name, result.channels, result.start_channel);
                    (Some(ServerMessage::Aes67Subscribed {
                        stream_id: result.stream_id,
                        stream_name: result.stream_name,
                        channels: result.channels,
                        start_channel: result.start_channel,
                    }), true) // BROADCAST!
                }
                Err(e) => (Some(ServerMessage::Error {
                    code: "SUBSCRIBE_FAILED".to_string(),
                    message: e,
                }), false),
            }
        }
        
        ClientMessage::UnsubscribeAes67Stream { stream_id } => {
            let audio_cmd = match &state.audio_cmd {
                Some(cmd) => cmd.clone(),
                None => return (Some(ServerMessage::Error {
                    code: "NO_ENGINE".to_string(),
                    message: "AudioEngine not available".to_string(),
                }), false),
            };
            
            match audio_cmd.unsubscribe_stream(stream_id.clone()).await {
                Ok(_) => {
                    info!("🔇 Client {} unsubscribed from '{}'", &client_id[..8], stream_id);
                    (Some(ServerMessage::Aes67Unsubscribed { stream_id }), true) // BROADCAST!
                }
                Err(e) => (Some(ServerMessage::Error {
                    code: "UNSUBSCRIBE_FAILED".to_string(),
                    message: e,
                }), false),
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
            
            // Broadcast damit alle Clients die neue Liste sehen
            (Some(ServerMessage::Aes67Streams(streams)), true) // BROADCAST!
        }
        
        _ => {
            warn!("Unbehandelte Nachricht von {}: {:?}", &client_id[..8], msg);
            (Some(ServerMessage::Error {
                code: "NOT_IMPLEMENTED".to_string(),
                message: "Diese Funktion ist noch nicht implementiert".to_string(),
            }), false)
        }
    }
}
