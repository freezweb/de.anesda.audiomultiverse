//! Audio-Engine Modul
//! 
//! Audio-Verarbeitung und Device-Management

mod engine;
pub mod eq;
pub mod metering;

#[cfg(test)]
mod metering_tests;

pub use engine::{AudioEngine, AudioCommandSender, AudioCommand};

// EQ-Typen
pub use eq::{ParametricEq, EqBand, EqBandParams, FilterType, HighPassFilter};

// Metering-Typen
pub use metering::{ChannelMeter, StereoMeter, LufsMeter, MeterState, StereoMeterState, LufsMeterState};
