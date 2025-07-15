//! Network manager implementation using libp2p

use crate::config::NetworkConfig;
use crate::events::{Event, EventBus};
use crate::network::{Connection, ConnectionState, Message, MessageType, Result, NetworkError};
use crate::network::behaviour::{CrossCopyBehaviour, CrossCopyEvent, ClipboardMessage};
use libp2p::{
    identity, noise, yamux, tcp,
    swarm::{Swarm, SwarmEvent},
    SwarmBuilder,
    PeerId, Multiaddr, Transport,
    request_response::ResponseChannel,
};
use log::{debug, info, warn, error};
use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::{RwLock, mpsc};
use futures::StreamExt;

/// Network statistics for monitoring
#[derive(Debug, Clone, Default)]
pub struct NetworkStats {
    pub peers_discovered: u64,
    pub peers_connected: u64,
    pub peers_disconnected: u64,
    pub messages_sent: u64,
    pub messages_received: u64,
    pub bytes_sent: u64,
    pub bytes_received: u64,
    pub discovery_cycles: u64,
}

/// Real libp2p-based network manager for CrossCopy
pub struct NetworkManager {
    config: NetworkConfig,
    event_bus: Arc<EventBus>,
    swarm: Option<Swarm<CrossCopyBehaviour>>,
    local_peer_id: PeerId,
    connections: Arc<RwLock<HashMap<PeerId, Connection>>>,
    stats: Arc<RwLock<NetworkStats>>,
    command_sender: Option<mpsc::UnboundedSender<NetworkCommand>>,
    running: Arc<RwLock<bool>>,
}

/// Commands that can be sent to the network manager
#[derive(Debug)]
pub enum NetworkCommand {
    BroadcastClipboard {
        content: Vec<u8>,
        content_type: String,
    },
    Shutdown,
}

impl NetworkManager {
    /// Create a new network manager with real libp2p implementation
    pub async fn new(config: NetworkConfig, event_bus: Arc<EventBus>) -> Result<Self> {
        info!("Creating libp2p network manager");

        // Generate a random key pair for this peer
        let local_key = identity::Keypair::generate_ed25519();
        let local_peer_id = PeerId::from(local_key.public());

        info!("Local peer ID: {}", local_peer_id);

        Ok(Self {
            config,
            event_bus,
            swarm: None,
            local_peer_id,
            connections: Arc::new(RwLock::new(HashMap::new())),
            stats: Arc::new(RwLock::new(NetworkStats::default())),
            command_sender: None,
            running: Arc::new(RwLock::new(false)),
        })
    }

    /// Start the network manager with real libp2p swarm
    pub async fn start(&mut self) -> Result<()> {
        info!("Starting libp2p network manager on port {}", self.config.listen_port);
        *self.running.write().await = true;

        // Generate keypair
        let local_key = identity::Keypair::generate_ed25519();
        let local_peer_id = PeerId::from(local_key.public());
        self.local_peer_id = local_peer_id;

        // Create behaviour
        let behaviour = CrossCopyBehaviour::new(local_peer_id)
            .map_err(|e| NetworkError::Libp2p(format!("Failed to create behaviour: {}", e)))?;

        // Create swarm using the new builder API
        let mut swarm = SwarmBuilder::with_existing_identity(local_key)
            .with_tokio()
            .with_tcp(
                tcp::Config::default(),
                noise::Config::new,
                yamux::Config::default,
            )
            .map_err(|e| NetworkError::Transport(format!("Failed to create transport: {}", e)))?
            .with_behaviour(|_| behaviour)
            .map_err(|e| NetworkError::Libp2p(format!("Failed to create behaviour: {}", e)))?
            .with_swarm_config(|c| c.with_idle_connection_timeout(Duration::from_secs(60)))
            .build();

        // Listen on the configured port
        let listen_addr: Multiaddr = format!("/ip4/0.0.0.0/tcp/{}", self.config.listen_port)
            .parse()
            .map_err(|e| NetworkError::Transport(format!("Invalid listen address: {}", e)))?;

        swarm.listen_on(listen_addr.clone())
            .map_err(|e| NetworkError::Transport(format!("Failed to listen: {}", e)))?;

        info!("Listening on: {}", listen_addr);

        // Create command channel
        let (command_sender, mut command_receiver) = mpsc::unbounded_channel();
        self.command_sender = Some(command_sender);

        // Store swarm
        self.swarm = Some(swarm);

        // Start the swarm event loop
        let event_bus = self.event_bus.clone();
        let connections = self.connections.clone();
        let stats = self.stats.clone();
        let running = self.running.clone();

        if let Some(mut swarm) = self.swarm.take() {
            tokio::spawn(async move {
                info!("Starting libp2p swarm event loop");

                loop {
                    if !*running.read().await {
                        break;
                    }

                    tokio::select! {
                        event = swarm.select_next_some() => {
                            Self::handle_swarm_event(event, &event_bus, &connections, &stats).await;
                        }
                        command = command_receiver.recv() => {
                            if let Some(cmd) = command {
                                if let Err(e) = Self::handle_command(cmd, &mut swarm).await {
                                    error!("Failed to handle command: {}", e);
                                }
                            }
                        }
                    }
                }

                info!("Swarm event loop stopped");
            });
        }

        info!("libp2p network manager started successfully");
        Ok(())
    }

