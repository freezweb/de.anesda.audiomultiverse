//! Szenen-Management
//!
//! Speichern und Laden von Mixer-Szenen

#![allow(dead_code)]

use std::collections::HashMap;
use std::path::Path;
use std::fs;
use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};
use tracing::{info, error};
use uuid::Uuid;

use audiomultiverse_protocol::{ChannelState, MixerState};

/// Szenen-Metadaten
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SceneMetadata {
    /// Eindeutige ID
    pub id: String,
    
    /// Szenen-Name
    pub name: String,
    
    /// Beschreibung
    pub description: Option<String>,
    
    /// Erstellungsdatum
    pub created_at: DateTime<Utc>,
    
    /// Letztes Änderungsdatum
    pub modified_at: DateTime<Utc>,
    
    /// Farbcode für UI
    pub color: Option<String>,
    
    /// Kategorie/Gruppe
    pub category: Option<String>,
    
    /// Autor
    pub author: Option<String>,
}

/// Recall-Filter - welche Teile sollen geladen werden
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct RecallFilter {
    /// Fader-Werte laden
    pub faders: bool,
    
    /// Mutes laden
    pub mutes: bool,
    
    /// Solos laden
    pub solos: bool,
    
    /// Pan laden
    pub pans: bool,
    
    /// EQ laden
    pub eq: bool,
    
    /// Routing laden
    pub routing: bool,
    
    /// Namen laden
    pub names: bool,
    
    /// Nur bestimmte Kanäle (leer = alle)
    pub channels: Vec<u32>,
}

impl RecallFilter {
    /// Alles laden
    pub fn all() -> Self {
        Self {
            faders: true,
            mutes: true,
            solos: true,
            pans: true,
            eq: true,
            routing: true,
            names: true,
            channels: vec![],
        }
    }
    
    /// Nur Fader und Mutes
    pub fn faders_only() -> Self {
        Self {
            faders: true,
            mutes: true,
            ..Default::default()
        }
    }
    
    /// Nur Routing
    pub fn routing_only() -> Self {
        Self {
            routing: true,
            ..Default::default()
        }
    }
}

/// EQ-Einstellungen pro Band
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct EqBand {
    /// Frequenz in Hz
    pub frequency: f32,
    
    /// Gain in dB (-15 bis +15)
    pub gain: f32,
    
    /// Q-Faktor (Bandbreite)
    pub q: f32,
    
    /// Filter-Typ (LowShelf, Peak, HighShelf, LowPass, HighPass)
    pub filter_type: EqFilterType,
    
    /// Band aktiv
    pub enabled: bool,
}

/// EQ Filter-Typ
#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq)]
pub enum EqFilterType {
    #[default]
    Peak,
    LowShelf,
    HighShelf,
    LowPass,
    HighPass,
}

/// Vollständiger Kanal-State für Szene
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SceneChannelState {
    /// Basis-State
    pub base: ChannelState,
    
    /// High-Pass Filter aktiviert
    pub hpf_enabled: bool,
    
    /// High-Pass Frequenz
    pub hpf_frequency: f32,
    
    /// EQ Bänder
    pub eq_bands: Vec<EqBand>,
    
    /// EQ aktiviert
    pub eq_enabled: bool,
    
    /// Phase invertiert
    pub phase_invert: bool,
    
    /// Aux Sends (Aux ID -> Pegel)
    pub aux_sends: HashMap<u32, f32>,
}

/// Vollständige Szene
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Scene {
    /// Metadaten
    pub metadata: SceneMetadata,
    
    /// Kanal-States
    pub channels: Vec<SceneChannelState>,
    
    /// Routing-Matrix
    pub routing: Vec<Vec<f32>>,
    
    /// Master-Einstellungen
    pub master: MasterSettings,
    
    /// Input/Output Anzahl bei Erstellung
    pub input_count: u32,
    pub output_count: u32,
}

