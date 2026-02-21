//! Erweitertes Metering-System
//! 
//! Implementiert:
//! - Peak Metering (mit Hold)
//! - RMS Metering (Root Mean Square)
//! - LUFS Metering (Loudness Units Full Scale)
//! - Correlation Meter (Stereo)
//! - Clipping Detection

#![allow(dead_code)]

use std::sync::atomic::{AtomicU32, AtomicBool, Ordering};
use serde::{Deserialize, Serialize};

/// Meter-Zeitkonstanten
pub struct MeterConstants {
    /// Peak Hold Zeit in Samples
    pub peak_hold_samples: usize,
    /// Peak Fall-off Rate pro Sample (in dB)
    pub peak_falloff: f32,
    /// RMS Integrationszeit in Samples
    pub rms_window: usize,
    /// LUFS Integration Window (400ms = Momentary)
    pub lufs_window: usize,
}

impl Default for MeterConstants {
    fn default() -> Self {
        let sample_rate = 48000;
        Self {
            peak_hold_samples: sample_rate * 2, // 2 Sekunden Hold
            peak_falloff: 0.0002, // ~10dB/s bei 48kHz
            rms_window: sample_rate / 10, // 100ms RMS
            lufs_window: (sample_rate as f32 * 0.4) as usize, // 400ms LUFS
        }
    }
}

/// Einzelkanal-Meter
pub struct ChannelMeter {
    /// Aktueller Peak-Wert (linear)
    peak: AtomicU32,
    /// Peak-Hold-Wert (für Anzeige)
    peak_hold: AtomicU32,
    /// Peak-Hold-Counter
    hold_counter: AtomicU32,
    /// RMS-Wert (linear)
    rms: AtomicU32,
    /// RMS Ringbuffer
    rms_buffer: Vec<f32>,
    rms_index: usize,
    rms_sum: f32,
    /// Clipping erkannt
    clipped: AtomicBool,
    /// Clipping-Counter (wie oft geclippt)
    clip_count: AtomicU32,
    /// Sample Rate für Berechnungen
    sample_rate: u32,
}

impl ChannelMeter {
    pub fn new(sample_rate: u32) -> Self {
        let rms_window = sample_rate as usize / 10; // 100ms
        Self {
            peak: AtomicU32::new(0),
            peak_hold: AtomicU32::new(0),
            hold_counter: AtomicU32::new(0),
            rms: AtomicU32::new(0),
            rms_buffer: vec![0.0; rms_window],
            rms_index: 0,
            rms_sum: 0.0,
            clipped: AtomicBool::new(false),
            clip_count: AtomicU32::new(0),
            sample_rate,
        }
    }

    /// Sample verarbeiten (wird im Audio-Thread aufgerufen)
    pub fn process(&mut self, sample: f32) {
        let abs_sample = sample.abs();
        
        // Peak Detection
        let current_peak = f32::from_bits(self.peak.load(Ordering::Relaxed));
        if abs_sample > current_peak {
            self.peak.store(abs_sample.to_bits(), Ordering::Relaxed);
            self.peak_hold.store(abs_sample.to_bits(), Ordering::Relaxed);
            self.hold_counter.store(self.sample_rate * 2, Ordering::Relaxed); // 2s Hold
        } else {
            // Peak Decay
            let decay = current_peak * 0.9999; // Langsamer Decay
            self.peak.store(decay.to_bits(), Ordering::Relaxed);
            
            // Hold Counter
            let hold = self.hold_counter.load(Ordering::Relaxed);
            if hold > 0 {
                self.hold_counter.store(hold - 1, Ordering::Relaxed);
            } else {
                // Peak Hold decay
                let hold_peak = f32::from_bits(self.peak_hold.load(Ordering::Relaxed));
                self.peak_hold.store((hold_peak * 0.9995).to_bits(), Ordering::Relaxed);
            }
        }
        
        // Clipping Detection (> 0dBFS)
        if abs_sample >= 1.0 {
            self.clipped.store(true, Ordering::Relaxed);
            self.clip_count.fetch_add(1, Ordering::Relaxed);
        }
        
        // RMS Berechnung (Ringbuffer)
        let old_sample = self.rms_buffer[self.rms_index];
        self.rms_sum -= old_sample * old_sample;
        self.rms_sum += sample * sample;
        self.rms_buffer[self.rms_index] = sample;
        self.rms_index = (self.rms_index + 1) % self.rms_buffer.len();
        
        let rms_value = (self.rms_sum / self.rms_buffer.len() as f32).sqrt();
        self.rms.store(rms_value.to_bits(), Ordering::Relaxed);
    }

