//! Network manager implementation

use crate::config::NetworkConfig;
use crate::events::EventBus;
use crate::network::{Connection, Message, Result};
use log::{debug, error, info, warn};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

/// Network manager for handling connections and communication
pub struct NetworkManager {
    config: NetworkConfig,
    event_bus: Arc<EventBus>,
    connections: Arc<RwLock<HashMap<String, Connection>>>,
    running: Arc<RwLock<bool>>,
}

impl NetworkManager {
    /// Create a new network manager
    pub async fn new(
        config: NetworkConfig,
        event_bus: Arc<EventBus>,
    ) -> Result<Self> {
        Ok(Self {
            config,
            event_bus,
            connections: Arc::new(RwLock::new(HashMap::new())),
            running: Arc::new(RwLock::new(false)),
        })
    }

    /// Start the network manager
    pub async fn start(&mut self) -> Result<()> {
        info!("Starting network manager on port {}", self.config.listen_port);
        *self.running.write().await = true;

        // Start server listener
        self.start_server().await?;

        // Connect to configured peers
        self.connect_to_peers().await?;

        Ok(())
    }

    /// Stop the network manager
    pub async fn stop(&mut self) -> Result<()> {
        info!("Stopping network manager");
        *self.running.write().await = false;

        // Close all connections
        let mut connections = self.connections.write().await;
        connections.clear();

        Ok(())
    }

    /// Broadcast clipboard content to all connected devices
    pub async fn broadcast_clipboard_content(&self, content: Vec<u8>) -> Result<()> {
        debug!("Broadcasting clipboard content to {} devices", self.get_connection_count().await);

        let message = Message::new(
            crate::network::MessageType::ClipboardSync,
            content,
            "local".to_string(),
        );

        let connections = self.connections.read().await;
        for (id, connection) in connections.iter() {
            if connection.is_active() {
                debug!("Sending clipboard content to connection: {}", id);
                // Implementation would send the message
            }
        }

        Ok(())
    }

    /// Get the number of active connections
    pub async fn get_connection_count(&self) -> usize {
        let connections = self.connections.read().await;
        connections.values().filter(|c| c.is_active()).count()
    }

    async fn start_server(&self) -> Result<()> {
        let addr = format!("0.0.0.0:{}", self.config.listen_port);
        info!("Starting WebSocket server on {}", addr);

        let listener = tokio::net::TcpListener::bind(&addr).await?;
        let connections = self.connections.clone();
        let event_bus = self.event_bus.clone();
        let running = self.running.clone();

        tokio::spawn(async move {
            while *running.read().await {
                match listener.accept().await {
                    Ok((stream, addr)) => {
                        info!("New connection from {}", addr);

                        let connections = connections.clone();
                        let event_bus = event_bus.clone();

                        tokio::spawn(async move {
                            if let Err(e) = Self::handle_incoming_connection(
                                stream, addr, connections, event_bus
                            ).await {
                                error!("Error handling connection from {}: {}", addr, e);
                            }
                        });
                    }
                    Err(e) => {
                        error!("Failed to accept connection: {}", e);
                    }
                }
            }
        });

        Ok(())
    }

    async fn handle_incoming_connection(
        stream: tokio::net::TcpStream,
        addr: std::net::SocketAddr,
        connections: Arc<RwLock<HashMap<String, Connection>>>,
        event_bus: Arc<EventBus>,
    ) -> Result<()> {
        use tokio_tungstenite::{accept_async, tungstenite::Message as WsMessage};
        use futures_util::{SinkExt, StreamExt};

        let ws_stream = accept_async(stream).await?;
        let (mut ws_sender, mut ws_receiver) = ws_stream.split();

        let connection_id = uuid::Uuid::new_v4().to_string();
        let mut connection = Connection::new(connection_id.clone());
        connection.set_state(crate::network::ConnectionState::Connected);

        // Add connection to the map
        {
            let mut connections_guard = connections.write().await;
            connections_guard.insert(connection_id.clone(), connection);
        }

        // Emit device connected event
        let event = crate::events::Event::DeviceConnected {
            device_id: connection_id.clone(),
        };
        if let Err(e) = event_bus.emit(event).await {
            error!("Failed to emit device connected event: {}", e);
        }

        // Handle incoming messages
        while let Some(msg) = ws_receiver.next().await {
            match msg {
                Ok(WsMessage::Binary(data)) => {
                    if let Err(e) = Self::handle_binary_message(
                        data, &connection_id, &event_bus
                    ).await {
                        error!("Error handling binary message: {}", e);
                    }
                }
                Ok(WsMessage::Text(text)) => {
                    debug!("Received text message: {}", text);
                }
                Ok(WsMessage::Close(_)) => {
                    info!("Connection {} closed", connection_id);
                    break;
                }
                Err(e) => {
                    error!("WebSocket error on connection {}: {}", connection_id, e);
                    break;
                }
                _ => {}
            }
        }

        // Remove connection and emit disconnected event
        {
            let mut connections_guard = connections.write().await;
            connections_guard.remove(&connection_id);
        }

        let event = crate::events::Event::DeviceDisconnected {
            device_id: connection_id,
        };
        if let Err(e) = event_bus.emit(event).await {
            error!("Failed to emit device disconnected event: {}", e);
        }

        Ok(())
    }