/// Master-Einstellungen
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct MasterSettings {
    /// Master-Fader
    pub fader: f32,
    
    /// Master Mute
    pub mute: bool,
    
    /// DIM aktiv
    pub dim: bool,
    
    /// DIM-Pegel in dB
    pub dim_level: f32,
    
    /// Mono-Summe
    pub mono: bool,
    
    /// Limiter aktiviert
    pub limiter_enabled: bool,
    
    /// Limiter Threshold
    pub limiter_threshold: f32,
}

/// Szenen-Manager
pub struct SceneManager {
    /// Speicherort für Szenen
    storage_path: String,
    
    /// Geladene Szenen (Cache)
    scenes: HashMap<String, Scene>,
    
    /// Aktuell aktive Szene
    current_scene: Option<String>,
    
    /// Crossfade State (für zeitbasierte Übergänge)
    crossfade_state: Option<CrossfadeState>,
}

/// Scene Crossfade State
#[derive(Debug, Clone)]
pub struct CrossfadeState {
    /// Start-Scene (von)
    pub from_scene: Option<String>,
    /// Ziel-Scene (nach)
    pub to_scene: String,
    /// Fade-Dauer in Sekunden
    pub duration_secs: f32,
    /// Verstrichene Zeit
    pub elapsed_secs: f32,
    /// Start-Werte (Kanal-ID -> Fader/Pan)
    pub start_values: HashMap<u32, CrossfadeChannelState>,
    /// Ziel-Werte
    pub target_values: HashMap<u32, CrossfadeChannelState>,
    /// Fade-Kurve
    pub curve: CrossfadeCurve,
}

/// Werte die gefadet werden
#[derive(Debug, Clone, Default)]
pub struct CrossfadeChannelState {
    pub fader: f32,
    pub pan: f32,
    pub gain: f32,
}

/// Crossfade-Kurve
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub enum CrossfadeCurve {
    /// Lineare Interpolation
    #[default]
    Linear,
    /// Logarithmisch (Audio-natürlich)
    Logarithmic,
    /// S-Kurve (Ease In/Out)
    SCurve,
    /// Kosinus (sanft)
    Cosine,
}

impl SceneManager {
    /// Neuen Manager erstellen
    pub fn new(storage_path: &str) -> Self {
        let mut manager = Self {
            storage_path: storage_path.to_string(),
            scenes: HashMap::new(),
            current_scene: None,
            crossfade_state: None,
        };
        
        // Szenen vom Disk laden
        manager.load_all_from_disk();
        
        manager
    }
    
    /// Alle Szenen vom Disk laden
    fn load_all_from_disk(&mut self) {
        let path = Path::new(&self.storage_path);
        
        if !path.exists() {
            if let Err(e) = fs::create_dir_all(path) {
                error!("Konnte Szenen-Verzeichnis nicht erstellen: {}", e);
                return;
            }
            info!("📁 Szenen-Verzeichnis erstellt: {}", self.storage_path);
            return;
        }
        
        let entries = match fs::read_dir(path) {
            Ok(e) => e,
            Err(e) => {
                error!("Konnte Szenen-Verzeichnis nicht lesen: {}", e);
                return;
            }
        };
        
        for entry in entries.flatten() {
            let file_path = entry.path();
            
            if file_path.extension().map_or(false, |e| e == "json") {
                match self.load_scene_file(&file_path) {
                    Ok(scene) => {
                        info!("   Szene geladen: {}", scene.metadata.name);
                        self.scenes.insert(scene.metadata.id.clone(), scene);
                    }
                    Err(e) => {
                        error!("Fehler beim Laden von {:?}: {}", file_path, e);
                    }
                }
            }
        }
        
        info!("🎬 {} Szenen geladen", self.scenes.len());
    }
    
    /// Einzelne Szene von Datei laden
    fn load_scene_file(&self, path: &Path) -> anyhow::Result<Scene> {
        let content = fs::read_to_string(path)?;
        let scene: Scene = serde_json::from_str(&content)?;
        Ok(scene)
    }
    
