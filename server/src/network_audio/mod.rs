//! Netzwerk-Audio Modul (AES67/DANTE)
//! 
//! Abstraktion für verschiedene Audio-Netzwerk-Backends

mod backend;

pub use backend::{AudioNetworkBackend, Aes67Backend};
