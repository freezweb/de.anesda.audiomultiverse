//! Szenen-Management
//!
//! Speichern und Laden von Mixer-Szenen

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
}

impl SceneManager {
    /// Neuen Manager erstellen
    pub fn new(storage_path: &str) -> Self {
        let mut manager = Self {
            storage_path: storage_path.to_string(),
            scenes: HashMap::new(),
            current_scene: None,
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
