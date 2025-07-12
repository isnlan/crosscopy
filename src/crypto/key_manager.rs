//! Key management implementation

use crate::crypto::{CryptoError, Result};
use log::{debug, info, warn};
use std::time::{Duration, Instant};

/// Key rotation policy
#[derive(Debug, Clone)]
pub enum KeyRotationPolicy {
    /// Never rotate keys
    Never,
    /// Rotate keys at fixed intervals
    Interval(Duration),
    /// Rotate keys after a certain number of operations
    OperationCount(u64),
    /// Rotate keys based on data volume
    DataVolume(u64),
}

/// Key manager for handling encryption key lifecycle
pub struct KeyManager {
    current_key: [u8; 32],
    previous_key: Option<[u8; 32]>,
    rotation_policy: KeyRotationPolicy,
    last_rotation: Instant,
    operation_count: u64,
    data_processed: u64,
}

impl KeyManager {
    /// Create a new key manager
    pub fn new(initial_key: [u8; 32], rotation_policy: KeyRotationPolicy) -> Self {
        Self {
            current_key: initial_key,
            previous_key: None,
            rotation_policy,
            last_rotation: Instant::now(),
            operation_count: 0,
            data_processed: 0,
        }
    }

    /// Get the current encryption key
    pub fn get_current_key(&self) -> &[u8; 32] {
        &self.current_key
    }

    /// Get the previous encryption key (for decrypting old data)
    pub fn get_previous_key(&self) -> Option<&[u8; 32]> {
        self.previous_key.as_ref()
    }

    /// Check if key rotation is needed
    pub fn should_rotate_key(&self) -> bool {
        match &self.rotation_policy {
            KeyRotationPolicy::Never => false,
            KeyRotationPolicy::Interval(interval) => {
                self.last_rotation.elapsed() >= *interval
            }
            KeyRotationPolicy::OperationCount(max_ops) => {
                self.operation_count >= *max_ops
            }
            KeyRotationPolicy::DataVolume(max_volume) => {
                self.data_processed >= *max_volume
            }
        }
    }

    /// Rotate the encryption key
    pub fn rotate_key(&mut self) -> Result<()> {
        info!("Rotating encryption key");
        
        // Store current key as previous
        self.previous_key = Some(self.current_key);
        
        // Generate new key
        self.current_key = self.generate_new_key()?;
        
        // Reset counters
        self.last_rotation = Instant::now();
        self.operation_count = 0;
        self.data_processed = 0;
        
        debug!("Key rotation completed");
        Ok(())
    }

    /// Record an encryption/decryption operation
    pub fn record_operation(&mut self, data_size: usize) {
        self.operation_count += 1;
        self.data_processed += data_size as u64;
        
        // Check if rotation is needed
        if self.should_rotate_key() {
            warn!("Key rotation needed based on policy");
        }
    }

    /// Force key rotation regardless of policy
    pub fn force_rotation(&mut self) -> Result<()> {
        info!("Forcing key rotation");
        self.rotate_key()
    }

    /// Generate a new random encryption key
    fn generate_new_key(&self) -> Result<[u8; 32]> {
        use rand::{RngCore, thread_rng};
        
        let mut key = [0u8; 32];
        thread_rng().fill_bytes(&mut key);
        
        Ok(key)
    }

    /// Get key rotation statistics
    pub fn get_stats(&self) -> KeyStats {
        KeyStats {
            current_key_age: self.last_rotation.elapsed(),
            operation_count: self.operation_count,
            data_processed: self.data_processed,
            has_previous_key: self.previous_key.is_some(),
            rotation_policy: self.rotation_policy.clone(),
        }
    }
}

/// Key management statistics
#[derive(Debug, Clone)]
pub struct KeyStats {
    pub current_key_age: Duration,
    pub operation_count: u64,
    pub data_processed: u64,
    pub has_previous_key: bool,
    pub rotation_policy: KeyRotationPolicy,
}

impl Default for KeyRotationPolicy {
    fn default() -> Self {
        // Default to rotating keys every 24 hours
        KeyRotationPolicy::Interval(Duration::from_secs(24 * 60 * 60))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_key_manager_creation() {
        let initial_key = [0u8; 32];
        let policy = KeyRotationPolicy::Never;
        let manager = KeyManager::new(initial_key, policy);
        
        assert_eq!(manager.get_current_key(), &initial_key);
        assert!(manager.get_previous_key().is_none());
    }

    #[test]
    fn test_operation_count_rotation() {
        let initial_key = [0u8; 32];
        let policy = KeyRotationPolicy::OperationCount(5);
        let mut manager = KeyManager::new(initial_key, policy);
        
        // Record operations
        for _ in 0..4 {
            manager.record_operation(100);
            assert!(!manager.should_rotate_key());
        }
        
        // Fifth operation should trigger rotation need
        manager.record_operation(100);
        assert!(manager.should_rotate_key());
    }

    #[test]
    fn test_key_rotation() {
        let initial_key = [1u8; 32];
        let policy = KeyRotationPolicy::Never;
        let mut manager = KeyManager::new(initial_key, policy);
        
        let old_key = *manager.get_current_key();
        manager.rotate_key().unwrap();
        
        let new_key = *manager.get_current_key();
        assert_ne!(old_key, new_key);
        assert_eq!(manager.get_previous_key(), Some(&old_key));
    }
}
