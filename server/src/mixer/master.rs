//! Master-Sektion
//!
//! Stereo Master Bus mit Limiter, DIM, Mono-Check

use std::sync::atomic::{AtomicBool, AtomicU32, Ordering};
use serde::{Deserialize, Serialize};
use tracing::info;

/// Master-Sektion State
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MasterState {
    /// Master-Fader (0.0 - 1.0)
    pub fader: f32,
    
    /// Master Mute
    pub mute: bool,
    
    /// DIM aktiviert (-20dB)
    pub dim: bool,
    
    /// DIM Absenkung in dB
    pub dim_level: f32,
    
    /// Mono-Summe aktiviert
    pub mono: bool,
    
    /// Limiter aktiviert
    pub limiter_enabled: bool,
    
    /// Limiter Threshold in dB
    pub limiter_threshold: f32,
    
    /// Limiter Ratio (z.B. 20:1)
    pub limiter_ratio: f32,
    
    /// Talkback aktiviert
    pub talkback: bool,
    
    /// Oszillator aktiviert
    pub oscillator: bool,
    
    /// Oszillator Frequenz (Hz)
    pub oscillator_freq: f32,
    
    /// Oszillator Pegel in dB
    pub oscillator_level: f32,
    
    /// Peak Level Links
    pub peak_left: f32,
    
    /// Peak Level Rechts
    pub peak_right: f32,
    
    /// Limiter GR (Gain Reduction) in dB
    pub limiter_gr: f32,
}

impl Default for MasterState {
    fn default() -> Self {
        Self {
            fader: 0.75,  // -6dB
            mute: false,
            dim: false,
            dim_level: -20.0,
            mono: false,
            limiter_enabled: true,
            limiter_threshold: -0.5,
            limiter_ratio: 20.0,
            talkback: false,
            oscillator: false,
            oscillator_freq: 1000.0,
            oscillator_level: -20.0,
            peak_left: -60.0,
            peak_right: -60.0,
            limiter_gr: 0.0,
        }
    }
}

/// Master-Sektion
pub struct MasterSection {
    /// Fader-Wert (IEEE 754 als u32 gespeichert für Atomics)
    fader_bits: AtomicU32,
    
    /// Mute
    mute: AtomicBool,
    
    /// DIM
    dim: AtomicBool,
    
    /// DIM Level in dB (als Bits)
    dim_level_bits: AtomicU32,
    
    /// Mono
    mono: AtomicBool,
    
    /// Limiter enabled
    limiter_enabled: AtomicBool,
    
    /// Limiter Threshold (als Bits)
    limiter_threshold_bits: AtomicU32,
    
    /// Limiter Ratio (als Bits)
    limiter_ratio_bits: AtomicU32,
    
    /// Talkback
    talkback: AtomicBool,
    
    /// Oszillator enabled
    oscillator: AtomicBool,
    
    /// Oszillator Frequenz (als Bits)
    oscillator_freq_bits: AtomicU32,
    
    /// Oszillator Level (als Bits)
    oscillator_level_bits: AtomicU32,
    
    /// Peak Level Links
    peak_left_bits: AtomicU32,
    
    /// Peak Level Rechts
    peak_right_bits: AtomicU32,
    
    /// Limiter Gain Reduction
    limiter_gr_bits: AtomicU32,
}

impl MasterSection {
    /// Neue Master-Sektion erstellen
    pub fn new() -> Self {
        let default = MasterState::default();
        
        Self {
            fader_bits: AtomicU32::new(default.fader.to_bits()),
            mute: AtomicBool::new(default.mute),
            dim: AtomicBool::new(default.dim),
            dim_level_bits: AtomicU32::new(default.dim_level.to_bits()),
            mono: AtomicBool::new(default.mono),
            limiter_enabled: AtomicBool::new(default.limiter_enabled),
            limiter_threshold_bits: AtomicU32::new(default.limiter_threshold.to_bits()),
            limiter_ratio_bits: AtomicU32::new(default.limiter_ratio.to_bits()),
            talkback: AtomicBool::new(default.talkback),
            oscillator: AtomicBool::new(default.oscillator),
            oscillator_freq_bits: AtomicU32::new(default.oscillator_freq.to_bits()),
            oscillator_level_bits: AtomicU32::new(default.oscillator_level.to_bits()),
            peak_left_bits: AtomicU32::new(default.peak_left.to_bits()),
            peak_right_bits: AtomicU32::new(default.peak_right.to_bits()),
            limiter_gr_bits: AtomicU32::new(0.0f32.to_bits()),
        }
    }
    
