//! Client-seitige MIDI-Eingabeverarbeitung
//! 
//! Empfängt MIDI-Daten vom Controller und sendet sie an die UI/Server

use std::collections::HashMap;
use std::collections::VecDeque;
use std::sync::{Arc, Mutex};
use midir::{MidiInput, MidiInputConnection};
use tracing::{info, debug};
use serde::{Serialize, Deserialize};
use audiomultiverse_protocol::MidiTarget;

/// MIDI-Event vom Controller
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum MidiEvent {
    /// Control Change
    ControlChange {
        device: String,
        channel: u8,
        cc: u8,
        value: u8,
    },
    /// Note On/Off
    Note {
        device: String,
        channel: u8,
        note: u8,
        velocity: u8,
        on: bool,
    },
    /// Pitch Bend (für Fader bei MCU)
    PitchBend {
        device: String,
        channel: u8,
        value: u16,
    },
}

/// Konvertiertes Mixer-Kommando - serialisierbar für Tauri
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum MixerCommand {
    SetFader { channel: u32, value: f32 },
    SetMute { channel: u32, muted: bool },
    SetSolo { channel: u32, solo: bool },
    SetPan { channel: u32, value: f32 },
    SetMasterFader { value: f32 },
}

/// Shared Queue für MIDI-Kommandos
pub type CommandQueue = Arc<Mutex<VecDeque<MixerCommand>>>;

/// Client MIDI Controller
pub struct ClientMidiController {
    /// Aktive MIDI-Eingabeverbindungen
    connections: Vec<MidiInputConnection<()>>,
    
    /// Mapping von MIDI-Events zu Mixer-Aktionen
    mappings: Arc<Mutex<HashMap<(String, u8, u8), MidiTarget>>>, // (device, channel, cc) -> target
    
    /// MCU-Modus aktiv
    mcu_mode: Arc<Mutex<bool>>,
    
    /// Queue für ausgehende Mixer-Kommandos (Thread-safe)
    command_queue: CommandQueue,
    
    /// Learn Mode aktiv
    learn_mode: Arc<Mutex<bool>>,
    learn_target: Arc<Mutex<Option<MidiTarget>>>,
}

impl ClientMidiController {
    /// Neuen Controller erstellen
    pub fn new() -> Self {
        Self {
            connections: Vec::new(),
            mappings: Arc::new(Mutex::new(HashMap::new())),
            mcu_mode: Arc::new(Mutex::new(false)),
            command_queue: Arc::new(Mutex::new(VecDeque::new())),
            learn_mode: Arc::new(Mutex::new(false)),
            learn_target: Arc::new(Mutex::new(None)),
        }
    }
    
    /// Command Queue abrufen (für Polling)
    pub fn get_command_queue(&self) -> CommandQueue {
        self.command_queue.clone()
    }
    
    /// Nächsten Befehl aus der Queue holen
    pub fn poll_command(&self) -> Option<MixerCommand> {
        self.command_queue.lock().ok()?.pop_front()
    }
    
    /// Verfügbare MIDI-Geräte auflisten
    pub fn list_devices(&self) -> Vec<String> {
        let mut devices = Vec::new();
        
        if let Ok(midi_in) = MidiInput::new("AudioMultiverse Client Scanner") {
            for port in midi_in.ports() {
                if let Ok(name) = midi_in.port_name(&port) {
                    devices.push(name);
                }
            }
        }
        
        devices
    }
    
    /// Mit einem MIDI-Gerät verbinden
    pub fn connect(&mut self, device_name: &str) -> anyhow::Result<()> {
        let midi_in = MidiInput::new("AudioMultiverse Client")?;
        
        let port = midi_in.ports()
            .into_iter()
            .find(|p| midi_in.port_name(p).map(|n| n == device_name).unwrap_or(false))
            .ok_or_else(|| anyhow::anyhow!("MIDI device not found: {}", device_name))?;
        
        let device = device_name.to_string();
        let command_queue = self.command_queue.clone();
        let mappings = self.mappings.clone();
        let mcu_mode = self.mcu_mode.clone();
        let learn_mode = self.learn_mode.clone();
        let learn_target = self.learn_target.clone();
        
        let conn = midi_in.connect(
            &port,
            "audiomultiverse-input",
            move |_timestamp, data, _| {
                if data.len() >= 2 {
                    let status = data[0] & 0xF0;
                    let channel = data[0] & 0x0F;
                    let is_mcu = *mcu_mode.lock().unwrap();
                    
                    match status {
                        0xB0 => {
                            // Control Change
                            let cc = data[1];
                            let value = data[2];
                            
                            // Check Learn Mode
                            if *learn_mode.lock().unwrap() {
                                if let Some(target) = learn_target.lock().unwrap().take() {
                                    debug!("MIDI Learn: CC {} auf Kanal {} -> {:?}", cc, channel, target);
                                    // Mapping speichern
                                    mappings.lock().unwrap().insert(
                                        (device.clone(), channel, cc), 
                                        target
                                    );
                                    *learn_mode.lock().unwrap() = false;
                                }
                                return;
                            }
                            
                            // Reguläre Verarbeitung
                            if is_mcu {
                                Self::process_mcu_cc(&command_queue, channel, cc, value);
                            } else if let Some(target) = mappings.lock().unwrap().get(&(device.clone(), channel, cc)).cloned() {
                                Self::process_mapped(&command_queue, &target, value);
                            }
                        }
                        0x90 => {
                            // Note On
                            let note = data[1];
                            let velocity = data[2];
                            
                            if is_mcu && velocity > 0 {
                                Self::process_mcu_button(&command_queue, note);
                            }
                        }
                        0xE0 => {
                            // Pitch Bend (MCU Fader)
                            if data.len() >= 3 && is_mcu {
                                let lsb = data[1] as u16;
                                let msb = data[2] as u16;
                                let value = (msb << 7) | lsb;
                                
                                // MCU: Channel 0-7 = Fader 1-8, Channel 8 = Master
                                if channel < 8 {
                                    let fader_value = value as f32 / 16383.0;
                                    if let Ok(mut queue) = command_queue.lock() {
                                        queue.push_back(MixerCommand::SetFader {
                                            channel: channel as u32,
                                            value: fader_value,
                                        });
                                    }
                                } else if channel == 8 {
                                    let fader_value = value as f32 / 16383.0;
                                    if let Ok(mut queue) = command_queue.lock() {
                                        queue.push_back(MixerCommand::SetMasterFader {
                                            value: fader_value,
                                        });
                                    }
                                }
                            }
                        }
                        _ => {}
                    }
                }
            },
            (),
        ).map_err(|e| anyhow::anyhow!("MIDI connect failed: {:?}", e))?;
        
        self.connections.push(conn);
        info!("MIDI Device verbunden: {}", device_name);
        
        Ok(())
    }
    
