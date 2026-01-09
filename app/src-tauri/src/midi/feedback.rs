//! Client-seitige MIDI-Feedback-Ausgabe
//! 
//! Sendet Feedback vom Server zurück an die MIDI-Controller

use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use midir::{MidiOutput, MidiOutputConnection};
use tracing::{info, warn, debug, error};
use audiomultiverse_protocol::MidiTarget;

/// MIDI Feedback Manager für den Client
pub struct ClientMidiFeedback {
    /// Aktive MIDI-Ausgabeverbindungen
    connections: Arc<Mutex<HashMap<String, MidiOutputConnection>>>,
    
    /// MCU-Modus aktiv
    mcu_mode: bool,
    
    /// Letzte Fader-Werte (um unnötige Updates zu vermeiden)
    last_fader_values: Arc<Mutex<HashMap<u32, f32>>>,
}

impl ClientMidiFeedback {
    /// Neuen Feedback Manager erstellen
    pub fn new() -> Self {
        Self {
            connections: Arc::new(Mutex::new(HashMap::new())),
            mcu_mode: false,
            last_fader_values: Arc::new(Mutex::new(HashMap::new())),
        }
    }
    
    /// Verfügbare MIDI-Ausgabegeräte auflisten
    pub fn list_devices(&self) -> Vec<String> {
        let mut devices = Vec::new();
        
        if let Ok(midi_out) = MidiOutput::new("AudioMultiverse Feedback Scanner") {
            for port in midi_out.ports() {
                if let Ok(name) = midi_out.port_name(&port) {
                    devices.push(name);
                }
            }
        }
        
        devices
    }
    
    /// Mit einem MIDI-Ausgabegerät verbinden
    pub fn connect(&mut self, device_name: &str) -> anyhow::Result<()> {
        let midi_out = MidiOutput::new("AudioMultiverse Feedback")?;
        
        let port = midi_out.ports()
            .into_iter()
            .find(|p| midi_out.port_name(p).map(|n| n == device_name).unwrap_or(false))
            .ok_or_else(|| anyhow::anyhow!("MIDI output device not found: {}", device_name))?;
        
        let conn = midi_out.connect(&port, "audiomultiverse-feedback")?;
        
        self.connections.lock().unwrap().insert(device_name.to_string(), conn);
        info!("MIDI Feedback verbunden: {}", device_name);
        
        Ok(())
    }
    
    /// MCU-Modus aktivieren
    pub fn enable_mcu_mode(&mut self) {
        self.mcu_mode = true;
        info!("MCU Feedback Mode aktiviert");
    }
    
    /// Fader-Feedback senden
    pub fn send_fader(&self, channel: u32, value: f32) {
        // Prüfen ob sich der Wert geändert hat (Hysterese)
        {
            let mut last_values = self.last_fader_values.lock().unwrap();
            if let Some(&last) = last_values.get(&channel) {
                if (last - value).abs() < 0.01 {
                    return; // Keine Änderung
                }
            }
            last_values.insert(channel, value);
        }
        
        if self.mcu_mode {
            self.send_mcu_fader(channel, value);
        } else {
            // Generic CC mode
            self.send_cc_all(0, (channel as u8) % 8, (value * 127.0) as u8);
        }
    }
    
    /// MCU Fader Pitch Bend senden
    fn send_mcu_fader(&self, channel: u32, value: f32) {
        if channel >= 9 {
            return; // MCU hat nur 9 Fader (8 + Master)
        }
        
        let pitch_value = (value * 16383.0) as u16;
        let lsb = (pitch_value & 0x7F) as u8;
        let msb = ((pitch_value >> 7) & 0x7F) as u8;
        
        // Pitch Bend: 0xE0 | channel
        let status = 0xE0 | (channel as u8);
        let message = [status, lsb, msb];
        
        if let Ok(conns) = self.connections.lock() {
            for (_, conn) in conns.iter() {
                // MidiOutputConnection hat kein send() - wir müssen es anders machen
                // midir connections können nicht direkt gesendet werden nach connect
                // Wir brauchen eine andere Architektur hier
            }
        }
    }
    
    /// Mute-LED Feedback senden
    pub fn send_mute(&self, channel: u32, muted: bool) {
        if self.mcu_mode {
            // MCU Mute LEDs sind Note 16-23
            if channel < 8 {
                let note = 16 + channel as u8;
                self.send_note_all(0, note, if muted { 127 } else { 0 });
            }
        } else {
            // Generic: CC auf Kanal 1 für Mutes
            self.send_cc_all(1, channel as u8, if muted { 127 } else { 0 });
        }
    }
    
