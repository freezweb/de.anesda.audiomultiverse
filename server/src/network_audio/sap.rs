//! SAP/SDP Discovery for AES67
//!
//! Session Announcement Protocol (SAP) is used to announce AES67 streams.
//! Session Description Protocol (SDP) describes the stream parameters.
//!
//! SAP Multicast Address: 224.2.127.254:9875
//! AES67 also uses mDNS for discovery (Ravenna compatible)

use std::net::{IpAddr, Ipv4Addr, SocketAddr, UdpSocket};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::Duration;
use std::collections::HashMap;
use anyhow::{Result, anyhow};
use parking_lot::RwLock;
use tracing::{info, warn, error, debug};

use super::backend::{NetworkDevice, NetworkDeviceType};

/// SAP Multicast address and port
pub const SAP_MULTICAST_ADDR: Ipv4Addr = Ipv4Addr::new(224, 2, 127, 254);
pub const SAP_PORT: u16 = 9875;

/// AES67 mDNS service type
pub const AES67_MDNS_SERVICE: &str = "_aes67._sub._ravenna._udp.local.";
pub const RAVENNA_MDNS_SERVICE: &str = "_ravenna._udp.local.";

/// Discovered AES67 stream
#[derive(Debug, Clone)]
pub struct Aes67Stream {
    /// Stream name
    pub name: String,
    /// Session ID
    pub session_id: String,
    /// Origin (source IP)
    pub origin: String,
    /// Multicast address
    pub multicast_addr: Ipv4Addr,
    /// RTP port
    pub port: u16,
    /// Number of channels
    pub channels: u8,
    /// Sample rate
    pub sample_rate: u32,
    /// Bits per sample (usually 24)
    pub bits_per_sample: u8,
    /// Packet time in microseconds (usually 1000 = 1ms)
    pub ptime_us: u32,
    /// Is this a sender or receiver
    pub direction: StreamDirection,
    /// Raw SDP content
    pub sdp: String,
}

/// Stream direction
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum StreamDirection {
    Send,
    Receive,
    SendReceive,
}

/// SAP/SDP Discovery service
pub struct SapDiscovery {
    /// Discovered streams
    streams: Arc<RwLock<HashMap<String, Aes67Stream>>>,
    /// Running flag
    running: Arc<AtomicBool>,
    /// Our announced streams
    announced: Arc<RwLock<Vec<Aes67Stream>>>,
}

impl SapDiscovery {
    /// Create a new SAP discovery service
    pub fn new() -> Self {
        Self {
            streams: Arc::new(RwLock::new(HashMap::new())),
            running: Arc::new(AtomicBool::new(false)),
            announced: Arc::new(RwLock::new(Vec::new())),
        }
    }

    /// Get discovered streams
    pub fn streams(&self) -> Vec<Aes67Stream> {
        self.streams.read().values().cloned().collect()
    }

    /// Convert streams to NetworkDevice format
    pub fn as_devices(&self) -> Vec<NetworkDevice> {
        self.streams.read().values().map(|s| {
            NetworkDevice {
                id: s.session_id.clone(),
                name: s.name.clone(),
                device_type: match s.direction {
                    StreamDirection::Send => NetworkDeviceType::Transmitter,
                    StreamDirection::Receive => NetworkDeviceType::Receiver,
                    StreamDirection::SendReceive => NetworkDeviceType::Both,
                },
                channels: s.channels as u32,
                sample_rate: s.sample_rate,
                ip_address: Some(s.origin.clone()),
                multicast_group: Some(s.multicast_addr.to_string()),
            }
        }).collect()
    }

    /// Start listening for SAP announcements
    pub fn start(&self) -> Result<()> {
        if self.running.load(Ordering::Relaxed) {
            return Ok(());
        }

        info!("📢 Starting SAP discovery on {}:{}", SAP_MULTICAST_ADDR, SAP_PORT);
        
        self.running.store(true, Ordering::Relaxed);
        
        let running = self.running.clone();
        let streams = self.streams.clone();
        
        std::thread::spawn(move || {
            if let Err(e) = run_sap_listener(running, streams) {
                error!("SAP listener error: {}", e);
            }
        });
        
        Ok(())
    }