    /// MCU-Modus aktivieren
    pub fn enable_mcu_mode(&self) {
        *self.mcu_mode.lock().unwrap() = true;
        info!("MCU Mode aktiviert");
    }
    
    /// MIDI Learn starten
    pub fn start_learn(&self, target: MidiTarget) {
        *self.learn_mode.lock().unwrap() = true;
        *self.learn_target.lock().unwrap() = Some(target);
        info!("MIDI Learn aktiviert");
    }
    
    /// MIDI Learn abbrechen
    pub fn cancel_learn(&self) {
        *self.learn_mode.lock().unwrap() = false;
        *self.learn_target.lock().unwrap() = None;
    }
    
    /// Mapping hinzufügen
    pub fn add_mapping(&self, device: &str, channel: u8, cc: u8, target: MidiTarget) {
        self.mappings.lock().unwrap().insert((device.to_string(), channel, cc), target);
    }
    
    /// MCU Control Change verarbeiten
    fn process_mcu_cc(queue: &CommandQueue, channel: u8, cc: u8, value: u8) {
        // MCU V-Pot (Pan) - CC 16-23
        if cc >= 16 && cc <= 23 {
            let mixer_channel = (cc - 16) as u32;
            // MCU V-Pot: 1-63 = links, 65-127 = rechts
            let pan = if value < 64 {
                -(64 - value as i32) as f32 / 63.0
            } else {
                (value as i32 - 64) as f32 / 63.0
            };
            if let Ok(mut q) = queue.lock() {
                q.push_back(MixerCommand::SetPan { channel: mixer_channel, value: pan });
            }
        }
    }
    
    /// MCU Button verarbeiten
    fn process_mcu_button(queue: &CommandQueue, note: u8) {
        // MCU REC/ARM buttons (0-7) als Mute
        if note <= 7 {
            // Toggle Mute - nur senden, Server verwaltet State
        }
        // MCU SOLO buttons (8-15)
        else if note >= 8 && note <= 15 {
            let channel = (note - 8) as u32;
            if let Ok(mut q) = queue.lock() {
                q.push_back(MixerCommand::SetSolo { channel, solo: true }); // Toggle wird vom Server gemacht
            }
        }
        // MCU MUTE buttons (16-23)
        else if note >= 16 && note <= 23 {
            let channel = (note - 16) as u32;
            if let Ok(mut q) = queue.lock() {
                q.push_back(MixerCommand::SetMute { channel, muted: true }); // Toggle wird vom Server gemacht
            }
        }
    }
    
    /// Gemapptes MIDI-Event verarbeiten
    fn process_mapped(queue: &CommandQueue, target: &MidiTarget, value: u8) {
        let cmd = match target {
            MidiTarget::Fader { channel } => {
                MixerCommand::SetFader {
                    channel: *channel,
                    value: value as f32 / 127.0,
                }
            }
            MidiTarget::Pan { channel } => {
                MixerCommand::SetPan {
                    channel: *channel,
                    value: (value as f32 / 127.0) * 2.0 - 1.0, // -1 to 1
                }
            }
            MidiTarget::Mute { channel } => {
                MixerCommand::SetMute {
                    channel: *channel,
                    muted: value > 63,
                }
            }
            MidiTarget::Solo { channel } => {
                MixerCommand::SetSolo {
                    channel: *channel,
                    solo: value > 63,
                }
            }
            MidiTarget::Master => {
                MixerCommand::SetMasterFader {
                    value: value as f32 / 127.0,
                }
            }
        };
        
        if let Ok(mut q) = queue.lock() {
            q.push_back(cmd);
        }
    }
}
