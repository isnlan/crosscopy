//! Performance metrics demonstration for CrossCopy
//!
//! This example demonstrates how to collect and analyze performance metrics
//! from CrossCopy operations.
//!
//! Run with: cargo run --example metrics_demo

use crosscopy::{
    clipboard::ClipboardContent,
    crypto::EncryptionService,
    network::{Message, MessageType},
    utils::{logger, metrics::PerformanceMetrics},
};
use log::info;
use std::time::Duration;
use tokio::time::sleep;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize logging
    logger::init_logger("info")?;
    
    info!("Starting CrossCopy performance metrics demonstration");

    // Create metrics collector
    let metrics = PerformanceMetrics::new();
    
    // Demonstrate clipboard content operations
    info!("\n--- Clipboard Content Metrics ---");
    await_clipboard_metrics(&metrics).await?;
    
    // Demonstrate encryption metrics
    info!("\n--- Encryption Metrics ---");
    await_encryption_metrics(&metrics).await?;
    
    // Demonstrate network message metrics
    info!("\n--- Network Message Metrics ---");
    await_network_metrics(&metrics).await?;
    
    // Demonstrate timer operations
    info!("\n--- Timer Metrics ---");
    await_timer_metrics(&metrics).await?;
    
    // Show final metrics summary
    info!("\n--- Final Metrics Summary ---");
    let summary = metrics.get_summary().await;
    
    info!("Counters:");
    for (name, value) in &summary.counters {
        info!("  {}: {}", name, value);
    }
    
    info!("Gauges:");
    for (name, value) in &summary.gauges {
        info!("  {}: {:.2}", name, value);
    }
    
    info!("Duration Statistics:");
    for (name, stats) in &summary.duration_stats {
        info!("  {}:", name);
        info!("    Count: {}", stats.count);
        info!("    Average: {:?}", stats.average);
        info!("    Min: {:?}", stats.min);
        info!("    Max: {:?}", stats.max);
        info!("    Total: {:?}", stats.total);
    }

    info!("\nPerformance metrics demonstration completed!");
    Ok(())
}

async fn await_clipboard_metrics(metrics: &PerformanceMetrics) -> Result<(), Box<dyn std::error::Error>> {
    info!("Testing clipboard content creation and processing...");
    
    let test_sizes = [100, 1000, 10000]; // Different content sizes
    
    for size in &test_sizes {
        let content_text = "A".repeat(*size);
        
        // Time content creation
        metrics.start_timer(&format!("clipboard_creation_{}", size)).await;
        let content = ClipboardContent::new_text(
            content_text,
            "metrics-demo".to_string(),
        );
        metrics.end_timer(&format!("clipboard_creation_{}", size)).await;
        
        // Count content operations
        metrics.increment_counter("clipboard_content_created").await;
        metrics.add_to_counter("clipboard_bytes_processed", content.metadata.size as u64).await;
        
        // Time integrity verification
        metrics.start_timer("clipboard_integrity_check").await;
        let is_valid = content.verify_integrity();
        metrics.end_timer("clipboard_integrity_check").await;
        
        if is_valid {
            metrics.increment_counter("clipboard_integrity_passed").await;
        }
        
        info!("  Processed {}B content", size);
    }
    
    // Set gauge for average content size
    let total_bytes = metrics.get_counter("clipboard_bytes_processed").await;
    let total_items = metrics.get_counter("clipboard_content_created").await;
    if total_items > 0 {
        let avg_size = total_bytes as f64 / total_items as f64;
        metrics.set_gauge("avg_clipboard_content_size", avg_size).await;
    }
    
    Ok(())
}

