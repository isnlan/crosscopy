//! Platform-specific utilities

use crate::utils::{Result, UtilError};

/// Get the platform-specific configuration directory
pub fn get_config_dir() -> Result<std::path::PathBuf> {
    dirs::config_dir()
        .map(|mut path| {
            path.push("crosscopy");
            path
        })
        .ok_or_else(|| UtilError::PlatformError("Failed to get config directory".to_string()))
}

/// Get the platform-specific data directory
pub fn get_data_dir() -> Result<std::path::PathBuf> {
    dirs::data_dir()
        .map(|mut path| {
            path.push("crosscopy");
            path
        })
        .ok_or_else(|| UtilError::PlatformError("Failed to get data directory".to_string()))
}

/// Get the platform-specific cache directory
pub fn get_cache_dir() -> Result<std::path::PathBuf> {
    dirs::cache_dir()
        .map(|mut path| {
            path.push("crosscopy");
            path
        })
        .ok_or_else(|| UtilError::PlatformError("Failed to get cache directory".to_string()))
}

/// Ensure directory exists, create if it doesn't
pub fn ensure_dir_exists(path: &std::path::Path) -> Result<()> {
    if !path.exists() {
        std::fs::create_dir_all(path)?;
    }
    Ok(())
}

/// Get system information
pub fn get_system_info() -> SystemInfo {
    SystemInfo {
        os: std::env::consts::OS.to_string(),
        arch: std::env::consts::ARCH.to_string(),
        hostname: hostname::get()
            .unwrap_or_default()
            .to_string_lossy()
            .to_string(),
    }
}

/// Get detailed system information including OS version
pub fn get_detailed_system_info() -> DetailedSystemInfo {
    let os_version = sysinfo::System::long_os_version()
        .unwrap_or_else(|| format!("{} {}", std::env::consts::OS, "Unknown"));

    let device_name = sysinfo::System::host_name()
        .unwrap_or_else(|| hostname::get()
            .unwrap_or_default()
            .to_string_lossy()
            .to_string());

    DetailedSystemInfo {
        device_name,
        device_system: os_version,
        hostname: sysinfo::System::host_name().unwrap_or_default(),
        os: std::env::consts::OS.to_string(),
        arch: std::env::consts::ARCH.to_string(),
    }
}

/// System information
#[derive(Debug, Clone)]
pub struct SystemInfo {
    pub os: String,
    pub arch: String,
    pub hostname: String,
}

/// Detailed system information including OS version and device name
#[derive(Debug, Clone)]
pub struct DetailedSystemInfo {
    pub device_name: String,
    pub device_system: String,
    pub hostname: String,
    pub os: String,
    pub arch: String,
}

/// Platform-specific clipboard access helpers
#[cfg(target_os = "windows")]
pub mod windows {
    use super::*;

    pub fn get_clipboard_formats() -> Result<Vec<String>> {
        // Windows-specific clipboard format enumeration
        // This would use winapi to get available clipboard formats
        Ok(vec!["CF_TEXT".to_string(), "CF_BITMAP".to_string()])
    }
}

#[cfg(target_os = "macos")]
pub mod macos {
    use super::*;

    pub fn get_pasteboard_types() -> Result<Vec<String>> {
        // macOS-specific pasteboard type enumeration
        // This would use Cocoa/AppKit to get available pasteboard types
        Ok(vec!["public.utf8-plain-text".to_string(), "public.png".to_string()])
    }
}

#[cfg(target_os = "linux")]
pub mod linux {
    use super::*;

    pub fn get_selection_targets() -> Result<Vec<String>> {
        // Linux-specific X11 selection target enumeration
        // This would use X11 or Wayland APIs to get available selection targets
        Ok(vec!["text/plain".to_string(), "image/png".to_string()])
    }
}
