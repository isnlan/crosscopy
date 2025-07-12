//! Encryption service implementation

use crate::clipboard::ClipboardContent;
use crate::config::SecurityConfig;
use crate::crypto::{CryptoError, Result};
use crate::network::Message;
use aes_gcm::{
    aead::{Aead, KeyInit},
    Aes256Gcm, Nonce,
};
use rand::{RngCore, thread_rng};
use sha2::{Sha256, Digest};

/// AES-GCM encryption service
pub struct EncryptionService {
    cipher: Aes256Gcm,
    key: [u8; 32],
}

impl EncryptionService {
    /// Create a new encryption service with a 32-byte key
    pub fn new(key: &[u8; 32]) -> Self {
        let cipher = Aes256Gcm::new(key.into());
        
        Self { 
            cipher,
            key: *key,
        }
    }

    /// Create encryption service from configuration
    pub fn from_config(config: &SecurityConfig) -> Result<Self> {
        let key = Self::derive_key_from_password(&config.secret_key)?;
        Ok(Self::new(&key))
    }

    /// Encrypt clipboard content
    pub fn encrypt_content(&self, content: &ClipboardContent) -> Result<Vec<u8>> {
        let serialized = serde_json::to_vec(content)
            .map_err(|e| CryptoError::EncryptionFailed(e.to_string()))?;
        
        self.encrypt(&serialized)
    }

    /// Decrypt network message
    pub fn decrypt_message(&self, message: &Message) -> Result<Vec<u8>> {
        self.decrypt(&message.payload)
    }

    /// Encrypt data using AES-GCM
    pub fn encrypt(&self, data: &[u8]) -> Result<Vec<u8>> {
        // Generate random nonce
        let mut nonce_bytes = [0u8; 12];
        thread_rng().fill_bytes(&mut nonce_bytes);
        let nonce = Nonce::from_slice(&nonce_bytes);
        
        // Encrypt the data
        let ciphertext = self.cipher.encrypt(nonce, data)
            .map_err(|e| CryptoError::EncryptionFailed(e.to_string()))?;
        
        // Combine nonce and ciphertext
        let mut result = Vec::with_capacity(12 + ciphertext.len());
        result.extend_from_slice(&nonce_bytes);
        result.extend_from_slice(&ciphertext);
        
        Ok(result)
    }

    /// Decrypt data using AES-GCM
    pub fn decrypt(&self, encrypted_data: &[u8]) -> Result<Vec<u8>> {
        if encrypted_data.len() < 12 {
            return Err(CryptoError::InvalidData("Data too short".to_string()));
        }
        
        // Split nonce and ciphertext
        let (nonce_bytes, ciphertext) = encrypted_data.split_at(12);
        let nonce = Nonce::from_slice(nonce_bytes);
        
        // Decrypt the data
        let plaintext = self.cipher.decrypt(nonce, ciphertext)
            .map_err(|e| CryptoError::DecryptionFailed(e.to_string()))?;
        
        Ok(plaintext)
    }

    /// Derive encryption key from password using SHA-256
    fn derive_key_from_password(password: &str) -> Result<[u8; 32]> {
        let mut hasher = Sha256::new();
        hasher.update(password.as_bytes());
        hasher.update(b"crosscopy-salt"); // Static salt for simplicity
        
        let result = hasher.finalize();
        let mut key = [0u8; 32];
        key.copy_from_slice(&result);
        
        Ok(key)
    }

    /// Generate a random encryption key
    pub fn generate_random_key() -> [u8; 32] {
        let mut key = [0u8; 32];
        thread_rng().fill_bytes(&mut key);
        key
    }

    /// Get the current encryption key (for key rotation)
    pub fn get_key(&self) -> &[u8; 32] {
        &self.key
    }

    /// Update the encryption key
    pub fn update_key(&mut self, new_key: &[u8; 32]) {
        self.key = *new_key;
        self.cipher = Aes256Gcm::new(new_key.into());
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_encryption_decryption() {
        let key = EncryptionService::generate_random_key();
        let service = EncryptionService::new(&key);
        
        let original_data = b"Hello, CrossCopy!";
        
        let encrypted = service.encrypt(original_data).unwrap();
        let decrypted = service.decrypt(&encrypted).unwrap();
        
        assert_eq!(original_data, &decrypted[..]);
    }

    #[test]
    fn test_key_derivation() {
        let password = "test-password";
        let key1 = EncryptionService::derive_key_from_password(password).unwrap();
        let key2 = EncryptionService::derive_key_from_password(password).unwrap();
        
        // Same password should produce same key
        assert_eq!(key1, key2);
        
        let different_key = EncryptionService::derive_key_from_password("different-password").unwrap();
        assert_ne!(key1, different_key);
    }

    #[test]
    fn test_invalid_data_decryption() {
        let key = EncryptionService::generate_random_key();
        let service = EncryptionService::new(&key);
        
        // Too short data
        let result = service.decrypt(&[1, 2, 3]);
        assert!(result.is_err());
        
        // Invalid encrypted data
        let invalid_data = vec![0u8; 50];
        let result = service.decrypt(&invalid_data);
        assert!(result.is_err());
    }
}
