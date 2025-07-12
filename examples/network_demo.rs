//! Network communication demonstration for CrossCopy
//!
//! This example demonstrates how to set up multiple CrossCopy instances
//! that communicate with each other over the network.
//!
//! Run with: cargo run --example network_demo

use crosscopy::{
    config::{AppConfig, NetworkConfig, SecurityConfig},
    utils::logger,
    CrossCopyApp,
};
use log::{info, warn};
use std::time::Duration;
use tokio::time::sleep;

#[tokio::main]
async fn main() -> crosscopy::Result<()> {
    // Initialize logging
    logger::init_logger("info")?;
    
    info!("Starting CrossCopy network communication demonstration");

    // Create two instances that will discover each other via mDNS
    let config1 = create_network_config(8881);
    let config2 = create_network_config(8882);

    info!("Configuration 1:");
    info!("  Device: {} (port {})", config1.device_name, config1.network.listen_port);
    info!("  mDNS Discovery: {}", config1.network.enable_mdns);

    info!("Configuration 2:");
    info!("  Device: {} (port {})", config2.device_name, config2.network.listen_port);
    info!("  mDNS Discovery: {}", config2.network.enable_mdns);

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
    info!("Note: In this demo, the two instances will discover each other via mDNS");
    info!("and establish connections automatically. Actual clipboard synchronization");
    info!("may not occur because we're running in a test environment without real clipboard access.");
    
    Ok(())
}

fn create_network_config(port: u16) -> AppConfig {
    let device_id = format!("device-{}", port);
    let device_name = format!("CrossCopy-{}", port);

    AppConfig {
        device_name,
        device_id,
        network: NetworkConfig {
            listen_port: port,
            connection_timeout: 5000,
            heartbeat_interval: 2000,
            max_connections: 10,
            enable_mdns: true,           // Enable mDNS for automatic discovery
            mdns_discovery_interval: 10, // Faster discovery for demo (10 seconds)
            idle_connection_timeout: 60, // Shorter timeout for demo (1 minute)
            enable_quic: false,          // TCP only for demo
            quic_port: None,
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
