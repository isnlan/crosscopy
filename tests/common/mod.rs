//! Common test utilities

use crosscopy::config::{AppConfig, ClipboardConfig, NetworkConfig, SecurityConfig, LoggingConfig};
use std::sync::Once;
use tempfile::TempDir;

static INIT: Once = Once::new();

/// Initialize test environment (logging, etc.)
pub fn init_test_env() {
    INIT.call_once(|| {
        // Initialize test logging
        let _ = env_logger::builder()
            .filter_level(log::LevelFilter::Debug)
            .is_test(true)
            .try_init();
    });
}

/// Create a test configuration with unique identifiers
pub fn create_test_config(port: u16) -> AppConfig {
    AppConfig {
        device_name: format!("test-device-{}", port),
        device_id: format!("device-{}", port),
        network: NetworkConfig {
            listen_port: port,
            connection_timeout: 1000, // Shorter for tests
            heartbeat_interval: 500,
            max_connections: 5,
            enable_mdns: false,       // Disable for controlled tests
            mdns_discovery_interval: 30,
            idle_connection_timeout: 60, // Shorter for tests
            enable_quic: false,       // TCP only for tests
            quic_port: None,
        },
        clipboard: ClipboardConfig {
            sync_images: false, // Disable for simpler tests
            sync_files: false,
            cooldown_millis: 50, // Very short for tests
            max_content_size: 1024 * 10, // 10KB for tests
            enable_compression: false,
            compression_threshold: 1024,
        },
        security: SecurityConfig {
            secret_key: format!("test-secret-{}", port),
            enable_encryption: false, // Disable for simpler tests
            key_rotation_interval: 3600,
            enable_authentication: false,
            max_message_age: 60,
        },
        logging: LoggingConfig {
            level: "debug".to_string(),
            file_path: None,
            structured: false,
            max_file_size: 1024 * 1024,
            max_files: 3,
        },
    }
}

/// Create a test configuration with encryption enabled
pub fn create_secure_test_config(port: u16) -> AppConfig {
    let mut config = create_test_config(port);
    config.security.enable_encryption = true;
    config.security.enable_authentication = true;
    config
}

/// Create a test configuration with mDNS discovery enabled
pub fn create_networked_test_config(port: u16, _peers: Vec<(String, String, u16)>) -> AppConfig {
    let mut config = create_test_config(port);

    // Enable mDNS discovery for networked tests
    config.network.enable_mdns = true;
    config.network.mdns_discovery_interval = 5; // Faster discovery for tests

    config
}

/// Test helper for creating temporary directories
pub struct TestDir {
    pub temp_dir: TempDir,
}

impl TestDir {
    pub fn new() -> Self {
        Self {
            temp_dir: tempfile::tempdir().expect("Failed to create temp directory"),
        }
    }

    pub fn path(&self) -> &std::path::Path {
        self.temp_dir.path()
    }

    pub fn config_path(&self) -> std::path::PathBuf {
        self.path().join("config.toml")
    }
}

/// Mock clipboard content for testing
pub fn create_test_clipboard_content(text: &str, device_id: &str) -> crosscopy::clipboard::ClipboardContent {
    crosscopy::clipboard::ClipboardContent::new_text(
        text.to_string(),
        device_id.to_string(),
    )
}

/// Mock network message for testing
pub fn create_test_message(
    msg_type: crosscopy::network::MessageType,
    payload: Vec<u8>,
    device_id: &str,
) -> crosscopy::network::Message {
    crosscopy::network::Message::new(msg_type, payload, device_id.to_string())
}

/// Test event emitter helper
pub struct TestEventEmitter {
    pub event_bus: std::sync::Arc<crosscopy::events::EventBus>,
}

impl TestEventEmitter {
    pub fn new() -> Self {
        Self {
            event_bus: std::sync::Arc::new(crosscopy::events::EventBus::new()),
        }
    }

    pub async fn emit_clipboard_change(&self, content: crosscopy::clipboard::ClipboardContent, device_id: &str) {
        let event = crosscopy::events::Event::ClipboardChanged {
            content,
            device_id: device_id.to_string(),
        };
        self.event_bus.emit(event).await.unwrap();
    }

