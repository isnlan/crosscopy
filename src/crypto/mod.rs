//! Cryptographic services module
//!
//! This module provides encryption and decryption services using AES-GCM
//! for secure clipboard content transmission.

pub mod encryption;
pub mod key_manager;

pub use encryption::EncryptionService;
pub use key_manager::{KeyManager, KeyRotationPolicy};

use thiserror::Error;

/// Cryptography-related errors
#[derive(Debug, Error)]
pub enum CryptoError {
    #[error("Encryption failed: {0}")]
    EncryptionFailed(String),

    #[error("Decryption failed: {0}")]
    DecryptionFailed(String),

    #[error("Invalid key format")]
    InvalidKey,

    #[error("Key derivation failed: {0}")]
    KeyDerivationFailed(String),

    #[error("Invalid data format: {0}")]
    InvalidData(String),

    #[error("Random number generation failed")]
    RandomGenerationFailed,
}

/// Result type for cryptographic operations
pub type Result<T> = std::result::Result<T, CryptoError>;
