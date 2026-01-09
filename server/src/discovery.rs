//! mDNS/DNS-SD Discovery
//!
//! Ermöglicht automatische Erkennung von AudioMultiverse Servern im Netzwerk

use anyhow::Result;
use mdns_sd::{ServiceDaemon, ServiceInfo};
use std::collections::HashMap;
use tracing::{info, error};

/// Service Type für AudioMultiverse
pub const SERVICE_TYPE: &str = "_audiomultiverse._tcp.local.";

/// mDNS Service Manager
pub struct DiscoveryService {
    daemon: ServiceDaemon,
    service_fullname: Option<String>,
}

impl DiscoveryService {
    /// Neuen Discovery Service erstellen
    pub fn new() -> Result<Self> {
        let daemon = ServiceDaemon::new()?;
        Ok(Self {
            daemon,
            service_fullname: None,
        })
    }

    /// Server im Netzwerk registrieren
    pub fn register_server(&mut self, name: &str, port: u16, version: &str) -> Result<()> {
        let host_name = hostname::get()
            .map(|h| h.to_string_lossy().to_string())
            .unwrap_or_else(|_| "audiomultiverse".to_string());
        
        let instance_name = format!("{} ({})", name, host_name);
        
        // Properties für den Service
        let mut properties = HashMap::new();
        properties.insert("version".to_string(), version.to_string());
        properties.insert("api".to_string(), "websocket".to_string());
        properties.insert("channels".to_string(), "32x32".to_string());
        
        let service = ServiceInfo::new(
            SERVICE_TYPE,
            &instance_name,
            &format!("{}.local.", host_name),
            (),  // Alle IPs
            port,
            properties,
        )?;
        
        self.service_fullname = Some(service.get_fullname().to_string());
        self.daemon.register(service)?;
        
        info!("🔊 mDNS Service registriert: {} auf Port {}", instance_name, port);
        
        Ok(())
    }

    /// Service deregistrieren
    pub fn unregister(&mut self) -> Result<()> {
        if let Some(fullname) = self.service_fullname.take() {
            self.daemon.unregister(&fullname)?;
            info!("🔇 mDNS Service deregistriert");
        }
        Ok(())
    }
}

impl Drop for DiscoveryService {
    fn drop(&mut self) {
        if let Err(e) = self.unregister() {
            error!("Fehler beim Deregistrieren des mDNS Service: {}", e);
        }
    }
}
