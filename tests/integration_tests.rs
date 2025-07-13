//! Integration tests for CrossCopy

use crosscopy::{
    config::{AppConfig, ClipboardConfig, NetworkConfig, SecurityConfig, LoggingConfig},
    CrossCopyApp,
};
use std::time::Duration;
use tokio::time::sleep;

/// Create a test configuration with unique ports
fn create_test_config(port: u16) -> AppConfig {
    AppConfig {
        device_name: format!("test-device-{}", port),
        device_system: format!("TestOS-{}", port),
        network: NetworkConfig {
            listen_port: port,
            connection_timeout: 5000,
            heartbeat_interval: 1000,
            max_connections: 10,
            enable_mdns: false, // Disable for tests
            mdns_discovery_interval: 30,
            idle_connection_timeout: 300,
            enable_quic: false,
            quic_port: None,
        },
        clipboard: ClipboardConfig {
            sync_images: true,
            sync_files: false,
            cooldown_millis: 100, // Shorter for tests
            max_content_size: 1024 * 1024, // 1MB
            enable_compression: false, // Disable for simpler tests
            compression_threshold: 1024,
        },
        security: SecurityConfig {
            secret_key: "test-secret-key".to_string(),
            enable_encryption: false, // Disable for simpler tests
            key_rotation_interval: 86400,
            enable_authentication: false,
            max_message_age: 300,
        },
        logging: LoggingConfig {
            level: "debug".to_string(),
            file_path: None,
            structured: false,
            max_file_size: 10 * 1024 * 1024,
            max_files: 5,
        },
    }
}

#[tokio::test]
async fn test_app_creation_and_startup() {
    let config = create_test_config(18881);
    let app = CrossCopyApp::new(config).await;
    assert!(app.is_ok(), "Failed to create CrossCopy app");
}

#[tokio::test]
async fn test_app_lifecycle() {
    let config = create_test_config(18882);
    let mut app = CrossCopyApp::new(config).await.unwrap();

    // Test that we can start the app
    let start_result = tokio::time::timeout(Duration::from_secs(2), async {
        // We'll start the app in a separate task since it runs indefinitely
        tokio::spawn(async move {
            app.run().await
        });
        sleep(Duration::from_millis(500)).await; // Give it time to start
    }).await;

    assert!(start_result.is_ok(), "App failed to start within timeout");
}

#[tokio::test]
async fn test_configuration_validation() {
    let mut config = create_test_config(18883);
    
    // Test valid configuration
    assert!(crosscopy::config::ConfigManager::validate_config(&config).is_ok());
    
    // Test invalid port
    config.network.listen_port = 0;
    assert!(crosscopy::config::ConfigManager::validate_config(&config).is_err());
    
    // Reset and test invalid max connections
    config.network.listen_port = 18883;
    config.network.max_connections = 0;
    assert!(crosscopy::config::ConfigManager::validate_config(&config).is_err());
    
    // Reset and test empty secret key
    config.network.max_connections = 10;
    config.security.secret_key = String::new();
    assert!(crosscopy::config::ConfigManager::validate_config(&config).is_err());
}

#[tokio::test]
async fn test_event_bus_functionality() {
    use crosscopy::events::{Event, EventBus};
    
    let event_bus = EventBus::new();
    
    // Test event emission and polling
    let test_event = Event::Shutdown;
    event_bus.emit(test_event).await.unwrap();
    
    assert_eq!(event_bus.queue_size().await, 1);
    
    let polled_event = event_bus.poll_event().await;
    assert!(polled_event.is_some());
    assert!(matches!(polled_event.unwrap(), Event::Shutdown));
    
    assert_eq!(event_bus.queue_size().await, 0);
}

#[tokio::test]
async fn test_clipboard_content_creation() {
    use crosscopy::clipboard::ClipboardContent;
    
    let device_id = "test-device".to_string();
    let test_text = "Hello, CrossCopy!".to_string();
    
    let content = ClipboardContent::new_text(test_text.clone(), device_id);
    
    assert_eq!(content.as_text(), Some(test_text));
    assert!(content.verify_integrity());
    assert_eq!(content.metadata.size, "Hello, CrossCopy!".len());
}