    /// State abrufen
    pub fn get_state(&self) -> MasterState {
        MasterState {
            fader: f32::from_bits(self.fader_bits.load(Ordering::Relaxed)),
            mute: self.mute.load(Ordering::Relaxed),
            dim: self.dim.load(Ordering::Relaxed),
            dim_level: f32::from_bits(self.dim_level_bits.load(Ordering::Relaxed)),
            mono: self.mono.load(Ordering::Relaxed),
            limiter_enabled: self.limiter_enabled.load(Ordering::Relaxed),
            limiter_threshold: f32::from_bits(self.limiter_threshold_bits.load(Ordering::Relaxed)),
            limiter_ratio: f32::from_bits(self.limiter_ratio_bits.load(Ordering::Relaxed)),
            talkback: self.talkback.load(Ordering::Relaxed),
            oscillator: self.oscillator.load(Ordering::Relaxed),
            oscillator_freq: f32::from_bits(self.oscillator_freq_bits.load(Ordering::Relaxed)),
            oscillator_level: f32::from_bits(self.oscillator_level_bits.load(Ordering::Relaxed)),
            peak_left: f32::from_bits(self.peak_left_bits.load(Ordering::Relaxed)),
            peak_right: f32::from_bits(self.peak_right_bits.load(Ordering::Relaxed)),
            limiter_gr: f32::from_bits(self.limiter_gr_bits.load(Ordering::Relaxed)),
        }
    }
    
    // === Setter Methoden ===
    
    /// Fader setzen
    pub fn set_fader(&self, value: f32) -> MasterState {
        let clamped = value.clamp(0.0, 1.0);
        self.fader_bits.store(clamped.to_bits(), Ordering::Relaxed);
        self.get_state()
    }
    
    /// Mute setzen
    pub fn set_mute(&self, muted: bool) -> MasterState {
        self.mute.store(muted, Ordering::Relaxed);
        self.get_state()
    }
    
    /// DIM setzen
    pub fn set_dim(&self, dim: bool) -> MasterState {
        self.dim.store(dim, Ordering::Relaxed);
        info!("Master DIM: {}", if dim { "ON" } else { "OFF" });
        self.get_state()
    }
    
    /// DIM Level setzen
    pub fn set_dim_level(&self, level_db: f32) -> MasterState {
        let clamped = level_db.clamp(-40.0, 0.0);
        self.dim_level_bits.store(clamped.to_bits(), Ordering::Relaxed);
        self.get_state()
    }
    
    /// Mono setzen
    pub fn set_mono(&self, mono: bool) -> MasterState {
        self.mono.store(mono, Ordering::Relaxed);
        info!("Master MONO: {}", if mono { "ON" } else { "OFF" });
        self.get_state()
    }
    
    /// Limiter aktivieren/deaktivieren
    pub fn set_limiter_enabled(&self, enabled: bool) -> MasterState {
        self.limiter_enabled.store(enabled, Ordering::Relaxed);
        self.get_state()
    }
    
    /// Limiter Threshold setzen
    pub fn set_limiter_threshold(&self, threshold_db: f32) -> MasterState {
        let clamped = threshold_db.clamp(-20.0, 0.0);
        self.limiter_threshold_bits.store(clamped.to_bits(), Ordering::Relaxed);
        self.get_state()
    }
    
    /// Limiter Ratio setzen
    pub fn set_limiter_ratio(&self, ratio: f32) -> MasterState {
        let clamped = ratio.clamp(1.0, 100.0);
        self.limiter_ratio_bits.store(clamped.to_bits(), Ordering::Relaxed);
        self.get_state()
    }
    
