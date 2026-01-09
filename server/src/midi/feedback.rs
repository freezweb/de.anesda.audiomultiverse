//! MIDI Output Feedback
//!
//! Sendet MIDI-Daten zurück an Controller (motorisierte Fader, LEDs, etc.)

use std::sync::{Arc, Mutex, RwLock};
use std::collections::HashMap;
use std::thread;
use std::time::Duration;
use anyhow::Result;
use tracing::{info, warn, error, debug};
use midir::{MidiOutput, MidiOutputConnection, MidiOutputPort};

use audiomultiverse_protocol::{MidiTarget, ChannelId};

/// MIDI Output Manager für Feedback
pub struct MidiOutputManager {
    /// Aktive MIDI Output Verbindungen
    connections: HashMap<String, Arc<Mutex<MidiOutputConnection>>>,
    
    /// Verfügbare Ports
    available_ports: Vec<MidiOutputPort>,
    
    /// MIDI Output Objekt
    midi_output: Option<MidiOutput>,
    
    /// Letzte gesendete Werte (um Spam zu vermeiden)
    last_sent: RwLock<HashMap<(String, u8, u8), u8>>,
}

impl MidiOutputManager {
    /// Neuen Output Manager erstellen
    pub fn new() -> Self {
        let midi_output = MidiOutput::new("AudioMultiverse Output").ok();
        let available_ports = midi_output.as_ref()
            .map(|m| m.ports())
            .unwrap_or_default();
        
        Self {
            connections: HashMap::new(),
            available_ports,
            midi_output,
            last_sent: RwLock::new(HashMap::new()),
        }
    }
    
    /// Verfügbare Output-Geräte auflisten
    pub fn list_devices(&self) -> Vec<String> {
        let Some(ref midi_out) = self.midi_output else {
            return vec![];
        };
        
        self.available_ports.iter()
            .filter_map(|port| midi_out.port_name(port).ok())
            .collect()
    }
    
    /// Mit MIDI Output verbinden
    pub fn connect(&mut self, device_name: &str) -> Result<()> {
        let midi_out = self.midi_output.take()
            .ok_or_else(|| anyhow::anyhow!("MIDI Output nicht verfügbar"))?;
        
        let port = self.available_ports.iter()
            .find(|p| midi_out.port_name(p).ok().as_deref() == Some(device_name))
            .ok_or_else(|| anyhow::anyhow!("MIDI Gerät nicht gefunden: {}", device_name))?
            .clone();
        
        let connection = midi_out.connect(&port, "audiomultiverse-feedback")
            .map_err(|e| anyhow::anyhow!("MIDI Connect Fehler: {}", e))?;
        
        info!("🎹 MIDI Output verbunden: {}", device_name);
        
        self.connections.insert(
            device_name.to_string(),
            Arc::new(Mutex::new(connection)),
        );
        
        Ok(())
    }
    
    /// Control Change senden
    pub fn send_cc(&self, device: &str, channel: u8, cc: u8, value: u8) {
        // Prüfen ob sich der Wert geändert hat
        let key = (device.to_string(), channel, cc);
        {
            let last = self.last_sent.read().unwrap();
            if last.get(&key) == Some(&value) {
                return; // Keine Änderung, nicht senden
            }
        }
        
        // Wert speichern
        {
            let mut last = self.last_sent.write().unwrap();
            last.insert(key, value);
        }
        
        // An alle verbundenen Geräte senden, wenn device "*" ist
        let devices: Vec<_> = if device == "*" {
            self.connections.keys().cloned().collect()
        } else {
            vec![device.to_string()]
        };
        
        for dev_name in devices {
            if let Some(conn) = self.connections.get(&dev_name) {
                let status = 0xB0 | (channel & 0x0F);
                let message = [status, cc, value];
                
                if let Ok(mut conn) = conn.lock() {
                    if let Err(e) = conn.send(&message) {
                        error!("MIDI Send Fehler: {}", e);
                    } else {
                        debug!("MIDI CC gesendet: ch={} cc={} val={}", channel, cc, value);
                    }
                }
            }
        }
    }
    
    /// Note On/Off senden (für LEDs)
    pub fn send_note(&self, device: &str, channel: u8, note: u8, velocity: u8) {
        let devices: Vec<_> = if device == "*" {
            self.connections.keys().cloned().collect()
        } else {
            vec![device.to_string()]
        };
        
        for dev_name in devices {
            if let Some(conn) = self.connections.get(&dev_name) {
                let status = if velocity > 0 { 0x90 } else { 0x80 } | (channel & 0x0F);
                let message = [status, note, velocity];
                
                if let Ok(mut conn) = conn.lock() {
                    if let Err(e) = conn.send(&message) {
                        error!("MIDI Note Send Fehler: {}", e);
                    }
                }
            }
        }
    }
    