    /// Handle swarm events
    async fn handle_swarm_event(
        event: SwarmEvent<CrossCopyEvent>,
        event_bus: &Arc<EventBus>,
        connections: &Arc<RwLock<HashMap<PeerId, Connection>>>,
        stats: &Arc<RwLock<NetworkStats>>,
    ) {
        match event {
            SwarmEvent::Behaviour(CrossCopyEvent::PeerDiscovered { peer_id, addresses }) => {
                info!("Discovered peer: {} at {:?}", peer_id, addresses);

                // Create connection entry
                let mut connection = Connection::new(peer_id.to_string());
                if let Some(addr) = addresses.first() {
                    connection.address = Some(addr.clone());
                }
                connection.peer_id = Some(peer_id);
                connection.set_state(ConnectionState::Connected);

                connections.write().await.insert(peer_id, connection);

                // Update stats
                {
                    let mut stats_guard = stats.write().await;
                    stats_guard.peers_discovered += 1;
                    stats_guard.peers_connected += 1;
                }

                // Emit event
                let event = Event::PeerDiscovered {
                    peer_id: peer_id.to_string(),
                    address: addresses.first().map(|a| a.to_string()).unwrap_or_default(),
                };
                let _ = event_bus.emit(event).await;
            }
            SwarmEvent::Behaviour(CrossCopyEvent::PeerExpired { peer_id }) => {
                info!("Peer expired: {}", peer_id);

                connections.write().await.remove(&peer_id);

                // Update stats
                {
                    let mut stats_guard = stats.write().await;
                    stats_guard.peers_disconnected += 1;
                    if stats_guard.peers_connected > 0 {
                        stats_guard.peers_connected -= 1;
                    }
                }

                // Emit event
                let event = Event::PeerDisconnected {
                    peer_id: peer_id.to_string(),
                };
                let _ = event_bus.emit(event).await;
            }
            // For now, we only handle mDNS events
            // Future: Add request-response handling here
            SwarmEvent::NewListenAddr { address, .. } => {
                info!("Listening on: {}", address);
            }
            SwarmEvent::ConnectionEstablished { peer_id, .. } => {
                info!("Connection established with: {}", peer_id);
            }
            SwarmEvent::ConnectionClosed { peer_id, cause, .. } => {
                info!("Connection closed with {}: {:?}", peer_id, cause);
                connections.write().await.remove(&peer_id);
            }
            _ => {
                debug!("Unhandled swarm event: {:?}", event);
            }
        }
    }

    /// Handle network commands
    async fn handle_command(
        command: NetworkCommand,
        _swarm: &mut Swarm<CrossCopyBehaviour>,
    ) -> Result<()> {
        match command {
            NetworkCommand::BroadcastClipboard { content, content_type } => {
                info!("Broadcasting clipboard content ({} bytes) of type {}", content.len(), content_type);
                // TODO: Implement actual broadcasting when we add request-response
                // For now, just log the broadcast attempt
            }
            NetworkCommand::Shutdown => {
                info!("Shutting down network manager");
                return Err(NetworkError::ConnectionFailed("Shutdown requested".to_string()));
            }
        }
        Ok(())
    }