    /// Talkback setzen
    pub fn set_talkback(&self, talkback: bool) -> MasterState {
        self.talkback.store(talkback, Ordering::Relaxed);
        if talkback {
            info!("🎙️ Talkback ON");
        }
        self.get_state()
    }
    
    /// Oszillator setzen
    pub fn set_oscillator(&self, enabled: bool) -> MasterState {
        self.oscillator.store(enabled, Ordering::Relaxed);
        if enabled {
            let freq = f32::from_bits(self.oscillator_freq_bits.load(Ordering::Relaxed));
            info!("🔊 Oszillator ON ({} Hz)", freq);
        }
        self.get_state()
    }
    
    /// Oszillator Frequenz setzen
    pub fn set_oscillator_freq(&self, freq_hz: f32) -> MasterState {
        let clamped = freq_hz.clamp(20.0, 20000.0);
        self.oscillator_freq_bits.store(clamped.to_bits(), Ordering::Relaxed);
        self.get_state()
    }
    
    /// Oszillator Level setzen
    pub fn set_oscillator_level(&self, level_db: f32) -> MasterState {
        let clamped = level_db.clamp(-60.0, 0.0);
        self.oscillator_level_bits.store(clamped.to_bits(), Ordering::Relaxed);
        self.get_state()
    }
    
    // === Audio Processing Methoden ===
    
    /// Peak Level aktualisieren (vom Audio-Thread)
    pub fn update_peaks(&self, left: f32, right: f32) {
        self.peak_left_bits.store(left.to_bits(), Ordering::Relaxed);
        self.peak_right_bits.store(right.to_bits(), Ordering::Relaxed);
    }
    
    /// Limiter GR aktualisieren (vom Audio-Thread)
    pub fn update_limiter_gr(&self, gr_db: f32) {
        self.limiter_gr_bits.store(gr_db.to_bits(), Ordering::Relaxed);
    }
    
    /// Effektives Gain berechnen (für Audio-Thread)
    pub fn get_effective_gain(&self) -> f32 {
        if self.mute.load(Ordering::Relaxed) {
            return 0.0;
        }
        
        let fader = f32::from_bits(self.fader_bits.load(Ordering::Relaxed));
        let mut gain = fader_to_db(fader);
        
        if self.dim.load(Ordering::Relaxed) {
            gain += f32::from_bits(self.dim_level_bits.load(Ordering::Relaxed));
        }
        
        db_to_linear(gain)
    }
    
    /// Oszillator Sample generieren
    pub fn generate_oscillator_sample(&self, phase: &mut f32, sample_rate: f32) -> (f32, f32) {
        if !self.oscillator.load(Ordering::Relaxed) {
            return (0.0, 0.0);
        }
        
        let freq = f32::from_bits(self.oscillator_freq_bits.load(Ordering::Relaxed));
        let level_db = f32::from_bits(self.oscillator_level_bits.load(Ordering::Relaxed));
        let amplitude = db_to_linear(level_db);
        
        // Sinus-Oszillator
        let sample = (*phase * std::f32::consts::TAU).sin() * amplitude;
        
        // Phase inkrementieren
        *phase += freq / sample_rate;
        if *phase >= 1.0 {
            *phase -= 1.0;
        }
        
        (sample, sample)
    }
    
    /// Mono-Summe anwenden
    pub fn apply_mono(&self, left: f32, right: f32) -> (f32, f32) {
        if self.mono.load(Ordering::Relaxed) {
            let mono = (left + right) * 0.5;
            (mono, mono)
        } else {
            (left, right)
        }
    }
    
