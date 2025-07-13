//! Configuration management demonstration for CrossCopy
//!
//! This example demonstrates how to load, save, and manage configuration
//! files for CrossCopy applications.
//!
//! Run with: cargo run --example config_management

use crosscopy::{
    config::{AppConfig, ConfigManager, ClipboardConfig, NetworkConfig, SecurityConfig, LoggingConfig},
    utils::logger,
};
use log::info;
use std::path::PathBuf;
use tempfile::tempdir;

#[tokio::main]
async fn main() -> crosscopy::Result<()> {
    // Initialize logging
    logger::init_logger("info")?;
    
    info!("Starting CrossCopy configuration management demonstration");

    // Create a temporary directory for this demo
    let temp_dir = tempdir()?;
    let config_path = temp_dir.path().join("crosscopy_demo.toml");
    
    info!("Using temporary config file: {}", config_path.display());

    // Demonstrate creating a config manager
    info!("\n--- Creating Configuration Manager ---");
    let config_manager = ConfigManager::new(Some(config_path.to_str().unwrap()))?;
    info!("✓ Configuration manager created");

    // Demonstrate loading default configuration (file doesn't exist yet)
    info!("\n--- Loading Default Configuration ---");
    let default_config = config_manager.load_config().await?;
    info!("✓ Default configuration loaded");
    info!("  Device Name: {}", default_config.device_name);
    info!("  Device System: {}", default_config.device_system);
    info!("  Listen Port: {}", default_config.network.listen_port);
    
    // Verify config file was created
    assert!(config_manager.config_exists(), "Config file should exist after loading");
    info!("✓ Configuration file created automatically");

    // Demonstrate creating a custom configuration
    info!("\n--- Creating Custom Configuration ---");
    let custom_config = create_custom_configuration();
    info!("✓ Custom configuration created");
    info!("  Device Name: {}", custom_config.device_name);
    info!("  Listen Port: {}", custom_config.network.listen_port);
    info!("  Encryption: {}", custom_config.security.enable_encryption);
    info!("  mDNS Discovery: {}", custom_config.network.enable_mdns);

    // Demonstrate saving custom configuration
    info!("\n--- Saving Custom Configuration ---");
    config_manager.save_config(&custom_config).await?;
    info!("✓ Custom configuration saved to file");

    // Demonstrate reloading configuration from file
    info!("\n--- Reloading Configuration ---");
    let reloaded_config = config_manager.reload_config().await?;
    info!("✓ Configuration reloaded from file");
    
    // Verify the reloaded config matches what we saved
    assert_eq!(custom_config.device_name, reloaded_config.device_name);
    assert_eq!(custom_config.network.listen_port, reloaded_config.network.listen_port);
    assert_eq!(custom_config.security.enable_encryption, reloaded_config.security.enable_encryption);
    info!("✓ Reloaded configuration matches saved configuration");

    // Demonstrate configuration validation
    info!("\n--- Configuration Validation ---");
    
    // Test valid configuration
    match ConfigManager::validate_config(&reloaded_config) {
        Ok(_) => info!("✓ Configuration validation passed"),
        Err(e) => panic!("Valid configuration failed validation: {}", e),
    }
    
    // Test invalid configurations
    let mut invalid_config = reloaded_config.clone();
    
    // Invalid port
    invalid_config.network.listen_port = 0;
    match ConfigManager::validate_config(&invalid_config) {
        Ok(_) => panic!("Invalid port should fail validation"),
        Err(_) => info!("✓ Invalid port properly rejected"),
    }
    
    // Invalid max connections
    invalid_config.network.listen_port = 8888; // Fix port
    invalid_config.network.max_connections = 0;
    match ConfigManager::validate_config(&invalid_config) {
        Ok(_) => panic!("Invalid max connections should fail validation"),
        Err(_) => info!("✓ Invalid max connections properly rejected"),
    }
    
    // Empty secret key
    invalid_config.network.max_connections = 10; // Fix max connections
    invalid_config.security.secret_key = String::new();
    match ConfigManager::validate_config(&invalid_config) {
        Ok(_) => panic!("Empty secret key should fail validation"),
        Err(_) => info!("✓ Empty secret key properly rejected"),
    }

    // Demonstrate configuration file watching
    info!("\n--- Configuration File Watching ---");
    let mut config_watcher = crosscopy::config::manager::ConfigWatcher::new(config_manager);
    
    // Check for changes (should be false initially)
    let has_changes = config_watcher.check_for_changes().await?;
    info!("Initial change check: {}", has_changes);
    
    // Modify the configuration and save it
    let mut modified_config = reloaded_config.clone();
    modified_config.device_name = "Modified-Device".to_string();
    config_watcher.get_manager().save_config(&modified_config).await?;
    
    // Check for changes again (should be true now)
    tokio::time::sleep(tokio::time::Duration::from_millis(100)).await; // Ensure file timestamp changes
    let has_changes = config_watcher.check_for_changes().await?;
    info!("After modification change check: {}", has_changes);

    // Demonstrate different configuration scenarios
    info!("\n--- Configuration Scenarios ---");
    
    // Minimal configuration
    let minimal_config = create_minimal_configuration();
    info!("Minimal configuration created:");
    info!("  Device Name: {}", minimal_config.device_name);
    info!("  Encryption: {}", minimal_config.security.enable_encryption);
    
    // High-security configuration
    let secure_config = create_high_security_configuration();
    info!("High-security configuration created:");
    info!("  Encryption: {}", secure_config.security.enable_encryption);
    info!("  Authentication: {}", secure_config.security.enable_authentication);
    info!("  Key Rotation: {}s", secure_config.security.key_rotation_interval);
    
    // Performance-optimized configuration
    let performance_config = create_performance_configuration();
    info!("Performance-optimized configuration created:");
    info!("  Max Content Size: {}MB", performance_config.clipboard.max_content_size / (1024 * 1024));
    info!("  Compression: {}", performance_config.clipboard.enable_compression);
    info!("  Max Connections: {}", performance_config.network.max_connections);

    // Show final configuration file content
    info!("\n--- Final Configuration File ---");
    let config_content = tokio::fs::read_to_string(config_path).await?;
    info!("Configuration file content:\n{}", config_content);

    info!("\nConfiguration management demonstration completed successfully!");
    info!("Temporary files will be cleaned up automatically.");
    
    Ok(())
}

