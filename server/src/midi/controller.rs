//! MIDI Controller
//! 
//! Verwaltet MIDI-Geräte und Mappings

use std::sync::Arc;
use std::collections::HashMap;
use anyhow::Result;
use tracing::{info, warn, error};

use audiomultiverse_protocol::{MidiMapping, MidiTarget, ChannelId};
use crate::mixer::Mixer;

/// MIDI Controller Manager
pub struct MidiController {
    /// Aktive MIDI-Verbindungen
    // TODO: midir Connections
    
    /// Mappings: (device, channel, cc) -> Target
    mappings: HashMap<(String, u8, u8), MidiTarget>,
    
    /// Mixer-Referenz
    mixer: Option<Arc<Mixer>>,
    
    /// MIDI-Learn Modus aktiv
    learn_mode: bool,
    
    /// Ziel für MIDI-Learn
    learn_target: Option<MidiTarget>,
}

impl MidiController {
    /// Neuen MIDI Controller erstellen
    pub fn new() -> Self {
        Self {
            mappings: HashMap::new(),
            mixer: None,
            learn_mode: false,
            learn_target: None,
        }
    }

    /// Mixer-Referenz setzen
    pub fn set_mixer(&mut self, mixer: Arc<Mixer>) {
        self.mixer = Some(mixer);
    }

    /// MIDI initialisieren
    pub fn init(&mut self) -> Result<()> {
        info!("🎹 MIDI Controller initialisiert");
        
        // Verfügbare MIDI-Geräte auflisten
        match midir::MidiInput::new("AudioMultiverse Input") {
            Ok(midi_in) => {
                let ports = midi_in.ports();
                if ports.is_empty() {
                    info!("   Keine MIDI-Eingabegeräte gefunden");
                } else {
                    for (i, port) in ports.iter().enumerate() {
                        if let Ok(name) = midi_in.port_name(port) {
                            info!("   [{}] {}", i, name);
                        }
                    }
                }
            }
            Err(e) => {
                warn!("MIDI Input konnte nicht initialisiert werden: {}", e);
            }
        }
        
        // Standard-Mappings laden
        self.load_default_mappings();
        
        Ok(())
    }

    /// Standard-Mappings laden (generisches CC-Layout)
    fn load_default_mappings(&mut self) {
        // Fader auf CC 0-31 (oder 0-7 für nanoKONTROL etc.)
        for i in 0..32 {
            self.mappings.insert(
                ("*".to_string(), 0, i as u8),
                MidiTarget::Fader { channel: i },
            );
        }
        
        // Mute auf CC 32-63
        for i in 0..32 {
            self.mappings.insert(
                ("*".to_string(), 0, (32 + i) as u8),
                MidiTarget::Mute { channel: i },
            );
        }
        
        // Solo auf CC 64-95
        for i in 0..32 {
            self.mappings.insert(
                ("*".to_string(), 0, (64 + i) as u8),
                MidiTarget::Solo { channel: i },
            );
        }
        
        info!("   Standard-Mappings geladen");
    }

    /// MIDI-Nachricht verarbeiten
    pub fn process_message(&mut self, device: &str, data: &[u8]) {
        if data.len() < 3 {
            return;
        }

        let status = data[0];
        let cc = data[1];
        let value = data[2];
        
        // Control Change (0xB0-0xBF)
        if status >= 0xB0 && status <= 0xBF {
            let channel = status - 0xB0;
            self.handle_cc(device, channel, cc, value);
        }
        // Note On/Off für Mute/Solo Buttons
        else if status >= 0x90 && status <= 0x9F {
            let channel = status - 0x90;
            self.handle_note(device, channel, data[1], value > 0);
        }
    }

    /// Control Change verarbeiten
    fn handle_cc(&mut self, device: &str, channel: u8, cc: u8, value: u8) {
        // MIDI-Learn Modus
        if self.learn_mode {
            if let Some(target) = &self.learn_target {
                self.mappings.insert(
                    (device.to_string(), channel, cc),
                    target.clone(),
                );
                info!("MIDI Learn: CC {} auf {:?} gemappt", cc, target);
                self.learn_mode = false;
                self.learn_target = None;
            }
            return;
        }

        // Mapping suchen
        let key = (device.to_string(), channel, cc);
        let wildcard_key = ("*".to_string(), channel, cc);
        
        let target = self.mappings.get(&key)
            .or_else(|| self.mappings.get(&wildcard_key));
        
        if let Some(target) = target {
            self.apply_midi_value(target.clone(), value);
        }
    }

    /// Note On/Off verarbeiten
    fn handle_note(&mut self, device: &str, channel: u8, note: u8, on: bool) {
        // Notes werden für Toggle-Buttons verwendet
        // TODO: Note-Mappings
    }

    /// MIDI-Wert auf Mixer anwenden
    fn apply_midi_value(&self, target: MidiTarget, value: u8) {
        let mixer = match &self.mixer {
            Some(m) => m,
            None => return,
        };

        match target {
            MidiTarget::Fader { channel } => {
                // MIDI 0-127 -> Fader 0.0-1.0
                let fader_value = value as f32 / 127.0;
                mixer.set_fader(channel, fader_value);
            }
            MidiTarget::Mute { channel } => {
                // Toggle bei Wert > 0
                if value > 0 {
                    if let Some(state) = mixer.get_channel(channel) {
                        mixer.set_mute(channel, !state.mute);
                    }
                }
            }
            MidiTarget::Solo { channel } => {
                if value > 0 {
                    if let Some(state) = mixer.get_channel(channel) {
                        mixer.set_solo(channel, !state.solo);
                    }
                }
            }
            MidiTarget::Pan { channel } => {
                // MIDI 0-127 -> Pan -1.0 bis +1.0
                let pan_value = (value as f32 / 63.5) - 1.0;
                mixer.set_pan(channel, pan_value);
            }
            MidiTarget::Master => {
                // TODO: Master Fader
            }
        }
    }

    /// MIDI-Learn Modus starten
    pub fn start_learn(&mut self, target: MidiTarget) {
        self.learn_mode = true;
        info!("MIDI Learn aktiviert für: {:?}", target);
        self.learn_target = Some(target);
    }

    /// MIDI-Learn Modus abbrechen
    pub fn cancel_learn(&mut self) {
        self.learn_mode = false;
        self.learn_target = None;
    }

    /// MIDI Feedback senden (für motorisierte Fader, LEDs)
    pub fn send_feedback(&self, target: &MidiTarget, value: u8) {
        // TODO: MIDI Output für Feedback
    }

    /// Mapping hinzufügen
    pub fn add_mapping(&mut self, mapping: MidiMapping) {
        self.mappings.insert(
            (mapping.device, mapping.channel, mapping.cc),
            mapping.target,
        );
    }

    /// Alle Mappings abrufen
    pub fn get_mappings(&self) -> Vec<MidiMapping> {
        self.mappings.iter()
            .map(|((device, channel, cc), target)| MidiMapping {
                device: device.clone(),
                channel: *channel,
                cc: *cc,
                target: target.clone(),
            })
            .collect()
    }
}

impl Default for MidiController {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_midi_controller_creation() {
        let controller = MidiController::new();
        assert!(!controller.learn_mode);
    }
}
