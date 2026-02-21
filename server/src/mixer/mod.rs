//! Mixer-Kernmodul
//! 
//! Verwaltet alle Kanäle, Routing und Audio-State

mod channel;
mod routing;
pub mod scenes;
pub mod master;
pub mod bus;

#[cfg(test)]
mod bus_tests;
#[cfg(test)]
mod scenes_tests;

pub use channel::Channel;
pub use audiomultiverse_protocol::ChannelState;
pub use routing::RoutingMatrix;
pub use scenes::{Scene, SceneManager, SceneMetadata, CrossfadeCurve, CrossfadeState};

// RecallFilter für zukünftige Verwendung
#[allow(unused_imports)]
pub use scenes::RecallFilter;
pub use master::{MasterSection, MasterState};
pub use bus::{BusManager, AuxBus, AuxBusState, AuxSend, AuxSendState, AuxSendMode, GroupBus, GroupBusState};

use std::sync::RwLock;
use audiomultiverse_protocol::{ChannelId, MixerState};

/// Haupt-Mixer Struktur
pub struct Mixer {
    /// Anzahl Eingänge
    pub input_count: usize,
    
    /// Anzahl Ausgänge
    pub output_count: usize,
    
    /// Eingangskanäle
    channels: RwLock<Vec<Channel>>,
    
    /// Routing-Matrix
    routing: RwLock<RoutingMatrix>,
    
    /// Solo-Modus aktiv (welche Kanäle)
    solo_active: RwLock<Vec<ChannelId>>,
    
    /// Bus-Manager (Aux/Groups)
    bus_manager: RwLock<BusManager>,
}

impl Mixer {
    /// Neuen Mixer erstellen
    pub fn new(input_count: usize, output_count: usize) -> Self {
        let channels: Vec<Channel> = (0..input_count)
            .map(|i| Channel::new(i as u32, format!("CH {}", i + 1)))
            .collect();

        let routing = RoutingMatrix::new(input_count, output_count);
        
        // 8 Aux Busse, 4 Groups
        let bus_manager = BusManager::new(8, 4);

        Self {
            input_count,
            output_count,
            channels: RwLock::new(channels),
            routing: RwLock::new(routing),
            solo_active: RwLock::new(vec![]),
            bus_manager: RwLock::new(bus_manager),
        }
    }

    /// Kanal abrufen
    pub fn get_channel(&self, id: ChannelId) -> Option<ChannelState> {
        let channels = self.channels.read().unwrap();
        channels.get(id as usize).map(|c| c.state())
    }

    /// Alle Kanäle abrufen
    pub fn get_all_channels(&self) -> Vec<ChannelState> {
        let channels = self.channels.read().unwrap();
        channels.iter().map(|c| c.state()).collect()
    }

    /// Fader-Wert setzen
    pub fn set_fader(&self, id: ChannelId, value: f32) -> Option<ChannelState> {
        let mut channels = self.channels.write().unwrap();
        if let Some(channel) = channels.get_mut(id as usize) {
            channel.set_fader(value);
            Some(channel.state())
        } else {
            None
        }
    }

    /// Mute setzen
    pub fn set_mute(&self, id: ChannelId, muted: bool) -> Option<ChannelState> {
        let mut channels = self.channels.write().unwrap();
        if let Some(channel) = channels.get_mut(id as usize) {
            channel.set_mute(muted);
            Some(channel.state())
        } else {
            None
        }
    }

    /// Solo setzen
    pub fn set_solo(&self, id: ChannelId, solo: bool) -> Option<ChannelState> {
        let mut channels = self.channels.write().unwrap();
        let mut solo_active = self.solo_active.write().unwrap();

        if let Some(channel) = channels.get_mut(id as usize) {
            channel.set_solo(solo);
            
            if solo {
                if !solo_active.contains(&id) {
                    solo_active.push(id);
                }
            } else {
                solo_active.retain(|&x| x != id);
            }
            
            Some(channel.state())
        } else {
            None
        }
    }

