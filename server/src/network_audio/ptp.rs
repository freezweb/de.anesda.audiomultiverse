//! PTP (IEEE 1588) Clock Synchronization for AES67
//!
//! Uses statime for Precision Time Protocol implementation.
//! AES67 requires PTP for sample-accurate synchronization across devices.

use std::net::{IpAddr, Ipv4Addr, SocketAddr, UdpSocket};
use std::sync::atomic::{AtomicBool, AtomicI64, Ordering};
use std::sync::Arc;
use std::time::{Duration, Instant};
use anyhow::{Result, anyhow};
use parking_lot::RwLock;
use tracing::{info, warn, error, debug};

/// PTP Clock state
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum PtpState {
    /// Not started
    Initializing,
    /// Listening for master clock
    Listening,
    /// Synchronized to master
    Slave,
    /// Acting as master clock
    Master,
    /// Lost sync
    Holdover,
}

/// PTP Clock statistics
#[derive(Debug, Clone, Default)]
pub struct PtpStats {
    /// Offset from master in nanoseconds
    pub offset_ns: i64,
    /// Path delay in nanoseconds
    pub path_delay_ns: i64,
    /// Number of sync messages received
    pub sync_count: u64,
    /// Clock accuracy estimate
    pub clock_accuracy_ns: u64,
    /// Steps removed from grandmaster
    pub steps_removed: u16,
}

/// PTP Clock for AES67 synchronization
pub struct PtpClock {
    /// Current state
    state: Arc<RwLock<PtpState>>,
    /// Running flag
    running: Arc<AtomicBool>,
    /// Current offset from master (nanoseconds)
    offset_ns: Arc<AtomicI64>,
    /// Network interface to use
    interface: String,
    /// PTP domain (default 0 for AES67)
    domain: u8,
    /// Statistics
    stats: Arc<RwLock<PtpStats>>,
    /// Last sync time
    last_sync: Arc<RwLock<Option<Instant>>>,
}

impl PtpClock {
    /// Create a new PTP clock
    pub fn new(interface: &str) -> Self {
        Self {
            state: Arc::new(RwLock::new(PtpState::Initializing)),
            running: Arc::new(AtomicBool::new(false)),
            offset_ns: Arc::new(AtomicI64::new(0)),
            interface: interface.to_string(),
            domain: 0, // AES67 default domain
            stats: Arc::new(RwLock::new(PtpStats::default())),
            last_sync: Arc::new(RwLock::new(None)),
        }
    }

    /// Set PTP domain (0-127)
    pub fn set_domain(&mut self, domain: u8) {
        self.domain = domain.min(127);
    }

    /// Get current state
    pub fn state(&self) -> PtpState {
        *self.state.read()
    }

    /// Get current offset from master in nanoseconds
    pub fn offset_ns(&self) -> i64 {
        self.offset_ns.load(Ordering::Relaxed)
    }

    /// Get statistics
    pub fn stats(&self) -> PtpStats {
        self.stats.read().clone()
    }

    /// Check if synchronized
    pub fn is_synchronized(&self) -> bool {
        matches!(self.state(), PtpState::Slave | PtpState::Master)
    }

    /// Start the PTP clock
    pub fn start(&self) -> Result<()> {
        if self.running.load(Ordering::Relaxed) {
            return Ok(());
        }

        info!("⏱️  PTP Clock starting on interface: {}", self.interface);
        info!("    Domain: {}", self.domain);

        self.running.store(true, Ordering::Relaxed);
        *self.state.write() = PtpState::Listening;

        // Clone for the thread
        let running = self.running.clone();
        let state = self.state.clone();
        let offset_ns = self.offset_ns.clone();
        let stats = self.stats.clone();
        let last_sync = self.last_sync.clone();
        let domain = self.domain;

        // Start PTP thread
        std::thread::spawn(move || {
            if let Err(e) = run_ptp_loop(running, state, offset_ns, stats, last_sync, domain) {
                error!("PTP loop error: {}", e);
            }
        });

        Ok(())
    }

    /// Stop the PTP clock
    pub fn stop(&self) {
        info!("⏱️  PTP Clock stopping");
        self.running.store(false, Ordering::Relaxed);
        *self.state.write() = PtpState::Initializing;
    }

