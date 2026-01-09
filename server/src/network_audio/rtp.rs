//! RTP (Real-time Transport Protocol) for AES67 Audio Streaming
//!
//! Implements RTP packet encoding/decoding for L24 (24-bit linear PCM) audio.
//! AES67 uses RTP with specific parameters:
//! - Payload type: 97 (dynamic)
//! - Clock rate: 48000 Hz
//! - Sample format: L24 (24-bit linear, big-endian)
//! - Channels: 1-8 typically
//! - Packet time: 1ms (48 samples at 48kHz)

use std::net::{IpAddr, Ipv4Addr, SocketAddr, UdpSocket};
use std::sync::atomic::{AtomicBool, AtomicU32, Ordering};
use std::sync::Arc;
use std::time::Duration;
use anyhow::{Result, anyhow};
use bytes::{BufMut, BytesMut};
use parking_lot::RwLock;
use tracing::{info, warn, error, debug, trace};

// Use PtpClock from parent module (either real or stub depending on platform)
use super::PtpClock;

/// RTP Header (12 bytes minimum)
/// 
/// ```text
///  0                   1                   2                   3
///  0 1 2 3 4 5 6 7 8 9 0 1 2 3 4 5 6 7 8 9 0 1 2 3 4 5 6 7 8 9 0 1
/// +-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+
/// |V=2|P|X|  CC   |M|     PT      |       sequence number         |
/// +-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+
/// |                           timestamp                           |
/// +-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+
/// |           synchronization source (SSRC) identifier            |
/// +-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+
/// ```
#[derive(Debug, Clone)]
pub struct RtpHeader {
    /// RTP version (always 2)
    pub version: u8,
    /// Padding flag
    pub padding: bool,
    /// Extension flag
    pub extension: bool,
    /// CSRC count
    pub csrc_count: u8,
    /// Marker bit
    pub marker: bool,
    /// Payload type (97 for AES67 dynamic)
    pub payload_type: u8,
    /// Sequence number
    pub sequence: u16,
    /// Timestamp (in samples at 48kHz)
    pub timestamp: u32,
    /// Synchronization source ID
    pub ssrc: u32,
}

impl RtpHeader {
    /// Create a new RTP header
    pub fn new(payload_type: u8, ssrc: u32) -> Self {
        Self {
            version: 2,
            padding: false,
            extension: false,
            csrc_count: 0,
            marker: false,
            payload_type,
            sequence: 0,
            timestamp: 0,
            ssrc,
        }
    }

    /// Serialize header to bytes
    pub fn to_bytes(&self) -> [u8; 12] {
        let mut buf = [0u8; 12];
        
        // Byte 0: V=2, P, X, CC
        buf[0] = (self.version << 6) 
            | ((self.padding as u8) << 5)
            | ((self.extension as u8) << 4)
            | (self.csrc_count & 0x0F);
        
        // Byte 1: M, PT
        buf[1] = ((self.marker as u8) << 7) | (self.payload_type & 0x7F);
        
        // Bytes 2-3: Sequence number
        buf[2..4].copy_from_slice(&self.sequence.to_be_bytes());
        
        // Bytes 4-7: Timestamp
        buf[4..8].copy_from_slice(&self.timestamp.to_be_bytes());
        
        // Bytes 8-11: SSRC
        buf[8..12].copy_from_slice(&self.ssrc.to_be_bytes());
        
        buf
    }

    /// Parse header from bytes
    pub fn from_bytes(data: &[u8]) -> Option<Self> {
        if data.len() < 12 {
            return None;
        }
        
        let version = (data[0] >> 6) & 0x03;
        if version != 2 {
            return None;
        }
        
        Some(Self {
            version,
            padding: (data[0] & 0x20) != 0,
            extension: (data[0] & 0x10) != 0,
            csrc_count: data[0] & 0x0F,
            marker: (data[1] & 0x80) != 0,
            payload_type: data[1] & 0x7F,
            sequence: u16::from_be_bytes([data[2], data[3]]),
            timestamp: u32::from_be_bytes([data[4], data[5], data[6], data[7]]),
            ssrc: u32::from_be_bytes([data[8], data[9], data[10], data[11]]),
        })
    }
}

/// AES67 Audio format
#[derive(Debug, Clone, Copy)]
pub struct Aes67Format {
    /// Sample rate (48000 for AES67)
    pub sample_rate: u32,
    /// Number of channels (1-8)
    pub channels: u8,
    /// Bits per sample (24 for L24)
    pub bits_per_sample: u8,
    /// Samples per packet (48 for 1ms at 48kHz)
    pub samples_per_packet: u16,
}

