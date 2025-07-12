//! Configuration management module
//!
//! This module handles application configuration, including loading from files,
//! environment variables, and providing default values.

pub mod manager;

pub use manager::ConfigManager;

use serde::{Deserialize, Serialize};
use std::time::Duration;
use thiserror::Error;

/// Configuration-related errors
#[derive(Debug, Error)]
pub enum ConfigError {
    #[error("Configuration file not found: {0}")]
    FileNotFound(String),

    #[error("Invalid configuration format: {0}")]
    InvalidFormat(String),

    #[error("Configuration validation failed: {0}")]
    ValidationFailed(String),

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("TOML parsing error: {0}")]
    TomlParsing(#[from] toml::de::Error),

    #[error("Serialization error: {0}")]
    Serialization(#[from] toml::ser::Error),
}

/// Result type for configuration operations
pub type Result<T> = std::result::Result<T, ConfigError>;

/// Main application configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppConfig {
    /// Device identification
    pub device_name: String,
    pub device_id: String,

    /// Network configuration
    pub network: NetworkConfig,

    /// Clipboard configuration
    pub clipboard: ClipboardConfig,

    /// Security configuration
    pub security: SecurityConfig,

    /// Logging configuration
    pub logging: LoggingConfig,
}

/// Network configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkConfig {
    /// Port to listen on for incoming connections
    pub listen_port: u16,

    /// List of peer devices to connect to
    pub peer_list: Vec<PeerConfig>,

    /// Connection timeout in milliseconds
    pub connection_timeout: u64,

    /// Heartbeat interval in milliseconds
    pub heartbeat_interval: u64,

    /// Maximum number of concurrent connections
    pub max_connections: usize,

    /// Enable automatic peer discovery
    pub auto_discovery: bool,

    /// Discovery broadcast port
    pub discovery_port: u16,
}

/// Peer device configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PeerConfig {
    /// Peer device ID
    pub device_id: String,

    /// Peer device name
    pub name: String,

    /// Peer IP address or hostname
    pub address: String,

    /// Peer port
    pub port: u16,

    /// Whether this peer is enabled
    pub enabled: bool,
}

/// Clipboard configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClipboardConfig {
    /// Enable image synchronization
    pub sync_images: bool,

    /// Enable file synchronization
    pub sync_files: bool,

    /// Cooldown period in milliseconds to prevent rapid updates
    pub cooldown_millis: u64,

    /// Maximum content size in bytes
    pub max_content_size: usize,

    /// Enable content compression
    pub enable_compression: bool,

    /// Compression threshold in bytes
    pub compression_threshold: usize,
}

/// Security configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecurityConfig {
    /// Shared secret key for encryption
    pub secret_key: String,

    /// Enable end-to-end encryption
    pub enable_encryption: bool,

    /// Key rotation interval in seconds
    pub key_rotation_interval: u64,

    /// Enable message authentication
    pub enable_authentication: bool,

    /// Maximum message age in seconds (for replay protection)
    pub max_message_age: u64,
}

/// Logging configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoggingConfig {
    /// Log level (error, warn, info, debug, trace)
    pub level: String,

    /// Optional log file path
    pub file_path: Option<String>,

    /// Enable structured logging (JSON format)
    pub structured: bool,

    /// Maximum log file size in bytes
    pub max_file_size: usize,

    /// Number of log files to keep
    pub max_files: usize,
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            device_name: format!("CrossCopy-{}", uuid::Uuid::new_v4().to_string()[..8].to_uppercase()),
            device_id: uuid::Uuid::new_v4().to_string(),
            network: NetworkConfig::default(),
            clipboard: ClipboardConfig::default(),
            security: SecurityConfig::default(),
            logging: LoggingConfig::default(),
        }
    }
}

impl Default for NetworkConfig {
    fn default() -> Self {
        Self {
            listen_port: 8888,
            peer_list: Vec::new(),
            connection_timeout: 5000,
            heartbeat_interval: 1000,
            max_connections: 10,
            auto_discovery: true,
            discovery_port: 8889,
        }
    }
}

impl Default for ClipboardConfig {
    fn default() -> Self {
        Self {
            sync_images: true,
            sync_files: false,
            cooldown_millis: 300,
            max_content_size: 10 * 1024 * 1024, // 10MB
            enable_compression: true,
            compression_threshold: 1024, // 1KB
        }
    }
}

impl Default for SecurityConfig {
    fn default() -> Self {
        Self {
            secret_key: "default-secret-key".to_string(),
            enable_encryption: true,
            key_rotation_interval: 86400, // 24 hours
            enable_authentication: true,
            max_message_age: 300, // 5 minutes
        }
    }
}

impl Default for LoggingConfig {
    fn default() -> Self {
        Self {
            level: "info".to_string(),
            file_path: None,
            structured: false,
            max_file_size: 10 * 1024 * 1024, // 10MB
            max_files: 5,
        }
    }
}

impl NetworkConfig {
    /// Get connection timeout as Duration
    pub fn connection_timeout_duration(&self) -> Duration {
        Duration::from_millis(self.connection_timeout)
    }

    /// Get heartbeat interval as Duration
    pub fn heartbeat_interval_duration(&self) -> Duration {
        Duration::from_millis(self.heartbeat_interval)
    }
}

impl ClipboardConfig {
    /// Get cooldown period as Duration
    pub fn cooldown_duration(&self) -> Duration {
        Duration::from_millis(self.cooldown_millis)
    }
}

impl SecurityConfig {
    /// Get key rotation interval as Duration
    pub fn key_rotation_duration(&self) -> Duration {
        Duration::from_secs(self.key_rotation_interval)
    }

    /// Get max message age as Duration
    pub fn max_message_age_duration(&self) -> Duration {
        Duration::from_secs(self.max_message_age)
    }
}
