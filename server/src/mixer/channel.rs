//! Einzelner Mixer-Kanal
//! 
//! Repräsentiert einen Eingangskanal mit allen Parametern

use audiomultiverse_protocol::ChannelState;
use std::sync::atomic::{AtomicU32, Ordering};

/// Ein Mixer-Kanal
pub struct Channel {
    /// Kanal-ID (0-basiert)
    pub id: u32,
    
    /// Kanalname
    name: String,
    
    /// Fader-Position (0.0 = -∞, 1.0 = 0dB, 1.25 = +10dB)
    fader: f32,
    
    /// Mute-Status
    mute: bool,
    
    /// Solo-Status
    solo: bool,
    
    /// Pan (-1.0 = L, 0.0 = C, 1.0 = R)
    pan: f32,
    
    /// Gain/Trim in dB (-20 bis +20)
    gain: f32,
    
    /// Phase invertiert
    phase_invert: bool,
    
    /// Kanal-Farbe (RGB Hex)
    color: String,
    
    /// Aktueller Peak-Meter Wert (0.0 - 1.0)
    /// Atomic für lock-free Audio-Thread Updates
    meter_peak: AtomicU32,
    
    /// Peak-Hold Wert
    meter_peak_hold: AtomicU32,
}

impl Channel {
    /// Neuen Kanal erstellen
    pub fn new(id: u32, name: String) -> Self {
        Self {
            id,
            name,
            fader: 0.75,      // ca. -6dB
            mute: false,
            solo: false,
            pan: 0.0,
            gain: 0.0,
            phase_invert: false,
            color: "#3B82F6".to_string(), // Blau
            meter_peak: AtomicU32::new(0),
            meter_peak_hold: AtomicU32::new(0),
        }
    }

    /// Kanal-State für API/UI
    pub fn state(&self) -> ChannelState {
        ChannelState {
            id: self.id,
            name: self.name.clone(),
            fader: self.fader,
            mute: self.mute,
            solo: self.solo,
            pan: self.pan,
            gain: self.gain,
            phase_invert: self.phase_invert,
            color: self.color.clone(),
            meter: self.get_meter(),
        }
    }

    // === Setter ===

    pub fn set_fader(&mut self, value: f32) {
        self.fader = value.clamp(0.0, 1.25); // bis +10dB
    }

    pub fn set_mute(&mut self, muted: bool) {
        self.mute = muted;
    }

    pub fn set_solo(&mut self, solo: bool) {
        self.solo = solo;
    }

    pub fn set_pan(&mut self, pan: f32) {
        self.pan = pan.clamp(-1.0, 1.0);
    }

    pub fn set_gain(&mut self, gain: f32) {
        self.gain = gain.clamp(-20.0, 20.0);
    }

    pub fn set_phase_invert(&mut self, invert: bool) {
        self.phase_invert = invert;
    }

    pub fn set_name(&mut self, name: String) {
        self.name = name;
    }

    pub fn set_color(&mut self, color: String) {
        self.color = color;
    }

    // === Metering ===

    /// Aktuellen Peak-Wert abrufen
    pub fn get_meter(&self) -> f32 {
        f32::from_bits(self.meter_peak.load(Ordering::Relaxed))
    }

    /// Peak-Wert vom Audio-Thread setzen (lock-free)
    pub fn update_meter(&self, peak: f32) {
        let current = self.get_meter();
        
        // Attack: Sofort übernehmen wenn höher
        // Release: Langsam abfallen
        let new_peak = if peak > current {
            peak
        } else {
            current * 0.95 // ~50ms Release bei 20 Updates/s
        };
        
        self.meter_peak.store(new_peak.to_bits(), Ordering::Relaxed);
        
        // Peak Hold aktualisieren
        let hold = f32::from_bits(self.meter_peak_hold.load(Ordering::Relaxed));
        if peak > hold {
            self.meter_peak_hold.store(peak.to_bits(), Ordering::Relaxed);
        }
    }

    /// Effektive Gain berechnen (für Audio-Processing)
    pub fn effective_gain(&self) -> f32 {
        if self.mute {
            return 0.0;
        }
        
        // Fader zu dB konvertieren
        // 0.0 = -inf, 0.75 = 0dB, 1.0 = 0dB, 1.25 = +10dB
        let fader_db = if self.fader < 0.001 {
            f32::NEG_INFINITY
        } else if self.fader <= 0.75 {
            // -inf bis 0dB
            20.0 * (self.fader / 0.75).log10()
        } else {
            // 0dB bis +10dB
            ((self.fader - 0.75) / 0.25) * 10.0
        };
        
        // dB zu linearer Gain
        let fader_linear = 10.0_f32.powf(fader_db / 20.0);
        let gain_linear = 10.0_f32.powf(self.gain / 20.0);
        
        // Phase Invert
        let phase = if self.phase_invert { -1.0 } else { 1.0 };
        
        fader_linear * gain_linear * phase
    }

    /// Stereo-Gains für Links/Rechts (Pan berücksichtigt)
    pub fn stereo_gains(&self) -> (f32, f32) {
        let gain = self.effective_gain();
        
        // Constant Power Pan Law
        // Bei Center: L = R = 0.707 (-3dB)
        // Bei Full L: L = 1.0, R = 0.0
        let pan_rad = (self.pan + 1.0) * std::f32::consts::FRAC_PI_4; // 0 bis π/2
        let left = pan_rad.cos() * gain;
        let right = pan_rad.sin() * gain;
        
        (left, right)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_channel_defaults() {
        let ch = Channel::new(0, "Test".to_string());
        assert_eq!(ch.fader, 0.75);
        assert!(!ch.mute);
        assert!(!ch.solo);
        assert_eq!(ch.pan, 0.0);
    }

    #[test]
    fn test_effective_gain() {
        let mut ch = Channel::new(0, "Test".to_string());
        
        // Fader auf Unity (0dB)
        ch.set_fader(0.75);
        ch.set_gain(0.0);
        let gain = ch.effective_gain();
        assert!((gain - 1.0).abs() < 0.01);
        
        // Muted
        ch.set_mute(true);
        assert_eq!(ch.effective_gain(), 0.0);
    }

    #[test]
    fn test_stereo_pan() {
        let mut ch = Channel::new(0, "Test".to_string());
        ch.set_fader(0.75);
        ch.set_gain(0.0);
        
        // Center
        ch.set_pan(0.0);
        let (l, r) = ch.stereo_gains();
        assert!((l - r).abs() < 0.01); // Links = Rechts
        
        // Hard Left
        ch.set_pan(-1.0);
        let (l, r) = ch.stereo_gains();
        assert!(l > 0.9);
        assert!(r < 0.1);
    }
}
