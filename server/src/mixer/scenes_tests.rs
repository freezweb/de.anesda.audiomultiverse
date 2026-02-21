//! Unit Tests für das Scene-System
//!
//! Testet Szenen-Speicherung, Recall und Crossfade

use super::scenes::*;
use audiomultiverse_protocol::MixerState;
use tempfile::tempdir;

fn create_test_mixer_state() -> MixerState {
    MixerState {
        channels: vec![],
        routing: vec![],
        input_count: 32,
        output_count: 32,
    }
}

#[test]
fn test_scene_manager_creation() {
    let temp = tempdir().unwrap();
    let path = temp.path().to_str().unwrap();
    
    let manager = SceneManager::new(path);
    assert!(manager.list_scenes().is_empty());
}

#[test]
fn test_create_scene() {
    let temp = tempdir().unwrap();
    let path = temp.path().to_str().unwrap();
    
    let mut manager = SceneManager::new(path);
    let mixer_state = create_test_mixer_state();
    
    let scene = manager.create_scene("Test Scene", &mixer_state, None);
    
    assert_eq!(scene.metadata.name, "Test Scene");
    assert!(!scene.metadata.id.is_empty());
}

#[test]
fn test_scene_list() {
    let temp = tempdir().unwrap();
    let path = temp.path().to_str().unwrap();
    
    let mut manager = SceneManager::new(path);
    let mixer_state = create_test_mixer_state();
    
    manager.create_scene("Scene 1", &mixer_state, None);
    manager.create_scene("Scene 2", &mixer_state, None);
    
    let scenes = manager.list_scenes();
    assert_eq!(scenes.len(), 2);
}

#[test]
fn test_get_scene() {
    let temp = tempdir().unwrap();
    let path = temp.path().to_str().unwrap();
    
    let mut manager = SceneManager::new(path);
    let mixer_state = create_test_mixer_state();
    
    let created = manager.create_scene("Find Me", &mixer_state, None);
    let scene_id = created.metadata.id.clone();
    
    let found = manager.get_scene(&scene_id);
    assert!(found.is_some());
    assert_eq!(found.unwrap().metadata.name, "Find Me");
}

#[test]
fn test_delete_scene() {
    let temp = tempdir().unwrap();
    let path = temp.path().to_str().unwrap();
    
    let mut manager = SceneManager::new(path);
    let mixer_state = create_test_mixer_state();
    
    let scene = manager.create_scene("Delete Me", &mixer_state, None);
    let scene_id = scene.metadata.id.clone();
    
    assert!(manager.get_scene(&scene_id).is_some());
    
    let result = manager.delete_scene(&scene_id);
    assert!(result.is_ok());
    
    assert!(manager.get_scene(&scene_id).is_none());
}

#[test]
fn test_crossfade_curve_types() {
    // Teste dass alle Varianten existieren
    let _linear = CrossfadeCurve::Linear;
    let _log = CrossfadeCurve::Logarithmic;
    let _scurve = CrossfadeCurve::SCurve;
    let _cosine = CrossfadeCurve::Cosine;
}
