//! Clipboard monitoring implementation

use crate::clipboard::{ClipboardContent, ContentType, Result};
use crate::config::ClipboardConfig;
use crate::events::{Event, EventBus};
use arboard::Clipboard;
use log::{debug, error, info, warn};
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::RwLock;
use tokio::time::interval;

/// Clipboard monitor that watches for clipboard changes
pub struct ClipboardMonitor {
    clipboard: Arc<RwLock<Clipboard>>,
    config: ClipboardConfig,
    event_bus: Arc<EventBus>,
    last_content: Arc<RwLock<Option<String>>>,
    last_update: Arc<RwLock<Instant>>,
    running: Arc<RwLock<bool>>,
    device_id: String,
}

impl ClipboardMonitor {
    /// Create a new clipboard monitor
    pub fn new(
        config: ClipboardConfig,
        event_bus: Arc<EventBus>,
    ) -> Result<Self> {
        let clipboard = Clipboard::new()
            .map_err(|e| crate::clipboard::ClipboardError::AccessFailed(e.to_string()))?;

        Ok(Self {
            clipboard: Arc::new(RwLock::new(clipboard)),
            config,
            event_bus,
            last_content: Arc::new(RwLock::new(None)),
            last_update: Arc::new(RwLock::new(Instant::now())),
            running: Arc::new(RwLock::new(false)),
            device_id: uuid::Uuid::new_v4().to_string(),
        })
    }

    /// Start monitoring clipboard changes
    pub async fn start(&mut self) -> Result<()> {
        info!("Starting clipboard monitor");
        *self.running.write().await = true;

        let clipboard = self.clipboard.clone();
        let config = self.config.clone();
        let event_bus = self.event_bus.clone();
        let last_content = self.last_content.clone();
        let last_update = self.last_update.clone();
        let running = self.running.clone();
        let device_id = self.device_id.clone();

        tokio::spawn(async move {
            let mut interval = interval(Duration::from_millis(100));

            while *running.read().await {
                interval.tick().await;

                if let Err(e) = Self::check_clipboard_change(
                    &clipboard,
                    &config,
                    &event_bus,
                    &last_content,
                    &last_update,
                    &device_id,
                ).await {
                    error!("Error checking clipboard: {}", e);
                }
            }
        });

        Ok(())
    }

    /// Stop monitoring clipboard changes
    pub async fn stop(&mut self) -> Result<()> {
        info!("Stopping clipboard monitor");
        *self.running.write().await = false;
        Ok(())
    }

    /// Update clipboard with new content
    pub async fn update_clipboard(&self, content: Vec<u8>) -> Result<()> {
        debug!("Updating clipboard with {} bytes", content.len());

        // Convert bytes to string (assuming text content for now)
        let text = String::from_utf8(content)
            .map_err(|e| crate::clipboard::ClipboardError::AccessFailed(e.to_string()))?;

        let mut clipboard = self.clipboard.write().await;
        clipboard.set_text(&text)
            .map_err(|e| crate::clipboard::ClipboardError::AccessFailed(e.to_string()))?;

        // Update last content to prevent echo
        *self.last_content.write().await = Some(text);
        *self.last_update.write().await = Instant::now();

        Ok(())
    }

    async fn check_clipboard_change(
        clipboard: &Arc<RwLock<Clipboard>>,
        config: &ClipboardConfig,
        event_bus: &Arc<EventBus>,
        last_content: &Arc<RwLock<Option<String>>>,
        last_update: &Arc<RwLock<Instant>>,
        device_id: &str,
    ) -> Result<()> {
        let now = Instant::now();

        // Check cooldown period
        {
            let last_update_time = *last_update.read().await;
            if now.duration_since(last_update_time) < config.cooldown_duration() {
                return Ok(());
            }
        }

        // Try to get different types of clipboard content
        let clipboard_content = {
            let mut clipboard = clipboard.write().await;

            // Try text first
            if let Ok(text) = clipboard.get_text() {
                Some(ClipboardContent::new_text(text, device_id.to_string()))
            } else if config.sync_images {
                // Try image content
                if let Ok(image) = clipboard.get_image() {
                    let image_data = Self::convert_image_to_bytes(image)?;
                    Some(ClipboardContent::new_image(
                        image_data,
                        "image/png".to_string(),
                        device_id.to_string(),
                    ))
                } else {
                    None
                }
            } else {
                None
            }
        };

        if let Some(content) = clipboard_content {
            // Check if content has changed
            let should_process = {
                let last_content_guard = last_content.read().await;
                match &*last_content_guard {
                    Some(last) => {
                        // For text content, compare the actual text
                        if let Some(current_text) = content.as_text() {
                            last != &current_text
                        } else {
                            // For non-text content, always process (could be improved with checksum comparison)
                            true
                        }
                    }
                    None => true,
                }
            };

            if should_process {
                debug!("Clipboard content changed: {} bytes", content.metadata.size);

                // Check content size limit
                if content.metadata.size > config.max_content_size {
                    warn!(
                        "Clipboard content too large: {} bytes (max: {} bytes)",
                        content.metadata.size,
                        config.max_content_size
                    );
                    return Ok(());
                }

                // Compress content if enabled and above threshold
                let mut final_content = content;
                #[cfg(feature = "compression")]
                if config.enable_compression && final_content.metadata.size > config.compression_threshold {
                    if let Err(e) = final_content.compress() {
                        warn!("Failed to compress clipboard content: {}", e);
                    } else {
                        debug!("Compressed content from {} to {} bytes",
                               final_content.metadata.size, final_content.data.len());
                    }
                }

                // Emit clipboard changed event
                let event = Event::ClipboardChanged {
                    content: final_content.clone(),
                    device_id: device_id.to_string(),
                };

                if let Err(e) = event_bus.emit(event).await {
                    error!("Failed to emit clipboard changed event: {}", e);
                }

                // Update last content and timestamp
                if let Some(text) = final_content.as_text() {
                    *last_content.write().await = Some(text);
                }
                *last_update.write().await = now;
            }
        }

        Ok(())
    }

    /// Convert arboard image to bytes
    fn convert_image_to_bytes(image: arboard::ImageData) -> Result<Vec<u8>> {
        // This is a simplified conversion - in a real implementation,
        // you would properly encode the image data to PNG or another format
        Ok(image.bytes.into_owned())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::ClipboardConfig;
    use crate::events::EventBus;

    #[tokio::test]
    async fn test_clipboard_monitor_creation() {
        let config = ClipboardConfig::default();
        let event_bus = Arc::new(EventBus::new());
        
        let monitor = ClipboardMonitor::new(config, event_bus);
        assert!(monitor.is_ok());
    }
}
