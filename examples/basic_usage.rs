//! Basic usage example for CrossCopy
//!
//! This example demonstrates how to set up and run a basic CrossCopy instance
//! with default configuration.
//!
//! Run with: cargo run --example basic_usage

use crosscopy::{
    config::AppConfig,
    utils::logger,
    CrossCopyApp,
};
use log::info;
use std::time::Duration;
use tokio::signal;

#[tokio::main]
async fn main() -> crosscopy::Result<()> {
    // Initialize logging
    logger::init_logger("info")?;
    
    info!("Starting CrossCopy basic usage example");

    // Create default configuration
    let config = AppConfig::default();
    
    info!("Configuration:");
    info!("  Device Name: {}", config.device_name);
    info!("  Device ID: {}", config.device_id);
    info!("  Listen Port: {}", config.network.listen_port);
    info!("  Encryption: {}", config.security.enable_encryption);

    // Create and start the application
    let mut app = CrossCopyApp::new(config).await?;
    
    info!("CrossCopy application created successfully");
    info!("Starting application... Press Ctrl+C to stop");

    // Set up graceful shutdown
    let shutdown_signal = signal::ctrl_c();
    
    // Run the application with timeout for this example
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
        _ = tokio::time::sleep(Duration::from_secs(30)) => {
            info!("Example timeout reached, stopping CrossCopy...");
            app.shutdown().await?;
        }
    }

    info!("CrossCopy basic usage example completed");
    Ok(())
}
