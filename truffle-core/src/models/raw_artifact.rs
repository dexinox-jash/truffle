//! RawArtifact Model
//!
//! The RawArtifact represents an immutable capture of visual data (screenshot).
//! It is content-addressable (UUID derived from SHA256 hash) and contains
//! metadata about the capture context.
//!
//! Per Section 2.1.1 of the Enterprise Specification:
//! - NO foreign keys - Raw is autonomous
//! - Content-addressable UUID (SHA256 of image)
//! - Local filesystem storage only
//! - Privacy-preserving device fingerprint and geohash

use serde::{Deserialize, Serialize};
use uuid::Uuid;
use chrono::{DateTime, Utc};

use super::{BlobPointer, DeviceFingerprint, Geohash, content_addressable_uuid};

/// Status of a raw artifact in the ingestion pipeline
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Default)]
#[serde(rename_all = "snake_case")]
pub enum IngestionStatus {
    /// Just captured, awaiting processing
    #[default]
    Pending,
    /// Currently being processed
    Processing,
    /// Successfully compiled to wiki
    Compiled,
    /// Failed to process (quarantined)
    Failed,
    /// User-marked for exclusion
    Excluded,
}

/// Application context captured at screenshot time
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct AppContext {
    /// Bundle identifier (e.g., com.apple.Safari)
    pub bundle_id: String,
    /// Human-readable app name
    pub app_name: String,
    /// Window title (macOS Accessibility API, user opt-in)
    pub window_title: Option<String>,
}

/// Image metadata extracted at capture time
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct RawArtifactMetadata {
    /// File size in bytes
    pub size_bytes: u64,
    /// Image width in pixels
    pub width: u32,
    /// Image height in pixels
    pub height: u32,
    /// OCR text (populated post-compilation)
    pub ocr_text: Option<String>,
    /// Semantic embedding vector (MiniLM-L6-v2, 384 dimensions)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub embedding_vector: Option<Vec<f32>>,
}

impl RawArtifactMetadata {
    /// Create new metadata from image dimensions
    pub fn new(size_bytes: u64, width: u32, height: u32) -> Self {
        Self {
            size_bytes,
            width,
            height,
            ocr_text: None,
            embedding_vector: None,
        }
    }
    
    /// Set OCR text (called after compilation)
    pub fn with_ocr(mut self, text: String) -> Self {
        self.ocr_text = Some(text);
        self
    }
    
    /// Set embedding vector (called after compilation)
    pub fn with_embedding(mut self, vector: Vec<f32>) -> Self {
        self.embedding_vector = Some(vector);
        self
    }
    
    /// Validate embedding vector dimensions
    pub fn validate_embedding(&self) -> bool {
        match &self.embedding_vector {
            None => true, // No embedding is valid
            Some(v) => v.len() == 384, // MiniLM-L6-v2 produces 384-dim vectors
        }
    }
}

/// The RawArtifact - immutable capture of visual data
/// 
/// This is the foundational data type in Truffle. Every screenshot becomes
/// a RawArtifact before being compiled into the knowledge graph.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RawArtifact {
    /// Content-addressable UUID (SHA256 of image content)
    pub uuid: Uuid,
    /// Original capture filename
    pub filename: String,
    /// Pointer to local filesystem storage
    pub binary: BlobPointer,
    /// Device timestamp (timezone-aware)
    pub captured_at: DateTime<Utc>,
    /// Privacy-preserving device fingerprint
    pub device_id: DeviceFingerprint,
    /// Optional application context
    #[serde(skip_serializing_if = "Option::is_none")]
    pub app_context: Option<AppContext>,
    /// Privacy-safe location (4-char geohash = ~20km)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub geohash: Option<Geohash>,
    /// Image metadata
    pub metadata: RawArtifactMetadata,
    /// Current ingestion status
    #[serde(default)]
    pub status: IngestionStatus,
    /// Retry count for failed processing
    #[serde(default)]
    pub retry_count: u32,
    /// When the artifact was added to the system
    pub created_at: DateTime<Utc>,
    /// When the artifact was last updated
    pub updated_at: DateTime<Utc>,
}