    /// Solo-LED Feedback senden
    pub fn send_solo(&self, channel: u32, solo: bool) {
        if self.mcu_mode {
            // MCU Solo LEDs sind Note 8-15
            if channel < 8 {
                let note = 8 + channel as u8;
                self.send_note_all(0, note, if solo { 127 } else { 0 });
            }
        } else {
            // Generic: CC auf Kanal 2 für Solos
            self.send_cc_all(2, channel as u8, if solo { 127 } else { 0 });
        }
    }
    
    /// Pan-Ring Feedback senden
    pub fn send_pan(&self, channel: u32, pan: f32) {
        if self.mcu_mode {
            // MCU V-Pot LED Ring: CC 48-55
            if channel < 8 {
                let cc = 48 + channel as u8;
                // Pan -1..1 zu MCU Ring 0-11
                let ring_pos = ((pan + 1.0) * 5.5) as u8; // 0-11
                // MCU Ring Mode: 0x3X = Dot, 0x2X = Boost/Cut
                let ring_value = 0x20 | ring_pos.min(11);
                self.send_cc_all(0, cc, ring_value);
            }
        }
    }
    
    /// Meter-Anzeige senden
    pub fn send_meter(&self, channel: u32, level_db: f32) {
        if self.mcu_mode {
            // MCU Meter via Channel Pressure oder SysEx
            // Hier vereinfacht via CC
            if channel < 8 {
                // dB zu Segments (0-12)
                let segments = if level_db <= -60.0 {
                    0u8
                } else if level_db >= 0.0 {
                    12u8
                } else {
                    ((level_db + 60.0) / 5.0) as u8
                };
                
                // MCU Meter: Channel Pressure pro Kanal
                // D0 | (channel << 4) | segments
                // Da Channel Pressure anders funktioniert, 
                // verwenden wir hier eine vereinfachte CC-Variante
            }
        }
    }
    
    /// LCD-Text senden (MCU)
    pub fn send_lcd_text(&self, row: u8, col: u8, text: &str) {
        if !self.mcu_mode {
            return;
        }
        
        // MCU LCD Update via SysEx
        // F0 00 00 66 14 12 <pos> <text...> F7
        let pos = row * 56 + col;
        let mut sysex = vec![
            0xF0, 0x00, 0x00, 0x66, 0x14, 0x12, pos
        ];
        
        for c in text.chars().take(7) {
            sysex.push(c as u8 & 0x7F);
        }
        sysex.push(0xF7);
        
        self.send_sysex_all(&sysex);
    }
    
    /// Timecode-Display senden (MCU)
    pub fn send_timecode(&self, text: &str) {
        if !self.mcu_mode {
            return;
        }
        
        // MCU Timecode via SysEx
        // F0 00 00 66 14 10 <10 chars> F7
        let mut sysex = vec![
            0xF0, 0x00, 0x00, 0x66, 0x14, 0x10
        ];
        
        for c in text.chars().take(10) {
            sysex.push(c as u8 & 0x7F);
        }
        
        // Padding falls nötig
        while sysex.len() < 17 {
            sysex.push(0x20); // Space
        }
        sysex.push(0xF7);
        
        self.send_sysex_all(&sysex);
    }
    
    /// Kompletten Channel-State senden
    pub fn update_channel(&self, channel: u32, fader: f32, muted: bool, solo: bool, pan: f32, name: &str) {
        self.send_fader(channel, fader);
        self.send_mute(channel, muted);
        self.send_solo(channel, solo);
        self.send_pan(channel, pan);
        
        if self.mcu_mode && channel < 8 {
            // LCD Update mit Kanalnamen
            let col = (channel as u8) * 7;
            self.send_lcd_text(0, col, name);
        }
    }
    
    // ============ Hilfsfunktionen ============
    
    /// CC an alle verbundenen Geräte senden
    fn send_cc_all(&self, channel: u8, cc: u8, value: u8) {
        let message = [0xB0 | (channel & 0x0F), cc & 0x7F, value & 0x7F];
        debug!("MIDI Out CC: ch={} cc={} val={}", channel, cc, value);
        // TODO: Actual sending requires connection redesign
    }
    
    /// Note an alle verbundenen Geräte senden
    fn send_note_all(&self, channel: u8, note: u8, velocity: u8) {
        let status = if velocity > 0 { 0x90 } else { 0x80 } | (channel & 0x0F);
        let message = [status, note & 0x7F, velocity & 0x7F];
        debug!("MIDI Out Note: ch={} note={} vel={}", channel, note, velocity);
        // TODO: Actual sending
    }
    
    /// SysEx an alle verbundenen Geräte senden
    fn send_sysex_all(&self, data: &[u8]) {
        debug!("MIDI Out SysEx: {} bytes", data.len());
        // TODO: Actual sending
    }
}

impl Default for ClientMidiFeedback {
    fn default() -> Self {
        Self::new()
    }
}
