//! Network discovery demonstration
//! 
//! This example demonstrates the network discovery functionality
//! including mDNS peer discovery and connection management.

use crosscopy::{
    config::{AppConfig, NetworkConfig},
    events::EventBus,
    network::NetworkManager,
};
use log::info;
use std::sync::Arc;
use tokio::time::{sleep, Duration};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize logging
    env_logger::init();
    
    info!("CrossCopy Network Discovery Demo");
    info!("================================");
    
    // Create network configuration with mDNS enabled
    let config = NetworkConfig {
        listen_port: 8888,
        enable_mdns: true,
        mdns_discovery_interval: 5, // Discover every 5 seconds for demo
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
    
    // Create event bus and network manager
    let event_bus = Arc::new(EventBus::new());
    let mut network_manager = NetworkManager::new(config, event_bus).await?;
    
    // Start the network manager
    info!("Starting network manager...");
    network_manager.start().await?;
    
    // Let it run for a while to demonstrate discovery
    info!("Running discovery for 30 seconds...");
    for i in 1..=6 {
        sleep(Duration::from_secs(5)).await;
        
        let connection_count = network_manager.get_connection_count().await;
        let connected_peers = network_manager.get_connected_peers().await;
        
        info!("Status update {} - Connections: {}", i, connection_count);
        if !connected_peers.is_empty() {
            info!("  Connected peers: {:?}", connected_peers);
        }
        
        // Test clipboard broadcast
        if connection_count > 0 {
            let test_content = format!("Test clipboard content {}", i);
            info!("Broadcasting clipboard content: {}", test_content);
            network_manager.broadcast_clipboard_content(test_content.into_bytes()).await?;
        }
    }
    
    // Stop the network manager
    info!("Stopping network manager...");
    network_manager.stop().await?;
    
    info!("Demo completed successfully!");
    Ok(())
}