impl RawArtifact {
    /// Create a new RawArtifact from image content
    /// 
    /// # Arguments
    /// * `filename` - Original capture name
    /// * `content` - Raw image bytes
    /// * `storage_path` - Where to store the image locally
    /// * `device_id` - Privacy-preserving device fingerprint
    /// * `captured_at` - When the screenshot was captured
    /// 
    /// # Returns
    /// A new RawArtifact with content-addressable UUID
    pub fn new(
        filename: impl Into<String>,
        content: &[u8],
        storage_path: std::path::PathBuf,
        device_id: DeviceFingerprint,
        captured_at: DateTime<Utc>,
    ) -> anyhow::Result<Self> {
        // Generate content-addressable UUID
        let uuid = content_addressable_uuid(content);
        
        // Create blob pointer with content hash
        let binary = BlobPointer::new(storage_path, content);
        
        // Extract image dimensions
        let (width, height) = extract_image_dimensions(content)?;
        
        let metadata = RawArtifactMetadata::new(
            content.len() as u64,
            width,
            height,
        );
        
        let now = Utc::now();
        
        Ok(Self {
            uuid,
            filename: filename.into(),
            binary,
            captured_at,
            device_id,
            app_context: None,
            geohash: None,
            metadata,
            status: IngestionStatus::Pending,
            retry_count: 0,
            created_at: now,
            updated_at: now,
        })
    }
    
    /// Add application context
    pub fn with_app_context(mut self, context: AppContext) -> Self {
        self.app_context = Some(context);
        self
    }
    
    /// Add geolocation (privacy-safe, 4-char geohash)
    pub fn with_geolocation(mut self, latitude: f64, longitude: f64) -> Self {
        self.geohash = Some(Geohash::new(latitude, longitude));
        self
    }
    
    /// Mark as processing
    pub fn mark_processing(&mut self) {
        self.status = IngestionStatus::Processing;
        self.updated_at = Utc::now();
    }
    
    /// Mark as compiled
    pub fn mark_compiled(&mut self) {
        self.status = IngestionStatus::Compiled;
        self.updated_at = Utc::now();
    }
    
    /// Mark as failed
    pub fn mark_failed(&mut self) {
        self.status = IngestionStatus::Failed;
        self.retry_count += 1;
        self.updated_at = Utc::now();
    }
    
    /// Mark as excluded
    pub fn mark_excluded(&mut self) {
        self.status = IngestionStatus::Excluded;
        self.updated_at = Utc::now();
    }
    
    /// Check if this artifact should be retried
    pub fn should_retry(&self, max_retries: u32) -> bool {
        self.status == IngestionStatus::Failed && self.retry_count < max_retries
    }
    
    /// Update OCR text (called after compilation)
    pub fn set_ocr_text(&mut self, text: String) {
        self.metadata.ocr_text = Some(text);
        self.updated_at = Utc::now();
    }
    
    /// Update embedding vector (called after compilation)
    pub fn set_embedding(&mut self, vector: Vec<f32>) {
        self.metadata.embedding_vector = Some(vector);
        self.updated_at = Utc::now();
    }
    
    /// Verify file integrity
    pub fn verify_integrity(&self) -> anyhow::Result<bool> {
        self.binary.verify()
    }
    
    /// Get the storage path
    pub fn storage_path(&self) -> &std::path::Path {
        &self.binary.path
    }
    
    /// Calculate aspect ratio
    pub fn aspect_ratio(&self) -> f64 {
        self.metadata.width as f64 / self.metadata.height.max(1) as f64
    }
    
    /// Check if this is a landscape image
    pub fn is_landscape(&self) -> bool {
        self.metadata.width > self.metadata.height
    }
    
    /// Get priority score for ingestion queue
    /// P0: User-initiated (highest)
    /// P1: Messaging context (temporal relevance decays fast)
    /// P2: Background batch (lowest)
    pub fn priority_score(&self) -> u32 {
        match &self.app_context {
            Some(ctx) => {
                let bundle_lower = ctx.bundle_id.to_lowercase();
                if bundle_lower.contains("message") || 
                   bundle_lower.contains("slack") ||
                   bundle_lower.contains("telegram") ||
                   bundle_lower.contains("whatsapp") ||
                   bundle_lower.contains("signal") {
                    1 // P1: Messaging apps
                } else {
                    2 // P2: Everything else
                }
            }
            None => 2, // P2: No context
        }
    }
}