    /// SysEx senden (für Display-Text etc.)
    pub fn send_sysex(&self, device: &str, data: &[u8]) {
        if let Some(conn) = self.connections.get(device) {
            if let Ok(mut conn) = conn.lock() {
                if let Err(e) = conn.send(data) {
                    error!("MIDI SysEx Send Fehler: {}", e);
                }
            }
        }
    }
}

impl Default for MidiOutputManager {
    fn default() -> Self {
        Self::new()
    }
}

/// MIDI Feedback Handler
/// Übersetzt Mixer-State-Änderungen in MIDI-Nachrichten
pub struct MidiFeedback {
    /// Output Manager
    output: Arc<Mutex<MidiOutputManager>>,
    
    /// Feedback Mappings: Target -> (device, channel, cc)
    mappings: HashMap<MidiTarget, (String, u8, u8)>,
    
    /// MCU Modus aktiviert
    mcu_mode: bool,
}

impl MidiFeedback {
    /// Neuen Feedback Handler erstellen
    pub fn new(output: Arc<Mutex<MidiOutputManager>>) -> Self {
        Self {
            output,
            mappings: HashMap::new(),
            mcu_mode: false,
        }
    }
    
    /// MCU Modus aktivieren
    pub fn enable_mcu_mode(&mut self) {
        self.mcu_mode = true;
        self.load_mcu_mappings();
        info!("🎹 Mackie Control Modus aktiviert");
    }
    
    /// Standard MCU Mappings laden
    fn load_mcu_mappings(&mut self) {
        // MCU Fader Pitch Bend (Ch 1-8)
        // MCU Meter auf speziellen SysEx
        // MCU Rec/Solo/Mute LEDs auf Notes
        
        // V-Pot (Encoder) auf CC 16-23
        for i in 0..8 {
            self.mappings.insert(
                MidiTarget::Pan { channel: i },
                ("*".to_string(), 0, (16 + i) as u8),
            );
        }
        
        // Solo LEDs auf Notes 8-15
        for i in 0..8 {
            self.mappings.insert(
                MidiTarget::Solo { channel: i },
                ("*".to_string(), 0, (8 + i) as u8),
            );
        }
        
        // Mute LEDs auf Notes 16-23
        for i in 0..8 {
            self.mappings.insert(
                MidiTarget::Mute { channel: i },
                ("*".to_string(), 0, (16 + i) as u8),
            );
        }
    }
    
    /// Generisches Feedback Mapping hinzufügen
    pub fn add_mapping(&mut self, target: MidiTarget, device: &str, channel: u8, cc: u8) {
        self.mappings.insert(target, (device.to_string(), channel, cc));
    }
    
    /// Fader-Position senden
    pub fn send_fader(&self, channel: ChannelId, value: f32) {
        let midi_value = (value.clamp(0.0, 1.0) * 127.0) as u8;
        
        if self.mcu_mode {
            // MCU: Pitch Bend für motorisierte Fader
            self.send_mcu_fader(channel, value);
        } else {
            // Generic: CC
            let target = MidiTarget::Fader { channel };
            if let Some((device, ch, cc)) = self.mappings.get(&target) {
                if let Ok(output) = self.output.lock() {
                    output.send_cc(device, *ch, *cc, midi_value);
                }
            }
        }
    }
    
    /// MCU Fader über Pitch Bend senden
    fn send_mcu_fader(&self, channel: ChannelId, value: f32) {
        if channel >= 8 {
            return; // MCU hat nur 8 Fader
        }
        
        // 14-bit Auflösung (0-16383)
        let pitch_value = ((value.clamp(0.0, 1.0) * 16383.0) as u16).min(16383);
        let lsb = (pitch_value & 0x7F) as u8;
        let msb = ((pitch_value >> 7) & 0x7F) as u8;
        
        // Pitch Bend: 0xE0 + channel
        if let Some((device, _, _)) = self.mappings.get(&MidiTarget::Fader { channel }) {
            // Manuell Pitch Bend senden
            // TODO: Spezielle Methode für Pitch Bend
        }
    }
    
