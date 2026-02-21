//! Bus-System für Aux Sends und Gruppen
//! 
//! Implementiert:
//! - 8x Stereo Aux Sends (Pre/Post Fader)
//! - 4x Stereo Gruppen/Subgruppen
//! - Matrix Outputs

#![allow(dead_code)]

use serde::{Deserialize, Serialize};
use std::sync::atomic::{AtomicU32, Ordering};

/// Bus-Typ
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum BusType {
    /// Aux Send (für Monitoring, Effekte)
    Aux,
    /// Gruppe/Subgruppe (für Submixing)
    Group,
    /// Matrix Output (flexible Routing)
    Matrix,
}

/// Aux Send Modus
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize, Default)]
pub enum AuxSendMode {
    /// Pre-Fader (unabhängig vom Channel-Fader)
    PreFader,
    /// Post-Fader (folgt Channel-Fader)
    #[default]
    PostFader,
}

/// Aux Send pro Kanal
#[derive(Debug)]
pub struct AuxSend {
    /// Aux Bus ID (0-7)
    pub aux_id: u8,
    /// Send Level (0.0 - 1.0)
    level: AtomicU32,
    /// Pan (-1.0 = L, 0.0 = C, 1.0 = R)
    pan: AtomicU32,
    /// Pre/Post Fader Modus
    pub mode: AuxSendMode,
    /// Send aktiv
    pub enabled: bool,
}

impl AuxSend {
    pub fn new(aux_id: u8) -> Self {
        Self {
            aux_id,
            level: AtomicU32::new(0.0_f32.to_bits()),
            pan: AtomicU32::new(0.0_f32.to_bits()),
            mode: AuxSendMode::PostFader,
            enabled: true,
        }
    }

    pub fn level(&self) -> f32 {
        f32::from_bits(self.level.load(Ordering::Relaxed))
    }

    pub fn set_level(&self, level: f32) {
        self.level.store(level.clamp(0.0, 1.0).to_bits(), Ordering::Relaxed);
    }

    pub fn pan(&self) -> f32 {
        f32::from_bits(self.pan.load(Ordering::Relaxed))
    }

    pub fn set_pan(&self, pan: f32) {
        self.pan.store(pan.clamp(-1.0, 1.0).to_bits(), Ordering::Relaxed);
    }
    
    pub fn set_enabled(&mut self, enabled: bool) {
        self.enabled = enabled;
    }
    
    pub fn set_mode(&mut self, mode: AuxSendMode) {
        self.mode = mode;
    }
    
    /// Bus ID für externe Nutzung (API-Kompatibilität)
    pub fn aux_bus_id(&self) -> u32 {
        self.aux_id as u32
    }
}

/// Aux Send State (für API/Protokoll)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuxSendState {
    pub aux_id: u8,
    pub level: f32,
    pub pan: f32,
    pub mode: AuxSendMode,
    pub enabled: bool,
}

impl AuxSend {
    pub fn state(&self) -> AuxSendState {
        AuxSendState {
            aux_id: self.aux_id,
            level: self.level(),
            pan: self.pan(),
            mode: self.mode,
            enabled: self.enabled,
        }
    }
}

/// Stereo Aux Bus
#[derive(Debug)]
pub struct AuxBus {
    /// Bus ID (0-7)
    pub id: u8,
    /// Bus Name
    pub name: String,
    /// Master Level (0.0 - 1.25, >1.0 = Boost)
    level: AtomicU32,
    /// Mute
    pub muted: bool,
    /// Solo (AFL)
    pub solo: bool,
    /// Stereo Link
    pub stereo_linked: bool,
    /// Metering (Peak L/R)
    meter_left: AtomicU32,
    meter_right: AtomicU32,
}

impl AuxBus {
    pub fn new(id: u8, name: impl Into<String>) -> Self {
        Self {
            id,
            name: name.into(),
            level: AtomicU32::new(0.75_f32.to_bits()), // Unity (0dB)
            muted: false,
            solo: false,
            stereo_linked: true,
            meter_left: AtomicU32::new(0),
            meter_right: AtomicU32::new(0),
        }
    }

