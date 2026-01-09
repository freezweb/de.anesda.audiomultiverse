//! MIDI Controller Modul
//! 
//! MIDI Input/Output und Controller-Mapping

mod controller;
mod feedback;

pub use controller::MidiController;
pub use feedback::{MidiOutputManager, MidiFeedback};