    /// Stop discovery
    pub fn stop(&self) {
        info!("📢 Stopping SAP discovery");
        self.running.store(false, Ordering::Relaxed);
    }

    /// Announce our own stream via SAP
    pub fn announce(&self, stream: Aes67Stream) -> Result<()> {
        let sdp = generate_sdp(&stream)?;
        let sap_packet = build_sap_packet(&stream.session_id, &sdp, false)?;
        
        // Send announcement
        let socket = UdpSocket::bind("0.0.0.0:0")?;
        socket.set_multicast_ttl_v4(64)?;
        socket.send_to(&sap_packet, (SAP_MULTICAST_ADDR, SAP_PORT))?;
        
        info!("📢 Announced stream: {}", stream.name);
        
        // Store for periodic re-announcement
        self.announced.write().push(stream);
        
        Ok(())
    }

    /// Remove stream announcement
    pub fn remove_announcement(&self, session_id: &str) -> Result<()> {
        // Send deletion announcement
        let sap_packet = build_sap_packet(session_id, "", true)?;
        
        let socket = UdpSocket::bind("0.0.0.0:0")?;
        socket.set_multicast_ttl_v4(64)?;
        socket.send_to(&sap_packet, (SAP_MULTICAST_ADDR, SAP_PORT))?;
        
        // Remove from announced list
        self.announced.write().retain(|s| s.session_id != session_id);
        
        info!("📢 Removed stream announcement: {}", session_id);
        
        Ok(())
    }
}

/// Run the SAP listener thread
fn run_sap_listener(
    running: Arc<AtomicBool>,
    streams: Arc<RwLock<HashMap<String, Aes67Stream>>>,
) -> Result<()> {
    use socket2::{Socket, Domain, Type, Protocol};
    
    // Create multicast socket
    let socket = Socket::new(Domain::IPV4, Type::DGRAM, Some(Protocol::UDP))?;
    socket.set_reuse_address(true)?;
    
    #[cfg(unix)]
    socket.set_reuse_port(true)?;
    
    let addr = SocketAddr::new(IpAddr::V4(Ipv4Addr::UNSPECIFIED), SAP_PORT);
    socket.bind(&addr.into())?;
    socket.join_multicast_v4(&SAP_MULTICAST_ADDR, &Ipv4Addr::UNSPECIFIED)?;
    
    let socket: UdpSocket = socket.into();
    socket.set_read_timeout(Some(Duration::from_millis(500)))?;
    
    let mut buf = [0u8; 4096];
    
    while running.load(Ordering::Relaxed) {
        match socket.recv_from(&mut buf) {
            Ok((len, src)) => {
                if len < 4 {
                    continue;
                }
                
                if let Some(stream) = parse_sap_packet(&buf[..len], src.ip()) {
                    let session_id = stream.session_id.clone();
                    
                    // Check for deletion
                    if buf[0] & 0x04 != 0 {
                        // Deletion bit set
                        streams.write().remove(&session_id);
                        debug!("SAP: Removed stream {}", session_id);
                    } else {
                        debug!("SAP: Discovered stream {} from {}", stream.name, src);
                        streams.write().insert(session_id, stream);
                    }
                }
            }
            Err(ref e) if e.kind() == std::io::ErrorKind::WouldBlock => {
                // Timeout, continue - this is normal
            }
            Err(ref e) if e.kind() == std::io::ErrorKind::TimedOut => {
                // Timeout on Windows, continue - this is normal
            }
            Err(e) => {
                // Only log real errors, not timeouts
                // Error 10060 is WSAETIMEDOUT on Windows - treat as normal
                let is_timeout = e.raw_os_error() == Some(10060);
                if !is_timeout {
                    debug!("SAP receive: {} (this is normal if no AES67 devices on network)", e);
                }
            }
        }
    }
    
    info!("SAP listener stopped");
    Ok(())
}

