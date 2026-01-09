//! Audio Engine
//! 
//! Hauptmodul für Audio-Verarbeitung mit cpal und AES67 Integration

use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use anyhow::{Result, anyhow};
use tracing::{info, warn, error};
use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use cpal::{Device, Host, Stream, StreamConfig, SampleFormat};
use tokio::sync::mpsc;

use crate::mixer::{Mixer, MasterSection};
use crate::network_audio::{Aes67Backend, Aes67Config, AudioNetworkBackend, NetworkDevice, SapDiscovery, PtpClock};

/// Befehle für die Audio Engine (von WebSocket/API)
#[derive(Debug)]
pub enum AudioCommand {
    /// Zu AES67 Stream subscriben
    SubscribeStream {
        stream_id: String,
        start_channel: Option<u32>,
        /// Antwort-Kanal
        response: tokio::sync::oneshot::Sender<Result<Aes67SubscribeResult, String>>,
    },
    /// AES67 Stream Subscription beenden
    UnsubscribeStream {
        stream_id: String,
        response: tokio::sync::oneshot::Sender<Result<(), String>>,
    },
}

/// Ergebnis einer erfolgreichen Stream-Subscription
#[derive(Debug, Clone)]
pub struct Aes67SubscribeResult {
    pub stream_id: String,
    pub stream_name: String,
    pub channels: u8,
    pub start_channel: u32,
}

/// Audio Device Info
#[derive(Debug, Clone)]
pub struct AudioDeviceInfo {
    pub name: String,
    pub is_input: bool,
    pub is_output: bool,
    pub is_default: bool,
}

/// Audio source type
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum AudioSource {
    /// Local soundcard (cpal)
    Local,
    /// AES67 network audio
    Aes67,
}

/// Audio Engine
pub struct AudioEngine {
    host: Host,
    sample_rate: u32,
    buffer_size: usize,
    running: Arc<AtomicBool>,
    input_stream: Option<Stream>,
    output_stream: Option<Stream>,
    mixer: Option<Arc<Mixer>>,
    master: Option<Arc<MasterSection>>,
    /// AES67 Backend for network audio
    aes67_backend: Option<Aes67Backend>,
    /// Current audio source
    audio_source: AudioSource,
    /// Command receiver for async control
    command_rx: Option<mpsc::Receiver<AudioCommand>>,
}

/// Handle für Befehle an die AudioEngine (thread-safe, cloneable)
#[derive(Clone)]
pub struct AudioCommandSender {
    tx: mpsc::Sender<AudioCommand>,
}

impl AudioCommandSender {
    /// Subscribe to an AES67 stream
    pub async fn subscribe_stream(&self, stream_id: String, start_channel: Option<u32>) -> Result<Aes67SubscribeResult, String> {
        let (response_tx, response_rx) = tokio::sync::oneshot::channel();
        
        self.tx.send(AudioCommand::SubscribeStream {
            stream_id,
            start_channel,
            response: response_tx,
        }).await.map_err(|e| format!("Command send error: {}", e))?;
        
        response_rx.await.map_err(|e| format!("Response error: {}", e))?
    }
    
    /// Unsubscribe from an AES67 stream
    pub async fn unsubscribe_stream(&self, stream_id: String) -> Result<(), String> {
        let (response_tx, response_rx) = tokio::sync::oneshot::channel();
        
        self.tx.send(AudioCommand::UnsubscribeStream {
            stream_id,
            response: response_tx,
        }).await.map_err(|e| format!("Command send error: {}", e))?;
        
        response_rx.await.map_err(|e| format!("Response error: {}", e))?
    }
}

impl AudioEngine {
    /// Neue Audio Engine erstellen
    pub fn new(sample_rate: u32, buffer_size: usize) -> Self {
        let host = cpal::default_host();
        
        Self {
            host,
            sample_rate,
            buffer_size,
            running: Arc::new(AtomicBool::new(false)),
            input_stream: None,
            output_stream: None,
            mixer: None,
            master: None,
            aes67_backend: None,
            audio_source: AudioSource::Local,
            command_rx: None,
        }
    }
    