    pub async fn emit_network_message(&self, message: crosscopy::network::Message, sender: &str) {
        let event = crosscopy::events::Event::NetworkMessage {
            message,
            sender: sender.to_string(),
        };
        self.event_bus.emit(event).await.unwrap();
    }

    pub async fn wait_for_event(&self) -> Option<crosscopy::events::Event> {
        // Simple polling with timeout
        for _ in 0..100 {
            if let Some(event) = self.event_bus.poll_event().await {
                return Some(event);
            }
            tokio::time::sleep(tokio::time::Duration::from_millis(10)).await;
        }
        None
    }
}

/// Test metrics collector
pub struct TestMetrics {
    pub metrics: crosscopy::utils::metrics::PerformanceMetrics,
}

impl TestMetrics {
    pub fn new() -> Self {
        Self {
            metrics: crosscopy::utils::metrics::PerformanceMetrics::new(),
        }
    }

    pub async fn assert_counter(&self, name: &str, expected: u64) {
        let actual = self.metrics.get_counter(name).await;
        assert_eq!(actual, expected, "Counter '{}' mismatch", name);
    }

    pub async fn assert_gauge(&self, name: &str, expected: f64) {
        let actual = self.metrics.get_gauge(name).await;
        assert_eq!(actual, Some(expected), "Gauge '{}' mismatch", name);
    }

    pub async fn assert_timer_exists(&self, name: &str) {
        let avg = self.metrics.get_average_duration(name).await;
        assert!(avg.is_some(), "Timer '{}' should exist", name);
    }
}

/// Async test timeout helper
pub async fn with_timeout<F, T>(duration: std::time::Duration, future: F) -> Result<T, &'static str>
where
    F: std::future::Future<Output = T>,
{
    tokio::time::timeout(duration, future)
        .await
        .map_err(|_| "Test timed out")
}

/// Port allocation for tests to avoid conflicts
static mut NEXT_PORT: u16 = 19000;

pub fn get_test_port() -> u16 {
    unsafe {
        let port = NEXT_PORT;
        NEXT_PORT += 1;
        port
    }
}

/// Test assertion helpers
#[macro_export]
macro_rules! assert_event_type {
    ($event:expr, $pattern:pat) => {
        match $event {
            $pattern => {},
            _ => panic!("Event type mismatch: expected {}, got {:?}", stringify!($pattern), $event),
        }
    };
}

#[macro_export]
macro_rules! assert_within_duration {
    ($duration:expr, $actual:expr) => {
        assert!(
            $actual <= $duration,
            "Duration {} exceeds expected maximum {}",
            $actual.as_millis(),
            $duration.as_millis()
        );
    };
}

/// Test data generators
pub mod generators {
    /// Generate test text of specified size
    pub fn generate_text(size: usize) -> String {
        "A".repeat(size)
    }

    /// Generate test binary data
    pub fn generate_binary_data(size: usize) -> Vec<u8> {
        (0..size).map(|i| (i % 256) as u8).collect()
    }

    /// Generate random device system info
    pub fn generate_device_system() -> String {
        format!("TestOS-{}", uuid::Uuid::new_v4())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_config_creation() {
        let config = create_test_config(8888);
        assert_eq!(config.network.listen_port, 8888);
        assert_eq!(config.device_name, "test-device-8888");
    }

    #[test]
    fn test_port_allocation() {
        let port1 = get_test_port();
        let port2 = get_test_port();
        assert_ne!(port1, port2);
        assert!(port2 > port1);
    }

    #[tokio::test]
    async fn test_event_emitter() {
        let emitter = TestEventEmitter::new();
        let content = create_test_clipboard_content("test", "device1");
        
        emitter.emit_clipboard_change(content, "device1").await;
        
        let event = emitter.wait_for_event().await;
        assert!(event.is_some());
        assert_event_type!(event.unwrap(), crosscopy::events::Event::ClipboardChanged { .. });
    }
}
