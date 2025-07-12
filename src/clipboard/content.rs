//! Clipboard content types and serialization

use serde::{Deserialize, Serialize};
use std::fmt;

/// Types of clipboard content
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ContentType {
    /// Plain text content
    Text,
    /// Image content (PNG, JPEG, etc.)
    Image,
    /// File paths or file content
    File,
    /// Rich text (HTML, RTF)
    RichText,
    /// Custom binary data
    Binary,
}

impl fmt::Display for ContentType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ContentType::Text => write!(f, "text"),
            ContentType::Image => write!(f, "image"),
            ContentType::File => write!(f, "file"),
            ContentType::RichText => write!(f, "rich_text"),
            ContentType::Binary => write!(f, "binary"),
        }
    }
}

/// Clipboard content container
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClipboardContent {
    /// Type of content
    pub content_type: ContentType,
    /// Raw content data
    pub data: Vec<u8>,
    /// Content metadata
    pub metadata: ContentMetadata,
    /// Content checksum for integrity verification
    pub checksum: String,
}

/// Content metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContentMetadata {
    /// MIME type
    pub mime_type: Option<String>,
    /// Original filename (for files)
    pub filename: Option<String>,
    /// Content size in bytes
    pub size: usize,
    /// Creation timestamp
    pub created_at: u64,
    /// Source device ID
    pub source_device: String,
    /// Whether content is compressed
    pub compressed: bool,
}

impl ClipboardContent {
    /// Create new text content
    pub fn new_text(text: String, source_device: String) -> Self {
        let data = text.into_bytes();
        let size = data.len();
        let checksum = Self::calculate_checksum(&data);

        Self {
            content_type: ContentType::Text,
            data,
            metadata: ContentMetadata {
                mime_type: Some("text/plain".to_string()),
                filename: None,
                size,
                created_at: chrono::Utc::now().timestamp_millis() as u64,
                source_device,
                compressed: false,
            },
            checksum,
        }
    }

    /// Create new image content
    pub fn new_image(image_data: Vec<u8>, mime_type: String, source_device: String) -> Self {
        let size = image_data.len();
        let checksum = Self::calculate_checksum(&image_data);

        Self {
            content_type: ContentType::Image,
            data: image_data,
            metadata: ContentMetadata {
                mime_type: Some(mime_type),
                filename: None,
                size,
                created_at: chrono::Utc::now().timestamp_millis() as u64,
                source_device,
                compressed: false,
            },
            checksum,
        }
    }

    /// Create new file content
    pub fn new_file(
        file_data: Vec<u8>,
        filename: String,
        mime_type: Option<String>,
        source_device: String,
    ) -> Self {
        let size = file_data.len();
        let checksum = Self::calculate_checksum(&file_data);

        Self {
            content_type: ContentType::File,
            data: file_data,
            metadata: ContentMetadata {
                mime_type,
                filename: Some(filename),
                size,
                created_at: chrono::Utc::now().timestamp_millis() as u64,
                source_device,
                compressed: false,
            },
            checksum,
        }
    }

    /// Get content as text (if it's text content)
    pub fn as_text(&self) -> Option<String> {
        if self.content_type == ContentType::Text {
            String::from_utf8(self.data.clone()).ok()
        } else {
            None
        }
    }

    /// Get content as bytes
    pub fn to_bytes(&self) -> Vec<u8> {
        self.data.clone()
    }

    /// Verify content integrity
    pub fn verify_integrity(&self) -> bool {
        let calculated_checksum = Self::calculate_checksum(&self.data);
        calculated_checksum == self.checksum
    }

    /// Calculate SHA-256 checksum
    fn calculate_checksum(data: &[u8]) -> String {
        use sha2::{Digest, Sha256};
        let mut hasher = Sha256::new();
        hasher.update(data);
        format!("{:x}", hasher.finalize())
    }

    /// Compress content data
    #[cfg(feature = "compression")]
    pub fn compress(&mut self) -> crate::clipboard::Result<()> {
        use flate2::{write::GzEncoder, Compression};
        use std::io::Write;

        if self.metadata.compressed {
            return Ok(());
        }

        let mut encoder = GzEncoder::new(Vec::new(), Compression::default());
        encoder.write_all(&self.data)?;
        let compressed_data = encoder.finish()?;

        // Only use compression if it actually reduces size
        if compressed_data.len() < self.data.len() {
            self.data = compressed_data;
            self.metadata.compressed = true;
            self.checksum = Self::calculate_checksum(&self.data);
        }

        Ok(())
    }

    /// Decompress content data
    #[cfg(feature = "compression")]
    pub fn decompress(&mut self) -> crate::clipboard::Result<()> {
        use flate2::read::GzDecoder;
        use std::io::Read;

        if !self.metadata.compressed {
            return Ok(());
        }

        let mut decoder = GzDecoder::new(&self.data[..]);
        let mut decompressed_data = Vec::new();
        decoder.read_to_end(&mut decompressed_data)?;

        self.data = decompressed_data;
        self.metadata.compressed = false;
        self.metadata.size = self.data.len();
        self.checksum = Self::calculate_checksum(&self.data);

        Ok(())
    }
}

impl fmt::Display for ClipboardContent {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "ClipboardContent(type: {}, size: {} bytes, source: {})",
            self.content_type, self.metadata.size, self.metadata.source_device
        )
    }
}
