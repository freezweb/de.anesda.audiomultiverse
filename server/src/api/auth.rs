//! API Authentifizierung
//! 
//! Token-basierte Authentifizierung für die REST/WebSocket API

#![allow(dead_code)]

use std::collections::HashMap;
use std::sync::RwLock;
use std::time::{Duration, Instant};
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use axum::{
    extract::Request,
    http::{HeaderMap, StatusCode},
    middleware::Next,
    response::Response,
};

/// Benutzer-Rolle
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum UserRole {
    /// Voller Zugriff
    Admin,
    /// Kann alles steuern, aber keine System-Einstellungen
    Operator,
    /// Nur lesen (Monitoring)
    Viewer,
}

impl UserRole {
    /// Kann schreibend auf Mixer zugreifen
    pub fn can_control(&self) -> bool {
        matches!(self, UserRole::Admin | UserRole::Operator)
    }
    
    /// Kann System-Einstellungen ändern
    pub fn can_configure(&self) -> bool {
        matches!(self, UserRole::Admin)
    }
}

/// Benutzer
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct User {
    pub id: String,
    pub username: String,
    #[serde(skip_serializing)]
    pub password_hash: String,
    pub role: UserRole,
    pub display_name: Option<String>,
    pub enabled: bool,
}

/// Auth Token
#[derive(Debug, Clone)]
pub struct AuthToken {
    pub token: String,
    pub user_id: String,
    pub role: UserRole,
    pub created_at: Instant,
    pub expires_at: Instant,
    pub client_ip: Option<String>,
}

impl AuthToken {
    pub fn is_expired(&self) -> bool {
        Instant::now() > self.expires_at
    }
}

/// Auth Token Response (für API)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuthTokenResponse {
    pub token: String,
    pub expires_in_secs: u64,
    pub role: UserRole,
    pub username: String,
}

/// Login Request
#[derive(Debug, Deserialize)]
pub struct LoginRequest {
    pub username: String,
    pub password: String,
}

/// Auth Manager
pub struct AuthManager {
    /// Registrierte Benutzer
    users: RwLock<HashMap<String, User>>,
    /// Aktive Tokens
    tokens: RwLock<HashMap<String, AuthToken>>,
    /// Token-Gültigkeitsdauer
    token_lifetime: Duration,
    /// Auth aktiviert
    enabled: bool,
    /// API Key (für Hausautomation)
    api_keys: RwLock<HashMap<String, ApiKey>>,
}

/// API Key für Hausautomation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApiKey {
    pub key: String,
    pub name: String,
    pub role: UserRole,
    pub enabled: bool,
    pub created_at: chrono::DateTime<chrono::Utc>,
}

impl AuthManager {
    pub fn new(enabled: bool) -> Self {
        let mut manager = Self {
            users: RwLock::new(HashMap::new()),
            tokens: RwLock::new(HashMap::new()),
            token_lifetime: Duration::from_secs(3600 * 24), // 24 Stunden
            enabled,
            api_keys: RwLock::new(HashMap::new()),
        };
        
        // Default Admin-User erstellen wenn aktiviert
        if enabled {
            manager.create_default_admin();
        }
        
        manager
    }
    
    /// Default Admin erstellen
    fn create_default_admin(&mut self) {
        let admin = User {
            id: Uuid::new_v4().to_string(),
            username: "admin".to_string(),
            password_hash: hash_password("admin"), // ÄNDERN IN PRODUKTION!
            role: UserRole::Admin,
            display_name: Some("Administrator".to_string()),
            enabled: true,
        };
        
        self.users.write().unwrap().insert(admin.username.clone(), admin);
    }
    
    /// Auth aktiviert?
    pub fn is_enabled(&self) -> bool {
        self.enabled
    }
    