/// Parse a SAP packet and extract stream info
fn parse_sap_packet(data: &[u8], source: IpAddr) -> Option<Aes67Stream> {
    if data.len() < 8 {
        return None;
    }
    
    // SAP Header:
    // Byte 0: V=1, A, R, T, E, C, Message Type (Announce=0, Delete=1)
    // Byte 1: Auth length (usually 0)
    // Bytes 2-3: Message ID Hash
    // Bytes 4-7 (or more): Originating source
    // Then optional auth data
    // Then MIME type (usually "application/sdp\0")
    // Then SDP payload
    
    let version = (data[0] >> 5) & 0x07;
    if version != 1 {
        return None;
    }
    
    let auth_len = data[1] as usize * 4;
    let addr_type = if data[0] & 0x10 != 0 { 6 } else { 4 }; // IPv6 or IPv4
    
    let header_len = 4 + addr_type + auth_len;
    if data.len() < header_len {
        return None;
    }
    
    // Find MIME type and SDP
    let payload = &data[header_len..];
    
    // Look for "application/sdp\0" or just parse as SDP
    let sdp_start = if let Some(pos) = payload.iter().position(|&b| b == 0) {
        pos + 1
    } else {
        0
    };
    
    if sdp_start >= payload.len() {
        return None;
    }
    
    let sdp_str = String::from_utf8_lossy(&payload[sdp_start..]);
    
    parse_sdp(&sdp_str, source.to_string())
}

/// Parse SDP content
fn parse_sdp(sdp: &str, default_origin: String) -> Option<Aes67Stream> {
    let mut name = String::new();
    let mut session_id = String::new();
    let mut origin = default_origin;
    let mut multicast_addr = Ipv4Addr::new(0, 0, 0, 0);
    let mut port: u16 = 5004;
    let mut channels: u8 = 2;
    let mut sample_rate: u32 = 48000;
    let mut bits_per_sample: u8 = 24;
    let mut ptime_us: u32 = 1000;
    
    for line in sdp.lines() {
        let line = line.trim();
        
        if line.starts_with("s=") {
            name = line[2..].to_string();
        } else if line.starts_with("o=") {
            // o=<username> <sess-id> <sess-version> <nettype> <addrtype> <unicast-address>
            let parts: Vec<&str> = line[2..].split_whitespace().collect();
            if parts.len() >= 6 {
                session_id = format!("{}_{}", parts[0], parts[1]);
                origin = parts[5].to_string();
            }
        } else if line.starts_with("c=") {
            // c=IN IP4 <multicast-address>/<ttl>
            let parts: Vec<&str> = line[2..].split_whitespace().collect();
            if parts.len() >= 3 {
                let addr_part = parts[2].split('/').next().unwrap_or("");
                if let Ok(addr) = addr_part.parse() {
                    multicast_addr = addr;
                }
            }
        } else if line.starts_with("m=audio ") {
            // m=audio <port> RTP/AVP <payload-type>
            let parts: Vec<&str> = line[8..].split_whitespace().collect();
            if let Some(p) = parts.first() {
                port = p.parse().unwrap_or(5004);
            }
        } else if line.starts_with("a=rtpmap:") {
            // a=rtpmap:<payload> L24/<sample-rate>/<channels>
            if line.contains("L24/") || line.contains("L16/") {
                let parts: Vec<&str> = line.split('/').collect();
                if parts.len() >= 2 {
                    sample_rate = parts[1].parse().unwrap_or(48000);
                }
                if parts.len() >= 3 {
                    channels = parts[2].parse().unwrap_or(2);
                }
                bits_per_sample = if line.contains("L24") { 24 } else { 16 };
            }
        } else if line.starts_with("a=ptime:") {
            // a=ptime:<ms>
            if let Ok(ptime_ms) = line[8..].parse::<f32>() {
                ptime_us = (ptime_ms * 1000.0) as u32;
            }
        }
    }
    
    if name.is_empty() {
        name = format!("AES67 Stream {}", &session_id);
    }
    
    if session_id.is_empty() {
        return None;
    }
    
    Some(Aes67Stream {
        name,
        session_id,
        origin,
        multicast_addr,
        port,
        channels,
        sample_rate,
        bits_per_sample,
        ptime_us,
        direction: StreamDirection::Send,
        sdp: sdp.to_string(),
    })
}

