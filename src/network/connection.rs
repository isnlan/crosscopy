//! Network connection management

use crate::network::{Message, Result};
use libp2p::{PeerId, Multiaddr};
use log::{debug, error, info};
use std::fmt;
use tokio::sync::mpsc;

/// Connection state
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ConnectionState {
    Disconnected,
    Connecting,
    Connected,
    Authenticated,
    Error,
}

impl fmt::Display for ConnectionState {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ConnectionState::Disconnected => write!(f, "Disconnected"),
            ConnectionState::Connecting => write!(f, "Connecting"),
            ConnectionState::Connected => write!(f, "Connected"),
            ConnectionState::Authenticated => write!(f, "Authenticated"),
            ConnectionState::Error => write!(f, "Error"),
        }
    }
}

/// Network connection wrapper for libp2p
pub struct Connection {
    pub id: String,
    pub peer_id: Option<PeerId>,
    pub device_id: Option<String>,
    pub state: ConnectionState,
    pub address: Option<Multiaddr>,
    pub message_sender: Option<mpsc::UnboundedSender<Message>>,
    pub last_heartbeat: Option<std::time::Instant>,
}

impl Clone for Connection {
    fn clone(&self) -> Self {
        Self {
            id: self.id.clone(),
            peer_id: self.peer_id,
            device_id: self.device_id.clone(),
            state: self.state,
            address: self.address.clone(),
            message_sender: self.message_sender.clone(), // UnboundedSender does implement Clone
            last_heartbeat: self.last_heartbeat,
        }
    }
}

impl Connection {
    /// Create a new connection
    pub fn new(id: String) -> Self {
        Self {
            id,
            peer_id: None,
            device_id: None,
            state: ConnectionState::Disconnected,
            address: None,
            message_sender: None,
            last_heartbeat: None,
        }
    }

    /// Create a new connection with peer ID
    pub fn new_with_peer(id: String, peer_id: PeerId, address: Multiaddr) -> Self {
        Self {
            id,
            peer_id: Some(peer_id),
            device_id: None,
            state: ConnectionState::Connecting,
            address: Some(address),
            message_sender: None,
            last_heartbeat: None,
        }
    }

    /// Set connection state
    pub fn set_state(&mut self, state: ConnectionState) {
        debug!("Connection {} state changed: {} -> {}", self.id, self.state, state);
        self.state = state;
    }

    /// Check if connection is active
    pub fn is_active(&self) -> bool {
        matches!(self.state, ConnectionState::Connected | ConnectionState::Authenticated)
    }

    /// Set message sender for this connection
    pub fn set_message_sender(&mut self, sender: mpsc::UnboundedSender<Message>) {
        self.message_sender = Some(sender);
    }

    /// Send a message through this connection
    pub async fn send_message(&mut self, message: Message) -> Result<()> {
        if let Some(ref sender) = self.message_sender {
            debug!("Sending message through connection {}: {:?}", self.id, message.header.message_type);

            sender.send(message)
                .map_err(|_| crate::network::NetworkError::ConnectionFailed(
                    "Failed to send message through channel".to_string()
                ))?;

            Ok(())
        } else {
            Err(crate::network::NetworkError::ConnectionFailed(
                "No active message sender".to_string()
            ))
        }
    }

    /// Get peer ID if available
    pub fn peer_id(&self) -> Option<&PeerId> {
        self.peer_id.as_ref()
    }

    /// Get connection address if available
    pub fn address(&self) -> Option<&Multiaddr> {
        self.address.as_ref()
    }

    /// Update last heartbeat timestamp
    pub fn update_heartbeat(&mut self) {
        self.last_heartbeat = Some(std::time::Instant::now());
    }

    /// Check if connection has timed out
    pub fn is_timed_out(&self, timeout: std::time::Duration) -> bool {
        if let Some(last_heartbeat) = self.last_heartbeat {
            last_heartbeat.elapsed() > timeout
        } else {
            false
        }
    }
}

impl fmt::Display for Connection {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "Connection(id: {}, device: {:?}, state: {})",
            self.id,
            self.device_id,
            self.state
        )
    }
}