    pub fn level(&self) -> f32 {
        f32::from_bits(self.level.load(Ordering::Relaxed))
    }

    pub fn set_level(&self, level: f32) {
        self.level.store(level.clamp(0.0, 1.25).to_bits(), Ordering::Relaxed);
    }
    
    pub fn set_mute(&mut self, muted: bool) {
        self.muted = muted;
    }
    
    pub fn set_pan(&mut self, _pan: f32) {
        // Aux Buses haben keinen Pan (sind stereo-linked)
    }

    pub fn update_meters(&self, left: f32, right: f32) {
        self.meter_left.store(left.to_bits(), Ordering::Relaxed);
        self.meter_right.store(right.to_bits(), Ordering::Relaxed);
    }

    pub fn meters(&self) -> (f32, f32) {
        (
            f32::from_bits(self.meter_left.load(Ordering::Relaxed)),
            f32::from_bits(self.meter_right.load(Ordering::Relaxed)),
        )
    }
}

/// Aux Bus State (für API/Protokoll)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuxBusState {
    pub id: u8,
    pub name: String,
    pub level: f32,
    pub muted: bool,
    pub solo: bool,
    pub stereo_linked: bool,
    pub meter_left: f32,
    pub meter_right: f32,
}

impl AuxBus {
    pub fn state(&self) -> AuxBusState {
        let (meter_left, meter_right) = self.meters();
        AuxBusState {
            id: self.id,
            name: self.name.clone(),
            level: self.level(),
            muted: self.muted,
            solo: self.solo,
            stereo_linked: self.stereo_linked,
            meter_left,
            meter_right,
        }
    }
}

/// Stereo Gruppe/Subgruppe
#[derive(Debug)]
pub struct GroupBus {
    /// Bus ID (0-3)
    pub id: u8,
    /// Bus Name
    pub name: String,
    /// Master Fader (0.0 - 1.25)
    fader: AtomicU32,
    /// Pan (-1.0 = L, 0.0 = C, 1.0 = R)
    pan: AtomicU32,
    /// Mute
    pub muted: bool,
    /// Solo
    pub solo: bool,
    /// An Master routen
    pub to_master: bool,
    /// Metering (Peak L/R)
    meter_left: AtomicU32,
    meter_right: AtomicU32,
}

impl GroupBus {
    pub fn new(id: u8, name: impl Into<String>) -> Self {
        Self {
            id,
            name: name.into(),
            fader: AtomicU32::new(0.75_f32.to_bits()), // Unity
            pan: AtomicU32::new(0.0_f32.to_bits()),
            muted: false,
            solo: false,
            to_master: true,
            meter_left: AtomicU32::new(0),
            meter_right: AtomicU32::new(0),
        }
    }

    pub fn fader(&self) -> f32 {
        f32::from_bits(self.fader.load(Ordering::Relaxed))
    }

    pub fn set_fader(&self, fader: f32) {
        self.fader.store(fader.clamp(0.0, 1.25).to_bits(), Ordering::Relaxed);
    }

    pub fn pan(&self) -> f32 {
        f32::from_bits(self.pan.load(Ordering::Relaxed))
    }

    pub fn set_pan(&self, pan: f32) {
        self.pan.store(pan.clamp(-1.0, 1.0).to_bits(), Ordering::Relaxed);
    }
    
    pub fn set_level(&self, level: f32) {
        self.set_fader(level);
    }
    
    pub fn set_mute(&mut self, muted: bool) {
        self.muted = muted;
    }
    
    pub fn set_to_master(&mut self, to_master: bool) {
        self.to_master = to_master;
    }

    pub fn update_meters(&self, left: f32, right: f32) {
        self.meter_left.store(left.to_bits(), Ordering::Relaxed);
        self.meter_right.store(right.to_bits(), Ordering::Relaxed);
    }