    /// Command-Sender erstellen für API/WebSocket Zugriff
    pub fn create_command_sender(&mut self) -> AudioCommandSender {
        let (tx, rx) = mpsc::channel(32);
        self.command_rx = Some(rx);
        AudioCommandSender { tx }
    }
    
    /// Pending Commands verarbeiten (sollte regelmäßig aufgerufen werden)
    pub fn process_commands(&mut self) {
        // Sammle alle Commands erst, dann verarbeite sie
        let mut commands = Vec::new();
        
        if let Some(ref mut rx) = self.command_rx {
            while let Ok(cmd) = rx.try_recv() {
                commands.push(cmd);
            }
        }
        
        // Jetzt können wir self frei mutieren
        for cmd in commands {
            match cmd {
                AudioCommand::SubscribeStream { stream_id, start_channel, response } => {
                    let result = self.handle_subscribe_stream(&stream_id, start_channel);
                    let _ = response.send(result);
                }
                AudioCommand::UnsubscribeStream { stream_id, response } => {
                    let result = self.handle_unsubscribe_stream(&stream_id);
                    let _ = response.send(result);
                }
            }
        }
    }
    
    /// Subscribe to stream (internal)
    fn handle_subscribe_stream(&mut self, stream_id: &str, start_channel: Option<u32>) -> Result<Aes67SubscribeResult, String> {
        let backend = self.aes67_backend.as_ref()
            .ok_or("AES67 not initialized")?;
        
        // Stream in Discovery finden
        let stream = backend.sap_discovery()
            .streams()
            .into_iter()
            .find(|s| s.session_id == stream_id)
            .ok_or_else(|| format!("Stream '{}' not found", stream_id))?;
        
        let device = NetworkDevice {
            id: stream.session_id.clone(),
            name: stream.name.clone(),
            device_type: crate::network_audio::NetworkDeviceType::Transmitter,
            channels: stream.channels as u32,
            sample_rate: stream.sample_rate,
            ip_address: Some(stream.origin.clone()),
            multicast_group: Some(stream.multicast_addr.to_string()),
        };
        
        let start_ch = start_channel.unwrap_or(0);
        
        // Connect to stream
        self.connect_aes67(&device)
            .map_err(|e| format!("Connection failed: {}", e))?;
        
        info!("🔊 Subscribed to '{}' ({} channels) -> channel {}", 
              stream.name, stream.channels, start_ch);
        
        Ok(Aes67SubscribeResult {
            stream_id: stream.session_id,
            stream_name: stream.name,
            channels: stream.channels,
            start_channel: start_ch,
        })
    }
    
    /// Unsubscribe from stream (internal)
    fn handle_unsubscribe_stream(&mut self, _stream_id: &str) -> Result<(), String> {
        self.disconnect_aes67()
            .map_err(|e| format!("Disconnect failed: {}", e))?;
        
        info!("🔇 Unsubscribed from stream");
        Ok(())
    }

    /// Mixer-Referenz setzen
    pub fn set_mixer(&mut self, mixer: Arc<Mixer>) {
        self.mixer = Some(mixer);
    }

    /// Master-Sektion setzen
    pub fn set_master(&mut self, master: Arc<MasterSection>) {
        self.master = Some(master);
    }

    /// Initialize AES67 backend
    pub fn init_aes67(&mut self, config: Option<Aes67Config>) -> Result<()> {
        info!("🌐 Initializing AES67 backend...");
        
        let config = config.unwrap_or_default();
        let mut backend = Aes67Backend::with_config(config);
        backend.init()?;
        
        self.aes67_backend = Some(backend);
        
        Ok(())
    }

    /// Set audio source
    pub fn set_audio_source(&mut self, source: AudioSource) {
        self.audio_source = source;
        info!("🔊 Audio source set to: {:?}", source);
    }

    /// Get current audio source
    pub fn audio_source(&self) -> AudioSource {
        self.audio_source
    }

    /// Get AES67 backend reference
    pub fn aes67_backend(&self) -> Option<&Aes67Backend> {
        self.aes67_backend.as_ref()
    }