    /// Get media clock timestamp for RTP
    /// Returns the current media clock in 48kHz samples since epoch
    pub fn media_timestamp(&self) -> u32 {
        // For AES67: 48kHz sample clock
        // RTP timestamp = (system_time + offset) * 48000 / 1_000_000_000
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default();
        
        let offset = self.offset_ns.load(Ordering::Relaxed);
        let corrected_ns = now.as_nanos() as i128 + offset as i128;
        
        // Convert to 48kHz samples (wrap at 32 bits)
        ((corrected_ns * 48000 / 1_000_000_000) & 0xFFFFFFFF) as u32
    }
}

/// PTP Message types
#[derive(Debug, Clone, Copy)]
#[repr(u8)]
enum PtpMessageType {
    Sync = 0x0,
    DelayReq = 0x1,
    FollowUp = 0x8,
    DelayResp = 0x9,
    Announce = 0xB,
}

/// Run the PTP synchronization loop
fn run_ptp_loop(
    running: Arc<AtomicBool>,
    state: Arc<RwLock<PtpState>>,
    offset_ns: Arc<AtomicI64>,
    stats: Arc<RwLock<PtpStats>>,
    last_sync: Arc<RwLock<Option<Instant>>>,
    domain: u8,
) -> Result<()> {
    // PTP uses two ports:
    // - 319 (event port) for Sync, Delay_Req, Pdelay_Req, Pdelay_Resp
    // - 320 (general port) for Announce, Follow_Up, Delay_Resp, etc.
    
    // Multicast addresses for PTP
    let ptp_primary_multicast: Ipv4Addr = "224.0.1.129".parse()?;
    let ptp_pdelay_multicast: Ipv4Addr = "224.0.0.107".parse()?;
    
    // Create event socket (port 319)
    let event_socket = create_ptp_socket(319, ptp_primary_multicast)?;
    event_socket.set_read_timeout(Some(Duration::from_millis(100)))?;
    
    // Create general socket (port 320)  
    let general_socket = create_ptp_socket(320, ptp_primary_multicast)?;
    general_socket.set_read_timeout(Some(Duration::from_millis(100)))?;

    info!("📡 PTP sockets bound, listening for sync messages...");

    let mut buf = [0u8; 1024];
    let mut sync_sequence: u16 = 0;
    let mut t1: Option<i64> = None; // Sync timestamp from master
    let mut t2: Option<i64> = None; // Local time when Sync received
    
    while running.load(Ordering::Relaxed) {
        // Listen for Sync messages on event port
        match event_socket.recv_from(&mut buf) {
            Ok((len, src)) => {
                if len >= 34 {
                    let msg_type = buf[0] & 0x0F;
                    let msg_domain = buf[4];
                    
                    if msg_domain != domain {
                        continue;
                    }
                    
                    match msg_type {
                        0x0 => { // Sync
                            t2 = Some(get_system_time_ns());
                            sync_sequence = u16::from_be_bytes([buf[30], buf[31]]);
                            
                            // Two-step clock: timestamp in Follow_Up
                            // One-step clock: timestamp in Sync message
                            let two_step = (buf[0] & 0x02) != 0;
                            
                            if !two_step {
                                // One-step: extract timestamp from Sync
                                t1 = Some(extract_timestamp(&buf[34..44]));
                                process_sync(t1, t2, &offset_ns, &stats, &last_sync, &state);
                            }
                            
                            debug!("Received Sync seq={} from {}", sync_sequence, src);
                        }
                        0x1 => { // Delay_Req - we send these
                            debug!("Received Delay_Req from {}", src);
                        }
                        _ => {}
                    }
                }
            }
            Err(ref e) if e.kind() == std::io::ErrorKind::WouldBlock => {
                // Timeout, check if still running
            }
            Err(e) => {
                warn!("Event socket error: {}", e);
            }
        }
        
        // Listen for Follow_Up and Delay_Resp on general port
        match general_socket.recv_from(&mut buf) {
            Ok((len, src)) => {
                if len >= 34 {
                    let msg_type = buf[0] & 0x0F;
                    let msg_domain = buf[4];
                    
                    if msg_domain != domain {
                        continue;
                    }
                    
                    match msg_type {
                        0x8 => { // Follow_Up
                            let seq = u16::from_be_bytes([buf[30], buf[31]]);
                            if seq == sync_sequence {
                                t1 = Some(extract_timestamp(&buf[34..44]));
                                process_sync(t1, t2, &offset_ns, &stats, &last_sync, &state);
                            }
                            debug!("Received Follow_Up seq={} from {}", seq, src);
                        }
                        0xB => { // Announce
                            debug!("Received Announce from {}", src);
                            // Could extract grandmaster info here
                        }
                        _ => {}
                    }
                }
            }
            Err(ref e) if e.kind() == std::io::ErrorKind::WouldBlock => {}
            Err(e) => {
                warn!("General socket error: {}", e);
            }
        }
        
        // Check for sync timeout (holdover if no sync for 3 seconds)
        if let Some(last) = *last_sync.read() {
            if last.elapsed() > Duration::from_secs(3) {
                if *state.read() == PtpState::Slave {
                    warn!("⚠️  PTP sync lost, entering holdover");
                    *state.write() = PtpState::Holdover;
                }
            }
        }
    }
    
    info!("PTP loop stopped");
    Ok(())
}