    /// Pan setzen (-1.0 = Links, 0.0 = Mitte, 1.0 = Rechts)
    pub fn set_pan(&self, id: ChannelId, pan: f32) -> Option<ChannelState> {
        let mut channels = self.channels.write().unwrap();
        if let Some(channel) = channels.get_mut(id as usize) {
            channel.set_pan(pan);
            Some(channel.state())
        } else {
            None
        }
    }

    /// Gain/Trim setzen (-20dB bis +20dB)
    pub fn set_gain(&self, id: ChannelId, gain: f32) -> Option<ChannelState> {
        let mut channels = self.channels.write().unwrap();
        if let Some(channel) = channels.get_mut(id as usize) {
            channel.set_gain(gain);
            Some(channel.state())
        } else {
            None
        }
    }

    /// Phase Invert setzen
    pub fn set_phase_invert(&self, id: ChannelId, invert: bool) -> Option<ChannelState> {
        let mut channels = self.channels.write().unwrap();
        if let Some(channel) = channels.get_mut(id as usize) {
            channel.set_phase_invert(invert);
            Some(channel.state())
        } else {
            None
        }
    }

    /// Kanalfarbe setzen
    pub fn set_channel_color(&self, id: ChannelId, color: String) -> Option<ChannelState> {
        let mut channels = self.channels.write().unwrap();
        if let Some(channel) = channels.get_mut(id as usize) {
            channel.set_color(color);
            Some(channel.state())
        } else {
            None
        }
    }

    /// Kanalname setzen
    pub fn set_channel_name(&self, id: ChannelId, name: String) -> Option<ChannelState> {
        let mut channels = self.channels.write().unwrap();
        if let Some(channel) = channels.get_mut(id as usize) {
            channel.set_name(name);
            Some(channel.state())
        } else {
            None
        }
    }

    /// Routing-Punkt setzen
    pub fn set_routing(&self, input: usize, output: usize, gain: f32) -> bool {
        let mut routing = self.routing.write().unwrap();
        routing.set(input, output, gain)
    }

    /// Routing-Matrix abrufen
    pub fn get_routing(&self) -> Vec<Vec<f32>> {
        let routing = self.routing.read().unwrap();
        routing.matrix.clone()
    }

    /// Kompletten Mixer-State abrufen
    pub fn get_state(&self) -> MixerState {
        MixerState {
            channels: self.get_all_channels(),
            routing: self.get_routing(),
            input_count: self.input_count as u32,
            output_count: self.output_count as u32,
        }
    }

    /// Meter-Werte abrufen (Peak pro Kanal)
    pub fn get_meters(&self) -> Vec<f32> {
        let channels = self.channels.read().unwrap();
        channels.iter().map(|c| c.get_meter()).collect()
    }

    /// Meter-Wert für Audio-Thread setzen
    pub fn update_meter(&self, id: ChannelId, peak: f32) {
        let mut channels = self.channels.write().unwrap();
        if let Some(channel) = channels.get_mut(id as usize) {
            channel.update_meter(peak);
        }
    }
    
    // === Bus-System Methoden ===
    
    /// Alle Aux-Busse abrufen
    pub fn get_aux_buses(&self) -> Vec<AuxBusState> {
        let bus_manager = self.bus_manager.read().unwrap();
        bus_manager.aux_buses.iter().map(|b| b.state()).collect()
    }
    
    /// Einzelnen Aux-Bus abrufen
    pub fn get_aux_bus(&self, id: u32) -> Option<AuxBusState> {
        let bus_manager = self.bus_manager.read().unwrap();
        bus_manager.aux_buses.get(id as usize).map(|b| b.state())
    }
    
    /// Aux-Bus aktualisieren
    pub fn update_aux_bus(&self, id: u32, level: Option<f32>, mute: Option<bool>, pan: Option<f32>, name: Option<String>) -> Option<AuxBusState> {
        let mut bus_manager = self.bus_manager.write().unwrap();
        if let Some(bus) = bus_manager.aux_buses.get_mut(id as usize) {
            if let Some(l) = level { bus.set_level(l); }
            if let Some(m) = mute { bus.set_mute(m); }
            if let Some(p) = pan { bus.set_pan(p); }
            if let Some(n) = name { bus.name = n; }
            Some(bus.state())
        } else {
            None
        }
    }
    
