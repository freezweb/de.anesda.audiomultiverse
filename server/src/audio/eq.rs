//! EQ-Modul (Equalizer)
//!
//! Parametrischer 4-Band EQ mit verschiedenen Filtertypen

use std::f32::consts::PI;
use serde::{Deserialize, Serialize};

/// Filter-Typ
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize, Default)]
pub enum FilterType {
    /// Parametrischer Peak/Notch
    #[default]
    Peak,
    /// Low Shelf
    LowShelf,
    /// High Shelf
    HighShelf,
    /// Low Pass
    LowPass,
    /// High Pass
    HighPass,
    /// Band Pass
    BandPass,
    /// Notch Filter
    Notch,
}

/// Ein EQ-Band Parameter
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EqBandParams {
    /// Frequenz in Hz
    pub frequency: f32,
    
    /// Gain in dB (-15 bis +15)
    pub gain: f32,
    
    /// Q-Faktor (0.1 bis 10)
    pub q: f32,
    
    /// Filter-Typ
    pub filter_type: FilterType,
    
    /// Band aktiviert
    pub enabled: bool,
}

impl Default for EqBandParams {
    fn default() -> Self {
        Self {
            frequency: 1000.0,
            gain: 0.0,
            q: 1.0,
            filter_type: FilterType::Peak,
            enabled: true,
        }
    }
}

/// Biquad Koeffizienten
#[derive(Debug, Clone, Copy, Default)]
struct BiquadCoeffs {
    b0: f32,
    b1: f32,
    b2: f32,
    a1: f32,
    a2: f32,
}

/// Biquad Filter State
#[derive(Debug, Clone, Default)]
struct BiquadState {
    x1: f32,
    x2: f32,
    y1: f32,
    y2: f32,
}

/// Ein EQ-Band (Filter)
#[derive(Debug, Clone)]
pub struct EqBand {
    /// Parameter
    params: EqBandParams,
    
    /// Koeffizienten
    coeffs: BiquadCoeffs,
    
    /// Zustand für linken Kanal
    state_l: BiquadState,
    
    /// Zustand für rechten Kanal
    state_r: BiquadState,
    
    /// Sample Rate
    sample_rate: f32,
}

impl EqBand {
    /// Neues EQ-Band erstellen
    pub fn new(params: EqBandParams, sample_rate: f32) -> Self {
        let mut band = Self {
            params: params.clone(),
            coeffs: BiquadCoeffs::default(),
            state_l: BiquadState::default(),
            state_r: BiquadState::default(),
            sample_rate,
        };
        band.update_coefficients();
        band
    }
    