    /// Szene auf Disk speichern
    fn save_scene_to_disk(&self, scene: &Scene) -> anyhow::Result<()> {
        let path = Path::new(&self.storage_path);
        let file_path = path.join(format!("{}.json", scene.metadata.id));
        
        let content = serde_json::to_string_pretty(scene)?;
        fs::write(file_path, content)?;
        
        Ok(())
    }
    
    /// Neue Szene erstellen
    pub fn create_scene(
        &mut self,
        name: &str,
        mixer_state: &MixerState,
        description: Option<String>,
    ) -> Scene {
        let now = Utc::now();
        
        let metadata = SceneMetadata {
            id: Uuid::new_v4().to_string(),
            name: name.to_string(),
            description,
            created_at: now,
            modified_at: now,
            color: None,
            category: None,
            author: None,
        };
        
        // Channel States erweitern
        let channels: Vec<SceneChannelState> = mixer_state.channels
            .iter()
            .map(|ch| SceneChannelState {
                base: ch.clone(),
                hpf_enabled: false,
                hpf_frequency: 80.0,
                eq_bands: Self::default_eq_bands(),
                eq_enabled: true,
                phase_invert: false,
                aux_sends: HashMap::new(),
            })
            .collect();
        
        let scene = Scene {
            metadata,
            channels,
            routing: mixer_state.routing.clone(),
            master: MasterSettings::default(),
            input_count: mixer_state.input_count,
            output_count: mixer_state.output_count,
        };
        
        // In Cache und auf Disk speichern
        self.scenes.insert(scene.metadata.id.clone(), scene.clone());
        
        if let Err(e) = self.save_scene_to_disk(&scene) {
            error!("Fehler beim Speichern der Szene: {}", e);
        }
        
        info!("🎬 Neue Szene erstellt: {}", name);
        
        scene
    }
    
    /// Standard EQ-Bänder
    fn default_eq_bands() -> Vec<EqBand> {
        vec![
            EqBand {
                frequency: 80.0,
                gain: 0.0,
                q: 0.7,
                filter_type: EqFilterType::LowShelf,
                enabled: true,
            },
            EqBand {
                frequency: 500.0,
                gain: 0.0,
                q: 1.0,
                filter_type: EqFilterType::Peak,
                enabled: true,
            },
            EqBand {
                frequency: 2500.0,
                gain: 0.0,
                q: 1.0,
                filter_type: EqFilterType::Peak,
                enabled: true,
            },
            EqBand {
                frequency: 12000.0,
                gain: 0.0,
                q: 0.7,
                filter_type: EqFilterType::HighShelf,
                enabled: true,
            },
        ]
    }
    
    /// Szene aktualisieren
    pub fn update_scene(&mut self, scene: Scene) -> anyhow::Result<()> {
        let mut updated = scene;
        updated.metadata.modified_at = Utc::now();
        
        self.scenes.insert(updated.metadata.id.clone(), updated.clone());
        self.save_scene_to_disk(&updated)?;
        
        info!("🎬 Szene aktualisiert: {}", updated.metadata.name);
        
        Ok(())
    }
    
    /// Szene löschen
    pub fn delete_scene(&mut self, id: &str) -> anyhow::Result<()> {
        if let Some(scene) = self.scenes.remove(id) {
            let path = Path::new(&self.storage_path).join(format!("{}.json", id));
            
            if path.exists() {
                fs::remove_file(path)?;
            }
            
            info!("🎬 Szene gelöscht: {}", scene.metadata.name);
        }
        
        Ok(())
    }
    
    /// Szene abrufen
    pub fn get_scene(&self, id: &str) -> Option<&Scene> {
        self.scenes.get(id)
    }
    
