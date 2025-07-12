//! Event system module
//!
//! This module provides an event bus for inter-module communication,
//! allowing different parts of the application to communicate asynchronously.

pub mod bus;
pub mod handlers;

pub use bus::EventBus;
pub use handlers::EventHandler;

use crate::clipboard::ClipboardContent;
use crate::network::Message;
use serde::{Deserialize, Serialize};
use std::fmt;
use thiserror::Error;

/// Event system errors
#[derive(Debug, Error)]
pub enum EventError {
    #[error("Event bus is full")]
    BusFull,

    #[error("Event handler registration failed: {0}")]
    HandlerRegistrationFailed(String),

    #[error("Event processing failed: {0}")]
    ProcessingFailed(String),

    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),
}

/// Result type for event operations
pub type Result<T> = std::result::Result<T, EventError>;

/// Application events
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Event {
    /// Clipboard content changed
    ClipboardChanged {
        content: ClipboardContent,
        device_id: String,
    },

    /// Network message received
    NetworkMessage {
        message: Message,
        sender: String,
    },

    /// Device connected
    DeviceConnected {
        device_id: String,
    },

    /// Device disconnected
    DeviceDisconnected {
        device_id: String,
    },

    /// Application error occurred
    Error {
        error: String,
    },

    /// Heartbeat event
    Heartbeat {
        device_id: String,
        timestamp: u64,
    },

    /// Configuration changed
    ConfigChanged {
        section: String,
    },

    /// Shutdown requested
    Shutdown,
}

impl fmt::Display for Event {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Event::ClipboardChanged { device_id, .. } => {
                write!(f, "ClipboardChanged(device_id: {})", device_id)
            }
            Event::NetworkMessage { sender, .. } => {
                write!(f, "NetworkMessage(sender: {})", sender)
            }
            Event::DeviceConnected { device_id } => {
                write!(f, "DeviceConnected(device_id: {})", device_id)
            }
            Event::DeviceDisconnected { device_id } => {
                write!(f, "DeviceDisconnected(device_id: {})", device_id)
            }
            Event::Error { error } => {
                write!(f, "Error({})", error)
            }
            Event::Heartbeat { device_id, timestamp } => {
                write!(f, "Heartbeat(device_id: {}, timestamp: {})", device_id, timestamp)
            }
            Event::ConfigChanged { section } => {
                write!(f, "ConfigChanged(section: {})", section)
            }
            Event::Shutdown => {
                write!(f, "Shutdown")
            }
        }
    }
}

/// Event priority levels
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum EventPriority {
    Low = 0,
    Normal = 1,
    High = 2,
    Critical = 3,
}

impl Default for EventPriority {
    fn default() -> Self {
        EventPriority::Normal
    }
}

/// Event with metadata
#[derive(Debug, Clone)]
pub struct EventWithMetadata {
    pub event: Event,
    pub priority: EventPriority,
    pub timestamp: u64,
    pub source: String,
}

impl EventWithMetadata {
    pub fn new(event: Event, source: String) -> Self {
        Self {
            event,
            priority: EventPriority::default(),
            timestamp: chrono::Utc::now().timestamp_millis() as u64,
            source,
        }
    }

    pub fn with_priority(mut self, priority: EventPriority) -> Self {
        self.priority = priority;
        self
    }
}
