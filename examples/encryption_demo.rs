//! Encryption demonstration example for CrossCopy
//!
//! This example demonstrates the encryption and decryption capabilities
//! of CrossCopy, showing how clipboard content is secured during transmission.
//!
//! Run with: cargo run --example encryption_demo

use crosscopy::{
    clipboard::ClipboardContent,
    crypto::EncryptionService,
    utils::logger,
};
use log::{info, debug};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize logging
    logger::init_logger("debug")?;
    
    info!("Starting CrossCopy encryption demonstration");

    // Generate a random encryption key
    let key = EncryptionService::generate_random_key();
    info!("Generated encryption key: {:02x?}", &key[..8]); // Show first 8 bytes only
    
    // Create encryption service
    let encryption_service = EncryptionService::new(&key);
    
    // Create sample clipboard content
    let original_text = "This is sensitive clipboard content that needs to be encrypted!";
    let clipboard_content = ClipboardContent::new_text(
        original_text.to_string(),
        "demo-device".to_string(),
    );
    
    info!("Original content: '{}'", original_text);
    info!("Content size: {} bytes", clipboard_content.metadata.size);
    info!("Content checksum: {}", clipboard_content.checksum);

    // Demonstrate encryption
    info!("\n--- Encryption Process ---");
    let encrypted_data = encryption_service.encrypt_content(&clipboard_content)?;
    info!("Encrypted data size: {} bytes", encrypted_data.len());
    debug!("Encrypted data (first 32 bytes): {:02x?}", &encrypted_data[..32.min(encrypted_data.len())]);
    
    // Demonstrate that encrypted data is different from original
    let original_bytes = serde_json::to_vec(&clipboard_content)?;
    info!("Original serialized size: {} bytes", original_bytes.len());
    assert_ne!(encrypted_data, original_bytes, "Encrypted data should be different from original");
    info!("✓ Encryption successful - data is properly encrypted");

    // Demonstrate decryption
    info!("\n--- Decryption Process ---");
    let decrypted_data = encryption_service.decrypt(&encrypted_data)?;
    info!("Decrypted data size: {} bytes", decrypted_data.len());
    
    // Deserialize the decrypted content
    let decrypted_content: ClipboardContent = serde_json::from_slice(&decrypted_data)?;
    let decrypted_text = decrypted_content.as_text().unwrap();
    
    info!("Decrypted content: '{}'", decrypted_text);
    info!("Decrypted checksum: {}", decrypted_content.checksum);
    
    // Verify integrity
    assert!(decrypted_content.verify_integrity(), "Content integrity check failed");
    info!("✓ Content integrity verified");
    
    // Verify content matches original
    assert_eq!(original_text, decrypted_text, "Decrypted content doesn't match original");
    info!("✓ Decryption successful - content matches original");

    // Demonstrate key derivation from password
    info!("\n--- Password-based Key Derivation ---");
    let password = "my-secure-password-123";
    let config = crosscopy::config::SecurityConfig {
        secret_key: password.to_string(),
        enable_encryption: true,
        key_rotation_interval: 86400,
        enable_authentication: true,
        max_message_age: 300,
    };
    
    let password_service = EncryptionService::from_config(&config)?;
    info!("Created encryption service from password");
    
    // Test encryption with password-derived key
    let password_encrypted = password_service.encrypt_content(&clipboard_content)?;
    let password_decrypted = password_service.decrypt(&password_encrypted)?;
    let password_content: ClipboardContent = serde_json::from_slice(&password_decrypted)?;
    
    assert_eq!(
        clipboard_content.as_text(),
        password_content.as_text(),
        "Password-based encryption failed"
    );
    info!("✓ Password-based encryption/decryption successful");

    // Demonstrate encryption with different keys fails
    info!("\n--- Security Verification ---");
    let different_key = EncryptionService::generate_random_key();
    let different_service = EncryptionService::new(&different_key);
    
    match different_service.decrypt(&encrypted_data) {
        Ok(_) => panic!("Decryption with wrong key should fail!"),
        Err(e) => {
            info!("✓ Decryption with wrong key properly failed: {}", e);
        }
    }

    // Performance demonstration
    info!("\n--- Performance Test ---");
    let large_text = "A".repeat(10000); // 10KB of text
    let large_content = ClipboardContent::new_text(
        large_text,
        "demo-device".to_string(),
    );
    
    let start = std::time::Instant::now();
    let large_encrypted = encryption_service.encrypt_content(&large_content)?;
    let encrypt_time = start.elapsed();
    
    let start = std::time::Instant::now();
    let large_decrypted = encryption_service.decrypt(&large_encrypted)?;
    let decrypt_time = start.elapsed();
    
    info!("Large content (10KB) encryption time: {:?}", encrypt_time);
    info!("Large content (10KB) decryption time: {:?}", decrypt_time);
    info!("Encrypted size: {} bytes (overhead: {} bytes)", 
          large_encrypted.len(), 
          large_encrypted.len() - large_content.metadata.size);

    info!("\nCrossCopy encryption demonstration completed successfully!");
    info!("All security features are working correctly.");
    
    Ok(())
}
