use crosscopy::{
    config::{AppConfig, ConfigManager},
    utils::logger,
    CrossCopyApp, Result,
};
use log::{error, info};
use std::env;

#[tokio::main]
async fn main() -> Result<()> {
    // Parse command line arguments
    let args: Vec<String> = env::args().collect();
    let config_path = args.get(1).map(|s| s.as_str());

    // Load configuration
    let config_manager = ConfigManager::new(config_path)?;
    let config = config_manager.load_config().await?;

    // Initialize logger
    logger::init_logger(&config.logging.level)?;
    info!("CrossCopy v{} starting...", env!("CARGO_PKG_VERSION"));

    // Create and start the application
    let mut app = CrossCopyApp::new(config).await?;
    
    // Handle graceful shutdown
    let shutdown_signal = tokio::signal::ctrl_c();
    
    tokio::select! {
        result = app.run() => {
            match result {
                Ok(_) => info!("CrossCopy stopped normally"),
                Err(e) => error!("CrossCopy stopped with error: {}", e),
            }
        }
        _ = shutdown_signal => {
            info!("Received shutdown signal, stopping CrossCopy...");
            app.shutdown().await?;
        }
    }

    Ok(())
}
