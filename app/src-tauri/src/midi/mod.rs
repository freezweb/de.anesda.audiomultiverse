//! Client-seitige MIDI-Verarbeitung
//! 
//! MIDI-Controller werden am Client angeschlossen und die Eingaben
//! werden über WebSocket an den Server weitergeleitet.

mod controller;
mod feedback;

pub use controller::{ClientMidiController, MixerCommand, MidiEvent};
pub use feedback::ClientMidiFeedback;