    /// Mute LED Status senden
    pub fn send_mute(&self, channel: ChannelId, muted: bool) {
        let target = MidiTarget::Mute { channel };
        
        if self.mcu_mode {
            // MCU: Note On/Off für LED
            if let Ok(output) = self.output.lock() {
                let note = 16 + (channel as u8 % 8);
                output.send_note("*", 0, note, if muted { 127 } else { 0 });
            }
        } else if let Some((device, ch, cc)) = self.mappings.get(&target) {
            if let Ok(output) = self.output.lock() {
                output.send_cc(device, *ch, *cc, if muted { 127 } else { 0 });
            }
        }
    }
    
    /// Solo LED Status senden
    pub fn send_solo(&self, channel: ChannelId, solo: bool) {
        let target = MidiTarget::Solo { channel };
        
        if self.mcu_mode {
            if let Ok(output) = self.output.lock() {
                let note = 8 + (channel as u8 % 8);
                output.send_note("*", 0, note, if solo { 127 } else { 0 });
            }
        } else if let Some((device, ch, cc)) = self.mappings.get(&target) {
            if let Ok(output) = self.output.lock() {
                output.send_cc(device, *ch, *cc, if solo { 127 } else { 0 });
            }
        }
    }
    
    /// Pan Position senden (LED Ring)
    pub fn send_pan(&self, channel: ChannelId, pan: f32) {
        // Pan -1.0 bis +1.0 -> MIDI 0-127 (64 = Mitte)
        let midi_value = ((pan + 1.0) / 2.0 * 127.0) as u8;
        
        if self.mcu_mode {
            // MCU V-Pot LED Ring
            // Werte 0-11 = LED Position, Bit 4-5 = Mode
            let led_pos = ((pan + 1.0) / 2.0 * 11.0) as u8;
            let vpot_value = 0x40 | led_pos; // Single dot mode
            
            if let Ok(output) = self.output.lock() {
                let cc = 48 + (channel as u8 % 8); // MCU V-Pot LEDs
                output.send_cc("*", 0, cc, vpot_value);
            }
        } else {
            let target = MidiTarget::Pan { channel };
            if let Some((device, ch, cc)) = self.mappings.get(&target) {
                if let Ok(output) = self.output.lock() {
                    output.send_cc(device, *ch, *cc, midi_value);
                }
            }
        }
    }
    
    /// MCU Kanal-Text senden (LCD)
    pub fn send_mcu_text(&self, channel: ChannelId, text: &str) {
        if !self.mcu_mode || channel >= 8 {
            return;
        }
        
        // MCU LCD SysEx: F0 00 00 66 14 12 <offset> <chars...> F7
        let offset = (channel as u8) * 7; // 7 Zeichen pro Kanal
        let text_bytes: Vec<u8> = text.chars()
            .take(7)
            .map(|c| {
                let ascii = c as u8;
                if ascii >= 0x20 && ascii <= 0x7E { ascii } else { 0x20 }
            })
            .collect();
        
        let mut sysex = vec![0xF0, 0x00, 0x00, 0x66, 0x14, 0x12, offset];
        sysex.extend(text_bytes);
        sysex.push(0xF7);
        
        if let Ok(output) = self.output.lock() {
            output.send_sysex("*", &sysex);
        }
    }
    
    /// MCU Meter senden
    pub fn send_mcu_meter(&self, channel: ChannelId, level: f32) {
        if !self.mcu_mode || channel >= 8 {
            return;
        }
        
        // MCU Meter: Channel Pressure mit Kanal in High Nibble, Level in Low Nibble
        // Level: 0-12 (0=aus, 1-12=Segmente, +0x10 für Clip)
        let segments = (level.clamp(0.0, 1.0) * 12.0) as u8;
        let meter_value = ((channel as u8 % 8) << 4) | segments;
        
        // Channel Pressure (0xD0)
        if let Ok(output) = self.output.lock() {
            // Manuell senden, da kein CC
            // TODO: Raw MIDI Methode
        }
    }
    
    /// Alle Feedback-Werte für einen Kanal senden
    pub fn send_channel_state(
        &self,
        channel: ChannelId,
        fader: f32,
        pan: f32,
        mute: bool,
        solo: bool,
        name: Option<&str>,
    ) {
        self.send_fader(channel, fader);
        self.send_pan(channel, pan);
        self.send_mute(channel, mute);
        self.send_solo(channel, solo);
        
        if let Some(text) = name {
            self.send_mcu_text(channel, text);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_output_manager_creation() {
        let manager = MidiOutputManager::new();
        // Kann je nach System MIDI-Geräte haben oder nicht
        let _ = manager.list_devices();
    }
    
    #[test]
    fn test_feedback_creation() {
        let output = Arc::new(Mutex::new(MidiOutputManager::new()));
        let feedback = MidiFeedback::new(output);
        assert!(!feedback.mcu_mode);
    }
}
