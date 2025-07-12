//! # CrossCopy
//!
//! CrossCopy is a cross-platform clipboard synchronization tool that enables
//! real-time clipboard content sharing between multiple devices over a network.
//!
//! ## Features
//!
//! - **Cross-platform**: Supports Windows, macOS, and Linux
//! - **Real-time sync**: Millisecond-level clipboard synchronization
//! - **Secure**: End-to-end encryption with AES-GCM
//! - **Lightweight**: Minimal system resource usage
//! - **Extensible**: Modular architecture for easy feature extension
//!
//! ## Quick Start
//!
//! ```rust,no_run
//! use crosscopy::{CrossCopyApp, config::AppConfig};
//!
//! #[tokio::main]
//! async fn main() -> crosscopy::Result<()> {
//!     let config = AppConfig::default();
//!     let mut app = CrossCopyApp::new(config).await?;
//!     app.run().await?;
//!     Ok(())
//! }
//! ```

pub mod clipboard;
pub mod config;
pub mod crypto;
pub mod events;
pub mod network;
pub mod utils;

use config::AppConfig;
use events::EventBus;
use log::{error, info, warn};
use std::sync::Arc;
use tokio::sync::RwLock;

/// Main application error type
pub type Result<T> = std::result::Result<T, Box<dyn std::error::Error + Send + Sync>>;

/// Main CrossCopy application
pub struct CrossCopyApp {
    config: AppConfig,
    event_bus: Arc<EventBus>,
    clipboard_monitor: Option<clipboard::ClipboardMonitor>,
    network_manager: Option<network::NetworkManager>,
    encryption_service: Option<crypto::EncryptionService>,
    running: Arc<RwLock<bool>>,
}

impl CrossCopyApp {
    /// Create a new CrossCopy application instance
    pub async fn new(config: AppConfig) -> Result<Self> {
        info!("Initializing CrossCopy application");

        let event_bus = Arc::new(EventBus::new());
        
        Ok(Self {
            config,
            event_bus,
            clipboard_monitor: None,
            network_manager: None,
            encryption_service: None,
            running: Arc::new(RwLock::new(false)),
        })
    }

    /// Start the CrossCopy application
    pub async fn run(&mut self) -> Result<()> {
        info!("Starting CrossCopy application");
        
        // Set running state
        *self.running.write().await = true;

        // Initialize encryption service
        self.init_encryption_service().await?;

        // Initialize network manager
        self.init_network_manager().await?;

        // Initialize clipboard monitor
        self.init_clipboard_monitor().await?;

        // Start all services
        self.start_services().await?;

        // Main event loop
        self.event_loop().await?;

        Ok(())
    }

    /// Shutdown the application gracefully
    pub async fn shutdown(&mut self) -> Result<()> {
        info!("Shutting down CrossCopy application");
        
        *self.running.write().await = false;

        // Stop clipboard monitor
        if let Some(monitor) = &mut self.clipboard_monitor {
            monitor.stop().await?;
        }

        // Stop network manager
        if let Some(manager) = &mut self.network_manager {
            manager.stop().await?;
        }

        info!("CrossCopy application shutdown complete");
        Ok(())
    }

    async fn init_encryption_service(&mut self) -> Result<()> {
        info!("Initializing encryption service");
        
        let encryption_service = crypto::EncryptionService::from_config(&self.config.security)?;
        self.encryption_service = Some(encryption_service);
        
        Ok(())
    }

    async fn init_network_manager(&mut self) -> Result<()> {
        info!("Initializing network manager");
        
        let network_manager = network::NetworkManager::new(
            self.config.network.clone(),
            self.event_bus.clone(),
        ).await?;
        
        self.network_manager = Some(network_manager);
        
        Ok(())
    }

    async fn init_clipboard_monitor(&mut self) -> Result<()> {
        info!("Initializing clipboard monitor");
        
        let clipboard_monitor = clipboard::ClipboardMonitor::new(
            self.config.clipboard.clone(),
            self.event_bus.clone(),
        )?;
        
        self.clipboard_monitor = Some(clipboard_monitor);
        
        Ok(())
    }

    async fn start_services(&mut self) -> Result<()> {
        info!("Starting all services");

        // Start network manager
        if let Some(manager) = &mut self.network_manager {
            manager.start().await?;
        }

        // Start clipboard monitor
        if let Some(monitor) = &mut self.clipboard_monitor {
            monitor.start().await?;
        }

        Ok(())
    }

    async fn event_loop(&self) -> Result<()> {
        info!("Entering main event loop");

        while *self.running.read().await {
            // Process events from the event bus
            if let Some(event) = self.event_bus.poll_event().await {
                if let Err(e) = self.handle_event(event).await {
                    error!("Error handling event: {}", e);
                }
            }

            // Small delay to prevent busy waiting
            tokio::time::sleep(tokio::time::Duration::from_millis(10)).await;
        }

        Ok(())
    }

    async fn handle_event(&self, event: events::Event) -> Result<()> {
        match event {
            events::Event::ClipboardChanged { content, device_id } => {
                self.handle_clipboard_change(content, device_id).await?;
            }
            events::Event::NetworkMessage { message, sender } => {
                self.handle_network_message(message, sender).await?;
            }
            events::Event::DeviceConnected { device_id } => {
                info!("Device connected: {}", device_id);
            }
            events::Event::DeviceDisconnected { device_id } => {
                warn!("Device disconnected: {}", device_id);
            }
            events::Event::Error { error } => {
                error!("Application error: {}", error);
            }
            events::Event::Heartbeat { device_id, timestamp } => {
                info!("Heartbeat from device {} at {}", device_id, timestamp);
            }
            events::Event::ConfigChanged { section } => {
                info!("Configuration changed in section: {}", section);
            }
            events::Event::Shutdown => {
                info!("Shutdown event received");
                *self.running.write().await = false;
            }
        }

        Ok(())
    }

    async fn handle_clipboard_change(
        &self,
        content: clipboard::ClipboardContent,
        device_id: String,
    ) -> Result<()> {
        info!("Handling clipboard change from device: {}", device_id);

        // Encrypt content if encryption is enabled
        let encrypted_content = if let Some(encryption_service) = &self.encryption_service {
            encryption_service.encrypt_content(&content)?
        } else {
            content.to_bytes()
        };

        // Send to network manager for distribution
        if let Some(network_manager) = &self.network_manager {
            network_manager.broadcast_clipboard_content(encrypted_content).await?;
        }

        Ok(())
    }

    async fn handle_network_message(
        &self,
        message: network::Message,
        sender: String,
    ) -> Result<()> {
        info!("Handling network message from: {}", sender);

        // Decrypt message if encryption is enabled
        let decrypted_content = if let Some(encryption_service) = &self.encryption_service {
            encryption_service.decrypt_message(&message)?
        } else {
            message.payload
        };

        // Update local clipboard
        if let Some(clipboard_monitor) = &self.clipboard_monitor {
            clipboard_monitor.update_clipboard(decrypted_content).await?;
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_app_creation() {
        let config = AppConfig::default();
        let app = CrossCopyApp::new(config).await;
        assert!(app.is_ok());
    }
}
