//! Audio Engine
//! 
//! Hauptmodul für Audio-Verarbeitung mit cpal

use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use anyhow::{Result, anyhow};
use tracing::{info, warn, error};
use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use cpal::{Device, Host, Stream, StreamConfig, SampleFormat};

use crate::mixer::Mixer;

/// Audio Device Info
#[derive(Debug, Clone)]
pub struct AudioDeviceInfo {
    pub name: String,
    pub is_input: bool,
    pub is_output: bool,
    pub is_default: bool,
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
        }
    }

    /// Mixer-Referenz setzen
    pub fn set_mixer(&mut self, mixer: Arc<Mixer>) {
        self.mixer = Some(mixer);
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
        
        // Output Stream
        let output_stream = output_device.build_output_stream(
            &config,
            move |data: &mut [f32], _: &cpal::OutputCallbackInfo| {
                // Samples aus Ringbuffer lesen und verarbeiten
                for sample in data.iter_mut() {
                    *sample = consumer.try_pop().unwrap_or(0.0);
                }
                
                // Mixer-Processing anwenden
                if let Some(ref mixer) = mixer {
                    process_audio(data, mixer, 2);
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

/// Audio-Processing im Output-Callback
fn process_audio(output: &mut [f32], mixer: &Mixer, channels: usize) {
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