/// Generate SDP for our stream
fn generate_sdp(stream: &Aes67Stream) -> Result<String> {
    let session_version = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)?
        .as_secs();
    
    let ptime_ms = stream.ptime_us as f32 / 1000.0;
    
    // Payload type 97 is commonly used for L24
    let payload_type = 97;
    
    let sdp = format!(
        "v=0\r\n\
        o=- {sess_id} {version} IN IP4 {origin}\r\n\
        s={name}\r\n\
        c=IN IP4 {mcast}/64\r\n\
        t=0 0\r\n\
        m=audio {port} RTP/AVP {pt}\r\n\
        a=rtpmap:{pt} L{bits}/{rate}/{ch}\r\n\
        a=ptime:{ptime:.3}\r\n\
        a=ts-refclk:ptp=IEEE1588-2008:00-00-00-00-00-00-00-00:0\r\n\
        a=mediaclk:direct=0\r\n",
        sess_id = stream.session_id,
        version = session_version,
        origin = stream.origin,
        name = stream.name,
        mcast = stream.multicast_addr,
        port = stream.port,
        pt = payload_type,
        bits = stream.bits_per_sample,
        rate = stream.sample_rate,
        ch = stream.channels,
        ptime = ptime_ms,
    );
    
    Ok(sdp)
}

/// Build a SAP packet
fn build_sap_packet(session_id: &str, sdp: &str, deletion: bool) -> Result<Vec<u8>> {
    let mut packet = Vec::with_capacity(8 + 16 + sdp.len());
    
    // SAP Header
    // Byte 0: V=1 (001), A=0, R=0, T=deletion, E=0, C=0
    let flags = if deletion { 0x24 } else { 0x20 }; // V=1, T=deletion
    packet.push(flags);
    
    // Byte 1: Auth length = 0
    packet.push(0);
    
    // Bytes 2-3: Message ID Hash (simple hash of session_id)
    let hash: u16 = session_id.bytes().fold(0u16, |acc, b| acc.wrapping_add(b as u16));
    packet.extend_from_slice(&hash.to_be_bytes());
    
    // Bytes 4-7: Originating source (0.0.0.0 for now)
    packet.extend_from_slice(&[0, 0, 0, 0]);
    
    // MIME type
    packet.extend_from_slice(b"application/sdp\0");
    
    // SDP payload
    packet.extend_from_slice(sdp.as_bytes());
    
    Ok(packet)
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_parse_sdp() {
        let sdp = r#"v=0
o=- 12345 1 IN IP4 192.168.1.100
s=Test AES67 Stream
c=IN IP4 239.69.1.1/64
t=0 0
m=audio 5004 RTP/AVP 97
a=rtpmap:97 L24/48000/2
a=ptime:1.000
"#;
        
        let stream = parse_sdp(sdp, "192.168.1.100".to_string()).unwrap();
        
        assert_eq!(stream.name, "Test AES67 Stream");
        assert_eq!(stream.channels, 2);
        assert_eq!(stream.sample_rate, 48000);
        assert_eq!(stream.bits_per_sample, 24);
        assert_eq!(stream.port, 5004);
        assert_eq!(stream.multicast_addr, Ipv4Addr::new(239, 69, 1, 1));
    }
    
    #[test]
    fn test_generate_sdp() {
        let stream = Aes67Stream {
            name: "Test Stream".to_string(),
            session_id: "test123".to_string(),
            origin: "192.168.1.1".to_string(),
            multicast_addr: Ipv4Addr::new(239, 69, 1, 1),
            port: 5004,
            channels: 8,
            sample_rate: 48000,
            bits_per_sample: 24,
            ptime_us: 1000,
            direction: StreamDirection::Send,
            sdp: String::new(),
        };
        
        let sdp = generate_sdp(&stream).unwrap();
        
        assert!(sdp.contains("s=Test Stream"));
        assert!(sdp.contains("L24/48000/8"));
        assert!(sdp.contains("239.69.1.1"));
    }
}