    /// Alle Szenen-Metadaten abrufen
    pub fn list_scenes(&self) -> Vec<SceneMetadata> {
        self.scenes.values()
            .map(|s| s.metadata.clone())
            .collect()
    }
    
    /// Szenen nach Kategorie filtern
    pub fn list_by_category(&self, category: &str) -> Vec<SceneMetadata> {
        self.scenes.values()
            .filter(|s| s.metadata.category.as_deref() == Some(category))
            .map(|s| s.metadata.clone())
            .collect()
    }
    
    /// Aktuelle Szene setzen
    pub fn set_current(&mut self, id: Option<String>) {
        self.current_scene = id;
    }
    
    /// Aktuelle Szene abrufen
    pub fn get_current(&self) -> Option<&Scene> {
        self.current_scene.as_ref().and_then(|id| self.scenes.get(id))
    }
    
    /// Szene in Datei exportieren
    pub fn export_scene(&self, id: &str, export_path: &str) -> anyhow::Result<()> {
        let scene = self.scenes.get(id)
            .ok_or_else(|| anyhow::anyhow!("Szene nicht gefunden: {}", id))?;
        
        let content = serde_json::to_string_pretty(scene)?;
        fs::write(export_path, content)?;
        
        info!("📤 Szene exportiert nach: {}", export_path);
        
        Ok(())
    }
    
    /// Szene aus Datei importieren
    pub fn import_scene(&mut self, import_path: &str) -> anyhow::Result<Scene> {
        let content = fs::read_to_string(import_path)?;
        let mut scene: Scene = serde_json::from_str(&content)?;
        
        // Neue ID vergeben um Konflikte zu vermeiden
        scene.metadata.id = Uuid::new_v4().to_string();
        scene.metadata.modified_at = Utc::now();
        
        self.scenes.insert(scene.metadata.id.clone(), scene.clone());
        self.save_scene_to_disk(&scene)?;
        
        info!("📥 Szene importiert: {}", scene.metadata.name);
        
        Ok(scene)
    }
    
    /// Scene Crossfade starten
    /// 
    /// Startet einen zeitbasierten Übergang von der aktuellen Szene zur Ziel-Szene
    pub fn start_crossfade(
        &mut self,
        to_scene_id: &str,
        duration_secs: f32,
        curve: CrossfadeCurve,
        current_mixer_state: &MixerState,
    ) -> anyhow::Result<()> {
        // Ziel-Szene muss existieren
        let to_scene = self.scenes.get(to_scene_id)
            .ok_or_else(|| anyhow::anyhow!("Ziel-Szene nicht gefunden: {}", to_scene_id))?
            .clone();
        
        // Start-Werte aus aktuellem Mixer-State
        let mut start_values = HashMap::new();
        for (i, ch) in current_mixer_state.channels.iter().enumerate() {
            start_values.insert(i as u32, CrossfadeChannelState {
                fader: ch.fader,
                pan: ch.pan,
                gain: ch.gain,
            });
        }
        
        // Ziel-Werte aus Scene
        let mut target_values = HashMap::new();
        for (i, ch) in to_scene.channels.iter().enumerate() {
            target_values.insert(i as u32, CrossfadeChannelState {
                fader: ch.base.fader,
                pan: ch.base.pan,
                gain: ch.base.gain,
            });
        }
        
        self.crossfade_state = Some(CrossfadeState {
            from_scene: self.current_scene.clone(),
            to_scene: to_scene_id.to_string(),
            duration_secs: duration_secs.max(0.1), // Min 100ms
            elapsed_secs: 0.0,
            start_values,
            target_values,
            curve,
        });
        
        info!("🔀 Crossfade gestartet -> {} ({:.1}s)", to_scene.metadata.name, duration_secs);
        
        Ok(())
    }
    