    /// Koeffizienten neu berechnen
    pub fn update_coefficients(&mut self) {
        let f0 = self.params.frequency;
        let q = self.params.q.max(0.1);
        let gain_db = self.params.gain;
        let fs = self.sample_rate;
        
        // Normalisierte Frequenz
        let omega = 2.0 * PI * f0 / fs;
        let sin_omega = omega.sin();
        let cos_omega = omega.cos();
        let alpha = sin_omega / (2.0 * q);
        
        // A für Shelving und Peak
        let a = 10.0_f32.powf(gain_db / 40.0);
        
        let (b0, b1, b2, a0, a1, a2) = match self.params.filter_type {
            FilterType::Peak => {
                let alpha_a = alpha * a;
                let alpha_div_a = alpha / a;
                (
                    1.0 + alpha_a,
                    -2.0 * cos_omega,
                    1.0 - alpha_a,
                    1.0 + alpha_div_a,
                    -2.0 * cos_omega,
                    1.0 - alpha_div_a,
                )
            }
            FilterType::LowShelf => {
                let ap1 = a + 1.0;
                let am1 = a - 1.0;
                let two_sqrt_a_alpha = 2.0 * a.sqrt() * alpha;
                (
                    a * ((ap1) - am1 * cos_omega + two_sqrt_a_alpha),
                    2.0 * a * (am1 - ap1 * cos_omega),
                    a * (ap1 - am1 * cos_omega - two_sqrt_a_alpha),
                    ap1 + am1 * cos_omega + two_sqrt_a_alpha,
                    -2.0 * (am1 + ap1 * cos_omega),
                    ap1 + am1 * cos_omega - two_sqrt_a_alpha,
                )
            }
            FilterType::HighShelf => {
                let ap1 = a + 1.0;
                let am1 = a - 1.0;
                let two_sqrt_a_alpha = 2.0 * a.sqrt() * alpha;
                (
                    a * (ap1 + am1 * cos_omega + two_sqrt_a_alpha),
                    -2.0 * a * (am1 + ap1 * cos_omega),
                    a * (ap1 + am1 * cos_omega - two_sqrt_a_alpha),
                    ap1 - am1 * cos_omega + two_sqrt_a_alpha,
                    2.0 * (am1 - ap1 * cos_omega),
                    ap1 - am1 * cos_omega - two_sqrt_a_alpha,
                )
            }
            FilterType::LowPass => (
                (1.0 - cos_omega) / 2.0,
                1.0 - cos_omega,
                (1.0 - cos_omega) / 2.0,
                1.0 + alpha,
                -2.0 * cos_omega,
                1.0 - alpha,
            ),
            FilterType::HighPass => (
                (1.0 + cos_omega) / 2.0,
                -(1.0 + cos_omega),
                (1.0 + cos_omega) / 2.0,
                1.0 + alpha,
                -2.0 * cos_omega,
                1.0 - alpha,
            ),
            FilterType::BandPass => (
                alpha,
                0.0,
                -alpha,
                1.0 + alpha,
                -2.0 * cos_omega,
                1.0 - alpha,
            ),
            FilterType::Notch => (
                1.0,
                -2.0 * cos_omega,
                1.0,
                1.0 + alpha,
                -2.0 * cos_omega,
                1.0 - alpha,
            ),
        };
        
        // Normalisieren
        self.coeffs = BiquadCoeffs {
            b0: b0 / a0,
            b1: b1 / a0,
            b2: b2 / a0,
            a1: a1 / a0,
            a2: a2 / a0,
        };
    }
    
    /// Parameter setzen und Koeffizienten aktualisieren
    pub fn set_params(&mut self, params: EqBandParams) {
        self.params = params;
        self.update_coefficients();
    }
    
    /// Frequenz setzen
    pub fn set_frequency(&mut self, freq: f32) {
        self.params.frequency = freq.clamp(20.0, 20000.0);
        self.update_coefficients();
    }
    
    /// Gain setzen
    pub fn set_gain(&mut self, gain_db: f32) {
        self.params.gain = gain_db.clamp(-15.0, 15.0);
        self.update_coefficients();
    }
    
    /// Q setzen
    pub fn set_q(&mut self, q: f32) {
        self.params.q = q.clamp(0.1, 10.0);
        self.update_coefficients();
    }
    
    /// Filter-Typ setzen
    pub fn set_filter_type(&mut self, filter_type: FilterType) {
        self.params.filter_type = filter_type;
        self.update_coefficients();
    }
    
    /// Band aktivieren/deaktivieren
    pub fn set_enabled(&mut self, enabled: bool) {
        self.params.enabled = enabled;
    }
    
    /// Ein Sample filtern (Mono)
    fn process_sample(sample: f32, coeffs: &BiquadCoeffs, state: &mut BiquadState) -> f32 {
        let output = coeffs.b0 * sample 
            + coeffs.b1 * state.x1 
            + coeffs.b2 * state.x2
            - coeffs.a1 * state.y1 
            - coeffs.a2 * state.y2;
        
        state.x2 = state.x1;
        state.x1 = sample;
        state.y2 = state.y1;
        state.y1 = output;
        
        output
    }
    
    /// Stereo-Sample filtern
    pub fn process(&mut self, left: f32, right: f32) -> (f32, f32) {
        if !self.params.enabled {
            return (left, right);
        }
        
        let out_l = Self::process_sample(left, &self.coeffs, &mut self.state_l);
        let out_r = Self::process_sample(right, &self.coeffs, &mut self.state_r);
        
        (out_l, out_r)
    }
    
    /// Filter-Zustand zurücksetzen
    pub fn reset(&mut self) {
        self.state_l = BiquadState::default();
        self.state_r = BiquadState::default();
    }
    
    /// Parameter abrufen
    pub fn params(&self) -> &EqBandParams {
        &self.params
    }
}

/// 4-Band parametrischer EQ
#[derive(Debug, Clone)]
pub struct ParametricEq {
    /// EQ-Bänder
    bands: Vec<EqBand>,
    
    /// EQ aktiviert
    enabled: bool,
    