    async fn handle_binary_message(
        data: Vec<u8>,
        sender_id: &str,
        event_bus: &Arc<EventBus>,
    ) -> Result<()> {
        // Deserialize the message
        let message: Message = serde_json::from_slice(&data)
            .map_err(|e| crate::network::NetworkError::InvalidMessage(e.to_string()))?;

        // Verify message integrity
        if !message.verify() {
            warn!("Received message with invalid checksum from {}", sender_id);
            return Ok(());
        }

        debug!("Received {} message from {}", message.header.message_type, sender_id);

        // Emit network message event
        let event = crate::events::Event::NetworkMessage {
            message,
            sender: sender_id.to_string(),
        };

        event_bus.emit(event).await
            .map_err(|e| crate::network::NetworkError::InvalidMessage(e.to_string()))?;

        Ok(())
    }

    async fn connect_to_peers(&self) -> Result<()> {
        for peer in &self.config.peer_list {
            if peer.enabled {
                info!("Connecting to peer: {} ({}:{})", peer.name, peer.address, peer.port);

                let connections = self.connections.clone();
                let event_bus = self.event_bus.clone();
                let peer_config = peer.clone();

                tokio::spawn(async move {
                    if let Err(e) = Self::connect_to_peer(peer_config, connections, event_bus).await {
                        error!("Failed to connect to peer: {}", e);
                    }
                });
            }
        }

        Ok(())
    }

    async fn connect_to_peer(
        peer: crate::config::PeerConfig,
        connections: Arc<RwLock<HashMap<String, Connection>>>,
        event_bus: Arc<EventBus>,
    ) -> Result<()> {
        use tokio_tungstenite::{connect_async, tungstenite::Message as WsMessage};
        use futures_util::{SinkExt, StreamExt};

        let url = format!("ws://{}:{}", peer.address, peer.port);
        info!("Connecting to peer at {}", url);

        let (ws_stream, _) = connect_async(&url).await?;
        let (mut ws_sender, mut ws_receiver) = ws_stream.split();

        let connection_id = peer.device_id.clone();
        let mut connection = Connection::new(connection_id.clone());
        connection.device_id = Some(peer.device_id.clone());
        connection.set_state(crate::network::ConnectionState::Connected);

        // Add connection to the map
        {
            let mut connections_guard = connections.write().await;
            connections_guard.insert(connection_id.clone(), connection);
        }

        // Emit device connected event
        let event = crate::events::Event::DeviceConnected {
            device_id: connection_id.clone(),
        };
        if let Err(e) = event_bus.emit(event).await {
            error!("Failed to emit device connected event: {}", e);
        }

        // Handle incoming messages
        while let Some(msg) = ws_receiver.next().await {
            match msg {
                Ok(WsMessage::Binary(data)) => {
                    if let Err(e) = Self::handle_binary_message(
                        data, &connection_id, &event_bus
                    ).await {
                        error!("Error handling binary message: {}", e);
                    }
                }
                Ok(WsMessage::Close(_)) => {
                    info!("Connection to peer {} closed", connection_id);
                    break;
                }
                Err(e) => {
                    error!("WebSocket error on peer connection {}: {}", connection_id, e);
                    break;
                }
                _ => {}
            }
        }

        // Remove connection and emit disconnected event
        {
            let mut connections_guard = connections.write().await;
            connections_guard.remove(&connection_id);
        }

        let event = crate::events::Event::DeviceDisconnected {
            device_id: connection_id,
        };
        if let Err(e) = event_bus.emit(event).await {
            error!("Failed to emit device disconnected event: {}", e);
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::NetworkConfig;
    use crate::events::EventBus;

    #[tokio::test]
    async fn test_network_manager_creation() {
        let config = NetworkConfig::default();
        let event_bus = Arc::new(EventBus::new());
        
        let manager = NetworkManager::new(config, event_bus).await;
        assert!(manager.is_ok());
    }
}