    /// Peak-Wert abrufen (linear, 0.0-1.0+)
    pub fn peak(&self) -> f32 {
        f32::from_bits(self.peak.load(Ordering::Relaxed))
    }

    /// Peak-Hold-Wert abrufen
    pub fn peak_hold(&self) -> f32 {
        f32::from_bits(self.peak_hold.load(Ordering::Relaxed))
    }

    /// RMS-Wert abrufen (linear)
    pub fn rms(&self) -> f32 {
        f32::from_bits(self.rms.load(Ordering::Relaxed))
    }

    /// Peak in dB
    pub fn peak_db(&self) -> f32 {
        linear_to_db(self.peak())
    }

    /// RMS in dB
    pub fn rms_db(&self) -> f32 {
        linear_to_db(self.rms())
    }

    /// Clipping Status
    pub fn is_clipped(&self) -> bool {
        self.clipped.load(Ordering::Relaxed)
    }

    /// Clipping-Anzeige zurücksetzen
    pub fn reset_clip(&self) {
        self.clipped.store(false, Ordering::Relaxed);
    }

    /// Clipping-Counter abrufen
    pub fn clip_count(&self) -> u32 {
        self.clip_count.load(Ordering::Relaxed)
    }

    /// Alle Werte zurücksetzen
    pub fn reset(&mut self) {
        self.peak.store(0, Ordering::Relaxed);
        self.peak_hold.store(0, Ordering::Relaxed);
        self.hold_counter.store(0, Ordering::Relaxed);
        self.rms.store(0, Ordering::Relaxed);
        self.rms_buffer.fill(0.0);
        self.rms_sum = 0.0;
        self.rms_index = 0;
        self.clipped.store(false, Ordering::Relaxed);
        self.clip_count.store(0, Ordering::Relaxed);
    }
}

/// Stereo Meter (für Master, Gruppen, Aux)
pub struct StereoMeter {
    pub left: ChannelMeter,
    pub right: ChannelMeter,
    /// Correlation (-1.0 = Out of Phase, 0.0 = Uncorrelated, 1.0 = Mono)
    correlation: AtomicU32,
    /// Correlation Berechnung
    correlation_buffer_l: Vec<f32>,
    correlation_buffer_r: Vec<f32>,
    correlation_index: usize,
}

impl StereoMeter {
    pub fn new(sample_rate: u32) -> Self {
        let window = sample_rate as usize / 10; // 100ms
        Self {
            left: ChannelMeter::new(sample_rate),
            right: ChannelMeter::new(sample_rate),
            correlation: AtomicU32::new(0.0_f32.to_bits()),
            correlation_buffer_l: vec![0.0; window],
            correlation_buffer_r: vec![0.0; window],
            correlation_index: 0,
        }
    }

    /// Stereo-Samples verarbeiten
    pub fn process(&mut self, left: f32, right: f32) {
        self.left.process(left);
        self.right.process(right);
        
        // Correlation berechnen
        let idx = self.correlation_index;
        self.correlation_buffer_l[idx] = left;
        self.correlation_buffer_r[idx] = right;
        self.correlation_index = (idx + 1) % self.correlation_buffer_l.len();
        
        // Pearson Correlation berechnen (vereinfacht)
        if self.correlation_index == 0 {
            let corr = calculate_correlation(&self.correlation_buffer_l, &self.correlation_buffer_r);
            self.correlation.store(corr.to_bits(), Ordering::Relaxed);
        }
    }

    /// Correlation-Wert abrufen
    pub fn correlation(&self) -> f32 {
        f32::from_bits(self.correlation.load(Ordering::Relaxed))
    }
}

