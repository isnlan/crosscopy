//! Network communication module
//!
//! This module handles all network communication between devices, including
//! WebSocket connections, message protocols, and connection management.

pub mod connection;
pub mod manager;
pub mod protocol;

pub use connection::{Connection, ConnectionState};
pub use manager::NetworkManager;
pub use protocol::{Message, MessageType, ProtocolVersion};

use thiserror::Error;

/// Network-related errors
#[derive(Debug, Error)]
pub enum NetworkError {
    #[error("Connection failed: {0}")]
    ConnectionFailed(String),

    #[error("Connection timeout")]
    ConnectionTimeout,

    #[error("Invalid message format: {0}")]
    InvalidMessage(String),

    #[error("Protocol version mismatch: expected {expected}, got {actual}")]
    ProtocolMismatch { expected: u16, actual: u16 },

    #[error("Authentication failed")]
    AuthenticationFailed,

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("WebSocket error: {0}")]
    WebSocket(#[from] tokio_tungstenite::tungstenite::Error),

    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),
}

/// Result type for network operations
pub type Result<T> = std::result::Result<T, NetworkError>;