/// Extract image dimensions from content
fn extract_image_dimensions(content: &[u8]) -> anyhow::Result<(u32, u32)> {
    // Try to use the image crate for common formats
    match image::load_from_memory(content) {
        Ok(img) => Ok((img.width(), img.height())),
        Err(_) => {
            // Fallback: try to parse PNG/JPEG headers manually
            if content.starts_with(b"\x89PNG\r\n\x1a\n") {
                parse_png_dimensions(content)
            } else if content.starts_with(&[0xFF, 0xD8, 0xFF]) {
                parse_jpeg_dimensions(content)
            } else {
                Err(anyhow::anyhow!("Unsupported image format"))
            }
        }
    }
}

/// Parse PNG dimensions from header
fn parse_png_dimensions(content: &[u8]) -> anyhow::Result<(u32, u32)> {
    // PNG IHDR chunk starts at byte 16
    if content.len() < 24 {
        return Err(anyhow::anyhow!("PNG too small"));
    }
    
    let width = u32::from_be_bytes([content[16], content[17], content[18], content[19]]);
    let height = u32::from_be_bytes([content[20], content[21], content[22], content[23]]);
    
    Ok((width, height))
}

/// Parse JPEG dimensions from SOF markers
fn parse_jpeg_dimensions(content: &[u8]) -> anyhow::Result<(u32, u32)> {
    let mut i = 2; // Skip SOI marker
    
    while i < content.len() - 1 {
        if content[i] == 0xFF {
            let marker = content[i + 1];
            
            // SOF markers (Start of Frame)
            if (0xC0..=0xCF).contains(&marker) && marker != 0xC4 && marker != 0xC8 && marker != 0xCC {
                if i + 9 < content.len() {
                    let height = u16::from_be_bytes([content[i + 5], content[i + 6]]) as u32;
                    let width = u16::from_be_bytes([content[i + 7], content[i + 8]]) as u32;
                    return Ok((width, height));
                }
            }
            
            // Skip marker segment
            if marker != 0x00 && marker != 0x01 && (marker < 0xD0 || marker > 0xD9) {
                if i + 3 < content.len() {
                    let len = u16::from_be_bytes([content[i + 2], content[i + 3]]) as usize;
                    i += len + 2;
                    continue;
                }
            }
        }
        i += 1;
    }
    
    Err(anyhow::anyhow!("Could not find JPEG SOF marker"))
}

/// Ingestion queue entry for pipeline processing
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IngestionQueueEntry {
    /// Queue entry ID
    pub id: Uuid,
    /// Reference to raw artifact
    pub raw_uuid: Uuid,
    /// Processing status
    pub status: IngestionStatus,
    /// Retry count
    pub retry_count: u32,
    /// Priority (0 = highest)
    pub priority: u32,
    /// When added to queue
    pub created_at: DateTime<Utc>,
    /// When processing started (if applicable)
    pub started_at: Option<DateTime<Utc>>,
    /// When processing completed (if applicable)
    pub completed_at: Option<DateTime<Utc>>,
    /// Error message (if failed)
    pub error_message: Option<String>,
}

impl IngestionQueueEntry {
    pub fn new(raw_uuid: Uuid, priority: u32) -> Self {
        Self {
            id: Uuid::new_v4(),
            raw_uuid,
            status: IngestionStatus::Pending,
            retry_count: 0,
            priority,
            created_at: Utc::now(),
            started_at: None,
            completed_at: None,
            error_message: None,
        }
    }
    
    pub fn mark_started(&mut self) {
        self.status = IngestionStatus::Processing;
        self.started_at = Some(Utc::now());
    }
    
    pub fn mark_completed(&mut self) {
        self.status = IngestionStatus::Compiled;
        self.completed_at = Some(Utc::now());
    }
    