    pub fn meters(&self) -> (f32, f32) {
        (
            f32::from_bits(self.meter_left.load(Ordering::Relaxed)),
            f32::from_bits(self.meter_right.load(Ordering::Relaxed)),
        )
    }
}

/// Group Bus State (für API/Protokoll)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GroupBusState {
    pub id: u8,
    pub name: String,
    pub fader: f32,
    pub pan: f32,
    pub muted: bool,
    pub solo: bool,
    pub to_master: bool,
    pub meter_left: f32,
    pub meter_right: f32,
}

impl GroupBus {
    pub fn state(&self) -> GroupBusState {
        let (meter_left, meter_right) = self.meters();
        GroupBusState {
            id: self.id,
            name: self.name.clone(),
            fader: self.fader(),
            pan: self.pan(),
            muted: self.muted,
            solo: self.solo,
            to_master: self.to_master,
            meter_left,
            meter_right,
        }
    }
}

/// Bus-Manager für alle Aux Sends und Gruppen
pub struct BusManager {
    /// 8 Aux Busse
    pub aux_buses: Vec<AuxBus>,
    /// 4 Gruppen
    pub group_buses: Vec<GroupBus>,
    /// Anzahl Input-Kanäle (für Aux Sends pro Kanal)
    input_count: usize,
    /// Aux Sends pro Kanal (input_count * 8 AuxSends)
    aux_sends: Vec<Vec<AuxSend>>,
    /// Gruppen-Zuweisungen pro Kanal (input_count * 4 bools)
    group_assignments: Vec<[bool; 4]>,
    /// Channel -> Aux Sends Map (für API-Zugriff)
    pub channel_aux_sends: std::collections::HashMap<u32, Vec<AuxSend>>,
}

impl BusManager {
    /// Neuen BusManager erstellen
    /// 
    /// # Arguments
    /// * `aux_count` - Anzahl Aux Busse (default: 8)
    /// * `group_count` - Anzahl Gruppen (default: 4)
    pub fn new(aux_count: usize, group_count: usize) -> Self {
        // Aux Busse erstellen
        let aux_buses: Vec<AuxBus> = (0..aux_count)
            .map(|i| AuxBus::new(i as u8, format!("Aux {}", i + 1)))
            .collect();

        // Gruppen erstellen
        let group_buses: Vec<GroupBus> = (0..group_count)
            .map(|i| GroupBus::new(i as u8, format!("Group {}", i + 1)))
            .collect();

        Self {
            aux_buses,
            group_buses,
            input_count: 32, // Default 32 Kanäle
            aux_sends: vec![],
            group_assignments: vec![],
            channel_aux_sends: std::collections::HashMap::new(),
        }
    }
    
    /// Input-Kanäle initialisieren
    pub fn init_channels(&mut self, input_count: usize) {
        self.input_count = input_count;
        
        // Aux Sends pro Kanal erstellen
        self.aux_sends = (0..input_count)
            .map(|_| {
                (0..self.aux_buses.len())
                    .map(|aux_id| AuxSend::new(aux_id as u8))
                    .collect()
            })
            .collect();

        // Gruppen-Zuweisungen initialisieren (alle aus)
        self.group_assignments = vec![[false; 4]; input_count];
    }
    
    /// Kanal einer Gruppe zuweisen
    pub fn assign_channel_to_group(&mut self, channel_id: u32, group_id: u32) {
        let ch = channel_id as usize;
        let grp = group_id as usize;
        if ch < self.group_assignments.len() && grp < 4 {
            self.group_assignments[ch][grp] = true;
        }
    }
    
    /// Kanal von Gruppe entfernen
    pub fn remove_channel_from_group(&mut self, channel_id: u32, group_id: u32) {
        let ch = channel_id as usize;
        let grp = group_id as usize;
        if ch < self.group_assignments.len() && grp < 4 {
            self.group_assignments[ch][grp] = false;
        }
    }
    