    /// Einfacher Limiter (Soft Clip)
    pub fn apply_limiter(&self, left: f32, right: f32) -> (f32, f32) {
        if !self.limiter_enabled.load(Ordering::Relaxed) {
            return (left, right);
        }
        
        let threshold_db = f32::from_bits(self.limiter_threshold_bits.load(Ordering::Relaxed));
        let threshold = db_to_linear(threshold_db);
        let ratio = f32::from_bits(self.limiter_ratio_bits.load(Ordering::Relaxed));
        
        let limit = |sample: f32| -> f32 {
            let abs_sample = sample.abs();
            if abs_sample > threshold {
                let over = abs_sample - threshold;
                let compressed = threshold + over / ratio;
                compressed.copysign(sample)
            } else {
                sample
            }
        };
        
        let out_left = limit(left);
        let out_right = limit(right);
        
        // GR berechnen (vereinfacht)
        let max_in = left.abs().max(right.abs());
        let max_out = out_left.abs().max(out_right.abs());
        if max_in > 0.0 && max_in > max_out {
            let gr = 20.0 * (max_out / max_in).log10();
            self.update_limiter_gr(gr);
        } else {
            self.update_limiter_gr(0.0);
        }
        
        (out_left, out_right)
    }
    
    /// Komplette Master-Verarbeitung
    pub fn process(&self, left: f32, right: f32, osc_phase: &mut f32, sample_rate: f32) -> (f32, f32) {
        // Oszillator hinzumischen (wenn aktiv)
        let (osc_l, osc_r) = self.generate_oscillator_sample(osc_phase, sample_rate);
        let mut out_l = left + osc_l;
        let mut out_r = right + osc_r;
        
        // Mono-Summe
        let (mono_l, mono_r) = self.apply_mono(out_l, out_r);
        out_l = mono_l;
        out_r = mono_r;
        
        // Master Gain
        let gain = self.get_effective_gain();
        out_l *= gain;
        out_r *= gain;
        
        // Limiter
        let (lim_l, lim_r) = self.apply_limiter(out_l, out_r);
        
        // Peaks aktualisieren
        self.update_peaks(lim_l, lim_r);
        
        (lim_l, lim_r)
    }
}

impl Default for MasterSection {
    fn default() -> Self {
        Self::new()
    }
}

// === Helper Funktionen ===

/// Fader-Wert (0-1) zu dB
fn fader_to_db(fader: f32) -> f32 {
    if fader <= 0.0 {
        -100.0 // -∞
    } else {
        // Logarithmische Kurve: 0.75 = 0dB, 1.0 = +10dB
        20.0 * (fader / 0.75).log10()
    }
}

/// dB zu linearem Gain
fn db_to_linear(db: f32) -> f32 {
    if db <= -100.0 {
        0.0
    } else {
        10.0_f32.powf(db / 20.0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_master_creation() {
        let master = MasterSection::new();
        let state = master.get_state();
        
        assert!(!state.mute);
        assert!(!state.dim);
        assert!(state.limiter_enabled);
    }
    
    #[test]
    fn test_fader() {
        let master = MasterSection::new();
        
        let state = master.set_fader(0.5);
        assert!((state.fader - 0.5).abs() < 0.001);
    }
    
    #[test]
    fn test_dim() {
        let master = MasterSection::new();
        
        let state = master.set_dim(true);
        assert!(state.dim);
        
        let gain = master.get_effective_gain();
        // Mit DIM sollte der Gain niedriger sein
        assert!(gain < 1.0);
    }
    
    #[test]
    fn test_mono() {
        let master = MasterSection::new();
        master.set_mono(true);
        
        let (l, r) = master.apply_mono(1.0, 0.0);
        assert!((l - r).abs() < 0.001);
        assert!((l - 0.5).abs() < 0.001);
    }
    
    #[test]
    fn test_limiter() {
        let master = MasterSection::new();
        master.set_limiter_threshold(-6.0);
        
        let (out_l, out_r) = master.apply_limiter(2.0, 2.0);
        // Sollte limitiert sein
        assert!(out_l < 2.0);
        assert!(out_r < 2.0);
    }
    
    #[test]
    fn test_db_conversions() {
        assert!((db_to_linear(0.0) - 1.0).abs() < 0.001);
        assert!((db_to_linear(-6.0) - 0.5012).abs() < 0.01);
        assert!((db_to_linear(-100.0)).abs() < 0.001);
    }
}