    /// Crossfade-Progress aktualisieren (sollte regelmäßig aufgerufen werden)
    /// 
    /// Gibt die interpolierten Werte zurück, oder None wenn kein Crossfade aktiv
    pub fn update_crossfade(&mut self, delta_secs: f32) -> Option<HashMap<u32, CrossfadeChannelState>> {
        let state = self.crossfade_state.as_mut()?;
        
        state.elapsed_secs += delta_secs;
        
        // Progress berechnen (0.0 - 1.0)
        let raw_progress = (state.elapsed_secs / state.duration_secs).clamp(0.0, 1.0);
        
        // Kurve anwenden
        let progress = apply_curve(raw_progress, state.curve);
        
        // Interpolierte Werte berechnen
        let mut result = HashMap::new();
        
        for (channel_id, start) in &state.start_values {
            if let Some(target) = state.target_values.get(channel_id) {
                result.insert(*channel_id, CrossfadeChannelState {
                    fader: lerp(start.fader, target.fader, progress),
                    pan: lerp(start.pan, target.pan, progress),
                    gain: lerp(start.gain, target.gain, progress),
                });
            }
        }
        
        // Crossfade beendet?
        if raw_progress >= 1.0 {
            let to_scene = state.to_scene.clone();
            self.crossfade_state = None;
            self.current_scene = Some(to_scene.clone());
            info!("✅ Crossfade abgeschlossen -> {}", to_scene);
        }
        
        Some(result)
    }
    
    /// Crossfade abbrechen
    pub fn cancel_crossfade(&mut self) {
        if self.crossfade_state.is_some() {
            self.crossfade_state = None;
            info!("❌ Crossfade abgebrochen");
        }
    }
    
    /// Crossfade aktiv?
    pub fn is_crossfading(&self) -> bool {
        self.crossfade_state.is_some()
    }
    
    /// Crossfade Progress (0.0 - 1.0)
    pub fn crossfade_progress(&self) -> f32 {
        self.crossfade_state.as_ref()
            .map(|s| (s.elapsed_secs / s.duration_secs).clamp(0.0, 1.0))
            .unwrap_or(0.0)
    }
}

/// Lineare Interpolation
fn lerp(a: f32, b: f32, t: f32) -> f32 {
    a + (b - a) * t
}

/// Kurve auf Progress anwenden
fn apply_curve(t: f32, curve: CrossfadeCurve) -> f32 {
    match curve {
        CrossfadeCurve::Linear => t,
        CrossfadeCurve::Logarithmic => {
            // Logarithmische Kurve (natürlicher für Audio)
            if t <= 0.0 { 0.0 }
            else if t >= 1.0 { 1.0 }
            else { (t.ln() / 10.0_f32.ln() + 1.0).clamp(0.0, 1.0) * t.sqrt() }
        }
        CrossfadeCurve::SCurve => {
            // Smoothstep S-Kurve
            t * t * (3.0 - 2.0 * t)
        }
        CrossfadeCurve::Cosine => {
            // Kosinus-Interpolation
            (1.0 - (t * std::f32::consts::PI).cos()) * 0.5
        }
    }
}

impl Default for SceneManager {
    fn default() -> Self {
        Self::new("./scenes")
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;
    
    fn create_test_mixer_state() -> MixerState {
        MixerState {
            channels: vec![],
            routing: vec![vec![0.0; 32]; 32],
            input_count: 32,
            output_count: 32,
        }
    }
    
    #[test]
    fn test_create_scene() {
        let temp_dir = TempDir::new().unwrap();
        let mut manager = SceneManager::new(temp_dir.path().to_str().unwrap());
        
        let state = create_test_mixer_state();
        let scene = manager.create_scene("Test Scene", &state, None);
        
        assert_eq!(scene.metadata.name, "Test Scene");
        assert_eq!(manager.list_scenes().len(), 1);
    }
    
    #[test]
    fn test_recall_filter() {
        let all = RecallFilter::all();
        assert!(all.faders);
        assert!(all.routing);
        
        let faders = RecallFilter::faders_only();
        assert!(faders.faders);
        assert!(!faders.routing);
    }
}