impl Default for Aes67Format {
    fn default() -> Self {
        Self {
            sample_rate: 48000,
            channels: 2,
            bits_per_sample: 24,
            samples_per_packet: 48, // 1ms at 48kHz
        }
    }
}

impl Aes67Format {
    /// Bytes per packet for audio payload
    pub fn bytes_per_packet(&self) -> usize {
        self.samples_per_packet as usize * self.channels as usize * 3 // 24-bit = 3 bytes
    }
}

/// RTP Stream for sending audio
pub struct RtpSender {
    /// Socket for sending
    socket: UdpSocket,
    /// Destination address
    destination: SocketAddr,
    /// SSRC (randomly generated)
    ssrc: u32,
    /// Sequence number
    sequence: AtomicU32,
    /// Audio format
    format: Aes67Format,
    /// PTP clock for timestamps
    ptp_clock: Option<Arc<PtpClock>>,
    /// Running flag
    running: Arc<AtomicBool>,
    /// Payload type (97 for dynamic)
    payload_type: u8,
}

impl RtpSender {
    /// Create a new RTP sender
    pub fn new(multicast_addr: Ipv4Addr, port: u16, format: Aes67Format) -> Result<Self> {
        let socket = UdpSocket::bind("0.0.0.0:0")?;
        
        // Set multicast TTL
        socket.set_multicast_ttl_v4(64)?;
        
        let destination = SocketAddr::new(IpAddr::V4(multicast_addr), port);
        
        // Generate random SSRC
        let ssrc = rand::random();
        
        Ok(Self {
            socket,
            destination,
            ssrc,
            sequence: AtomicU32::new(rand::random::<u16>() as u32),
            format,
            ptp_clock: None,
            running: Arc::new(AtomicBool::new(false)),
            payload_type: 97, // Dynamic payload type for AES67
        })
    }

    /// Set PTP clock for accurate timestamps
    pub fn set_ptp_clock(&mut self, clock: Arc<PtpClock>) {
        self.ptp_clock = Some(clock);
    }

    /// Get SSRC
    pub fn ssrc(&self) -> u32 {
        self.ssrc
    }

    /// Send audio samples
    /// 
    /// Expects interleaved f32 samples, will convert to L24
    pub fn send(&self, samples: &[f32]) -> Result<()> {
        let samples_per_channel = samples.len() / self.format.channels as usize;
        let packets_needed = (samples_per_channel + self.format.samples_per_packet as usize - 1) 
            / self.format.samples_per_packet as usize;
        
        let mut offset = 0;
        for _ in 0..packets_needed {
            let samples_this_packet = std::cmp::min(
                self.format.samples_per_packet as usize * self.format.channels as usize,
                samples.len() - offset,
            );
            
            if samples_this_packet == 0 {
                break;
            }
            
            self.send_packet(&samples[offset..offset + samples_this_packet])?;
            offset += samples_this_packet;
        }
        
        Ok(())
    }

    /// Send a single RTP packet
    fn send_packet(&self, samples: &[f32]) -> Result<()> {
        // Get timestamp from PTP clock or system time
        let timestamp = self.ptp_clock
            .as_ref()
            .map(|c| c.media_timestamp())
            .unwrap_or_else(|| {
                (std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap_or_default()
                    .as_nanos() as u64 * 48000 / 1_000_000_000) as u32
            });
        
        // Get next sequence number
        let sequence = self.sequence.fetch_add(1, Ordering::Relaxed) as u16;
        
        // Build header
        let mut header = RtpHeader::new(self.payload_type, self.ssrc);
        header.sequence = sequence;
        header.timestamp = timestamp;
        
        // Build packet
        let mut packet = BytesMut::with_capacity(12 + samples.len() * 3);
        packet.extend_from_slice(&header.to_bytes());
        
        // Convert f32 samples to L24 (24-bit big-endian)
        for sample in samples {
            // Clamp and scale to 24-bit range
            let clamped = sample.clamp(-1.0, 1.0);
            let scaled = (clamped * 8388607.0) as i32; // 2^23 - 1
            
            // Write as 24-bit big-endian
            let bytes = scaled.to_be_bytes();
            packet.put_slice(&bytes[1..4]); // Skip the MSB, write 3 bytes
        }
        
        // Send packet
        self.socket.send_to(&packet, self.destination)?;
        
        trace!("Sent RTP packet: seq={}, ts={}, samples={}", 
            sequence, timestamp, samples.len());
        
        Ok(())
    }
}