    /// Sample Rate
    sample_rate: f32,
}

impl ParametricEq {
    /// Neuen 4-Band EQ erstellen mit Standardwerten
    pub fn new(sample_rate: f32) -> Self {
        let bands = vec![
            EqBand::new(
                EqBandParams {
                    frequency: 80.0,
                    gain: 0.0,
                    q: 0.7,
                    filter_type: FilterType::LowShelf,
                    enabled: true,
                },
                sample_rate,
            ),
            EqBand::new(
                EqBandParams {
                    frequency: 500.0,
                    gain: 0.0,
                    q: 1.0,
                    filter_type: FilterType::Peak,
                    enabled: true,
                },
                sample_rate,
            ),
            EqBand::new(
                EqBandParams {
                    frequency: 2500.0,
                    gain: 0.0,
                    q: 1.0,
                    filter_type: FilterType::Peak,
                    enabled: true,
                },
                sample_rate,
            ),
            EqBand::new(
                EqBandParams {
                    frequency: 12000.0,
                    gain: 0.0,
                    q: 0.7,
                    filter_type: FilterType::HighShelf,
                    enabled: true,
                },
                sample_rate,
            ),
        ];
        
        Self {
            bands,
            enabled: true,
            sample_rate,
        }
    }
    
    /// Stereo-Sample durch alle Bänder verarbeiten
    pub fn process(&mut self, left: f32, right: f32) -> (f32, f32) {
        if !self.enabled {
            return (left, right);
        }
        
        let mut l = left;
        let mut r = right;
        
        for band in &mut self.bands {
            let (out_l, out_r) = band.process(l, r);
            l = out_l;
            r = out_r;
        }
        
        (l, r)
    }
    
    /// Audio-Buffer verarbeiten (interleaved Stereo)
    pub fn process_buffer(&mut self, buffer: &mut [f32]) {
        if !self.enabled {
            return;
        }
        
        for chunk in buffer.chunks_mut(2) {
            if chunk.len() == 2 {
                let (l, r) = self.process(chunk[0], chunk[1]);
                chunk[0] = l;
                chunk[1] = r;
            }
        }
    }
    
    /// Band-Parameter setzen
    pub fn set_band_params(&mut self, band_index: usize, params: EqBandParams) {
        if let Some(band) = self.bands.get_mut(band_index) {
            band.set_params(params);
        }
    }
    
    /// Band-Frequenz setzen
    pub fn set_band_frequency(&mut self, band_index: usize, freq: f32) {
        if let Some(band) = self.bands.get_mut(band_index) {
            band.set_frequency(freq);
        }
    }
    
    /// Band-Gain setzen
    pub fn set_band_gain(&mut self, band_index: usize, gain_db: f32) {
        if let Some(band) = self.bands.get_mut(band_index) {
            band.set_gain(gain_db);
        }
    }
    
    /// Band-Q setzen
    pub fn set_band_q(&mut self, band_index: usize, q: f32) {
        if let Some(band) = self.bands.get_mut(band_index) {
            band.set_q(q);
        }
    }
    
    /// Band aktivieren/deaktivieren
    pub fn set_band_enabled(&mut self, band_index: usize, enabled: bool) {
        if let Some(band) = self.bands.get_mut(band_index) {
            band.set_enabled(enabled);
        }
    }
    
    /// EQ aktivieren/deaktivieren
    pub fn set_enabled(&mut self, enabled: bool) {
        self.enabled = enabled;
    }
    
    /// Ist EQ aktiviert?
    pub fn is_enabled(&self) -> bool {
        self.enabled
    }
    
    /// Alle Bänder zurücksetzen
    pub fn reset(&mut self) {
        for band in &mut self.bands {
            band.reset();
        }
    }
    
    /// Alle Band-Parameter abrufen
    pub fn get_all_params(&self) -> Vec<EqBandParams> {
        self.bands.iter().map(|b| b.params().clone()).collect()
    }
    
    /// Band-Anzahl
    pub fn band_count(&self) -> usize {
        self.bands.len()
    }
    
    /// Frequenzgang berechnen (für UI)
    pub fn get_frequency_response(&self, frequencies: &[f32]) -> Vec<f32> {
        frequencies.iter().map(|&freq| {
            let mut gain_db = 0.0;
            
            for band in &self.bands {
                if band.params.enabled {
                    // Vereinfachte Näherung des Frequenzgangs
                    gain_db += self.calculate_band_response(band, freq);
                }
            }
            
            gain_db
        }).collect()
    }
    
