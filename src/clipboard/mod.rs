//! Clipboard monitoring and management module
//!
//! This module provides cross-platform clipboard access and monitoring capabilities.
//! It detects clipboard changes and manages clipboard content synchronization.

pub mod content;
pub mod monitor;

pub use content::{ClipboardContent, ContentType};
pub use monitor::ClipboardMonitor;

use thiserror::Error;

/// Clipboard-related errors
#[derive(Debug, Error)]
pub enum ClipboardError {
    #[error("Failed to access clipboard: {0}")]
    AccessFailed(String),

    #[error("Unsupported content type")]
    UnsupportedContentType,

    #[error("Content too large: {size} bytes (max: {max_size} bytes)")]
    ContentTooLarge { size: usize, max_size: usize },

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),
}

/// Result type for clipboard operations
pub type Result<T> = std::result::Result<T, ClipboardError>;