    /// Stop the network manager
    pub async fn stop(&mut self) -> Result<()> {
        info!("Stopping libp2p network manager");
        *self.running.write().await = false;

        // Send shutdown command
        if let Some(sender) = &self.command_sender {
            let _ = sender.send(NetworkCommand::Shutdown);
        }

        // Clear connections
        let mut connections = self.connections.write().await;
        for (peer_id, mut connection) in connections.drain() {
            connection.set_state(ConnectionState::Disconnected);
            info!("Closed connection to peer: {}", peer_id);
        }

        // Drop swarm and command sender
        self.swarm = None;
        self.command_sender = None;

        info!("libp2p network manager stopped");
        Ok(())
    }

    /// Broadcast clipboard content to all connected devices
    pub async fn broadcast_clipboard_content(&self, content: Vec<u8>) -> Result<()> {
        let connection_count = self.get_connection_count().await;
        debug!("Broadcasting clipboard content to {} devices", connection_count);

        if connection_count == 0 {
            info!("No active peers to broadcast clipboard content to");
            return Ok(());
        }

        // Send broadcast command through channel
        if let Some(sender) = &self.command_sender {
            let command = NetworkCommand::BroadcastClipboard {
                content: content.clone(),
                content_type: "text/plain".to_string(), // Default content type
            };

            sender.send(command)
                .map_err(|_| NetworkError::ConnectionFailed("Failed to send broadcast command".to_string()))?;

            // Update statistics
            {
                let mut stats = self.stats.write().await;
                stats.messages_sent += connection_count as u64;
                stats.bytes_sent += (content.len() * connection_count) as u64;
            }

            info!("Clipboard content broadcast initiated for {} peers", connection_count);
        } else {
            return Err(NetworkError::ConnectionFailed("Network manager not started".to_string()));
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
        // Parse peer ID
        let _peer_id = peer_id.parse::<PeerId>()
            .map_err(|_| NetworkError::PeerNotFound(format!("Invalid peer ID: {}", peer_id)))?;

        let connections = self.connections.read().await;
        if let Some(connection) = connections.get(&_peer_id) {
            if connection.is_active() {
                let payload_len = message.payload.len();

                // For now, just log the message sending
                // TODO: Implement actual message sending when we add request-response
                info!("Would send message to peer {}: {:?} ({} bytes)", peer_id, message.header.message_type, payload_len);

                // Update statistics
                {
                    let mut stats = self.stats.write().await;
                    stats.messages_sent += 1;
                    stats.bytes_sent += payload_len as u64;
                }

                Ok(())
            } else {
                Err(NetworkError::ConnectionFailed(format!("Peer {} not connected", peer_id)))
            }
        } else {
            Err(NetworkError::PeerNotFound(peer_id.to_string()))
        }
    }

    /// Check if mDNS discovery is enabled
    pub fn is_mdns_enabled(&self) -> bool {
        self.config.enable_mdns
    }

    /// Get network configuration
    pub fn config(&self) -> &NetworkConfig {
        &self.config
    }

    /// Get list of connected peer IDs
    pub async fn get_connected_peers(&self) -> Vec<String> {
        let connections = self.connections.read().await;
        connections
            .iter()
            .filter(|(_, conn)| conn.is_active())
            .map(|(peer_id, _)| peer_id.to_string())
            .collect()
    }

    /// Get network statistics
    pub async fn get_network_stats(&self) -> NetworkStats {
        self.stats.read().await.clone()
    }

    /// Reset network statistics
    pub async fn reset_network_stats(&self) {
        let mut stats = self.stats.write().await;
        *stats = NetworkStats::default();
        info!("Network statistics reset");
    }

    /// Get detailed connection information
    pub async fn get_connection_details(&self) -> Vec<(String, ConnectionState, Option<std::time::Instant>)> {
        let connections = self.connections.read().await;
        connections
            .iter()
            .map(|(peer_id, conn)| (peer_id.to_string(), conn.state, conn.last_heartbeat))
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
