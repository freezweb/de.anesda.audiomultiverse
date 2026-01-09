//! Audio Network Backend Abstraktion
//! 
//! Ermöglicht verschiedene Backends: AES67, DANTE, etc.

use std::net::Ipv4Addr;
use std::sync::Arc;
use anyhow::{Result, anyhow};
use parking_lot::RwLock;
use tracing::{info, warn, error, debug};

// Use PtpClock from parent module (either real or stub depending on platform)
use super::PtpClock;
use super::rtp::{RtpSender, RtpReceiver, Aes67Format};
use super::sap::{SapDiscovery, Aes67Stream, StreamDirection};

/// Trait für Audio-Netzwerk Backends
pub trait AudioNetworkBackend: Send + Sync {
    /// Backend-Name
    fn name(&self) -> &'static str;
    
    /// Backend initialisieren
    fn init(&mut self) -> Result<()>;
    
    /// Verfügbare Geräte/Streams auflisten
    fn discover(&self) -> Result<Vec<NetworkDevice>>;
    
    /// Zu einem Gerät verbinden
    fn connect(&mut self, device: &NetworkDevice) -> Result<()>;
    
    /// Verbindung trennen
    fn disconnect(&mut self) -> Result<()>;
    
    /// Input-Samples lesen (für Audio-Thread)
    fn read_samples(&self, buffer: &mut [f32], channels: usize) -> usize;
    
    /// Output-Samples schreiben
    fn write_samples(&self, buffer: &[f32], channels: usize) -> usize;
    
    /// Aktuelle Latenz in Samples
    fn latency(&self) -> usize;
    
    /// Ist verbunden?
    fn is_connected(&self) -> bool;
}

/// Ein Netzwerk-Audio Gerät
#[derive(Debug, Clone)]
pub struct NetworkDevice {
    /// Eindeutige ID
    pub id: String,
    
    /// Anzeigename
    pub name: String,
    
    /// Gerätetyp (transmitter, receiver)
    pub device_type: NetworkDeviceType,
    
    /// Anzahl Kanäle
    pub channels: u32,
    
    /// Sample Rate
    pub sample_rate: u32,
    
    /// IP-Adresse
    pub ip_address: Option<String>,
    
    /// Multicast-Gruppe (für AES67)
    pub multicast_group: Option<String>,
}

/// Gerätetyp
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum NetworkDeviceType {
    Transmitter,
    Receiver,
    Both,
}

// ============================================================
// AES67 Backend - Full Implementation
// ============================================================

/// AES67 Backend configuration
#[derive(Debug, Clone)]
pub struct Aes67Config {
    /// Network interface to use
    pub interface: String,
    /// Number of input channels to receive
    pub input_channels: u8,
    /// Number of output channels to send
    pub output_channels: u8,
    /// Sample rate (48000 for AES67)
    pub sample_rate: u32,
    /// Multicast address for our output stream
    pub output_multicast: Ipv4Addr,
    /// RTP port for our output stream
    pub output_port: u16,
    /// Stream name
    pub stream_name: String,
    /// PTP domain (0-127)
    pub ptp_domain: u8,
}

impl Default for Aes67Config {
    fn default() -> Self {
        Self {
            interface: "eth0".to_string(),
            input_channels: 8,
            output_channels: 8,
            sample_rate: 48000,
            output_multicast: Ipv4Addr::new(239, 69, 1, 100),
            output_port: 5004,
            stream_name: "AudioMultiverse".to_string(),
            ptp_domain: 0,
        }
    }
}

/// AES67 Backend (DANTE-kompatibel via AES67)
pub struct Aes67Backend {
    /// Configuration
    config: Aes67Config,
    /// Connection state
    connected: bool,
    /// PTP Clock for synchronization
    ptp_clock: Arc<PtpClock>,
    /// SAP/SDP Discovery
    sap_discovery: Arc<SapDiscovery>,
    /// RTP Sender (for output)
    rtp_sender: Option<RtpSender>,
    /// RTP Receiver (for input)  
    rtp_receiver: Option<RtpReceiver>,
    /// Connected device
    connected_device: Option<NetworkDevice>,
    /// Our announced stream info
    our_stream: Option<Aes67Stream>,
}

impl Aes67Backend {
    /// Create a new AES67 backend with default config
    pub fn new() -> Self {
        Self::with_config(Aes67Config::default())
    }
    
    /// Create a new AES67 backend with custom config
    pub fn with_config(config: Aes67Config) -> Self {
        let mut ptp_clock = PtpClock::new(&config.interface);
        ptp_clock.set_domain(config.ptp_domain);
        
        Self {
            config,
            connected: false,
            ptp_clock: Arc::new(ptp_clock),
            sap_discovery: Arc::new(SapDiscovery::new()),
            rtp_sender: None,
            rtp_receiver: None,
            connected_device: None,
            our_stream: None,
        }
    }
    
