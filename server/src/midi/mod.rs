//! MIDI Controller Modul
//! 
//! MIDI Input/Output und Controller-Mapping

mod controller;
mod feedback;

pub use controller::MidiController;

// Export für zukünftige Verwendung
#[allow(unused_imports)]
pub use feedback::{MidiOutputManager, MidiFeedback};
