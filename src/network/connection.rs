//! Network connection management

use crate::network::{Message, Result};
use log::{debug, error, info};
use std::fmt;
use tokio::net::TcpStream;
use tokio_tungstenite::{WebSocketStream, MaybeTlsStream};

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

/// Network connection wrapper
pub struct Connection {
    pub id: String,
    pub device_id: Option<String>,
    pub state: ConnectionState,
    pub websocket: Option<WebSocketStream<MaybeTlsStream<TcpStream>>>,
    pub last_heartbeat: Option<std::time::Instant>,
}

impl Connection {
    /// Create a new connection
    pub fn new(id: String) -> Self {
        Self {
            id,
            device_id: None,
            state: ConnectionState::Disconnected,
            websocket: None,
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

    /// Send a message through this connection
    pub async fn send_message(&mut self, message: Message) -> Result<()> {
        use futures_util::SinkExt;
        use tokio_tungstenite::tungstenite::Message as WsMessage;

        if let Some(ref mut ws) = self.websocket {
            debug!("Sending message through connection {}: {:?}", self.id, message.header.message_type);

            // Serialize message to JSON
            let serialized = serde_json::to_vec(&message)
                .map_err(|e| crate::network::NetworkError::Serialization(e))?;

            // Send as binary WebSocket message
            ws.send(WsMessage::Binary(serialized)).await
                .map_err(|e| crate::network::NetworkError::WebSocket(e))?;

            Ok(())
        } else {
            Err(crate::network::NetworkError::ConnectionFailed(
                "No active WebSocket connection".to_string()
            ))
        }
    }

    /// Receive a message from this connection
    pub async fn receive_message(&mut self) -> Result<Option<Message>> {
        use futures_util::StreamExt;
        use tokio_tungstenite::tungstenite::Message as WsMessage;

        if let Some(ref mut ws) = self.websocket {
            match ws.next().await {
                Some(Ok(WsMessage::Binary(data))) => {
                    debug!("Received binary message on connection {}", self.id);

                    // Deserialize message from JSON
                    let message: Message = serde_json::from_slice(&data)
                        .map_err(|e| crate::network::NetworkError::Serialization(e))?;

                    Ok(Some(message))
                }
                Some(Ok(WsMessage::Text(text))) => {
                    debug!("Received text message on connection {}: {}", self.id, text);
                    Ok(None) // We don't handle text messages as clipboard data
                }
                Some(Ok(WsMessage::Close(_))) => {
                    debug!("Connection {} closed by peer", self.id);
                    self.set_state(ConnectionState::Disconnected);
                    Ok(None)
                }
                Some(Err(e)) => {
                    error!("WebSocket error on connection {}: {}", self.id, e);
                    self.set_state(ConnectionState::Error);
                    Err(crate::network::NetworkError::WebSocket(e))
                }
                None => {
                    debug!("WebSocket stream ended for connection {}", self.id);
                    self.set_state(ConnectionState::Disconnected);
                    Ok(None)
                }
                _ => Ok(None),
            }
        } else {
            Err(crate::network::NetworkError::ConnectionFailed(
                "No active WebSocket connection".to_string()
            ))
        }
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
