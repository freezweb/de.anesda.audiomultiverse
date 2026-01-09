//! Audio-Engine Modul
//! 
//! Audio-Verarbeitung und Device-Management

mod engine;
pub mod eq;

pub use engine::{AudioEngine, AudioDeviceInfo};
pub use eq::{ParametricEq, EqBand, EqBandParams, FilterType, HighPassFilter};