fn create_custom_configuration() -> AppConfig {
    AppConfig {
        device_name: "Demo-CrossCopy".to_string(),
        device_system: "DemoOS 1.0".to_string(),
        
        network: NetworkConfig {
            listen_port: 9876,
            connection_timeout: 8000,
            heartbeat_interval: 3000,
            max_connections: 15,
            enable_mdns: true,           // Enable mDNS automatic peer discovery
            mdns_discovery_interval: 30, // 30 seconds
            idle_connection_timeout: 300, // 5 minutes
            enable_quic: false,          // TCP only for this demo
            quic_port: None,
        },
        
        clipboard: ClipboardConfig {
            sync_images: true,
            sync_files: true,
            cooldown_millis: 250,
            max_content_size: 25 * 1024 * 1024, // 25MB
            enable_compression: true,
            compression_threshold: 5 * 1024, // 5KB
        },
        
        security: SecurityConfig {
            secret_key: "demo-secret-key-2024".to_string(),
            enable_encryption: true,
            key_rotation_interval: 12 * 60 * 60, // 12 hours
            enable_authentication: true,
            max_message_age: 180, // 3 minutes
        },
        
        logging: LoggingConfig {
            level: "info".to_string(),
            file_path: Some("crosscopy_demo.log".to_string()),
            structured: false,
            max_file_size: 50 * 1024 * 1024, // 50MB
            max_files: 7,
        },
    }
}

fn create_minimal_configuration() -> AppConfig {
    let mut config = AppConfig::default();
    config.device_name = "Minimal-CrossCopy".to_string();
    config.security.enable_encryption = false;
    config.clipboard.sync_images = false;
    config.clipboard.sync_files = false;
    config.network.enable_mdns = false; // Disable automatic discovery for minimal config
    config
}

fn create_high_security_configuration() -> AppConfig {
    let mut config = AppConfig::default();
    config.device_name = "Secure-CrossCopy".to_string();
    config.security.enable_encryption = true;
    config.security.enable_authentication = true;
    config.security.key_rotation_interval = 60 * 60; // 1 hour
    config.security.max_message_age = 30; // 30 seconds
    config.security.secret_key = "ultra-secure-key-with-high-entropy-2024".to_string();
    config
}

fn create_performance_configuration() -> AppConfig {
    let mut config = AppConfig::default();
    config.device_name = "Performance-CrossCopy".to_string();
    config.clipboard.max_content_size = 100 * 1024 * 1024; // 100MB
    config.clipboard.enable_compression = true;
    config.clipboard.compression_threshold = 1024; // 1KB
    config.clipboard.cooldown_millis = 100; // Very responsive
    config.network.max_connections = 50;
    config.network.connection_timeout = 2000; // Fast timeout
    config.network.heartbeat_interval = 1000; // Frequent heartbeats
    config
}