    /// Frequenzgang eines Bands berechnen
    fn calculate_band_response(&self, band: &EqBand, freq: f32) -> f32 {
        let params = band.params();
        
        match params.filter_type {
            FilterType::Peak => {
                // Bell-Kurve Näherung
                let ratio = (freq / params.frequency).ln() / (1.0 / params.q);
                params.gain * (-ratio * ratio).exp()
            }
            FilterType::LowShelf => {
                if freq < params.frequency {
                    params.gain
                } else {
                    let ratio = (params.frequency / freq).ln().abs();
                    params.gain * (-ratio * params.q).exp()
                }
            }
            FilterType::HighShelf => {
                if freq > params.frequency {
                    params.gain
                } else {
                    let ratio = (freq / params.frequency).ln().abs();
                    params.gain * (-ratio * params.q).exp()
                }
            }
            _ => 0.0,
        }
    }
}

/// High-Pass Filter (für Kanal-Strip)
#[derive(Debug, Clone)]
pub struct HighPassFilter {
    /// EQ-Band (als HPF konfiguriert)
    band: EqBand,
    
    /// Aktiviert
    enabled: bool,
}

impl HighPassFilter {
    /// Neuen HPF erstellen
    pub fn new(frequency: f32, sample_rate: f32) -> Self {
        let band = EqBand::new(
            EqBandParams {
                frequency,
                gain: 0.0,
                q: 0.707,  // Butterworth
                filter_type: FilterType::HighPass,
                enabled: true,
            },
            sample_rate,
        );
        
        Self { band, enabled: false }
    }
    
    /// Frequenz setzen (typisch 80Hz oder 120Hz)
    pub fn set_frequency(&mut self, freq: f32) {
        self.band.set_frequency(freq);
    }
    
    /// Aktivieren/Deaktivieren
    pub fn set_enabled(&mut self, enabled: bool) {
        self.enabled = enabled;
    }
    
    /// Sample verarbeiten
    pub fn process(&mut self, left: f32, right: f32) -> (f32, f32) {
        if self.enabled {
            self.band.process(left, right)
        } else {
            (left, right)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_eq_band_creation() {
        let band = EqBand::new(EqBandParams::default(), 48000.0);
        assert!((band.params().frequency - 1000.0).abs() < 0.1);
    }
    
    #[test]
    fn test_eq_processing() {
        let mut eq = ParametricEq::new(48000.0);
        
        // Ohne Gain-Änderungen sollte das Signal unverändert bleiben
        let (out_l, out_r) = eq.process(0.5, 0.5);
        assert!((out_l - 0.5).abs() < 0.01);
        assert!((out_r - 0.5).abs() < 0.01);
    }
    
    #[test]
    fn test_eq_gain() {
        let mut eq = ParametricEq::new(48000.0);
        eq.set_band_gain(1, 6.0);  // +6dB auf Band 2 (500Hz)
        
        // Bei Frequenzen nahe 500Hz sollte das Signal lauter werden
        // (Test mit DC, vereinfacht)
        let (out_l, _) = eq.process(0.1, 0.1);
        // Da DC nicht bei 500Hz ist, Änderung minimal
        assert!((out_l - 0.1).abs() < 0.1);
    }
    
    #[test]
    fn test_hpf() {
        let mut hpf = HighPassFilter::new(80.0, 48000.0);
        hpf.set_enabled(true);
        
        // HPF sollte tiefe Frequenzen dämpfen
        // Mit DC (0 Hz) sollte Ausgabe gegen 0 gehen
        let mut output = 0.0;
        for _ in 0..1000 {
            let (l, _) = hpf.process(0.5, 0.5);
            output = l;
        }
        // Nach Einschwingen sollte DC gefiltert sein
        assert!(output.abs() < 0.01);
    }
    
    #[test]
    fn test_frequency_response() {
        let eq = ParametricEq::new(48000.0);
        
        let freqs: Vec<f32> = vec![100.0, 1000.0, 10000.0];
        let response = eq.get_frequency_response(&freqs);
        
        // Bei neutralem EQ sollte alles ~0dB sein
        for gain in response {
            assert!(gain.abs() < 1.0);
        }
    }
}