    /// Login
    pub fn login(&self, username: &str, password: &str, client_ip: Option<String>) -> Result<AuthTokenResponse, AuthError> {
        let users = self.users.read().unwrap();
        
        let user = users.get(username)
            .ok_or(AuthError::InvalidCredentials)?;
        
        if !user.enabled {
            return Err(AuthError::UserDisabled);
        }
        
        if !verify_password(password, &user.password_hash) {
            return Err(AuthError::InvalidCredentials);
        }
        
        // Token erstellen
        let now = Instant::now();
        let token = AuthToken {
            token: generate_token(),
            user_id: user.id.clone(),
            role: user.role,
            created_at: now,
            expires_at: now + self.token_lifetime,
            client_ip,
        };
        
        let response = AuthTokenResponse {
            token: token.token.clone(),
            expires_in_secs: self.token_lifetime.as_secs(),
            role: user.role,
            username: user.username.clone(),
        };
        
        // Token speichern
        self.tokens.write().unwrap().insert(token.token.clone(), token);
        
        Ok(response)
    }
    
    /// Token validieren
    pub fn validate_token(&self, token: &str) -> Result<AuthToken, AuthError> {
        if !self.enabled {
            // Auth deaktiviert = alles erlaubt
            return Ok(AuthToken {
                token: String::new(),
                user_id: String::new(),
                role: UserRole::Admin,
                created_at: Instant::now(),
                expires_at: Instant::now() + Duration::from_secs(3600),
                client_ip: None,
            });
        }
        
        let tokens = self.tokens.read().unwrap();
        
        let auth_token = tokens.get(token)
            .ok_or(AuthError::InvalidToken)?;
        
        if auth_token.is_expired() {
            return Err(AuthError::TokenExpired);
        }
        
        Ok(auth_token.clone())
    }
    
    /// API Key validieren
    pub fn validate_api_key(&self, key: &str) -> Result<UserRole, AuthError> {
        if !self.enabled {
            return Ok(UserRole::Admin);
        }
        
        let keys = self.api_keys.read().unwrap();
        
        let api_key = keys.get(key)
            .ok_or(AuthError::InvalidApiKey)?;
        
        if !api_key.enabled {
            return Err(AuthError::ApiKeyDisabled);
        }
        
        Ok(api_key.role)
    }
    
    /// Logout (Token invalidieren)
    pub fn logout(&self, token: &str) {
        self.tokens.write().unwrap().remove(token);
    }
    
    /// Benutzer erstellen
    pub fn create_user(&self, username: &str, password: &str, role: UserRole) -> Result<User, AuthError> {
        let mut users = self.users.write().unwrap();
        
        if users.contains_key(username) {
            return Err(AuthError::UserExists);
        }
        
        let user = User {
            id: Uuid::new_v4().to_string(),
            username: username.to_string(),
            password_hash: hash_password(password),
            role,
            display_name: None,
            enabled: true,
        };
        
        users.insert(username.to_string(), user.clone());
        
        Ok(user)
    }
    
    /// API Key erstellen
    pub fn create_api_key(&self, name: &str, role: UserRole) -> ApiKey {
        let key = ApiKey {
            key: generate_api_key(),
            name: name.to_string(),
            role,
            enabled: true,
            created_at: chrono::Utc::now(),
        };
        
        self.api_keys.write().unwrap().insert(key.key.clone(), key.clone());
        
        key
    }
    
    /// Abgelaufene Tokens aufräumen
    pub fn cleanup_expired_tokens(&self) {
        let mut tokens = self.tokens.write().unwrap();
        tokens.retain(|_, t| !t.is_expired());
    }
    
    /// Alle Benutzer auflisten
    pub fn list_users(&self) -> Vec<User> {
        self.users.read().unwrap().values().cloned().collect()
    }
    
    /// Alle API Keys auflisten
    pub fn list_api_keys(&self) -> Vec<ApiKey> {
        self.api_keys.read().unwrap().values().cloned().collect()
    }
}

/// Auth Fehler
#[derive(Debug, Clone)]
pub enum AuthError {
    InvalidCredentials,
    InvalidToken,
    TokenExpired,
    UserDisabled,
    UserExists,
    InvalidApiKey,
    ApiKeyDisabled,
    InsufficientPermissions,
}

