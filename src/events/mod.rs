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
        device_system: String,
    },

    /// Network message received
    NetworkMessage {
        message: Message,
        sender: String,
    },

    /// Device connected
    DeviceConnected {
        device_system: String,
    },

    /// Device disconnected
    DeviceDisconnected {
        device_system: String,
    },

    /// Peer discovered via mDNS
    PeerDiscovered {
        peer_id: String,
        address: String,
    },

    /// Peer connected
    PeerConnected {
        peer_id: String,
    },

    /// Peer disconnected
    PeerDisconnected {
        peer_id: String,
    },

    /// Clipboard synced from peer
    ClipboardSynced {
        from_peer: String,
        content_size: usize,
    },

    /// Application error occurred
    Error {
        error: String,
    },

    /// Heartbeat event
    Heartbeat {
        device_system: String,
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
            Event::ClipboardChanged { device_system, .. } => {
                write!(f, "ClipboardChanged(device_system: {})", device_system)
            }
            Event::NetworkMessage { sender, .. } => {
                write!(f, "NetworkMessage(sender: {})", sender)
            }
            Event::DeviceConnected { device_system } => {
                write!(f, "DeviceConnected(device_system: {})", device_system)
            }
            Event::DeviceDisconnected { device_system } => {
                write!(f, "DeviceDisconnected(device_system: {})", device_system)
            }
            Event::Error { error } => {
                write!(f, "Error({})", error)
            }
            Event::Heartbeat { device_system, timestamp } => {
                write!(f, "Heartbeat(device_system: {}, timestamp: {})", device_system, timestamp)
            }
            Event::ConfigChanged { section } => {
                write!(f, "ConfigChanged(section: {})", section)
            }
            Event::Shutdown => {
                write!(f, "Shutdown")
            }
            Event::PeerDiscovered { peer_id, address } => {
                write!(f, "PeerDiscovered(peer_id: {}, address: {})", peer_id, address)
            }
            Event::PeerConnected { peer_id } => {
                write!(f, "PeerConnected(peer_id: {})", peer_id)
            }
            Event::PeerDisconnected { peer_id } => {
                write!(f, "PeerDisconnected(peer_id: {})", peer_id)
            }
            Event::ClipboardSynced { from_peer, content_size } => {
                write!(f, "ClipboardSynced(from_peer: {}, content_size: {})", from_peer, content_size)
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