async fn await_encryption_metrics(metrics: &PerformanceMetrics) -> Result<(), Box<dyn std::error::Error>> {
    info!("Testing encryption performance...");
    
    let key = EncryptionService::generate_random_key();
    let service = EncryptionService::new(&key);
    
    let test_data_sizes = [1024, 10240, 102400]; // 1KB, 10KB, 100KB
    
    for size in &test_data_sizes {
        let test_data = vec![0u8; *size];
        
        // Time encryption
        metrics.start_timer(&format!("encryption_{}", size)).await;
        let encrypted = service.encrypt(&test_data)?;
        metrics.end_timer(&format!("encryption_{}", size)).await;
        
        // Time decryption
        metrics.start_timer(&format!("decryption_{}", size)).await;
        let _decrypted = service.decrypt(&encrypted)?;
        metrics.end_timer(&format!("decryption_{}", size)).await;
        
        // Count operations
        metrics.increment_counter("encryption_operations").await;
        metrics.increment_counter("decryption_operations").await;
        metrics.add_to_counter("encryption_bytes_processed", *size as u64).await;
        
        // Calculate overhead
        let overhead = encrypted.len() - test_data.len();
        metrics.set_gauge(&format!("encryption_overhead_{}", size), overhead as f64).await;
        
        info!("  Encrypted/decrypted {}B (overhead: {}B)", size, overhead);
    }
    
    Ok(())
}

async fn await_network_metrics(metrics: &PerformanceMetrics) -> Result<(), Box<dyn std::error::Error>> {
    info!("Testing network message processing...");
    
    let message_types = [
        MessageType::Handshake,
        MessageType::Heartbeat,
        MessageType::ClipboardSync,
        MessageType::DeviceInfo,
    ];
    
    for msg_type in &message_types {
        let payload = format!("Test payload for {:?}", msg_type).into_bytes();
        
        // Time message creation
        metrics.start_timer("message_creation").await;
        let message = Message::new(*msg_type, payload.clone(), "metrics-demo".to_string());
        metrics.end_timer("message_creation").await;
        
        // Time message serialization
        metrics.start_timer("message_serialization").await;
        let serialized = serde_json::to_vec(&message)?;
        metrics.end_timer("message_serialization").await;
        
        // Time message deserialization
        metrics.start_timer("message_deserialization").await;
        let _deserialized: Message = serde_json::from_slice(&serialized)?;
        metrics.end_timer("message_deserialization").await;
        
        // Time message verification
        metrics.start_timer("message_verification").await;
        let is_valid = message.verify();
        metrics.end_timer("message_verification").await;
        
        // Count operations
        metrics.increment_counter("messages_created").await;
        metrics.increment_counter("messages_serialized").await;
        metrics.increment_counter("messages_deserialized").await;
        
        if is_valid {
            metrics.increment_counter("messages_verified").await;
        }
        
        metrics.add_to_counter("message_bytes_processed", serialized.len() as u64).await;
        
        info!("  Processed {:?} message ({}B)", msg_type, serialized.len());
    }
    
    Ok(())
}

async fn await_timer_metrics(metrics: &PerformanceMetrics) -> Result<(), Box<dyn std::error::Error>> {
    info!("Testing timer operations...");
    
    // Test different duration operations
    let operations = [
        ("fast_operation", 10),
        ("medium_operation", 50),
        ("slow_operation", 100),
    ];
    
    for (op_name, duration_ms) in &operations {
        for i in 0..5 {
            metrics.start_timer(&format!("{}_{}", op_name, i)).await;
            sleep(Duration::from_millis(*duration_ms)).await;
            let actual_duration = metrics.end_timer(&format!("{}_{}", op_name, i)).await;
            
            if let Some(duration) = actual_duration {
                info!("  {} iteration {}: {:?}", op_name, i, duration);
            }
        }
        
        // Get average for this operation type
        if let Some(avg) = metrics.get_average_duration(&format!("{}_0", op_name)).await {
            info!("  Average for {}: {:?}", op_name, avg);
        }
    }
    
    // Test timer guard (automatic timing)
    {
        let _guard = metrics.timer_guard("auto_timed_operation").await;
        sleep(Duration::from_millis(25)).await;
        // Timer automatically ends when guard is dropped
    }
    
    Ok(())
}