    /// Get AES67 backend mutable reference
    pub fn aes67_backend_mut(&mut self) -> Option<&mut Aes67Backend> {
        self.aes67_backend.as_mut()
    }

    /// Get SAP discovery reference (thread-safe, can be shared with API)
    pub fn sap_discovery(&self) -> Option<Arc<SapDiscovery>> {
        self.aes67_backend.as_ref().map(|b| b.sap_discovery())
    }

    /// Get PTP clock reference (thread-safe, can be shared with API)
    pub fn ptp_clock(&self) -> Option<Arc<PtpClock>> {
        self.aes67_backend.as_ref().map(|b| b.ptp_clock())
    }

    /// Discover AES67 devices
    pub fn discover_aes67_devices(&self) -> Result<Vec<NetworkDevice>> {
        if let Some(backend) = &self.aes67_backend {
            backend.discover()
        } else {
            Ok(vec![])
        }
    }

    /// Connect to AES67 device
    pub fn connect_aes67(&mut self, device: &NetworkDevice) -> Result<()> {
        if let Some(backend) = &mut self.aes67_backend {
            backend.connect(device)?;
            self.audio_source = AudioSource::Aes67;
            Ok(())
        } else {
            Err(anyhow!("AES67 backend not initialized"))
        }
    }

    /// Disconnect from AES67 device
    pub fn disconnect_aes67(&mut self) -> Result<()> {
        if let Some(backend) = &mut self.aes67_backend {
            backend.disconnect()?;
            self.audio_source = AudioSource::Local;
            Ok(())
        } else {
            Ok(())
        }
    }

    /// Check if AES67 PTP is synchronized
    pub fn is_aes67_synchronized(&self) -> bool {
        self.aes67_backend.as_ref()
            .map(|b| b.is_ptp_synchronized())
            .unwrap_or(false)
    }

    /// Engine starten
    pub fn start(&mut self) -> Result<()> {
        info!("🔊 Audio Engine startet...");
        info!("   Sample Rate: {} Hz", self.sample_rate);
        info!("   Buffer Size: {} samples", self.buffer_size);
        
        let input_device = self.host.default_input_device()
            .ok_or_else(|| anyhow!("Kein Eingabegerät gefunden"))?;
        let output_device = self.host.default_output_device()
            .ok_or_else(|| anyhow!("Kein Ausgabegerät gefunden"))?;
        
        info!("   Input:  {}", input_device.name().unwrap_or_default());
        info!("   Output: {}", output_device.name().unwrap_or_default());
        
        let config = StreamConfig {
            channels: 2,
            sample_rate: cpal::SampleRate(self.sample_rate),
            buffer_size: cpal::BufferSize::Fixed(self.buffer_size as u32),
        };
        
        // Ringbuffer für Audio-Daten zwischen Input und Output
        let (producer, consumer) = create_ring_buffer(self.buffer_size * 4);
        
        let mixer = self.mixer.clone();
        let master = self.master.clone();
        let sample_rate = self.sample_rate as f32;
        let running = self.running.clone();
        
        // Input Stream
        let input_stream = input_device.build_input_stream(
            &config,
            move |data: &[f32], _: &cpal::InputCallbackInfo| {
                // Input-Samples in Ringbuffer schreiben
                for sample in data {
                    let _ = producer.try_push(*sample);
                }
            },
            |err| error!("Input Stream Fehler: {}", err),
            None,
        )?;
        
        // Oszillator-Phase für Master (muss im Closure bleiben)
        let mut osc_phase = 0.0f32;
        
        // Output Stream
        let output_stream = output_device.build_output_stream(
            &config,
            move |data: &mut [f32], _: &cpal::OutputCallbackInfo| {
                // Samples aus Ringbuffer lesen und verarbeiten
                for sample in data.iter_mut() {
                    *sample = consumer.try_pop().unwrap_or(0.0);
                }
                
                // Channel-Mixer-Processing anwenden
                if let Some(ref mixer) = mixer {
                    process_channels(data, mixer, 2);
                }
                
                // Master-Processing anwenden (Limiter, Mono, Oszillator etc.)
                if let Some(ref master) = master {
                    process_master(data, master, &mut osc_phase, sample_rate, 2);
                }
            },
            |err| error!("Output Stream Fehler: {}", err),
            None,
        )?;
        
        input_stream.play()?;
        output_stream.play()?;
        
        self.input_stream = Some(input_stream);
        self.output_stream = Some(output_stream);
        self.running.store(true, Ordering::SeqCst);
        
        info!("✅ Audio Engine läuft");
        Ok(())
    }