    /// Aux Sends für einen Kanal abrufen (für API)
    pub fn get_channel_aux_sends(&self, channel_id: u32) -> Vec<AuxSend> {
        if let Some(sends) = self.channel_aux_sends.get(&channel_id) {
            return sends.iter().map(|s| AuxSend::new(s.aux_id)).collect();
        }
        // Default: Leere Sends erstellen
        (0..self.aux_buses.len() as u8)
            .map(AuxSend::new)
            .collect()
    }

    /// Aux Send Level für einen Kanal setzen
    pub fn set_aux_send(&mut self, channel: usize, aux: usize, level: f32) -> bool {
        if channel < self.input_count && aux < 8 {
            self.aux_sends[channel][aux].set_level(level);
            true
        } else {
            false
        }
    }

    /// Aux Send Mode für einen Kanal setzen
    pub fn set_aux_send_mode(&mut self, channel: usize, aux: usize, mode: AuxSendMode) -> bool {
        if channel < self.input_count && aux < 8 {
            self.aux_sends[channel][aux].mode = mode;
            true
        } else {
            false
        }
    }

    /// Aux Send State für einen Kanal abrufen
    pub fn get_aux_send(&self, channel: usize, aux: usize) -> Option<AuxSendState> {
        if channel < self.input_count && aux < 8 {
            Some(self.aux_sends[channel][aux].state())
        } else {
            None
        }
    }

    /// Alle Aux Sends für einen Kanal abrufen (Vec<AuxSendState>)
    pub fn get_channel_aux_send_states(&self, channel: usize) -> Vec<AuxSendState> {
        if channel < self.aux_sends.len() {
            self.aux_sends[channel].iter().map(|s| s.state()).collect()
        } else {
            vec![]
        }
    }

    /// Kanal einer Gruppe zuweisen
    pub fn assign_to_group(&mut self, channel: usize, group: usize, assigned: bool) -> bool {
        if channel < self.input_count && group < 4 {
            self.group_assignments[channel][group] = assigned;
            true
        } else {
            false
        }
    }

    /// Gruppen-Zuweisungen für einen Kanal abrufen
    pub fn get_group_assignments(&self, channel: usize) -> Option<[bool; 4]> {
        self.group_assignments.get(channel).copied()
    }

    /// Aux Bus Level setzen
    pub fn set_aux_bus_level(&mut self, aux: usize, level: f32) -> bool {
        if let Some(bus) = self.aux_buses.get(aux) {
            bus.set_level(level);
            true
        } else {
            false
        }
    }

    /// Aux Bus Mute setzen
    pub fn set_aux_bus_mute(&mut self, aux: usize, muted: bool) -> bool {
        if let Some(bus) = self.aux_buses.get_mut(aux) {
            bus.muted = muted;
            true
        } else {
            false
        }
    }

    /// Gruppe Fader setzen
    pub fn set_group_fader(&mut self, group: usize, fader: f32) -> bool {
        if let Some(bus) = self.group_buses.get(group) {
            bus.set_fader(fader);
            true
        } else {
            false
        }
    }

    /// Gruppe Mute setzen
    pub fn set_group_mute(&mut self, group: usize, muted: bool) -> bool {
        if let Some(bus) = self.group_buses.get_mut(group) {
            bus.muted = muted;
            true
        } else {
            false
        }
    }

    /// Alle Aux Bus States abrufen
    pub fn get_aux_buses(&self) -> Vec<AuxBusState> {
        self.aux_buses.iter().map(|b| b.state()).collect()
    }

    /// Alle Gruppen States abrufen
    pub fn get_groups(&self) -> Vec<GroupBusState> {
        self.group_buses.iter().map(|g| g.state()).collect()
    }

