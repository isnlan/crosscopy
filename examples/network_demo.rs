//! Network communication demonstration for CrossCopy
//!
//! This example demonstrates how to set up multiple CrossCopy instances
//! that communicate with each other over the network.
//!
//! Run with: cargo run --example network_demo

use crosscopy::{
    config::{AppConfig, NetworkConfig, PeerConfig, SecurityConfig},
    utils::logger,
    CrossCopyApp,
};
use log::{info, warn};
use std::time::Duration;
use tokio::time::sleep;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize logging
    logger::init_logger("info")?;
    
    info!("Starting CrossCopy network communication demonstration");

    // Create two instances that will communicate with each other
    let config1 = create_network_config(8881, vec![(8882, "device-2")]);
    let config2 = create_network_config(8882, vec![(8881, "device-1")]);

    info!("Configuration 1:");
    info!("  Device: {} (port {})", config1.device_name, config1.network.listen_port);
    info!("  Peers: {:?}", config1.network.peer_list.iter().map(|p| &p.name).collect::<Vec<_>>());

    info!("Configuration 2:");
    info!("  Device: {} (port {})", config2.device_name, config2.network.listen_port);
    info!("  Peers: {:?}", config2.network.peer_list.iter().map(|p| &p.name).collect::<Vec<_>>());

    // Create applications
    let mut app1 = CrossCopyApp::new(config1).await?;
    let mut app2 = CrossCopyApp::new(config2).await?;

    info!("Created two CrossCopy instances");

    // Start both applications in separate tasks
    info!("Starting applications...");
    
    let app1_handle = tokio::spawn(async move {
        if let Err(e) = app1.run().await {
            warn!("App1 error: {}", e);
        }
    });

    let app2_handle = tokio::spawn(async move {
        if let Err(e) = app2.run().await {
            warn!("App2 error: {}", e);
        }
    });

    // Give applications time to start and connect
    info!("Waiting for applications to start and connect...");
    sleep(Duration::from_secs(3)).await;

    // Simulate some network activity
    info!("Applications should now be running and attempting to connect");
    info!("In a real scenario, clipboard changes on one device would be");
    info!("automatically synchronized to the other device.");

    // Let them run for a while to demonstrate connection attempts
    info!("Letting applications run for 10 seconds...");
    sleep(Duration::from_secs(10)).await;

    // Stop the applications
    info!("Stopping applications...");
    app1_handle.abort();
    app2_handle.abort();

    // Wait a bit for cleanup
    sleep(Duration::from_millis(500)).await;

    info!("Network communication demonstration completed");
    info!("Note: In this demo, actual clipboard synchronization may not occur");
    info!("because we're running in a test environment without real clipboard access.");
    
    Ok(())
}

fn create_network_config(port: u16, peers: Vec<(u16, &str)>) -> AppConfig {
    let device_id = format!("device-{}", port);
    let device_name = format!("CrossCopy-{}", port);

    let peer_list = peers
        .into_iter()
        .map(|(peer_port, peer_id)| PeerConfig {
            device_id: peer_id.to_string(),
            name: format!("Peer-{}", peer_port),
            address: "127.0.0.1".to_string(), // localhost for demo
            port: peer_port,
            enabled: true,
        })
        .collect();

    AppConfig {
        device_name,
        device_id,
        network: NetworkConfig {
            listen_port: port,
            peer_list,
            connection_timeout: 5000,
            heartbeat_interval: 2000,
            max_connections: 10,
            auto_discovery: false, // Disabled for controlled demo
            discovery_port: port + 1000,
        },
        clipboard: crosscopy::config::ClipboardConfig {
            sync_images: false, // Simplified for demo
            sync_files: false,
            cooldown_millis: 300,
            max_content_size: 1024 * 1024, // 1MB
            enable_compression: false,
            compression_threshold: 1024,
        },
        security: SecurityConfig {
            secret_key: "demo-secret-key".to_string(),
            enable_encryption: false, // Disabled for simpler demo
            key_rotation_interval: 86400,
            enable_authentication: false,
            max_message_age: 300,
        },
        logging: crosscopy::config::LoggingConfig {
            level: "info".to_string(),
            file_path: None,
            structured: false,
            max_file_size: 10 * 1024 * 1024,
            max_files: 5,
        },
    }
}
