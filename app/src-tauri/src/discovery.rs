//! mDNS/DNS-SD Client Discovery
//!
//! Ermöglicht das Finden von AudioMultiverse Servern im Netzwerk

use mdns_sd::{ServiceDaemon, ServiceEvent, Receiver};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::time::Duration;
use tracing::{info, debug, warn};

/// Service Type für AudioMultiverse
pub const SERVICE_TYPE: &str = "_audiomultiverse._tcp.local.";

/// Ein im Netzwerk gefundener Server
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiscoveredServer {
    pub name: String,
    pub host: String,
    pub port: u16,
    pub version: String,
    pub addresses: Vec<String>,
}

/// Discovery Client für das Finden von Servern
pub struct DiscoveryClient {
    daemon: ServiceDaemon,
}

impl DiscoveryClient {
    /// Neuen Discovery Client erstellen
    pub fn new() -> anyhow::Result<Self> {
        let daemon = ServiceDaemon::new()?;
        Ok(Self { daemon })
    }

    /// Nach Servern im Netzwerk suchen
    /// 
    /// Sucht für die angegebene Dauer nach Servern und gibt alle gefundenen zurück
    pub fn discover_servers(&self, timeout_ms: u64) -> anyhow::Result<Vec<DiscoveredServer>> {
        let receiver = self.daemon.browse(SERVICE_TYPE)?;
        let servers = Arc::new(Mutex::new(HashMap::<String, DiscoveredServer>::new()));
        let servers_clone = servers.clone();
        
        // Timeout für das Browsen
        let deadline = std::time::Instant::now() + Duration::from_millis(timeout_ms);
        
        debug!("Starte Server-Suche für {}ms...", timeout_ms);
        
        loop {
            if std::time::Instant::now() >= deadline {
                break;
            }
            
            // Receive mit Timeout
            match receiver.recv_timeout(Duration::from_millis(100)) {
                Ok(event) => {
                    match event {
                        ServiceEvent::ServiceResolved(info) => {
                            let name = info.get_fullname().to_string();
                            let host = info.get_hostname().to_string();
                            let port = info.get_port();
                            
                            // Properties auslesen
                            let version = info.get_properties()
                                .get("version")
                                .map(|v| v.val_str().to_string())
                                .unwrap_or_else(|| "unknown".to_string());
                            
                            // IP-Adressen
                            let addresses: Vec<String> = info.get_addresses()
                                .iter()
                                .map(|a| a.to_string())
                                .collect();
                            
                            info!("🔍 Server gefunden: {} @ {}:{}", name, host, port);
                            
                            let server = DiscoveredServer {
                                name: info.get_fullname()
                                    .trim_end_matches(SERVICE_TYPE)
                                    .trim_end_matches('.')
                                    .to_string(),
                                host: if addresses.is_empty() {
                                    host.trim_end_matches('.').to_string()
                                } else {
                                    addresses[0].clone()
                                },
                                port,
                                version,
                                addresses,
                            };
                            
                            servers_clone.lock().unwrap().insert(name, server);
                        }
                        ServiceEvent::ServiceRemoved(_, fullname) => {
                            debug!("Server entfernt: {}", fullname);
                            servers_clone.lock().unwrap().remove(&fullname);
                        }
                        _ => {}
                    }
                }
                Err(std::sync::mpsc::RecvTimeoutError::Timeout) => {
                    // Weiter warten
                }
                Err(e) => {
                    warn!("Discovery Fehler: {:?}", e);
                    break;
                }
            }
        }
        
        // Browse stoppen
        let _ = self.daemon.stop_browse(SERVICE_TYPE);
        
        let result: Vec<DiscoveredServer> = servers.lock().unwrap().values().cloned().collect();
        info!("Discovery abgeschlossen: {} Server gefunden", result.len());
        
        Ok(result)
    }
}