    /// Alle Group-Busse abrufen
    pub fn get_group_buses(&self) -> Vec<GroupBusState> {
        let bus_manager = self.bus_manager.read().unwrap();
        bus_manager.group_buses.iter().map(|b| b.state()).collect()
    }
    
    /// Einzelnen Group-Bus abrufen
    pub fn get_group_bus(&self, id: u32) -> Option<GroupBusState> {
        let bus_manager = self.bus_manager.read().unwrap();
        bus_manager.group_buses.get(id as usize).map(|b| b.state())
    }
    
    /// Group-Bus aktualisieren
    pub fn update_group_bus(&self, id: u32, level: Option<f32>, mute: Option<bool>, pan: Option<f32>, name: Option<String>, to_master: Option<bool>) -> Option<GroupBusState> {
        let mut bus_manager = self.bus_manager.write().unwrap();
        if let Some(bus) = bus_manager.group_buses.get_mut(id as usize) {
            if let Some(l) = level { bus.set_level(l); }
            if let Some(m) = mute { bus.set_mute(m); }
            if let Some(p) = pan { bus.set_pan(p); }
            if let Some(n) = name { bus.name = n; }
            if let Some(tm) = to_master { bus.set_to_master(tm); }
            Some(bus.state())
        } else {
            None
        }
    }
    
    /// Aux-Send für einen Kanal abrufen
    pub fn get_channel_aux_sends(&self, channel_id: u32) -> Vec<AuxSendState> {
        let bus_manager = self.bus_manager.read().unwrap();
        bus_manager.get_channel_aux_sends(channel_id)
            .into_iter()
            .map(|s| s.state())
            .collect()
    }
    
    /// Aux-Send aktualisieren
    pub fn update_channel_aux_send(&self, channel_id: u32, aux_id: u32, level: Option<f32>, enabled: Option<bool>, mode: Option<AuxSendMode>, pan: Option<f32>) -> Option<AuxSendState> {
        let mut bus_manager = self.bus_manager.write().unwrap();
        
        // Send abrufen oder erstellen
        let sends = bus_manager.channel_aux_sends.entry(channel_id).or_insert_with(Vec::new);
        
        // Finde oder erstelle den Send
        let send = if let Some(s) = sends.iter_mut().find(|s| s.aux_bus_id() == aux_id) {
            s
        } else {
            let new_send = AuxSend::new(aux_id as u8);
            sends.push(new_send);
            sends.last_mut().unwrap()
        };
        
        if let Some(l) = level { send.set_level(l); }
        if let Some(e) = enabled { send.set_enabled(e); }
        if let Some(m) = mode { send.set_mode(m); }
        if let Some(p) = pan { send.set_pan(p); }
        
        Some(send.state())
    }
    
    /// Kanal einer Group zuweisen
    pub fn assign_channel_to_group(&self, channel_id: u32, group_id: u32) {
        let mut bus_manager = self.bus_manager.write().unwrap();
        bus_manager.assign_channel_to_group(channel_id, group_id);
    }
    
    /// Kanal von Group entfernen
    pub fn remove_channel_from_group(&self, channel_id: u32, group_id: u32) {
        let mut bus_manager = self.bus_manager.write().unwrap();
        bus_manager.remove_channel_from_group(channel_id, group_id);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_mixer_creation() {
        let mixer = Mixer::new(32, 32);
        assert_eq!(mixer.input_count, 32);
        assert_eq!(mixer.output_count, 32);
    }

    #[test]
    fn test_fader_control() {
        let mixer = Mixer::new(8, 2);
        
        // Fader auf -6dB setzen
        let state = mixer.set_fader(0, 0.5).unwrap();
        assert_eq!(state.fader, 0.5);
    }

    #[test]
    fn test_mute_control() {
        let mixer = Mixer::new(8, 2);
        
        let state = mixer.set_mute(0, true).unwrap();
        assert!(state.mute);
        
        let state = mixer.set_mute(0, false).unwrap();
        assert!(!state.mute);
    }
}