    pub fn mark_failed(&mut self, error: impl Into<String>) {
        self.status = IngestionStatus::Failed;
        self.retry_count += 1;
        self.error_message = Some(error.into());
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    
    fn create_test_png() -> Vec<u8> {
        // Minimal 1x1 PNG
        vec![
            0x89, 0x50, 0x4E, 0x47, 0x0D, 0x0A, 0x1A, 0x0A, // PNG signature
            0x00, 0x00, 0x00, 0x0D, // IHDR length
            0x49, 0x48, 0x44, 0x52, // IHDR
            0x00, 0x00, 0x00, 0x01, // width: 1
            0x00, 0x00, 0x00, 0x01, // height: 1
            0x08, 0x02, 0x00, 0x00, 0x00, // bit depth, color type, etc.
            0x90, 0x77, 0x53, 0xDE, // CRC
        ]
    }
    
    #[test]
    fn test_raw_artifact_creation() {
        let content = create_test_png();
        let temp_dir = tempfile::tempdir().expect("Failed to create temp dir");
        let storage_path = temp_dir.path().join("test.png");
        
        let device_id = DeviceFingerprint::new("test_device", b"salt");
        let captured_at = Utc::now();
        
        let artifact = RawArtifact::new(
            "screenshot.png",
            &content,
            storage_path.clone(),
            device_id.clone(),
            captured_at,
        ).expect("Failed to create artifact");
        
        assert_eq!(artifact.filename, "screenshot.png");
        assert_eq!(artifact.metadata.width, 1);
        assert_eq!(artifact.metadata.height, 1);
        assert_eq!(artifact.status, IngestionStatus::Pending);
        assert_eq!(artifact.device_id, device_id);
    }
    
    #[test]
    fn test_content_addressable_uuid_determinism() {
        let content1 = b"test screenshot data";
        let content2 = b"test screenshot data";
        
        let uuid1 = content_addressable_uuid(content1);
        let uuid2 = content_addressable_uuid(content2);
        
        assert_eq!(uuid1, uuid2);
    }
    
    #[test]
    fn test_priority_score() {
        let content = create_test_png();
        let temp_dir = tempfile::tempdir().expect("Failed to create temp dir");
        let storage_path = temp_dir.path().join("test.png");
        let device_id = DeviceFingerprint::new("test_device", b"salt");
        
        let mut artifact = RawArtifact::new(
            "test.png",
            &content,
            storage_path,
            device_id,
            Utc::now(),
        ).expect("Failed to create artifact");
        
        // No context = P2
        assert_eq!(artifact.priority_score(), 2);
        
        // Messaging app = P1
        artifact.app_context = Some(AppContext {
            bundle_id: "com.apple.MobileSMS".to_string(),
            app_name: "Messages".to_string(),
            window_title: None,
        });
        assert_eq!(artifact.priority_score(), 1);
        
        // Slack = P1
        artifact.app_context = Some(AppContext {
            bundle_id: "com.tinyspeck.slackmacgap".to_string(),
            app_name: "Slack".to_string(),
            window_title: None,
        });
        assert_eq!(artifact.priority_score(), 1);
    }
    
    #[test]
    fn test_ingestion_status_transitions() {
        let content = create_test_png();
        let temp_dir = tempfile::tempdir().expect("Failed to create temp dir");
        let storage_path = temp_dir.path().join("test.png");
        let device_id = DeviceFingerprint::new("test_device", b"salt");
        
        let mut artifact = RawArtifact::new(
            "test.png",
            &content,
            storage_path,
            device_id,
            Utc::now(),
        ).expect("Failed to create artifact");
        
        assert_eq!(artifact.status, IngestionStatus::Pending);
        
        artifact.mark_processing();
        assert_eq!(artifact.status, IngestionStatus::Processing);
        
        artifact.mark_compiled();
        assert_eq!(artifact.status, IngestionStatus::Compiled);
        
        // Test retry logic
        let mut artifact2 = RawArtifact::new(
            "test2.png",
            &content,
            temp_dir.path().join("test2.png"),
            DeviceFingerprint::new("test_device2", b"salt"),
            Utc::now(),
        ).expect("Failed to create artifact");
        
        artifact2.mark_failed();
        assert_eq!(artifact2.status, IngestionStatus::Failed);
        assert_eq!(artifact2.retry_count, 1);
        assert!(artifact2.should_retry(3));
        assert!(!artifact2.should_retry(1));
    }
    
    #[test]
    fn test_ingestion_queue_entry() {
        let raw_uuid = Uuid::new_v4();
        let mut entry = IngestionQueueEntry::new(raw_uuid, 1);
        
        assert_eq!(entry.status, IngestionStatus::Pending);
        assert_eq!(entry.priority, 1);
        
        entry.mark_started();
        assert_eq!(entry.status, IngestionStatus::Processing);
        assert!(entry.started_at.is_some());
        
        entry.mark_completed();
        assert_eq!(entry.status, IngestionStatus::Compiled);
        assert!(entry.completed_at.is_some());
    }
    
    #[test]
    fn test_png_dimension_parsing() {
        let png = create_test_png();
        let (width, height) = parse_png_dimensions(&png).expect("Failed to parse PNG dimensions");
        assert_eq!(width, 1);
        assert_eq!(height, 1);
    }
}