    /// Get PTP clock reference
    pub fn ptp_clock(&self) -> Arc<PtpClock> {
        self.ptp_clock.clone()
    }
    
    /// Get SAP discovery reference
    pub fn sap_discovery(&self) -> Arc<SapDiscovery> {
        self.sap_discovery.clone()
    }
    
    /// Check if PTP is synchronized
    pub fn is_ptp_synchronized(&self) -> bool {
        self.ptp_clock.is_synchronized()
    }
    
    /// Get our output stream info
    pub fn our_stream(&self) -> Option<&Aes67Stream> {
        self.our_stream.as_ref()
    }
}

impl AudioNetworkBackend for Aes67Backend {
    fn name(&self) -> &'static str {
        "AES67"
    }
    
    fn init(&mut self) -> Result<()> {
        info!("🌐 AES67 Backend initialisiert");
        info!("   Interface: {}", self.config.interface);
        info!("   Channels: {} in / {} out", self.config.input_channels, self.config.output_channels);
        info!("   Sample Rate: {} Hz", self.config.sample_rate);
        info!("   PTP Domain: {}", self.config.ptp_domain);
        
        // Start PTP Clock synchronization
        info!("⏱️  Starting PTP clock synchronization...");
        self.ptp_clock.start()?;
        
        // Start SAP/SDP discovery
        info!("📢 Starting SAP/SDP discovery...");
        self.sap_discovery.start()?;
        
        // Create output stream
        let format = Aes67Format {
            sample_rate: self.config.sample_rate,
            channels: self.config.output_channels,
            bits_per_sample: 24,
            samples_per_packet: 48, // 1ms at 48kHz
        };
        
        // Create RTP sender for output
        let mut sender = RtpSender::new(
            self.config.output_multicast,
            self.config.output_port,
            format,
        )?;
        sender.set_ptp_clock(self.ptp_clock.clone());
        
        // Announce our output stream via SAP
        let our_stream = Aes67Stream {
            name: self.config.stream_name.clone(),
            session_id: format!("audiomultiverse_{}", sender.ssrc()),
            origin: get_local_ip().unwrap_or_else(|| "0.0.0.0".to_string()),
            multicast_addr: self.config.output_multicast,
            port: self.config.output_port,
            channels: self.config.output_channels,
            sample_rate: self.config.sample_rate,
            bits_per_sample: 24,
            ptime_us: 1000, // 1ms
            direction: StreamDirection::Send,
            sdp: String::new(),
        };
        
        self.sap_discovery.announce(our_stream.clone())?;
        self.our_stream = Some(our_stream);
        self.rtp_sender = Some(sender);
        
        info!("✅ AES67 Backend ready");
        info!("   Output: {}:{} ({} channels)", 
            self.config.output_multicast, 
            self.config.output_port,
            self.config.output_channels
        );
        
        Ok(())
    }
    
    fn discover(&self) -> Result<Vec<NetworkDevice>> {
        // Get discovered streams from SAP
        let devices = self.sap_discovery.as_devices();
        
        if devices.is_empty() {
            debug!("No AES67 devices discovered yet");
        } else {
            debug!("Discovered {} AES67 devices", devices.len());
        }
        
        Ok(devices)
    }
    
    fn connect(&mut self, device: &NetworkDevice) -> Result<()> {
        info!("🔌 Connecting to AES67 device: {}", device.name);
        
        // Get multicast address from device
        let multicast_str = device.multicast_group.as_ref()
            .ok_or_else(|| anyhow!("Device has no multicast group"))?;
        let multicast_addr: Ipv4Addr = multicast_str.parse()
            .map_err(|_| anyhow!("Invalid multicast address"))?;
        
        // Assume port 5004 if not specified (standard AES67 port)
        let port = 5004;
        
        // Create format based on device info
        let format = Aes67Format {
            sample_rate: device.sample_rate,
            channels: device.channels as u8,
            bits_per_sample: 24,
            samples_per_packet: 48,
        };
        
        // Create RTP receiver for this stream
        let receiver = RtpReceiver::new(multicast_addr, port, format)?;
        receiver.start()?;
        
        self.rtp_receiver = Some(receiver);
        self.connected_device = Some(device.clone());
        self.connected = true;
        
        info!("✅ Connected to {} ({}:{}, {} channels)", 
            device.name, multicast_addr, port, device.channels);
        
        Ok(())
    }
    
    fn disconnect(&mut self) -> Result<()> {
        if let Some(device) = &self.connected_device {
            info!("🔌 Disconnecting from AES67 device: {}", device.name);
        }
        
        if let Some(receiver) = &self.rtp_receiver {
            receiver.stop();
        }
        
        self.rtp_receiver = None;
        self.connected_device = None;
        self.connected = false;
        
        Ok(())
    }
    
    fn read_samples(&self, buffer: &mut [f32], _channels: usize) -> usize {
        if let Some(receiver) = &self.rtp_receiver {
            receiver.read(buffer)
        } else {
            // No receiver connected, return silence
            for sample in buffer.iter_mut() {
                *sample = 0.0;
            }
            buffer.len()
        }
    }
    
    fn write_samples(&self, buffer: &[f32], _channels: usize) -> usize {
        if let Some(sender) = &self.rtp_sender {
            match sender.send(buffer) {
                Ok(_) => buffer.len(),
                Err(e) => {
                    warn!("RTP send error: {}", e);
                    0
                }
            }
        } else {
            0
        }
    }
    
    fn latency(&self) -> usize {
        // AES67 typical latency: 1ms packet time + jitter buffer (~3-4ms total)
        // At 48kHz: 48 samples = 1ms
        // We use ~4ms jitter buffer = 192 samples
        48 * 4
    }
    
    fn is_connected(&self) -> bool {
        self.connected
    }
}

