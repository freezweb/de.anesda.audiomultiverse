//! Unit Tests für das Bus-System
//!
//! Testet Aux Sends, Groups und Audio-Routing

use super::bus::*;

#[test]
fn test_aux_send_creation() {
    let send = AuxSend::new(1);
    assert_eq!(send.aux_bus_id(), 1);
    assert!((send.level() - 0.0).abs() < 0.001);
}

#[test]
fn test_aux_send_level() {
    let send = AuxSend::new(1);
    send.set_level(0.75);
    
    assert!((send.level() - 0.75).abs() < 0.001);
}

#[test]
fn test_aux_send_pan() {
    let send = AuxSend::new(1);
    send.set_pan(-0.5);
    
    assert!((send.pan() - (-0.5)).abs() < 0.001);
}

#[test]
fn test_aux_bus_creation() {
    let bus = AuxBus::new(1, "FX 1");
    assert_eq!(bus.id, 1);
    assert_eq!(bus.name, "FX 1");
    assert!(!bus.muted);
    assert!((bus.level() - 0.75).abs() < 0.001); // Unity at 0.75
}

#[test]
fn test_aux_bus_level() {
    let bus = AuxBus::new(1, "Aux 1");
    bus.set_level(0.9);
    
    assert!((bus.level() - 0.9).abs() < 0.001);
}

#[test]
fn test_aux_bus_state() {
    let bus = AuxBus::new(2, "Monitor");
    bus.set_level(0.8);
    
    let state = bus.state();
    assert_eq!(state.id, 2);
    assert_eq!(state.name, "Monitor");
    assert!((state.level - 0.8).abs() < 0.001);
}

#[test]
fn test_group_bus_creation() {
    let group = GroupBus::new(1, "Drums");
    assert_eq!(group.id, 1);
    assert_eq!(group.name, "Drums");
    assert!(!group.muted);
    assert!(group.to_master);
}

#[test]
fn test_group_bus_fader() {
    let group = GroupBus::new(0, "Group 1");
    group.set_fader(1.0);
    
    assert!((group.fader() - 1.0).abs() < 0.001);
}

#[test]
fn test_group_bus_pan() {
    let group = GroupBus::new(0, "Group 1");
    group.set_pan(0.5);
    
    assert!((group.pan() - 0.5).abs() < 0.001);
}

#[test]
fn test_group_bus_state() {
    let group = GroupBus::new(3, "Vocals");
    group.set_fader(0.9);
    group.set_pan(-0.3);
    
    let state = group.state();
    assert_eq!(state.id, 3);
    assert_eq!(state.name, "Vocals");
    assert!((state.fader - 0.9).abs() < 0.001);
    assert!((state.pan - (-0.3)).abs() < 0.001);
    assert!(state.to_master);
}

#[test]
fn test_bus_manager_creation() {
    let manager = BusManager::new(8, 4);
    
    assert_eq!(manager.aux_buses.len(), 8);
    assert_eq!(manager.group_buses.len(), 4);
}

#[test]
fn test_bus_manager_init_channels() {
    let mut manager = BusManager::new(8, 4);
    manager.init_channels(16);
    
    // Nach init sollten wir Gruppen-Zuweisungen für 16 Kanäle haben
    let assignments = manager.get_group_assignments(0);
    assert!(assignments.is_some());
    assert_eq!(assignments.unwrap(), [false, false, false, false]);
}

#[test]
fn test_channel_group_assignment() {
    let mut manager = BusManager::new(8, 4);
    manager.init_channels(8);
    
    // Kanal 0 zu Gruppe 0 zuweisen
    manager.assign_channel_to_group(0, 0);
    
    let assignments = manager.get_group_assignments(0).unwrap();
    assert!(assignments[0]);
    assert!(!assignments[1]);
    
    // Wieder entfernen
    manager.remove_channel_from_group(0, 0);
    let assignments = manager.get_group_assignments(0).unwrap();
    assert!(!assignments[0]);
}

#[test]
fn test_aux_bus_get_set() {
    let manager = BusManager::new(4, 2);
    
    let aux_buses = manager.get_aux_buses();
    assert_eq!(aux_buses.len(), 4);
    
    let groups = manager.get_groups();
    assert_eq!(groups.len(), 2);
}
