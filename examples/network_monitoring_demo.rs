//! Network monitoring and statistics demonstration
//! 
//! This example demonstrates the enhanced network monitoring capabilities
//! including statistics tracking, event notifications, and connection details.

use crosscopy::{
    config::NetworkConfig,
    events::{Event, EventBus, EventHandler, Result as EventResult},
    network::NetworkManager,
};
use log::info;
use std::sync::Arc;
use tokio::time::{sleep, Duration};

/// Simple event handler for network monitoring
struct NetworkEventHandler;

impl EventHandler for NetworkEventHandler {
    fn name(&self) -> &str {
        "NetworkEventHandler"
    }

    fn handle(&self, event: &Event) -> EventResult<()> {
        match event {
            Event::PeerDiscovered { peer_id, address } => {
                info!("🔍 EVENT: Peer discovered - {} at {}", peer_id, address);
            }
            Event::PeerConnected { peer_id } => {
                info!("🔗 EVENT: Peer connected - {}", peer_id);
            }
            Event::PeerDisconnected { peer_id } => {
                info!("❌ EVENT: Peer disconnected - {}", peer_id);
            }
            Event::ClipboardSynced { from_peer, content_size } => {
                info!("📋 EVENT: Clipboard synced from {} ({} bytes)", from_peer, content_size);
            }
            _ => {}
        }
        Ok(())
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize logging
    env_logger::init();
    
    info!("CrossCopy Network Monitoring Demo");
    info!("==================================");
    
    // Create network configuration with faster discovery for demo
    let config = NetworkConfig {
        listen_port: 8888,
        enable_mdns: true,
        mdns_discovery_interval: 3, // Faster discovery for demo
        connection_timeout: 10000,
        heartbeat_interval: 30000,
        max_connections: 10,
        idle_connection_timeout: 300,
        enable_quic: false,
        quic_port: None,
    };
    
    info!("Network Configuration:");
    info!("  Listen Port: {}", config.listen_port);
    info!("  mDNS Discovery: {}", config.enable_mdns);
    info!("  Discovery Interval: {}s", config.mdns_discovery_interval);
    
    // Create event bus and register event handler
    let event_bus = Arc::new(EventBus::new());

    // Register network event handler
    let handler = Box::new(NetworkEventHandler);
    event_bus.register_handler(handler).await?;

    let event_bus_clone = event_bus.clone();

    // Start event processing task
    tokio::spawn(async move {
        loop {
            if let Err(e) = event_bus_clone.process_events().await {
                eprintln!("Error processing events: {}", e);
            }
            tokio::time::sleep(Duration::from_millis(100)).await;
        }
    });
    
    // Create network manager
    let mut network_manager = NetworkManager::new(config, event_bus.clone()).await?;
    
    // Start the network manager
    info!("Starting network manager...");
    network_manager.start().await?;
    
    // Monitor network for 45 seconds with detailed statistics
    info!("Running network monitoring for 45 seconds...");
    for i in 1..=15 {
        sleep(Duration::from_secs(3)).await;
        
        // Get basic connection info
        let connection_count = network_manager.get_connection_count().await;
        let connected_peers = network_manager.get_connected_peers().await;
        
        // Get detailed statistics
        let stats = network_manager.get_network_stats().await;
        let connection_details = network_manager.get_connection_details().await;
        
        info!("📊 Status Update {} - Connections: {}", i, connection_count);
        
        if !connected_peers.is_empty() {
            info!("  🔗 Connected peers: {:?}", connected_peers);
        }
        
        // Display network statistics
        info!("  📈 Network Stats:");
        info!("    - Peers discovered: {}", stats.peers_discovered);
        info!("    - Peers connected: {}", stats.peers_connected);
        info!("    - Messages sent: {}", stats.messages_sent);
        info!("    - Bytes sent: {}", stats.bytes_sent);
        info!("    - Discovery cycles: {}", stats.discovery_cycles);
        
        // Display connection details
        if !connection_details.is_empty() {
            info!("  🔍 Connection Details:");
            for (peer_id, state, last_heartbeat) in connection_details {
                let heartbeat_info = if let Some(time) = last_heartbeat {
                    format!("last heartbeat: {:?} ago", time.elapsed())
                } else {
                    "no heartbeat".to_string()
                };
                info!("    - {}: {:?} ({})", peer_id, state, heartbeat_info);
            }
        }
        
        // Test clipboard broadcast every 6 seconds (every 2nd iteration)
        if i % 2 == 0 && connection_count > 0 {
            let test_content = format!("Test clipboard content #{} - timestamp: {}", i/2, chrono::Utc::now().format("%H:%M:%S"));
            info!("📋 Broadcasting clipboard content: {}", test_content);
            network_manager.broadcast_clipboard_content(test_content.into_bytes()).await?;
        }
        
        // Reset stats every 15 seconds for demo
        if i % 5 == 0 {
            info!("🔄 Resetting network statistics...");
            network_manager.reset_network_stats().await;
        }
        
        println!(); // Add spacing between updates
    }
    
    // Final statistics summary
    info!("📊 Final Network Statistics Summary:");
    let final_stats = network_manager.get_network_stats().await;
    info!("  - Total peers discovered: {}", final_stats.peers_discovered);
    info!("  - Total peers connected: {}", final_stats.peers_connected);
    info!("  - Total messages sent: {}", final_stats.messages_sent);
    info!("  - Total bytes sent: {}", final_stats.bytes_sent);
    info!("  - Total discovery cycles: {}", final_stats.discovery_cycles);
    
    // Stop the network manager
    info!("Stopping network manager...");
    network_manager.stop().await?;
    
    info!("Network monitoring demo completed successfully!");
    Ok(())
}