/// RTP Stream for receiving audio
pub struct RtpReceiver {
    /// Socket for receiving
    socket: UdpSocket,
    /// Audio format
    format: Aes67Format,
    /// Running flag
    running: Arc<AtomicBool>,
    /// Jitter buffer (in samples)
    jitter_buffer: Arc<RwLock<JitterBuffer>>,
    /// Last received sequence
    last_sequence: Arc<AtomicU32>,
    /// Expected SSRC (None = accept any)
    expected_ssrc: Option<u32>,
}

impl RtpReceiver {
    /// Create a new RTP receiver
    pub fn new(multicast_addr: Ipv4Addr, port: u16, format: Aes67Format) -> Result<Self> {
        use socket2::{Socket, Domain, Type, Protocol};
        
        // Create socket with reuse
        let socket = Socket::new(Domain::IPV4, Type::DGRAM, Some(Protocol::UDP))?;
        socket.set_reuse_address(true)?;
        
        #[cfg(unix)]
        socket.set_reuse_port(true)?;
        
        // Bind to the port
        let addr = SocketAddr::new(IpAddr::V4(Ipv4Addr::UNSPECIFIED), port);
        socket.bind(&addr.into())?;
        
        // Join multicast group
        socket.join_multicast_v4(&multicast_addr, &Ipv4Addr::UNSPECIFIED)?;
        
        let socket: UdpSocket = socket.into();
        socket.set_read_timeout(Some(Duration::from_millis(10)))?;
        
        Ok(Self {
            socket,
            format,
            running: Arc::new(AtomicBool::new(false)),
            jitter_buffer: Arc::new(RwLock::new(JitterBuffer::new(format.samples_per_packet as usize * 4))),
            last_sequence: Arc::new(AtomicU32::new(0)),
            expected_ssrc: None,
        })
    }

    /// Set expected SSRC (filter packets)
    pub fn set_expected_ssrc(&mut self, ssrc: u32) {
        self.expected_ssrc = Some(ssrc);
    }

    /// Start receiving (spawns background thread)
    pub fn start(&self) -> Result<()> {
        if self.running.load(Ordering::Relaxed) {
            return Ok(());
        }
        
        self.running.store(true, Ordering::Relaxed);
        
        let socket = self.socket.try_clone()?;
        let running = self.running.clone();
        let jitter_buffer = self.jitter_buffer.clone();
        let expected_ssrc = self.expected_ssrc;
        let last_sequence = self.last_sequence.clone();
        let channels = self.format.channels;
        
        std::thread::spawn(move || {
            let mut buf = [0u8; 2048];
            
            while running.load(Ordering::Relaxed) {
                match socket.recv_from(&mut buf) {
                    Ok((len, _src)) => {
                        if len < 12 {
                            continue;
                        }
                        
                        if let Some(header) = RtpHeader::from_bytes(&buf[..len]) {
                            // Check SSRC if filtering
                            if let Some(expected) = expected_ssrc {
                                if header.ssrc != expected {
                                    continue;
                                }
                            }
                            
                            // Check sequence for packet loss
                            let last_seq = last_sequence.swap(header.sequence as u32, Ordering::Relaxed) as u16;
                            let expected_seq = last_seq.wrapping_add(1);
                            if header.sequence != expected_seq && last_seq != 0 {
                                let lost = header.sequence.wrapping_sub(expected_seq);
                                if lost > 0 && lost < 100 {
                                    warn!("Packet loss detected: {} packets", lost);
                                }
                            }
                            
                            // Decode L24 payload to f32
                            let payload = &buf[12..len];
                            let samples = decode_l24_to_f32(payload, channels);
                            
                            // Add to jitter buffer
                            jitter_buffer.write().push_samples(&samples);
                        }
                    }
                    Err(ref e) if e.kind() == std::io::ErrorKind::WouldBlock => {
                        // Timeout, continue
                    }
                    Err(e) => {
                        warn!("RTP receive error: {}", e);
                    }
                }
            }
            
            info!("RTP receiver stopped");
        });
        
        Ok(())
    }

    /// Stop receiving
    pub fn stop(&self) {
        self.running.store(false, Ordering::Relaxed);
    }

    /// Read samples from jitter buffer
    pub fn read(&self, buffer: &mut [f32]) -> usize {
        self.jitter_buffer.write().pop_samples(buffer)
    }

    /// Check if receiving data
    pub fn is_receiving(&self) -> bool {
        self.jitter_buffer.read().available() > 0
    }
}

