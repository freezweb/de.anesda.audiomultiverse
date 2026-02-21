//! API Server Module
//! 
//! REST API und WebSocket für Client-Kommunikation
//! Unterstützt Multi-Client-Synchronisation via Broadcast-Channel

mod routes;
mod websocket;
pub mod auth;

pub use routes::start_api_server;
pub use auth::{AuthManager, AuthError, UserRole, AuthToken, User, LoginRequest, AuthTokenResponse};
