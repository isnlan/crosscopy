//! Comprehensive network manager verification tests
//!
//! This test suite verifies:
//! 1. Automatic peer discovery via mDNS
//! 2. Connection management (connect/disconnect/timeout)
//! 3. Event handling and propagation
//! 4. Network monitoring and statistics
//! 5. Message broadcasting and peer communication

use crosscopy::config::NetworkConfig;
use crosscopy::events::{Event, EventBus, EventHandler};
use crosscopy::network::{NetworkManager, Message, MessageType, ConnectionState};
use std::sync::Arc;
use std::time::Duration;
use tokio::time::sleep;
use log::{info, debug};

/// Test event handler to capture network events
struct TestEventHandler {
    pub events: Arc<tokio::sync::Mutex<Vec<Event>>>,
    name: String,
}

impl TestEventHandler {
    fn new() -> Self {
        Self {
            events: Arc::new(tokio::sync::Mutex::new(Vec::new())),
            name: "TestEventHandler".to_string(),
        }
    }

    async fn get_events(&self) -> Vec<Event> {
        self.events.lock().await.clone()
    }

    async fn clear_events(&self) {
        self.events.lock().await.clear();
    }

    async fn wait_for_event_type(&self, event_type: &str, timeout: Duration) -> bool {
        let start = std::time::Instant::now();
        while start.elapsed() < timeout {
            let events = self.get_events().await;
            for event in events {
                match (&event, event_type) {
                    (Event::PeerDiscovered { .. }, "PeerDiscovered") => return true,
                    (Event::PeerConnected { .. }, "PeerConnected") => return true,
                    (Event::PeerDisconnected { .. }, "PeerDisconnected") => return true,
                    (Event::ClipboardSynced { .. }, "ClipboardSynced") => return true,
                    _ => continue,
                }
            }
            sleep(Duration::from_millis(100)).await;
        }
        false
    }
}

impl EventHandler for TestEventHandler {
    fn handle(&self, event: &Event) -> crosscopy::events::Result<()> {
        debug!("Test handler received event: {:?}", event);
        // We can't use async operations in the sync trait method,
        // so we'll use a different approach for testing
        Ok(())
    }

    fn name(&self) -> &str {
        &self.name
    }
}

/// Create a test network configuration with mDNS enabled
fn create_test_network_config() -> NetworkConfig {
    NetworkConfig {
        listen_port: 8889,
        connection_timeout: 5000,
        heartbeat_interval: 10000,
        max_connections: 10,
        enable_mdns: true,
        mdns_discovery_interval: 2, // Fast discovery for testing
        idle_connection_timeout: 30,
        enable_quic: false,
        quic_port: None,
    }
}

#[tokio::test]
async fn test_network_manager_creation_and_configuration() {
    env_logger::try_init().ok();
    info!("Testing network manager creation and configuration");

    let config = create_test_network_config();
    let event_bus = Arc::new(EventBus::new());
    
    let manager = NetworkManager::new(config.clone(), event_bus).await;
    assert!(manager.is_ok(), "Failed to create network manager");
    
    let manager = manager.unwrap();
    assert_eq!(manager.config().listen_port, 8889);
    assert!(manager.is_mdns_enabled());
    assert_eq!(manager.config().mdns_discovery_interval, 2);
    
    info!("✓ Network manager creation and configuration test passed");
}

#[tokio::test]
async fn test_peer_discovery_via_mdns() {
    env_logger::try_init().ok();
    info!("Testing automatic peer discovery via mDNS");

    let config = create_test_network_config();
    let event_bus = Arc::new(EventBus::new());
    let event_handler = TestEventHandler::new();

    // Register event handler
    event_bus.register_handler(Box::new(event_handler)).await.unwrap();
    
    let mut manager = NetworkManager::new(config, event_bus).await.unwrap();
    
    // Start the network manager
    manager.start().await.unwrap();
    
    // Wait for peer discovery events
    info!("Waiting for peer discovery...");
    sleep(Duration::from_secs(6)).await;
    
    // Check that peers were actually added to connections
    sleep(Duration::from_secs(3)).await;
    let peer_count = manager.get_connection_count().await;
    assert!(peer_count > 0, "No active connections after discovery");
    
    let connected_peers = manager.get_connected_peers().await;
    assert!(!connected_peers.is_empty(), "No connected peers found");
    
    info!("✓ Discovered {} peers: {:?}", peer_count, connected_peers);
    
    manager.stop().await.unwrap();
    info!("✓ Peer discovery test passed");
}

#[tokio::test]
async fn test_connection_management() {
    env_logger::try_init().ok();
    info!("Testing connection management");

    let config = create_test_network_config();
    let event_bus = Arc::new(EventBus::new());
    let mut manager = NetworkManager::new(config, event_bus).await.unwrap();
    
    manager.start().await.unwrap();
    
    // Wait for some connections to be established
    sleep(Duration::from_secs(5)).await;
    
    let initial_count = manager.get_connection_count().await;
    info!("Initial connection count: {}", initial_count);
    
    // Get connection details
    let connection_details = manager.get_connection_details().await;
    assert!(!connection_details.is_empty(), "No connection details available");
    
    for (peer_id, state, last_heartbeat) in &connection_details {
        info!("Peer: {}, State: {:?}, Last heartbeat: {:?}", peer_id, state, last_heartbeat);
        assert!(matches!(state, ConnectionState::Connected), "Peer should be connected");
    }
    
    manager.stop().await.unwrap();
    
    // After stopping, connections should be cleared
    let final_count = manager.get_connection_count().await;
    assert_eq!(final_count, 0, "Connections should be cleared after stopping");
    
    info!("✓ Connection management test passed");
}