/// Decode L24 (24-bit big-endian) audio to f32
fn decode_l24_to_f32(data: &[u8], channels: u8) -> Vec<f32> {
    let sample_count = data.len() / 3;
    let mut samples = Vec::with_capacity(sample_count);
    
    for i in 0..sample_count {
        let offset = i * 3;
        if offset + 3 > data.len() {
            break;
        }
        
        // Read 24-bit big-endian, sign-extend to 32-bit
        let raw = ((data[offset] as i32) << 16)
            | ((data[offset + 1] as i32) << 8)
            | (data[offset + 2] as i32);
        
        // Sign extend from 24-bit
        let signed = if raw & 0x800000 != 0 {
            raw | !0xFFFFFF // Sign extend negative
        } else {
            raw
        };
        
        // Convert to f32 (-1.0 to 1.0)
        let sample = signed as f32 / 8388607.0;
        samples.push(sample);
    }
    
    samples
}

/// Simple jitter buffer for smoothing out network jitter
struct JitterBuffer {
    buffer: Vec<f32>,
    read_pos: usize,
    write_pos: usize,
    capacity: usize,
}

impl JitterBuffer {
    fn new(capacity: usize) -> Self {
        Self {
            buffer: vec![0.0; capacity],
            read_pos: 0,
            write_pos: 0,
            capacity,
        }
    }

    fn push_samples(&mut self, samples: &[f32]) {
        for &sample in samples {
            self.buffer[self.write_pos] = sample;
            self.write_pos = (self.write_pos + 1) % self.capacity;
            
            // Prevent overflow (overwrite oldest)
            if self.write_pos == self.read_pos {
                self.read_pos = (self.read_pos + 1) % self.capacity;
            }
        }
    }

    fn pop_samples(&mut self, buffer: &mut [f32]) -> usize {
        let available = self.available();
        let to_read = std::cmp::min(buffer.len(), available);
        
        for sample in buffer.iter_mut().take(to_read) {
            *sample = self.buffer[self.read_pos];
            self.read_pos = (self.read_pos + 1) % self.capacity;
        }
        
        // Fill rest with silence if not enough data
        for sample in buffer.iter_mut().skip(to_read) {
            *sample = 0.0;
        }
        
        to_read
    }

    fn available(&self) -> usize {
        if self.write_pos >= self.read_pos {
            self.write_pos - self.read_pos
        } else {
            self.capacity - self.read_pos + self.write_pos
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_rtp_header_roundtrip() {
        let header = RtpHeader {
            version: 2,
            padding: false,
            extension: false,
            csrc_count: 0,
            marker: true,
            payload_type: 97,
            sequence: 12345,
            timestamp: 0xDEADBEEF,
            ssrc: 0xCAFEBABE,
        };
        
        let bytes = header.to_bytes();
        let parsed = RtpHeader::from_bytes(&bytes).unwrap();
        
        assert_eq!(parsed.version, 2);
        assert_eq!(parsed.marker, true);
        assert_eq!(parsed.payload_type, 97);
        assert_eq!(parsed.sequence, 12345);
        assert_eq!(parsed.timestamp, 0xDEADBEEF);
        assert_eq!(parsed.ssrc, 0xCAFEBABE);
    }
    
    #[test]
    fn test_l24_decode() {
        // Test: 0x7FFFFF = max positive = ~1.0
        let data = [0x7F, 0xFF, 0xFF];
        let samples = decode_l24_to_f32(&data, 1);
        assert!((samples[0] - 1.0).abs() < 0.001);
        
        // Test: 0x800000 = max negative = ~-1.0
        let data = [0x80, 0x00, 0x00];
        let samples = decode_l24_to_f32(&data, 1);
        assert!((samples[0] + 1.0).abs() < 0.001);
        
        // Test: 0x000000 = zero
        let data = [0x00, 0x00, 0x00];
        let samples = decode_l24_to_f32(&data, 1);
        assert_eq!(samples[0], 0.0);
    }
    
    #[test]
    fn test_jitter_buffer() {
        let mut jb = JitterBuffer::new(100);
        
        // Push samples
        jb.push_samples(&[0.1, 0.2, 0.3, 0.4]);
        assert_eq!(jb.available(), 4);
        
        // Pop samples
        let mut out = [0.0; 2];
        let read = jb.pop_samples(&mut out);
        assert_eq!(read, 2);
        assert_eq!(out[0], 0.1);
        assert_eq!(out[1], 0.2);
        assert_eq!(jb.available(), 2);
    }
}