/// Create a PTP multicast socket
fn create_ptp_socket(port: u16, multicast_addr: Ipv4Addr) -> Result<UdpSocket> {
    use socket2::{Socket, Domain, Type, Protocol};
    
    let socket = Socket::new(Domain::IPV4, Type::DGRAM, Some(Protocol::UDP))?;
    
    // Allow address reuse
    socket.set_reuse_address(true)?;
    
    #[cfg(unix)]
    socket.set_reuse_port(true)?;
    
    // Bind to any interface on the PTP port
    let addr = SocketAddr::new(IpAddr::V4(Ipv4Addr::UNSPECIFIED), port);
    socket.bind(&addr.into())?;
    
    // Join multicast group
    socket.join_multicast_v4(&multicast_addr, &Ipv4Addr::UNSPECIFIED)?;
    
    // Set multicast TTL
    socket.set_multicast_ttl_v4(64)?;
    
    Ok(socket.into())
}

/// Get current system time in nanoseconds since epoch
fn get_system_time_ns() -> i64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_nanos() as i64)
        .unwrap_or(0)
}

/// Extract PTP timestamp from message (48-bit seconds + 32-bit nanoseconds)
fn extract_timestamp(data: &[u8]) -> i64 {
    if data.len() < 10 {
        return 0;
    }
    
    // 48-bit seconds (6 bytes)
    let seconds = u64::from_be_bytes([0, 0, data[0], data[1], data[2], data[3], data[4], data[5]]);
    
    // 32-bit nanoseconds (4 bytes)
    let nanos = u32::from_be_bytes([data[6], data[7], data[8], data[9]]);
    
    (seconds as i64 * 1_000_000_000) + nanos as i64
}

/// Process sync and update offset
fn process_sync(
    t1: Option<i64>,
    t2: Option<i64>,
    offset_ns: &Arc<AtomicI64>,
    stats: &Arc<RwLock<PtpStats>>,
    last_sync: &Arc<RwLock<Option<Instant>>>,
    state: &Arc<RwLock<PtpState>>,
) {
    if let (Some(master_time), Some(local_time)) = (t1, t2) {
        // Simple offset calculation (without delay compensation for now)
        // offset = t1 - t2 (how much our clock is behind master)
        let offset = master_time - local_time;
        
        // Apply low-pass filter to smooth offset
        let current = offset_ns.load(Ordering::Relaxed);
        let filtered = if current == 0 {
            offset
        } else {
            // Exponential moving average
            (current * 7 + offset) / 8
        };
        
        offset_ns.store(filtered, Ordering::Relaxed);
        
        // Update stats
        {
            let mut s = stats.write();
            s.offset_ns = filtered;
            s.sync_count += 1;
            s.clock_accuracy_ns = (offset - current).unsigned_abs() as u64;
        }
        
        // Update last sync time
        *last_sync.write() = Some(Instant::now());
        
        // Transition to Slave state if not already
        if *state.read() != PtpState::Slave {
            info!("✅ PTP synchronized to master (offset: {} ns)", filtered);
            *state.write() = PtpState::Slave;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_extract_timestamp() {
        // Test timestamp: 1704067200 seconds, 500000000 nanoseconds
        let data: [u8; 10] = [
            0x00, 0x00, 0x65, 0x8D, 0x1E, 0x00, // seconds (1704067200)
            0x1D, 0xCD, 0x65, 0x00,             // nanoseconds (500000000)
        ];
        
        let ts = extract_timestamp(&data);
        let expected = 1704067200_i64 * 1_000_000_000 + 500000000;
        assert_eq!(ts, expected);
    }
    
    #[test]
    fn test_ptp_clock_creation() {
        let clock = PtpClock::new("eth0");
        assert_eq!(clock.state(), PtpState::Initializing);
        assert_eq!(clock.offset_ns(), 0);
        assert!(!clock.is_synchronized());
    }
}
