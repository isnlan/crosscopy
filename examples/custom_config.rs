//! Custom configuration example for CrossCopy
//!
//! This example demonstrates how to create and use a custom configuration
//! with specific settings for network, security, and clipboard options.
//!
//! Run with: cargo run --example custom_config

use crosscopy::{
    config::{
        AppConfig, ClipboardConfig, LoggingConfig, NetworkConfig, SecurityConfig,
    },
    utils::logger,
    CrossCopyApp,
};
use log::info;
use std::time::Duration;
use tokio::signal;

#[tokio::main]
async fn main() -> crosscopy::Result<()> {
    // Initialize logging with debug level
    logger::init_logger("debug")?;
    
    info!("Starting CrossCopy custom configuration example");

    // Create custom configuration
    let config = create_custom_config();
    
    info!("Custom Configuration:");
    info!("  Device Name: {}", config.device_name);
    info!("  Listen Port: {}", config.network.listen_port);
    info!("  Max Content Size: {} bytes", config.clipboard.max_content_size);
    info!("  Encryption Enabled: {}", config.security.enable_encryption);
    info!("  mDNS Discovery Enabled: {}", config.network.enable_mdns);

    // Create and start the application
    let mut app = CrossCopyApp::new(config).await?;
    
    info!("CrossCopy application created with custom configuration");
    info!("Starting application... Press Ctrl+C to stop");

    // Set up graceful shutdown
    let shutdown_signal = signal::ctrl_c();
    
    // Run the application
    tokio::select! {
        result = app.run() => {
            match result {
                Ok(_) => info!("CrossCopy stopped normally"),
                Err(e) => eprintln!("CrossCopy stopped with error: {}", e),
            }
        }
        _ = shutdown_signal => {
            info!("Received shutdown signal, stopping CrossCopy...");
            app.shutdown().await?;
        }
        _ = tokio::time::sleep(Duration::from_secs(60)) => {
            info!("Example timeout reached, stopping CrossCopy...");
            app.shutdown().await?;
        }
    }

    info!("CrossCopy custom configuration example completed");
    Ok(())
}

fn create_custom_config() -> AppConfig {
    AppConfig {
        device_name: "CustomCrossCopy".to_string(),
        device_id: uuid::Uuid::new_v4().to_string(),
        
        network: NetworkConfig {
            listen_port: 9999,
            connection_timeout: 10000, // 10 seconds
            heartbeat_interval: 2000,  // 2 seconds
            max_connections: 20,
            enable_mdns: true,         // Enable mDNS automatic peer discovery
            mdns_discovery_interval: 30, // 30 seconds
            idle_connection_timeout: 300, // 5 minutes
            enable_quic: false,        // TCP only for this example
            quic_port: None,
        },
        
        clipboard: ClipboardConfig {
            sync_images: true,
            sync_files: true,
            cooldown_millis: 500, // 500ms cooldown
            max_content_size: 50 * 1024 * 1024, // 50MB
            enable_compression: true,
            compression_threshold: 10 * 1024, // 10KB
        },
        
        security: SecurityConfig {
            secret_key: "my-super-secret-key-2024".to_string(),
            enable_encryption: true,
            key_rotation_interval: 24 * 60 * 60, // 24 hours
            enable_authentication: true,
            max_message_age: 600, // 10 minutes
        },
        
        logging: LoggingConfig {
            level: "debug".to_string(),
            file_path: Some("crosscopy.log".to_string()),
            structured: true,
            max_file_size: 100 * 1024 * 1024, // 100MB
            max_files: 10,
        },
    }
}
