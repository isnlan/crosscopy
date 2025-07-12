//! Network protocol implementation

use serde::{Deserialize, Serialize};
use std::fmt;

/// Protocol version
pub const PROTOCOL_VERSION: u16 = 1;

/// Protocol magic number
pub const PROTOCOL_MAGIC: u32 = 0x43505354; // "CPST"

/// Message types
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[repr(u16)]
pub enum MessageType {
    Handshake = 0x0001,
    Heartbeat = 0x0002,
    ClipboardSync = 0x0003,
    DeviceInfo = 0x0004,
    Ack = 0x0005,
    Error = 0x0006,
}

/// Protocol version information
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct ProtocolVersion {
    pub major: u8,
    pub minor: u8,
}

impl ProtocolVersion {
    pub const fn new(major: u8, minor: u8) -> Self {
        Self { major, minor }
    }

    pub const fn current() -> Self {
        Self::new(1, 0)
    }
}

impl fmt::Display for ProtocolVersion {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}.{}", self.major, self.minor)
    }
}

/// Network message
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Message {
    pub header: MessageHeader,
    pub payload: Vec<u8>,
}

/// Message header
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MessageHeader {
    pub magic: u32,
    pub version: u16,
    pub message_type: MessageType,
    pub length: u32,
    pub timestamp: u64,
    pub device_id: String,
    pub message_id: String,
    pub checksum: String,
}

impl Message {
    /// Create a new message
    pub fn new(
        message_type: MessageType,
        payload: Vec<u8>,
        device_id: String,
    ) -> Self {
        let header = MessageHeader {
            magic: PROTOCOL_MAGIC,
            version: PROTOCOL_VERSION,
            message_type,
            length: payload.len() as u32,
            timestamp: chrono::Utc::now().timestamp_millis() as u64,
            device_id,
            message_id: uuid::Uuid::new_v4().to_string(),
            checksum: Self::calculate_checksum(&payload),
        };

        Self { header, payload }
    }

    /// Verify message integrity
    pub fn verify(&self) -> bool {
        let calculated_checksum = Self::calculate_checksum(&self.payload);
        calculated_checksum == self.header.checksum
    }

    /// Calculate message checksum
    fn calculate_checksum(data: &[u8]) -> String {
        use sha2::{Digest, Sha256};
        let mut hasher = Sha256::new();
        hasher.update(data);
        format!("{:x}", hasher.finalize())
    }
}

impl fmt::Display for MessageType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            MessageType::Handshake => write!(f, "HANDSHAKE"),
            MessageType::Heartbeat => write!(f, "HEARTBEAT"),
            MessageType::ClipboardSync => write!(f, "CLIPBOARD_SYNC"),
            MessageType::DeviceInfo => write!(f, "DEVICE_INFO"),
            MessageType::Ack => write!(f, "ACK"),
            MessageType::Error => write!(f, "ERROR"),
        }
    }
}