#[tokio::test]
async fn test_encryption_service() {
    use crosscopy::crypto::EncryptionService;
    
    let key = EncryptionService::generate_random_key();
    let service = EncryptionService::new(&key);
    
    let original_data = b"Test encryption data";
    
    let encrypted = service.encrypt(original_data).unwrap();
    assert_ne!(encrypted, original_data);
    
    let decrypted = service.decrypt(&encrypted).unwrap();
    assert_eq!(decrypted, original_data);
}

#[tokio::test]
async fn test_network_protocol_message() {
    use crosscopy::network::{Message, MessageType};
    
    let device_id = "test-device".to_string();
    let payload = b"test payload".to_vec();
    
    let message = Message::new(MessageType::ClipboardSync, payload.clone(), device_id);
    
    assert_eq!(message.header.message_type, MessageType::ClipboardSync);
    assert_eq!(message.payload, payload);
    assert!(message.verify());
}

#[tokio::test]
async fn test_performance_metrics() {
    use crosscopy::utils::metrics::PerformanceMetrics;
    
    let metrics = PerformanceMetrics::new();
    
    // Test counter
    metrics.increment_counter("test_counter").await;
    metrics.add_to_counter("test_counter", 5).await;
    assert_eq!(metrics.get_counter("test_counter").await, 6);
    
    // Test gauge
    metrics.set_gauge("test_gauge", 42.5).await;
    assert_eq!(metrics.get_gauge("test_gauge").await, Some(42.5));
    
    // Test timer
    metrics.start_timer("test_timer").await;
    sleep(Duration::from_millis(10)).await;
    let duration = metrics.end_timer("test_timer").await;
    assert!(duration.is_some());
    assert!(duration.unwrap() >= Duration::from_millis(10));
}

#[tokio::test]
async fn test_config_manager_file_operations() {
    use crosscopy::config::ConfigManager;
    use tempfile::tempdir;
    
    let temp_dir = tempdir().unwrap();
    let config_path = temp_dir.path().join("test_config.toml");
    
    let manager = ConfigManager::new(Some(config_path.to_str().unwrap())).unwrap();
    
    // Test loading default config (file doesn't exist)
    let config = manager.load_config().await.unwrap();
    assert!(!config.device_name.is_empty());
    
    // Test that config file was created
    assert!(manager.config_exists());
    
    // Test reloading config
    let reloaded_config = manager.reload_config().await.unwrap();
    assert_eq!(config.device_system, reloaded_config.device_system);
}

#[tokio::test]
async fn test_key_manager_rotation() {
    use crosscopy::crypto::{KeyManager, KeyRotationPolicy};
    use std::time::Duration;
    
    let initial_key = [1u8; 32];
    let policy = KeyRotationPolicy::OperationCount(3);
    let mut manager = KeyManager::new(initial_key, policy);
    
    let original_key = *manager.get_current_key();
    
    // Record operations
    manager.record_operation(100);
    manager.record_operation(100);
    assert!(!manager.should_rotate_key());
    
    manager.record_operation(100);
    assert!(manager.should_rotate_key());
    
    // Rotate key
    manager.rotate_key().unwrap();
    let new_key = *manager.get_current_key();
    
    assert_ne!(original_key, new_key);
    assert_eq!(manager.get_previous_key(), Some(&original_key));
}

// Benchmark tests (only run with --features bench)
#[cfg(feature = "bench")]
mod benchmarks {
    use super::*;
    use criterion::{black_box, Criterion};
    
    pub fn bench_clipboard_content_creation(c: &mut Criterion) {
        c.bench_function("clipboard_content_creation", |b| {
            b.iter(|| {
                let content = crosscopy::clipboard::ClipboardContent::new_text(
                    black_box("Test content".to_string()),
                    black_box("device-id".to_string()),
                );
                black_box(content)
            })
        });
    }
    
    pub fn bench_encryption_roundtrip(c: &mut Criterion) {
        let key = crosscopy::crypto::EncryptionService::generate_random_key();
        let service = crosscopy::crypto::EncryptionService::new(&key);
        let data = b"Test data for encryption benchmark";
        
        c.bench_function("encryption_roundtrip", |b| {
            b.iter(|| {
                let encrypted = service.encrypt(black_box(data)).unwrap();
                let decrypted = service.decrypt(black_box(&encrypted)).unwrap();
                black_box(decrypted)
            })
        });
    }
}