/// LUFS Meter (ITU-R BS.1770)
pub struct LufsMeter {
    /// Momentary Loudness (400ms Integration)
    momentary: AtomicU32,
    /// Short-term Loudness (3s Integration)
    short_term: AtomicU32,
    /// Integrated Loudness (gesamte Messung)
    integrated: AtomicU32,
    /// True Peak
    true_peak: AtomicU32,
    /// K-Weighting Filter State (vereinfacht)
    k_state: KWeightingState,
    /// Momentary Buffer (400ms bei 48kHz)
    momentary_buffer: Vec<f32>,
    momentary_index: usize,
    /// Short-term Buffer (3s)
    short_term_buffer: Vec<f32>,
    short_term_index: usize,
    /// Integrated Samples
    integrated_samples: Vec<f32>,
    sample_rate: u32,
}

/// K-Weighting Filter State
struct KWeightingState {
    // High Shelf (+4dB @ 1681Hz)
    hs_z1: f32,
    hs_z2: f32,
    // High Pass (cutoff ~38Hz)
    hp_z1: f32,
    hp_z2: f32,
}

impl Default for KWeightingState {
    fn default() -> Self {
        Self {
            hs_z1: 0.0,
            hs_z2: 0.0,
            hp_z1: 0.0,
            hp_z2: 0.0,
        }
    }
}

impl LufsMeter {
    pub fn new(sample_rate: u32) -> Self {
        let momentary_samples = (sample_rate as f32 * 0.4) as usize; // 400ms
        let short_term_samples = sample_rate as usize * 3; // 3s
        
        Self {
            momentary: AtomicU32::new((-70.0_f32).to_bits()),
            short_term: AtomicU32::new((-70.0_f32).to_bits()),
            integrated: AtomicU32::new((-70.0_f32).to_bits()),
            true_peak: AtomicU32::new(0),
            k_state: KWeightingState::default(),
            momentary_buffer: vec![0.0; momentary_samples],
            momentary_index: 0,
            short_term_buffer: vec![0.0; short_term_samples],
            short_term_index: 0,
            integrated_samples: Vec::with_capacity(sample_rate as usize * 60), // 1 Minute
            sample_rate,
        }
    }

    /// Stereo-Samples verarbeiten (Left + Right gemittelt)
    pub fn process(&mut self, left: f32, right: f32) {
        // K-Weighting Filter (vereinfacht - echte Implementierung ist komplexer)
        let sum = (left + right) * 0.5;
        let weighted = self.apply_k_weighting(sum);
        let squared = weighted * weighted;
        
        // True Peak (ohne Oversampling - vereinfacht)
        let peak = left.abs().max(right.abs());
        let current_peak = f32::from_bits(self.true_peak.load(Ordering::Relaxed));
        if peak > current_peak {
            self.true_peak.store(peak.to_bits(), Ordering::Relaxed);
        }
        
        // Momentary Buffer
        self.momentary_buffer[self.momentary_index] = squared;
        self.momentary_index = (self.momentary_index + 1) % self.momentary_buffer.len();
        
        // Short-term Buffer
        self.short_term_buffer[self.short_term_index] = squared;
        self.short_term_index = (self.short_term_index + 1) % self.short_term_buffer.len();
        
        // Berechne LUFS wenn Buffer voll
        if self.momentary_index == 0 {
            let momentary_sum: f32 = self.momentary_buffer.iter().sum();
            let momentary_mean = momentary_sum / self.momentary_buffer.len() as f32;
            let momentary_lufs = if momentary_mean > 0.0 {
                -0.691 + 10.0 * momentary_mean.log10()
            } else {
                -70.0
            };
            self.momentary.store(momentary_lufs.to_bits(), Ordering::Relaxed);
        }
        
        if self.short_term_index == 0 {
            let short_term_sum: f32 = self.short_term_buffer.iter().sum();
            let short_term_mean = short_term_sum / self.short_term_buffer.len() as f32;
            let short_term_lufs = if short_term_mean > 0.0 {
                -0.691 + 10.0 * short_term_mean.log10()
            } else {
                -70.0
            };
            self.short_term.store(short_term_lufs.to_bits(), Ordering::Relaxed);
        }
    }

    /// Vereinfachte K-Weighting (echte Implementierung braucht Biquad Filter)
    fn apply_k_weighting(&mut self, sample: f32) -> f32 {
        // Vereinfachte High-Shelf Simulation (+4dB oberhalb 1681Hz)
        let hs_out = sample + (sample - self.k_state.hs_z1) * 0.3;
        self.k_state.hs_z1 = sample;
        
        // Vereinfachter High-Pass (removes <38Hz)
        let hp_out = hs_out - self.k_state.hp_z1;
        self.k_state.hp_z1 = hs_out * 0.995 + self.k_state.hp_z1 * 0.005;
        
        hp_out
    }

