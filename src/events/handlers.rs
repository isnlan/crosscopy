//! Event handlers implementation

use crate::events::{Event, Result};
use log::{debug, error, info, warn};

/// Re-export the EventHandler trait from bus module
pub use super::bus::EventHandler;

/// Default event handler that logs all events
pub struct LoggingEventHandler {
    name: String,
}

impl LoggingEventHandler {
    pub fn new() -> Self {
        Self {
            name: "LoggingEventHandler".to_string(),
        }
    }
}

impl EventHandler for LoggingEventHandler {
    fn handle(&self, event: &Event) -> Result<()> {
        match event {
            Event::ClipboardChanged { device_system, .. } => {
                info!("Clipboard changed on device: {}", device_system);
            }
            Event::NetworkMessage { sender, .. } => {
                debug!("Network message received from: {}", sender);
            }
            Event::DeviceConnected { device_system } => {
                info!("Device connected: {}", device_system);
            }
            Event::DeviceDisconnected { device_system } => {
                warn!("Device disconnected: {}", device_system);
            }
            Event::Error { error } => {
                error!("Application error: {}", error);
            }
            Event::Heartbeat { device_system, timestamp } => {
                debug!("Heartbeat from device {} at {}", device_system, timestamp);
            }
            Event::ConfigChanged { section } => {
                info!("Configuration changed in section: {}", section);
            }
            Event::Shutdown => {
                info!("Shutdown event received");
            }
        }

        Ok(())
    }

    fn name(&self) -> &str {
        &self.name
    }
}

impl Default for LoggingEventHandler {
    fn default() -> Self {
        Self::new()
    }
}

/// Event handler for metrics collection
pub struct MetricsEventHandler {
    name: String,
    // Add metrics collection fields here
}

impl MetricsEventHandler {
    pub fn new() -> Self {
        Self {
            name: "MetricsEventHandler".to_string(),
        }
    }
}

impl EventHandler for MetricsEventHandler {
    fn handle(&self, event: &Event) -> Result<()> {
        // Collect metrics based on event type
        match event {
            Event::ClipboardChanged { .. } => {
                // Increment clipboard change counter
                debug!("Metrics: Clipboard change recorded");
            }
            Event::NetworkMessage { .. } => {
                // Increment network message counter
                debug!("Metrics: Network message recorded");
            }
            Event::DeviceConnected { .. } => {
                // Update connected devices count
                debug!("Metrics: Device connection recorded");
            }
            Event::DeviceDisconnected { .. } => {
                // Update connected devices count
                debug!("Metrics: Device disconnection recorded");
            }
            Event::Error { .. } => {
                // Increment error counter
                debug!("Metrics: Error recorded");
            }
            _ => {
                // Handle other events for metrics
            }
        }
        
        Ok(())
    }

    fn name(&self) -> &str {
        &self.name
    }
}

impl Default for MetricsEventHandler {
    fn default() -> Self {
        Self::new()
    }
}
