//! Mixer-Kernmodul
//! 
//! Verwaltet alle Kanäle, Routing und Audio-State

mod channel;
mod routing;
pub mod scenes;
pub mod master;

pub use channel::Channel;
pub use audiomultiverse_protocol::ChannelState;
pub use routing::RoutingMatrix;
pub use scenes::{Scene, SceneManager, SceneMetadata, RecallFilter};
pub use master::{MasterSection, MasterState};

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
}

impl Mixer {
    /// Neuen Mixer erstellen
    pub fn new(input_count: usize, output_count: usize) -> Self {
        let channels: Vec<Channel> = (0..input_count)
            .map(|i| Channel::new(i as u32, format!("CH {}", i + 1)))
            .collect();

        let routing = RoutingMatrix::new(input_count, output_count);

        Self {
            input_count,
            output_count,
            channels: RwLock::new(channels),
            routing: RwLock::new(routing),
            solo_active: RwLock::new(vec![]),
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
