//! Netzwerk-Audio Modul (AES67/DANTE)
//! 
//! Abstraktion für verschiedene Audio-Netzwerk-Backends
//!
//! ## Modules
//! - `ptp` - PTP (IEEE 1588) Clock Synchronization (Linux only)
//! - `rtp` - RTP Audio Streaming (L24 format)
//! - `sap` - SAP/SDP Stream Discovery
//! - `backend` - High-level backend abstraction
//!
//! Note: Full AES67 support (PTP synchronization) requires Linux.
//! On Windows, a limited implementation without hardware timestamping is used.

// Common modules (all platforms)
pub mod rtp;
pub mod sap;
mod backend;

// PTP requires statime which is Linux-only
#[cfg(target_os = "linux")]
pub mod ptp;

// Re-exports
pub use backend::{AudioNetworkBackend, Aes67Backend, Aes67Config, NetworkDevice, NetworkDeviceType};
#[cfg(target_os = "linux")]
pub use ptp::{PtpClock, PtpState, PtpStats};
pub use rtp::{RtpSender, RtpReceiver, Aes67Format};
pub use sap::{SapDiscovery, Aes67Stream, StreamDirection};

// Stub types for non-Linux platforms
#[cfg(not(target_os = "linux"))]
pub mod ptp_stub {
    use std::sync::atomic::{AtomicBool, Ordering};
    use std::sync::Arc;
    
    /// PTP Clock state
    #[derive(Debug, Clone, Copy, PartialEq)]
    pub enum PtpState {
        Initializing,
        Listening,
        Slave,
        Master,
        Holdover,
    }
    
    /// PTP Clock statistics
    #[derive(Debug, Clone, Default)]
    pub struct PtpStats {
        pub offset_ns: i64,
        pub path_delay_ns: i64,
        pub sync_count: u64,
        pub clock_accuracy_ns: u64,
        pub steps_removed: u16,
    }
    
    /// Stub PTP Clock for non-Linux platforms
    /// Uses system clock without network synchronization
    pub struct PtpClock {
        running: Arc<AtomicBool>,
    }
    
    impl PtpClock {
        pub fn new(_interface: &str) -> Self {
            Self {
                running: Arc::new(AtomicBool::new(false)),
            }
        }
        
        pub fn set_domain(&mut self, _domain: u8) {}
        
        pub fn state(&self) -> PtpState {
            if self.running.load(Ordering::Relaxed) {
                PtpState::Master // Pretend to be master on Windows
            } else {
                PtpState::Initializing
            }
        }
        
        pub fn offset_ns(&self) -> i64 {
            0 // No offset tracking on Windows
        }
        
        pub fn stats(&self) -> PtpStats {
            PtpStats::default()
        }
        
        pub fn is_synchronized(&self) -> bool {
            self.running.load(Ordering::Relaxed)
        }
        
        pub fn start(&self) -> anyhow::Result<()> {
            self.running.store(true, Ordering::Relaxed);
            tracing::warn!("PTP not fully supported on Windows - using system clock");
            Ok(())
        }
        
        pub fn stop(&self) {
            self.running.store(false, Ordering::Relaxed);
        }
        
        pub fn media_timestamp(&self) -> u32 {
            use std::time::{SystemTime, UNIX_EPOCH};
            let now = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap_or_default();
            // 48kHz sample clock
            ((now.as_nanos() / 20833) as u32) & 0xFFFFFFFF
        }
    }
}

#[cfg(not(target_os = "linux"))]
pub use ptp_stub::{PtpClock, PtpState, PtpStats};