impl std::fmt::Display for AuthError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AuthError::InvalidCredentials => write!(f, "Ungültige Anmeldedaten"),
            AuthError::InvalidToken => write!(f, "Ungültiger Token"),
            AuthError::TokenExpired => write!(f, "Token abgelaufen"),
            AuthError::UserDisabled => write!(f, "Benutzer deaktiviert"),
            AuthError::UserExists => write!(f, "Benutzer existiert bereits"),
            AuthError::InvalidApiKey => write!(f, "Ungültiger API Key"),
            AuthError::ApiKeyDisabled => write!(f, "API Key deaktiviert"),
            AuthError::InsufficientPermissions => write!(f, "Keine Berechtigung"),
        }
    }
}

impl std::error::Error for AuthError {}

/// Passwort hashen (vereinfacht - in Produktion bcrypt/argon2 verwenden!)
fn hash_password(password: &str) -> String {
    use std::collections::hash_map::DefaultHasher;
    use std::hash::{Hash, Hasher};
    
    let mut hasher = DefaultHasher::new();
    password.hash(&mut hasher);
    "SIMPLE_HASH:".to_string() + &hasher.finish().to_string()
}

/// Passwort verifizieren
fn verify_password(password: &str, hash: &str) -> bool {
    hash_password(password) == hash
}

/// Token generieren
fn generate_token() -> String {
    format!("tok_{}", Uuid::new_v4().to_string().replace("-", ""))
}

/// API Key generieren
fn generate_api_key() -> String {
    format!("amv_{}", Uuid::new_v4().to_string().replace("-", ""))
}

/// Token aus Header extrahieren
pub fn extract_token(headers: &HeaderMap) -> Option<String> {
    headers.get("Authorization")
        .and_then(|v| v.to_str().ok())
        .and_then(|v| {
            if v.starts_with("Bearer ") {
                Some(v[7..].to_string())
            } else {
                None
            }
        })
}

/// API Key aus Header extrahieren
pub fn extract_api_key(headers: &HeaderMap) -> Option<String> {
    headers.get("X-API-Key")
        .and_then(|v| v.to_str().ok())
        .map(|v| v.to_string())
}

/// Auth Middleware für Axum
pub async fn auth_middleware(
    headers: HeaderMap,
    request: Request,
    next: Next,
) -> Result<Response, StatusCode> {
    // Token oder API Key extrahieren
    let _auth = if let Some(token) = extract_token(&headers) {
        Some(token)
    } else {
        extract_api_key(&headers)
    };
    
    // Hier würde normalerweise die Validierung stattfinden
    // Für jetzt durchreichen
    
    Ok(next.run(request).await)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_auth_manager() {
        let manager = AuthManager::new(true);
        
        // Default Admin sollte existieren
        let result = manager.login("admin", "admin", None);
        assert!(result.is_ok());
        
        let token = result.unwrap();
        assert!(token.token.starts_with("tok_"));
        assert_eq!(token.role, UserRole::Admin);
    }

    #[test]
    fn test_invalid_login() {
        let manager = AuthManager::new(true);
        
        let result = manager.login("admin", "wrong_password", None);
        assert!(matches!(result, Err(AuthError::InvalidCredentials)));
    }

    #[test]
    fn test_user_roles() {
        assert!(UserRole::Admin.can_control());
        assert!(UserRole::Admin.can_configure());
        
        assert!(UserRole::Operator.can_control());
        assert!(!UserRole::Operator.can_configure());
        
        assert!(!UserRole::Viewer.can_control());
        assert!(!UserRole::Viewer.can_configure());
    }

    #[test]
    fn test_api_key() {
        let manager = AuthManager::new(true);
        
        let key = manager.create_api_key("Home Assistant", UserRole::Operator);
        assert!(key.key.starts_with("amv_"));
        
        let role = manager.validate_api_key(&key.key);
        assert!(matches!(role, Ok(UserRole::Operator)));
    }
}