    /// Engine stoppen
    pub fn stop(&mut self) -> Result<()> {
        info!("🔇 Audio Engine stoppt...");
        
        self.running.store(false, Ordering::SeqCst);
        
        // Streams droppen (stoppt automatisch)
        self.input_stream = None;
        self.output_stream = None;
        
        info!("✅ Audio Engine gestoppt");
        Ok(())
    }

    /// Prüfen ob Engine läuft
    pub fn is_running(&self) -> bool {
        self.running.load(Ordering::SeqCst)
    }

    /// Verfügbare Audio-Geräte auflisten
    pub fn list_devices(&self) -> Vec<AudioDeviceInfo> {
        let mut devices = Vec::new();
        
        // Input-Geräte
        if let Ok(inputs) = self.host.input_devices() {
            let default_input = self.host.default_input_device()
                .and_then(|d| d.name().ok());
            
            for device in inputs {
                if let Ok(name) = device.name() {
                    devices.push(AudioDeviceInfo {
                        name: name.clone(),
                        is_input: true,
                        is_output: false,
                        is_default: default_input.as_ref() == Some(&name),
                    });
                }
            }
        }
        
        // Output-Geräte
        if let Ok(outputs) = self.host.output_devices() {
            let default_output = self.host.default_output_device()
                .and_then(|d| d.name().ok());
            
            for device in outputs {
                if let Ok(name) = device.name() {
                    // Prüfen ob schon als Input vorhanden
                    if let Some(existing) = devices.iter_mut().find(|d| d.name == name) {
                        existing.is_output = true;
                        if default_output.as_ref() == Some(&name) {
                            existing.is_default = true;
                        }
                    } else {
                        devices.push(AudioDeviceInfo {
                            name: name.clone(),
                            is_input: false,
                            is_output: true,
                            is_default: default_output.as_ref() == Some(&name),
                        });
                    }
                }
            }
        }
        
        devices
    }
}

/// Einfacher Lock-Free Ringbuffer
struct RingBuffer<T> {
    buffer: Vec<std::sync::atomic::AtomicU32>,
    head: std::sync::atomic::AtomicUsize,
    tail: std::sync::atomic::AtomicUsize,
    capacity: usize,
    _marker: std::marker::PhantomData<T>,
}

struct Producer {
    buffer: Arc<std::sync::RwLock<Vec<f32>>>,
    write_pos: Arc<std::sync::atomic::AtomicUsize>,
    capacity: usize,
}

struct Consumer {
    buffer: Arc<std::sync::RwLock<Vec<f32>>>,
    read_pos: Arc<std::sync::atomic::AtomicUsize>,
    capacity: usize,
}

impl Producer {
    fn try_push(&self, value: f32) -> Result<(), ()> {
        if let Ok(mut buf) = self.buffer.try_write() {
            let pos = self.write_pos.fetch_add(1, Ordering::SeqCst) % self.capacity;
            buf[pos] = value;
            Ok(())
        } else {
            Err(())
        }
    }
}

impl Consumer {
    fn try_pop(&self) -> Option<f32> {
        if let Ok(buf) = self.buffer.try_read() {
            let pos = self.read_pos.fetch_add(1, Ordering::SeqCst) % self.capacity;
            Some(buf[pos])
        } else {
            None
        }
    }
}

