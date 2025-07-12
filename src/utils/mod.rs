//! Utility modules
//!
//! This module contains various utility functions and helpers used throughout
//! the application, including logging, platform-specific code, and performance metrics.

pub mod logger;
pub mod platform;

pub mod metrics;

use thiserror::Error;

/// Utility-related errors
#[derive(Debug, Error)]
pub enum UtilError {
    #[error("Platform operation failed: {0}")]
    PlatformError(String),

    #[error("Logger initialization failed: {0}")]
    LoggerInitFailed(String),

    #[error("Metrics collection failed: {0}")]
    MetricsError(String),

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
}

/// Result type for utility operations
pub type Result<T> = std::result::Result<T, UtilError>;