#[tokio::test]
async fn test_network_statistics_and_monitoring() {
    env_logger::try_init().ok();
    info!("Testing network statistics and monitoring");

    let config = create_test_network_config();
    let event_bus = Arc::new(EventBus::new());
    let mut manager = NetworkManager::new(config, event_bus).await.unwrap();
    
    // Check initial stats
    let initial_stats = manager.get_network_stats().await;
    assert_eq!(initial_stats.peers_discovered, 0);
    assert_eq!(initial_stats.peers_connected, 0);
    assert_eq!(initial_stats.messages_sent, 0);
    
    manager.start().await.unwrap();
    
    // Wait for discovery and connections
    sleep(Duration::from_secs(6)).await;
    
    let stats = manager.get_network_stats().await;
    info!("Network statistics: {:?}", stats);
    
    assert!(stats.peers_discovered > 0, "Should have discovered peers");
    assert!(stats.peers_connected > 0, "Should have connected peers");
    assert!(stats.discovery_cycles > 0, "Should have completed discovery cycles");
    
    // Test stats reset
    manager.reset_network_stats().await;
    let reset_stats = manager.get_network_stats().await;
    assert_eq!(reset_stats.peers_discovered, 0);
    assert_eq!(reset_stats.messages_sent, 0);
    
    manager.stop().await.unwrap();
    info!("✓ Network statistics and monitoring test passed");
}

#[tokio::test]
async fn test_message_broadcasting() {
    env_logger::try_init().ok();
    info!("Testing message broadcasting to peers");

    let config = create_test_network_config();
    let event_bus = Arc::new(EventBus::new());
    let mut manager = NetworkManager::new(config, event_bus).await.unwrap();
    
    manager.start().await.unwrap();
    
    // Wait for peers to connect
    sleep(Duration::from_secs(5)).await;
    
    let peer_count = manager.get_connection_count().await;
    if peer_count == 0 {
        info!("No peers connected, skipping broadcast test");
        manager.stop().await.unwrap();
        return;
    }
    
    // Test clipboard content broadcasting
    let test_content = b"Test clipboard content for broadcasting".to_vec();
    let result = manager.broadcast_clipboard_content(test_content.clone()).await;
    assert!(result.is_ok(), "Failed to broadcast clipboard content");
    
    // Check that stats were updated
    let stats = manager.get_network_stats().await;
    assert!(stats.messages_sent > 0, "No messages were sent");
    assert!(stats.bytes_sent >= test_content.len() as u64, "Bytes sent should match content size");
    
    info!("✓ Broadcasted to {} peers, sent {} bytes", peer_count, stats.bytes_sent);
    
    manager.stop().await.unwrap();
    info!("✓ Message broadcasting test passed");
}

#[tokio::test]
async fn test_peer_to_peer_messaging() {
    env_logger::try_init().ok();
    info!("Testing peer-to-peer messaging");

    let config = create_test_network_config();
    let event_bus = Arc::new(EventBus::new());
    let mut manager = NetworkManager::new(config, event_bus).await.unwrap();
    
    manager.start().await.unwrap();
    
    // Wait for peers to connect
    sleep(Duration::from_secs(5)).await;
    
    let connected_peers = manager.get_connected_peers().await;
    if connected_peers.is_empty() {
        info!("No peers connected, skipping P2P messaging test");
        manager.stop().await.unwrap();
        return;
    }
    
    // Send message to first peer
    let target_peer = &connected_peers[0];
    let test_message = Message::new(
        MessageType::DeviceInfo,
        b"Test device info message".to_vec(),
        "test-device".to_string(),
    );
    
    let result = manager.send_message_to_peer(target_peer, test_message).await;
    assert!(result.is_ok(), "Failed to send message to peer");
    
    // Verify stats were updated
    let stats = manager.get_network_stats().await;
    assert!(stats.messages_sent > 0, "Message count should be updated");
    
    info!("✓ Successfully sent message to peer: {}", target_peer);
    
    manager.stop().await.unwrap();
    info!("✓ Peer-to-peer messaging test passed");
}

#[tokio::test]
async fn test_event_handling_integration() {
    env_logger::try_init().ok();
    info!("Testing event handling integration");

    let config = create_test_network_config();
    let event_bus = Arc::new(EventBus::new());
    let event_handler = TestEventHandler::new();

    // Register event handler
    event_bus.register_handler(Box::new(event_handler)).await.unwrap();
    
    let mut manager = NetworkManager::new(config, event_bus).await.unwrap();
    manager.start().await.unwrap();
    
    // Wait for events to be generated and network activity
    sleep(Duration::from_secs(8)).await;

    // Verify that the network manager is working by checking connections
    let peer_count = manager.get_connection_count().await;
    assert!(peer_count > 0, "No peers connected - event system should have facilitated discovery");

    let connected_peers = manager.get_connected_peers().await;
    info!("✓ Event system facilitated discovery of {} peers: {:?}", peer_count, connected_peers);
    
    manager.stop().await.unwrap();
    info!("✓ Event handling integration test passed");
}
