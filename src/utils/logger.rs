//! Logging utilities

use crate::utils::{Result, UtilError};
use env_logger::{Builder, Target};
use log::LevelFilter;
use std::io::Write;

/// Initialize the logger with the specified configuration
pub fn init_logger(level: &str) -> Result<()> {
    let log_level = parse_log_level(level)?;

    Builder::from_default_env()
        .target(Target::Stdout)
        .filter_level(log_level)
        .format(|buf, record| {
            writeln!(
                buf,
                "{} [{}] [{}:{}] - {}",
                chrono::Utc::now().format("%Y-%m-%d %H:%M:%S%.3f"),
                record.level(),
                record.file().unwrap_or("unknown"),
                record.line().unwrap_or(0),
                record.args()
            )
        })
        .try_init()
        .map_err(|e| UtilError::LoggerInitFailed(e.to_string()))?;

    Ok(())
}

/// Initialize file-based logger
pub fn init_file_logger(level: &str, file_path: &str) -> Result<()> {
    let log_level = parse_log_level(level)?;

    Builder::from_default_env()
        .target(Target::Pipe(Box::new(std::fs::File::create(file_path)?)))
        .filter_level(log_level)
        .format(|buf, record| {
            writeln!(
                buf,
                "{} [{}] [{}:{}] - {}",
                chrono::Utc::now().format("%Y-%m-%d %H:%M:%S%.3f"),
                record.level(),
                record.file().unwrap_or("unknown"),
                record.line().unwrap_or(0),
                record.args()
            )
        })
        .try_init()
        .map_err(|e| UtilError::LoggerInitFailed(e.to_string()))?;

    Ok(())
}

/// Parse log level string to LevelFilter
fn parse_log_level(level: &str) -> Result<LevelFilter> {
    match level.to_lowercase().as_str() {
        "error" => Ok(LevelFilter::Error),
        "warn" => Ok(LevelFilter::Warn),
        "info" => Ok(LevelFilter::Info),
        "debug" => Ok(LevelFilter::Debug),
        "trace" => Ok(LevelFilter::Trace),
        "off" => Ok(LevelFilter::Off),
        _ => Err(UtilError::LoggerInitFailed(format!(
            "Invalid log level: {}",
            level
        ))),
    }
}