    /// Audio-Processing für Aux Sends (wird im Audio-Thread aufgerufen)
    /// 
    /// Nimmt die Channel-Samples und berechnet die Aux-Summen
    pub fn process_aux_sends(
        &self,
        channel: usize,
        channel_fader: f32,
        sample_left: f32,
        sample_right: f32,
        aux_buffers: &mut [[f32; 2]; 8],
    ) {
        if channel >= self.aux_sends.len() {
            return;
        }

        for (aux_id, send) in self.aux_sends[channel].iter().enumerate() {
            if !send.enabled {
                continue;
            }

            let level = send.level();
            if level < 0.001 {
                continue;
            }

            // Pre/Post Fader Gain
            let gain = match send.mode {
                AuxSendMode::PreFader => level,
                AuxSendMode::PostFader => level * channel_fader,
            };

            // Pan Law (Constant Power)
            let pan = send.pan();
            let pan_rad = (pan + 1.0) * std::f32::consts::FRAC_PI_4;
            let left_gain = pan_rad.cos() * gain;
            let right_gain = pan_rad.sin() * gain;

            if aux_id < aux_buffers.len() {
                aux_buffers[aux_id][0] += sample_left * left_gain;
                aux_buffers[aux_id][1] += sample_right * right_gain;
            }
        }
    }

    /// Audio-Processing für Gruppen (wird im Audio-Thread aufgerufen)
    pub fn process_groups(
        &self,
        channel: usize,
        sample_left: f32,
        sample_right: f32,
        group_buffers: &mut [[f32; 2]; 4],
    ) {
        if channel >= self.group_assignments.len() {
            return;
        }

        let assignments = &self.group_assignments[channel];
        
        for (group_id, &assigned) in assignments.iter().enumerate() {
            if assigned && group_id < group_buffers.len() {
                group_buffers[group_id][0] += sample_left;
                group_buffers[group_id][1] += sample_right;
            }
        }
    }

    /// Master-Mix aus Gruppen berechnen
    pub fn sum_groups_to_master(&self, group_buffers: &[[f32; 2]; 4]) -> (f32, f32) {
        let mut left = 0.0;
        let mut right = 0.0;

        for (i, buffer) in group_buffers.iter().enumerate() {
            if i >= self.group_buses.len() {
                break;
            }
            let group = &self.group_buses[i];
            
            if group.muted || !group.to_master {
                continue;
            }

            let fader = group.fader();
            let pan = group.pan();
            
            // Pan Law
            let pan_rad = (pan + 1.0) * std::f32::consts::FRAC_PI_4;
            let left_gain = pan_rad.cos() * fader;
            let right_gain = pan_rad.sin() * fader;

            left += buffer[0] * left_gain;
            right += buffer[1] * right_gain;
        }

        (left, right)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_bus_manager_creation() {
        let manager = BusManager::new(8, 4);
        assert_eq!(manager.aux_buses.len(), 8);
        assert_eq!(manager.group_buses.len(), 4);
    }

    #[test]
    fn test_aux_send() {
        let send = AuxSend::new(0);
        send.set_level(0.5);
        assert_eq!(send.level(), 0.5);
    }

    #[test]
    fn test_group_assignment() {
        let mut manager = BusManager::new(8, 4);
        manager.init_channels(8);
        
        // Kanal 0 zu Gruppe 0 zuweisen
        manager.assign_channel_to_group(0, 0);
        
        let assignments = manager.get_group_assignments(0).unwrap();
        assert!(assignments[0]);
        assert!(!assignments[1]);
    }
    
    #[test]
    fn test_aux_bus_state() {
        let bus = AuxBus::new(0, "Test Aux");
        bus.set_level(0.8);
        
        let state = bus.state();
        assert_eq!(state.id, 0);
        assert_eq!(state.name, "Test Aux");
        assert!((state.level - 0.8).abs() < 0.001);
    }
    
    #[test]
    fn test_group_bus_state() {
        let bus = GroupBus::new(0, "Test Group");
        bus.set_fader(0.9);
        
        let state = bus.state();
        assert_eq!(state.id, 0);
        assert_eq!(state.name, "Test Group");
        assert!((state.fader - 0.9).abs() < 0.001);
    }
}