impl Default for Aes67Backend {
    fn default() -> Self {
        Self::new()
    }
}

impl Drop for Aes67Backend {
    fn drop(&mut self) {
        // Stop services
        self.ptp_clock.stop();
        self.sap_discovery.stop();
        
        // Remove our SAP announcement
        if let Some(stream) = &self.our_stream {
            let _ = self.sap_discovery.remove_announcement(&stream.session_id);
        }
        
        // Disconnect from any connected device
        let _ = self.disconnect();
    }
}

/// Get local IP address
fn get_local_ip() -> Option<String> {
    use std::net::UdpSocket;
    
    // Create a UDP socket and connect to a public address
    // This doesn't actually send anything, just determines the local IP
    let socket = UdpSocket::bind("0.0.0.0:0").ok()?;
    socket.connect("8.8.8.8:80").ok()?;
    let addr = socket.local_addr().ok()?;
    Some(addr.ip().to_string())
}

// ============================================================
// DANTE Backend (Placeholder für Audinate SDK Integration)
// ============================================================

/// DANTE Backend (benötigt Audinate SDK)
/// 
/// Dieser Backend ist vorbereitet für die Integration mit dem
/// Audinate DANTE SDK. Die Implementierung erfolgt, sobald
/// das SDK verfügbar ist.
#[cfg(feature = "dante")]
pub struct DanteBackend {
    connected: bool,
}

#[cfg(feature = "dante")]
impl DanteBackend {
    pub fn new() -> Self {
        Self {
            connected: false,
        }
    }
}

#[cfg(feature = "dante")]
impl AudioNetworkBackend for DanteBackend {
    fn name(&self) -> &'static str {
        "DANTE"
    }
    
    fn init(&mut self) -> Result<()> {
        info!("🎵 DANTE Backend initialisiert");
        // TODO: dante_sdk::initialize()
        Ok(())
    }
    
    fn discover(&self) -> Result<Vec<NetworkDevice>> {
        // TODO: dante_sdk::browse_devices()
        Ok(vec![])
    }
    
    fn connect(&mut self, device: &NetworkDevice) -> Result<()> {
        info!("Verbinde mit DANTE Gerät: {}", device.name);
        // TODO: dante_sdk::connect()
        self.connected = true;
        Ok(())
    }
    
    fn disconnect(&mut self) -> Result<()> {
        // TODO: dante_sdk::disconnect()
        self.connected = false;
        Ok(())
    }
    
    fn read_samples(&self, buffer: &mut [f32], _channels: usize) -> usize {
        // TODO: dante_sdk::read()
        for sample in buffer.iter_mut() {
            *sample = 0.0;
        }
        buffer.len()
    }
    
    fn write_samples(&self, buffer: &[f32], _channels: usize) -> usize {
        // TODO: dante_sdk::write()
        0
    }
    
    fn latency(&self) -> usize {
        // DANTE typical latency: configurable 0.15ms - 5ms
        48 * 5 // 5ms default
    }
    
    fn is_connected(&self) -> bool {
        self.connected
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_aes67_config_default() {
        let config = Aes67Config::default();
        assert_eq!(config.sample_rate, 48000);
        assert_eq!(config.input_channels, 8);
        assert_eq!(config.output_channels, 8);
        assert_eq!(config.ptp_domain, 0);
    }
    
    #[test]
    fn test_get_local_ip() {
        let ip = get_local_ip();
        // Should get some IP (may fail in some network configs)
        if let Some(ip) = ip {
            assert!(!ip.is_empty());
            assert!(!ip.contains("0.0.0.0"));
        }
    }
}