    /// Momentary LUFS
    pub fn momentary(&self) -> f32 {
        f32::from_bits(self.momentary.load(Ordering::Relaxed))
    }

    /// Short-term LUFS
    pub fn short_term(&self) -> f32 {
        f32::from_bits(self.short_term.load(Ordering::Relaxed))
    }

    /// Integrated LUFS
    pub fn integrated(&self) -> f32 {
        f32::from_bits(self.integrated.load(Ordering::Relaxed))
    }

    /// True Peak (linear)
    pub fn true_peak(&self) -> f32 {
        f32::from_bits(self.true_peak.load(Ordering::Relaxed))
    }

    /// True Peak in dBTP
    pub fn true_peak_db(&self) -> f32 {
        linear_to_db(self.true_peak())
    }

    /// Messung zurücksetzen
    pub fn reset(&mut self) {
        self.momentary.store((-70.0_f32).to_bits(), Ordering::Relaxed);
        self.short_term.store((-70.0_f32).to_bits(), Ordering::Relaxed);
        self.integrated.store((-70.0_f32).to_bits(), Ordering::Relaxed);
        self.true_peak.store(0, Ordering::Relaxed);
        self.momentary_buffer.fill(0.0);
        self.short_term_buffer.fill(0.0);
        self.integrated_samples.clear();
        self.k_state = KWeightingState::default();
    }
}

/// Meter State für API/WebSocket
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MeterState {
    pub channel_id: u32,
    pub peak: f32,
    pub peak_db: f32,
    pub peak_hold: f32,
    pub rms: f32,
    pub rms_db: f32,
    pub clipped: bool,
}

/// Stereo Meter State für API/WebSocket
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StereoMeterState {
    pub left: MeterState,
    pub right: MeterState,
    pub correlation: f32,
}

/// LUFS Meter State für API/WebSocket
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LufsMeterState {
    pub momentary: f32,
    pub short_term: f32,
    pub integrated: f32,
    pub true_peak: f32,
    pub true_peak_db: f32,
}

/// Hilfsfunktion: Linear zu dB
pub fn linear_to_db(value: f32) -> f32 {
    if value < 0.000001 {
        -120.0
    } else {
        20.0 * value.log10()
    }
}

/// Hilfsfunktion: Correlation berechnen
fn calculate_correlation(a: &[f32], b: &[f32]) -> f32 {
    if a.len() != b.len() || a.is_empty() {
        return 0.0;
    }
    
    let n = a.len() as f32;
    let sum_a: f32 = a.iter().sum();
    let sum_b: f32 = b.iter().sum();
    let sum_ab: f32 = a.iter().zip(b.iter()).map(|(x, y)| x * y).sum();
    let sum_a2: f32 = a.iter().map(|x| x * x).sum();
    let sum_b2: f32 = b.iter().map(|x| x * x).sum();
    
    let numerator = n * sum_ab - sum_a * sum_b;
    let denominator = ((n * sum_a2 - sum_a * sum_a) * (n * sum_b2 - sum_b * sum_b)).sqrt();
    
    if denominator.abs() < 0.0001 {
        0.0
    } else {
        (numerator / denominator).clamp(-1.0, 1.0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_channel_meter() {
        let mut meter = ChannelMeter::new(48000);
        
        // Stille
        for _ in 0..1000 {
            meter.process(0.0);
        }
        assert!(meter.peak() < 0.01);
        
        // Laut
        for _ in 0..1000 {
            meter.process(0.5);
        }
        assert!(meter.peak() > 0.4);
    }

    #[test]
    fn test_clipping_detection() {
        let mut meter = ChannelMeter::new(48000);
        
        meter.process(0.5);
        assert!(!meter.is_clipped());
        
        meter.process(1.0);
        assert!(meter.is_clipped());
    }

    #[test]
    fn test_db_conversion() {
        assert!((linear_to_db(1.0)).abs() < 0.01); // 0 dB
        assert!((linear_to_db(0.5) - (-6.02)).abs() < 0.1); // ~-6 dB
        assert!(linear_to_db(0.0) < -100.0); // -inf
    }
}
