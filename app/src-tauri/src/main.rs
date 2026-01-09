#![cfg_attr(
    all(not(debug_assertions), target_os = "windows"),
    windows_subsystem = "windows"
)]

mod midi;

use std::sync::{Arc, Mutex};
use tauri::State;
use midi::{ClientMidiController, ClientMidiFeedback, MixerCommand};

/// Globaler App-State
pub struct AppState {
    midi_controller: Arc<Mutex<ClientMidiController>>,
    midi_feedback: Arc<Mutex<ClientMidiFeedback>>,
    /// Server-Verbindung URL
    server_url: Arc<Mutex<Option<String>>>,
}

impl AppState {
    fn new() -> Self {
        Self {
            midi_controller: Arc::new(Mutex::new(ClientMidiController::new())),
            midi_feedback: Arc::new(Mutex::new(ClientMidiFeedback::new())),
            server_url: Arc::new(Mutex::new(None)),
        }
    }
}

// ============ MIDI Commands ============

/// Liste verfügbare MIDI-Eingabegeräte
#[tauri::command]
fn list_midi_inputs(state: State<AppState>) -> Vec<String> {
    if let Ok(controller) = state.midi_controller.lock() {
        return controller.list_devices();
    }
    Vec::new()
}

/// Liste verfügbare MIDI-Ausgabegeräte
#[tauri::command]
fn list_midi_outputs(state: State<AppState>) -> Vec<String> {
    if let Ok(feedback) = state.midi_feedback.lock() {
        return feedback.list_devices();
    }
    Vec::new()
}

/// MIDI-Eingabegerät verbinden
#[tauri::command]
fn connect_midi_input(state: State<AppState>, device_name: String) -> Result<(), String> {
    let mut controller_guard = state.midi_controller.lock()
        .map_err(|e| format!("Lock error: {}", e))?;
    
    controller_guard.connect(&device_name).map_err(|e| e.to_string())
}

/// MIDI-Ausgabegerät verbinden
#[tauri::command]
fn connect_midi_output(state: State<AppState>, device_name: String) -> Result<(), String> {
    let mut feedback = state.midi_feedback.lock()
        .map_err(|e| format!("Lock error: {}", e))?;
    
    feedback.connect(&device_name).map_err(|e| e.to_string())
}

/// MCU-Modus aktivieren
#[tauri::command]
fn enable_mcu_mode(state: State<AppState>) -> Result<(), String> {
    let controller_guard = state.midi_controller.lock()
        .map_err(|e| format!("Lock error: {}", e))?;
    controller_guard.enable_mcu_mode();
    
    let mut feedback = state.midi_feedback.lock()
        .map_err(|e| format!("Lock error: {}", e))?;
    feedback.enable_mcu_mode();
    
    Ok(())
}

/// MIDI Learn starten
#[tauri::command]
fn start_midi_learn(state: State<AppState>, target_type: String, channel: Option<u32>) -> Result<(), String> {
    use audiomultiverse_protocol::MidiTarget;
    
    let target = match target_type.as_str() {
        "fader" => MidiTarget::Fader { channel: channel.unwrap_or(0) },
        "mute" => MidiTarget::Mute { channel: channel.unwrap_or(0) },
        "solo" => MidiTarget::Solo { channel: channel.unwrap_or(0) },
        "pan" => MidiTarget::Pan { channel: channel.unwrap_or(0) },
        "master" => MidiTarget::Master,
        _ => return Err(format!("Unknown target type: {}", target_type)),
    };
    
    let controller_guard = state.midi_controller.lock()
        .map_err(|e| format!("Lock error: {}", e))?;
    
    controller_guard.start_learn(target);
    
    Ok(())
}

/// MIDI Learn abbrechen
#[tauri::command]
fn cancel_midi_learn(state: State<AppState>) -> Result<(), String> {
    let controller_guard = state.midi_controller.lock()
        .map_err(|e| format!("Lock error: {}", e))?;
    
    controller_guard.cancel_learn();
    
    Ok(())
}

/// Nächsten MIDI-Befehl aus der Queue holen (Polling)
#[tauri::command]
fn poll_midi_command(state: State<AppState>) -> Option<MixerCommand> {
    if let Ok(controller) = state.midi_controller.lock() {
        return controller.poll_command();
    }
    None
}

// ============ Feedback Commands (für Updates vom Server) ============

/// Fader-Feedback senden
#[tauri::command]
fn send_fader_feedback(state: State<AppState>, channel: u32, value: f32) {
    if let Ok(feedback) = state.midi_feedback.lock() {
        feedback.send_fader(channel, value);
    }
}

/// Mute-Feedback senden
#[tauri::command]
fn send_mute_feedback(state: State<AppState>, channel: u32, muted: bool) {
    if let Ok(feedback) = state.midi_feedback.lock() {
        feedback.send_mute(channel, muted);
    }
}

/// Solo-Feedback senden
#[tauri::command]
fn send_solo_feedback(state: State<AppState>, channel: u32, solo: bool) {
    if let Ok(feedback) = state.midi_feedback.lock() {
        feedback.send_solo(channel, solo);
    }
}

/// Channel komplett updaten
#[tauri::command]
fn update_channel_feedback(
    state: State<AppState>,
    channel: u32,
    fader: f32,
    muted: bool,
    solo: bool,
    pan: f32,
    name: String,
) {
    if let Ok(feedback) = state.midi_feedback.lock() {
        feedback.update_channel(channel, fader, muted, solo, pan, &name);
    }
}

// ============ Server Connection ============

/// Server-URL setzen
#[tauri::command]
fn set_server_url(state: State<AppState>, url: String) {
    if let Ok(mut server_url) = state.server_url.lock() {
        *server_url = Some(url);
    }
}

/// Server-URL abrufen
#[tauri::command]
fn get_server_url(state: State<AppState>) -> Option<String> {
    if let Ok(server_url) = state.server_url.lock() {
        return server_url.clone();
    }
    None
}

// ============ Main ============

fn main() {
    // Logging initialisieren
    tracing_subscriber::fmt::init();
    
    let state = AppState::new();
    
    tauri::Builder::default()
        .manage(state)
        .invoke_handler(tauri::generate_handler![
            // MIDI
            list_midi_inputs,
            list_midi_outputs,
            connect_midi_input,
            connect_midi_output,
            enable_mcu_mode,
            start_midi_learn,
            cancel_midi_learn,
            poll_midi_command,
            // Feedback
            send_fader_feedback,
            send_mute_feedback,
            send_solo_feedback,
            update_channel_feedback,
            // Server
            set_server_url,
            get_server_url,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
