//! API Server Module
//! 
//! REST API und WebSocket für Client-Kommunikation

mod routes;
mod websocket;

pub use routes::start_api_server;
pub use websocket::WebSocketHandler;
