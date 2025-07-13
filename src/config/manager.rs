//! Configuration manager implementation

use crate::config::{AppConfig, ConfigError, Result};
use crate::utils::platform;
use log::{debug, info, warn};
use std::path::{Path, PathBuf};

/// Configuration manager for loading and saving application configuration
pub struct ConfigManager {
    config_path: PathBuf,
}

impl ConfigManager {
    /// Create a new configuration manager
    pub fn new(config_path: Option<&str>) -> Result<Self> {
        let config_path = if let Some(path) = config_path {
            PathBuf::from(path)
        } else {
            Self::default_config_path()?
        };

        Ok(Self { config_path })
    }

    /// Load configuration from file or create default
    pub async fn load_config(&self) -> Result<AppConfig> {
        if self.config_path.exists() {
            info!("Loading configuration from: {}", self.config_path.display());
            self.load_from_file().await
        } else {
            info!("Configuration file not found, creating default configuration");
            let config = AppConfig::default();
            self.save_config(&config).await?;
            Ok(config)
        }
    }

    /// Save configuration to file
    pub async fn save_config(&self, config: &AppConfig) -> Result<()> {
        info!("Saving configuration to: {}", self.config_path.display());

        // Ensure config directory exists
        if let Some(parent) = self.config_path.parent() {
            platform::ensure_dir_exists(parent)
                .map_err(|e| ConfigError::ValidationFailed(e.to_string()))?;
        }

        // Serialize configuration to TOML
        let toml_content = toml::to_string_pretty(config)?;

        // Write to file
        tokio::fs::write(&self.config_path, toml_content).await?;

        debug!("Configuration saved successfully");
        Ok(())
    }

    /// Reload configuration from file
    pub async fn reload_config(&self) -> Result<AppConfig> {
        info!("Reloading configuration from file");
        self.load_from_file().await
    }

    /// Validate configuration
    pub fn validate_config(config: &AppConfig) -> Result<()> {
        // Validate network configuration
        if config.network.listen_port == 0 {
            return Err(ConfigError::ValidationFailed(
                "Listen port cannot be 0".to_string(),
            ));
        }

        if config.network.max_connections == 0 {
            return Err(ConfigError::ValidationFailed(
                "Max connections must be greater than 0".to_string(),
            ));
        }

        // Validate clipboard configuration
        if config.clipboard.max_content_size == 0 {
            return Err(ConfigError::ValidationFailed(
                "Max content size must be greater than 0".to_string(),
            ));
        }

        // Validate security configuration
        if config.security.secret_key.is_empty() {
            return Err(ConfigError::ValidationFailed(
                "Secret key cannot be empty".to_string(),
            ));
        }

        // Validate logging configuration
        let valid_levels = ["error", "warn", "info", "debug", "trace"];
        if !valid_levels.contains(&config.logging.level.as_str()) {
            return Err(ConfigError::ValidationFailed(
                format!("Invalid log level: {}", config.logging.level),
            ));
        }

        Ok(())
    }

    /// Get configuration file path
    pub fn get_config_path(&self) -> &Path {
        &self.config_path
    }

    /// Check if configuration file exists
    pub fn config_exists(&self) -> bool {
        self.config_path.exists()
    }

    async fn load_from_file(&self) -> Result<AppConfig> {
        let content = tokio::fs::read_to_string(&self.config_path).await?;
        let config: AppConfig = toml::from_str(&content)?;

        // Validate loaded configuration
        Self::validate_config(&config)?;

        debug!("Configuration loaded and validated successfully");
        Ok(config)
    }

    fn default_config_path() -> Result<PathBuf> {
        let mut config_dir = platform::get_config_dir()
            .map_err(|e| ConfigError::ValidationFailed(e.to_string()))?;
        
        config_dir.push("config.toml");
        Ok(config_dir)
    }
}

/// Configuration watcher for detecting file changes
pub struct ConfigWatcher {
    config_manager: ConfigManager,
    last_modified: Option<std::time::SystemTime>,
}

impl ConfigWatcher {
    /// Create a new configuration watcher
    pub fn new(config_manager: ConfigManager) -> Self {
        Self {
            config_manager,
            last_modified: None,
        }
    }

    /// Check if configuration file has been modified
    pub async fn check_for_changes(&mut self) -> Result<bool> {
        let config_path = self.config_manager.get_config_path();
        
        if !config_path.exists() {
            return Ok(false);
        }

        let metadata = tokio::fs::metadata(config_path).await?;
        let modified = metadata.modified()?;

        if let Some(last_modified) = self.last_modified {
            if modified > last_modified {
                self.last_modified = Some(modified);
                info!("Configuration file has been modified");
                return Ok(true);
            }
        } else {
            self.last_modified = Some(modified);
        }

        Ok(false)
    }

    /// Get the configuration manager
    pub fn get_manager(&self) -> &ConfigManager {
        &self.config_manager
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[tokio::test]
    async fn test_config_manager_creation() {
        let temp_dir = tempdir().unwrap();
        let config_path = temp_dir.path().join("test_config.toml");
        
        let manager = ConfigManager::new(Some(config_path.to_str().unwrap()));
        assert!(manager.is_ok());
    }

    #[tokio::test]
    async fn test_default_config_creation() {
        let temp_dir = tempdir().unwrap();
        let config_path = temp_dir.path().join("test_config.toml");
        
        let manager = ConfigManager::new(Some(config_path.to_str().unwrap())).unwrap();
        let config = manager.load_config().await.unwrap();
        
        // Should create default config
        assert!(!config.device_name.is_empty());
        assert!(!config.device_system.is_empty());
    }

    #[tokio::test]
    async fn test_config_validation() {
        let mut config = AppConfig::default();
        
        // Valid config should pass
        assert!(ConfigManager::validate_config(&config).is_ok());
        
        // Invalid port should fail
        config.network.listen_port = 0;
        assert!(ConfigManager::validate_config(&config).is_err());
    }
}
