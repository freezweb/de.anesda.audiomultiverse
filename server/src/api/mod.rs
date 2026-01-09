//! API Server Module
//! 
//! REST API und WebSocket für Client-Kommunikation
//! Unterstützt Multi-Client-Synchronisation via Broadcast-Channel

mod routes;
mod websocket;

pub use routes::start_api_server;