fn create_ring_buffer(capacity: usize) -> (Producer, Consumer) {
    let buffer = Arc::new(std::sync::RwLock::new(vec![0.0f32; capacity]));
    let write_pos = Arc::new(std::sync::atomic::AtomicUsize::new(0));
    let read_pos = Arc::new(std::sync::atomic::AtomicUsize::new(0));
    
    (
        Producer {
            buffer: buffer.clone(),
            write_pos: write_pos.clone(),
            capacity,
        },
        Consumer {
            buffer,
            read_pos,
            capacity,
        },
    )
}

/// Audio-Processing im Output-Callback (Channel-Mixer)
fn process_channels(output: &mut [f32], mixer: &Mixer, channels: usize) {
    let channel_states = mixer.get_all_channels();
    
    // Für jeden Sample-Frame
    let frames = output.len() / channels;
    for frame in 0..frames {
        for ch in 0..channels.min(channel_states.len()) {
            let idx = frame * channels + ch;
            if idx < output.len() {
                let state = &channel_states[ch];
                
                // Fader und Mute anwenden
                let gain = if state.mute {
                    0.0
                } else {
                    fader_to_gain(state.fader)
                };
                
                output[idx] *= gain;
                
                // Meter aktualisieren (Peak)
                mixer.update_meter(ch as u32, output[idx].abs());
            }
        }
    }
}

/// Master-Processing im Output-Callback (Limiter, Mono, Oszillator, Gain)
fn process_master(output: &mut [f32], master: &MasterSection, osc_phase: &mut f32, sample_rate: f32, channels: usize) {
    // Verarbeite Stereo-Paare (2 Kanäle)
    if channels >= 2 {
        let frames = output.len() / channels;
        
        for frame in 0..frames {
            let left_idx = frame * channels;
            let right_idx = frame * channels + 1;
            
            if left_idx < output.len() && right_idx < output.len() {
                let left = output[left_idx];
                let right = output[right_idx];
                
                // MasterSection-Processing anwenden (Oszillator, Mono, Gain, Limiter)
                let (out_left, out_right) = master.process(left, right, osc_phase, sample_rate);
                
                output[left_idx] = out_left;
                output[right_idx] = out_right;
            }
        }
    } else if channels == 1 {
        // Mono-Processing
        for idx in 0..output.len() {
            let sample = output[idx];
            let (out_left, _) = master.process(sample, sample, osc_phase, sample_rate);
            output[idx] = out_left;
        }
    }
}

/// Fader-Wert (0.0-1.25) zu linearem Gain konvertieren
fn fader_to_gain(fader: f32) -> f32 {
    if fader < 0.001 {
        0.0 // -∞ dB
    } else if fader <= 0.75 {
        // 0 bis 0.75 -> -∞ bis 0 dB (exponentiell)
        let normalized = fader / 0.75;
        normalized * normalized
    } else {
        // 0.75 bis 1.25 -> 0 bis +10 dB
        let db = ((fader - 0.75) / 0.25) * 10.0;
        10.0_f32.powf(db / 20.0)
    }
}

/// Audio-Callback Funktion (wird vom Audio-Thread aufgerufen)
/// 
/// Diese Funktion wird in Echtzeit aufgerufen und darf NICHT:
/// - Speicher allokieren
/// - Locks verwenden (außer try_lock)
/// - Auf Netzwerk/Dateien zugreifen
/// - Logging verwenden
pub fn audio_callback(
    input: &[f32],
    output: &mut [f32],
    _mixer: &Arc<crate::mixer::Mixer>,
    channels: usize,
) {
    // Einfacher Passthrough für den Anfang
    // TODO: Mixer-Processing implementieren
    
    let samples_per_channel = input.len() / channels;
    
    for sample_idx in 0..samples_per_channel {
        for ch in 0..channels.min(output.len() / samples_per_channel) {
            let in_idx = sample_idx * channels + ch;
            let out_idx = sample_idx * (output.len() / samples_per_channel) + ch;
            
            if in_idx < input.len() && out_idx < output.len() {
                output[out_idx] = input[in_idx];
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_engine_creation() {
        let engine = AudioEngine::new(48000, 256);
        assert_eq!(engine.sample_rate, 48000);
        assert_eq!(engine.buffer_size, 256);
        assert!(!engine.is_running());
    }
}
