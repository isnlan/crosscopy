//! Network manager implementation using libp2p

use crate::config::NetworkConfig;
use crate::events::EventBus;
use crate::network::{Connection, Message, Result, NetworkError};
use libp2p::{
    core::upgrade,
    futures::StreamExt,
    identity, mdns, noise, tcp, yamux, PeerId, Swarm, Transport,
    swarm::{SwarmEvent, NetworkBehaviour},
    request_response::{self, ProtocolSupport, RequestResponse, RequestResponseEvent},
    Multiaddr,
};
use log::{debug, error, info, warn};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::{mpsc, RwLock};

/// CrossCopy protocol for request-response communication
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CrossCopyRequest {
    pub message: Message,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CrossCopyResponse {
    pub success: bool,
    pub error: Option<String>,
}

/// Network behaviour for CrossCopy
#[derive(NetworkBehaviour)]
#[behaviour(out_event = "CrossCopyEvent")]
pub struct CrossCopyBehaviour {
    request_response: RequestResponse<CrossCopyCodec>,
    mdns: mdns::tokio::Behaviour,
}

/// Custom codec for CrossCopy protocol
#[derive(Clone)]
pub struct CrossCopyCodec;

/// Events from the CrossCopy network behaviour
#[derive(Debug)]
pub enum CrossCopyEvent {
    RequestResponse(RequestResponseEvent<CrossCopyRequest, CrossCopyResponse>),
    Mdns(mdns::Event),
}

impl From<RequestResponseEvent<CrossCopyRequest, CrossCopyResponse>> for CrossCopyEvent {
    fn from(event: RequestResponseEvent<CrossCopyRequest, CrossCopyResponse>) -> Self {
        CrossCopyEvent::RequestResponse(event)
    }
}

impl From<mdns::Event> for CrossCopyEvent {
    fn from(event: mdns::Event) -> Self {
        CrossCopyEvent::Mdns(event)
    }
}

/// Network manager for handling libp2p connections and communication
pub struct NetworkManager {
    config: NetworkConfig,
    event_bus: Arc<EventBus>,
    connections: Arc<RwLock<HashMap<String, Connection>>>,
    swarm: Option<Swarm<CrossCopyBehaviour>>,
    running: Arc<RwLock<bool>>,
    message_sender: Option<mpsc::UnboundedSender<(PeerId, Message)>>,
}

impl request_response::Codec for CrossCopyCodec {
    type Protocol = &'static str;
    type Request = CrossCopyRequest;
    type Response = CrossCopyResponse;

    async fn read_request<T>(
        &mut self,
        _: &Self::Protocol,
        io: &mut T,
    ) -> std::io::Result<Self::Request>
    where
        T: futures::AsyncRead + Unpin + Send,
    {
        // Implementation would read from the stream and deserialize
        // For now, return a placeholder
        Err(std::io::Error::new(
            std::io::ErrorKind::Other,
            "Not implemented",
        ))
    }

    async fn read_response<T>(
        &mut self,
        _: &Self::Protocol,
        io: &mut T,
    ) -> std::io::Result<Self::Response>
    where
        T: futures::AsyncRead + Unpin + Send,
    {
        // Implementation would read from the stream and deserialize
        // For now, return a placeholder
        Err(std::io::Error::new(
            std::io::ErrorKind::Other,
            "Not implemented",
        ))
    }

    async fn write_request<T>(
        &mut self,
        _: &Self::Protocol,
        io: &mut T,
        req: Self::Request,
    ) -> std::io::Result<()>
    where
        T: futures::AsyncWrite + Unpin + Send,
    {
        // Implementation would serialize and write to the stream
        // For now, return a placeholder
        Err(std::io::Error::new(
            std::io::ErrorKind::Other,
            "Not implemented",
        ))
    }

    async fn write_response<T>(
        &mut self,
        _: &Self::Protocol,
        io: &mut T,
        res: Self::Response,
    ) -> std::io::Result<()>
    where
        T: futures::AsyncWrite + Unpin + Send,
    {
        // Implementation would serialize and write to the stream
        // For now, return a placeholder
        Err(std::io::Error::new(
            std::io::ErrorKind::Other,
            "Not implemented",
        ))
    }
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
            swarm: None,
            running: Arc::new(RwLock::new(false)),
            message_sender: None,
        })
    }

    /// Start the network manager
    pub async fn start(&mut self) -> Result<()> {
        info!("Starting libp2p network manager on port {}", self.config.listen_port);
        *self.running.write().await = true;

        // Create libp2p identity
        let local_key = identity::Keypair::generate_ed25519();
        let local_peer_id = PeerId::from(local_key.public());
        info!("Local peer id: {}", local_peer_id);

        // Create transport
        let transport = tcp::tokio::Transport::new(tcp::Config::default().nodelay(true))
            .upgrade(upgrade::Version::V1)
            .authenticate(noise::Config::new(&local_key).unwrap())
            .multiplex(yamux::Config::default())
            .boxed();

        // Create network behaviour
        let request_response = RequestResponse::new(
            CrossCopyCodec,
            std::iter::once(("/crosscopy/1.0.0", ProtocolSupport::Full)),
            Default::default(),
        );

        let mdns_behaviour = mdns::tokio::Behaviour::new(mdns::Config::default(), local_peer_id)
            .map_err(|e| NetworkError::MdnsDiscoveryFailed(e.to_string()))?;

        let behaviour = CrossCopyBehaviour {
            request_response,
            mdns: mdns_behaviour,
        };

        // Create swarm
        let mut swarm = Swarm::with_tokio_executor(transport, behaviour, local_peer_id);

        // Listen on all interfaces
        let listen_addr: Multiaddr = format!("/ip4/0.0.0.0/tcp/{}", self.config.listen_port)
            .parse()
            .map_err(|e| NetworkError::Libp2p(format!("Invalid listen address: {}", e)))?;

        swarm.listen_on(listen_addr.clone())?;
        info!("Listening on {}", listen_addr);

        // Start the swarm event loop
        let connections = self.connections.clone();
        let event_bus = self.event_bus.clone();
        let running = self.running.clone();

        tokio::spawn(async move {
            loop {
                if !*running.read().await {
                    break;
                }

                match swarm.select_next_some().await {
                    SwarmEvent::NewListenAddr { address, .. } => {
                        info!("Listening on {}", address);
                    }
                    SwarmEvent::Behaviour(CrossCopyEvent::Mdns(mdns::Event::Discovered(list))) => {
                        for (peer_id, multiaddr) in list {
                            info!("Discovered peer: {} at {}", peer_id, multiaddr);

                            // Create new connection entry
                            let connection = Connection::new_with_peer(
                                peer_id.to_string(),
                                peer_id,
                                multiaddr.clone(),
                            );

                            connections.write().await.insert(peer_id.to_string(), connection);

                            // Attempt to dial the discovered peer
                            if let Err(e) = swarm.dial(multiaddr) {
                                error!("Failed to dial peer {}: {}", peer_id, e);
                            }
                        }
                    }
                    SwarmEvent::Behaviour(CrossCopyEvent::Mdns(mdns::Event::Expired(list))) => {
                        for (peer_id, _) in list {
                            info!("Peer expired: {}", peer_id);
                            connections.write().await.remove(&peer_id.to_string());
                        }
                    }
                    SwarmEvent::ConnectionEstablished { peer_id, .. } => {
                        info!("Connection established with {}", peer_id);
                        if let Some(connection) = connections.write().await.get_mut(&peer_id.to_string()) {
                            connection.set_state(crate::network::ConnectionState::Connected);
                        }
                    }
                    SwarmEvent::ConnectionClosed { peer_id, .. } => {
                        info!("Connection closed with {}", peer_id);
                        if let Some(connection) = connections.write().await.get_mut(&peer_id.to_string()) {
                            connection.set_state(crate::network::ConnectionState::Disconnected);
                        }
                    }
                    _ => {}
                }
            }
        });

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





    /// Send a message to a specific peer
    pub async fn send_message_to_peer(&self, peer_id: &str, message: Message) -> Result<()> {
        let connections = self.connections.read().await;
        if let Some(connection) = connections.get(peer_id) {
            if connection.is_active() {
                // In a real implementation, this would send through the libp2p swarm
                info!("Sending message to peer {}: {:?}", peer_id, message.header.message_type);
                Ok(())
            } else {
                Err(NetworkError::ConnectionFailed(format!("Peer {} not connected", peer_id)))
            }
        } else {
            Err(NetworkError::PeerNotFound(peer_id.to_string()))
        }
    }

    /// Get list of connected peers
    pub async fn get_connected_peers(&self) -> Vec<String> {
        let connections = self.connections.read().await;
        connections
            .iter()
            .filter(|(_, conn)| conn.is_active())
            .map(|(id, _)| id.clone())
            .collect()
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
