//! Unit Tests für das Metering-System
//!
//! Testet Peak, RMS und Korrelations-Messung

use super::metering::{ChannelMeter, StereoMeter, linear_to_db};

#[test]
fn test_channel_meter_creation() {
    let meter = ChannelMeter::new(48000);
    
    assert!((meter.peak() - 0.0).abs() < 0.001);
    assert!((meter.rms() - 0.0).abs() < 0.001);
    assert!(!meter.is_clipped());
}

#[test]
fn test_peak_detection() {
    let mut meter = ChannelMeter::new(48000);
    
    // Stille verarbeiten
    for _ in 0..100 {
        meter.process(0.0);
    }
    assert!((meter.peak() - 0.0).abs() < 0.01);
    
    // Signal mit bekanntem Peak
    meter.process(0.8);
    assert!((meter.peak() - 0.8).abs() < 0.01);
}

#[test]
fn test_clipping_detection() {
    let mut meter = ChannelMeter::new(48000);
    
    // Normales Signal
    for _ in 0..100 {
        meter.process(0.5);
    }
    assert!(!meter.is_clipped());
    
    // Clipping Signal (>= 1.0)
    meter.process(1.0);
    assert!(meter.is_clipped());
}

#[test]
fn test_rms_calculation() {
    let mut meter = ChannelMeter::new(48000);
    
    // Konstantes Signal = RMS sollte annähernd gleich sein
    for _ in 0..4800 {
        meter.process(0.5);
    }
    
    // RMS sollte nahe am Wert sein
    let rms = meter.rms();
    assert!(rms > 0.4 && rms < 0.6);
}

#[test]
fn test_peak_db() {
    let mut meter = ChannelMeter::new(48000);
    
    // 0.5 linear ≈ -6dB
    meter.process(0.5);
    let db = meter.peak_db();
    assert!(db > -7.0 && db < -5.0);
}

#[test]
fn test_clip_reset() {
    let mut meter = ChannelMeter::new(48000);
    
    // Clipping auslösen
    meter.process(1.0);
    assert!(meter.is_clipped());
    
    // Reset
    meter.reset_clip();
    assert!(!meter.is_clipped());
}

#[test]
fn test_stereo_meter_creation() {
    let meter = StereoMeter::new(48000);
    
    assert!((meter.left.peak() - 0.0).abs() < 0.001);
    assert!((meter.right.peak() - 0.0).abs() < 0.001);
}

#[test]
fn test_stereo_correlation_mono() {
    let mut meter = StereoMeter::new(48000);
    
    // Identische Signale = hohe Korrelation (Mono-Signal)
    for i in 0..4800 {
        let sample = (i as f32 / 100.0).sin() * 0.5;
        meter.process(sample, sample);
    }
    
    let corr = meter.correlation();
    // Sollte positiv sein für mono-kompatibles Material
    assert!(corr >= 0.0);
}

#[test]
fn test_linear_to_db() {
    // 1.0 = 0dB
    let db = linear_to_db(1.0);
    assert!((db - 0.0).abs() < 0.01);
    
    // 0.5 ≈ -6dB
    let db = linear_to_db(0.5);
    assert!(db > -7.0 && db < -5.0);
    
    // 0 = -inf
    let db = linear_to_db(0.0);
    assert!(db < -90.0);
}
