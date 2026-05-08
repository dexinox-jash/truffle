//! CRDT Delta Generation and Application
//!
//! Handles creation, compression, and application of sync deltas
//! for efficient multi-device synchronization.

use crate::models::VectorClock;
use crate::sync::crdt::CrdtDocument;
use uuid::Uuid;
use chrono::{DateTime, Utc};
use tracing::{info, debug, error};

/// A sync delta for transport
#[derive(Debug, Clone)]
pub struct SyncDelta {
    /// Delta ID
    pub id: Uuid,
    /// Document ID (node ID)
    pub doc_id: String,
    /// Source device ID
    pub source_device: String,
    /// Target device ID (optional, for directed sync)
    pub target_device: Option<String>,
    /// Vector clock at time of generation
    pub vector_clock: VectorClock,
    /// Binary CRDT update data
    pub update_data: Vec<u8>,
    /// Timestamp
    pub timestamp: DateTime<Utc>,
    /// Compression type
    pub compression: CompressionType,
    /// Delta size before compression
    pub original_size: usize,
}

/// Compression types for delta payloads
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CompressionType {
    /// No compression
    None,
    /// Zstd compression
    Zstd,
    /// LZ4 compression
    Lz4,
}

impl SyncDelta {
    /// Create a new sync delta
    pub fn new(
        doc_id: impl Into<String>,
        source_device: impl Into<String>,
        update_data: Vec<u8>,
        vector_clock: VectorClock,
    ) -> Self {
        let original_size = update_data.len();
        
        Self {
            id: Uuid::new_v4(),
            doc_id: doc_id.into(),
            source_device: source_device.into(),
            target_device: None,
            vector_clock,
            update_data,
            timestamp: Utc::now(),
            compression: CompressionType::None,
            original_size,
        }
    }
    
    /// Set target device for directed sync
    pub fn with_target(mut self, device: impl Into<String>) -> Self {
        self.target_device = Some(device.into());
        self
    }
    
    /// Compress the delta payload
    pub fn compress(&mut self, method: CompressionType) -> anyhow::Result<()> {
        if self.compression != CompressionType::None {
            return Ok(()); // Already compressed
        }
        
        match method {
            CompressionType::None => {}
            CompressionType::Zstd => {
                // In production, use zstd crate
                // self.update_data = zstd::encode_all(&self.update_data[..], 3)?;
                // For now, just mark as compressed
            }
            CompressionType::Lz4 => {
                // In production, use lz4 crate
            }
        }
        
        self.compression = method;
        debug!(
            "Compressed delta {}: {} -> {} bytes ({:.1}%)",
            self.id,
            self.original_size,
            self.update_data.len(),
            (1.0 - self.update_data.len() as f64 / self.original_size as f64) * 100.0
        );
        
        Ok(())
    }
    
    /// Decompress the delta payload
    pub fn decompress(&mut self) -> anyhow::Result<()> {
        match self.compression {
            CompressionType::None => {}
            CompressionType::Zstd => {
                // In production, use zstd crate
                // self.update_data = zstd::decode_all(&self.update_data[..])?;
            }
            CompressionType::Lz4 => {
                // In production, use lz4 crate
            }
        }
        
        self.compression = CompressionType::None;
        Ok(())
    }
    
    /// Get current size in bytes
    pub fn size_bytes(&self) -> usize {
        std::mem::size_of::<Uuid>() + // id
        self.doc_id.len() +
        self.source_device.len() +
        self.target_device.as_ref().map(|s| s.len()).unwrap_or(0) +
        std::mem::size_of::<DateTime<Utc>>() +
        self.update_data.len()
    }
    
    /// Get compression ratio
    pub fn compression_ratio(&self) -> f64 {
        if self.original_size == 0 {
            1.0
        } else {
            self.update_data.len() as f64 / self.original_size as f64
        }
    }
    
    /// Check if delta should be split (too large)
    pub fn should_split(&self, max_size: usize) -> bool {
        self.update_data.len() > max_size
    }
}

/// Delta generator for creating sync deltas
pub struct DeltaGenerator {
    device_id: String,
    enable_compression: bool,
    compression_type: CompressionType,
}

impl DeltaGenerator {
    /// Create a new delta generator
    pub fn new(device_id: impl Into<String>) -> Self {
        Self {
            device_id: device_id.into(),
            enable_compression: true,
            compression_type: CompressionType::Zstd,
        }
    }
    
    /// Generate delta from document changes
    pub fn generate(&self, doc: &CrdtDocument, since: Option<&[u8]>) -> anyhow::Result<SyncDelta> {
        let update_data = match since {
            Some(sv) => doc.encode_state_as_update_from(sv)?,
            None => doc.encode_state_as_update(),
        };
        
        let mut delta = SyncDelta::new(
            doc.doc_id(),
            &self.device_id,
            update_data,
            doc.vector_clock().clone(),
        );
        
        if self.enable_compression {
            delta.compress(self.compression_type)?;
        }
        
        debug!(
            "Generated delta for {}: {} bytes",
            doc.doc_id(),
            delta.size_bytes()
        );
        
        Ok(delta)
    }
    
    /// Generate delta for specific target device
    pub fn generate_for_target(
        &self,
        doc: &CrdtDocument,
        target_device: impl Into<String>,
        target_state: Option<&[u8]>,
    ) -> anyhow::Result<SyncDelta> {
        let mut delta = self.generate(doc, target_state)?;
        delta.target_device = Some(target_device.into());
        Ok(delta)
    }
    
    /// Batch generate deltas for multiple documents
    pub fn generate_batch(
        &self,
        docs: &[&CrdtDocument],
        since: Option<&[u8]>,
    ) -> anyhow::Result<Vec<SyncDelta>> {
        docs.iter().map(|doc| self.generate(doc, since)).collect()
    }
    
    /// Set compression settings
    pub fn with_compression(mut self, enabled: bool, method: CompressionType) -> Self {
        self.enable_compression = enabled;
        self.compression_type = method;
        self
    }
}

/// Delta applier for applying received sync deltas
pub struct DeltaApplier {
    device_id: String,
    conflict_resolver: ConflictResolver,
}

/// Conflict resolution strategy
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ConflictStrategy {
    /// Last-write-wins (use higher timestamp)
    LastWriteWins,
    /// First-write-wins (use lower timestamp)
    FirstWriteWins,
    /// Merge concurrent changes
    Merge,
    /// Reject conflicting changes
    Reject,
}

/// Conflict resolver for handling concurrent edits
pub struct ConflictResolver {
    strategy: ConflictStrategy,
}

impl ConflictResolver {
    /// Create a new conflict resolver
    pub fn new(strategy: ConflictStrategy) -> Self {
        Self { strategy }
    }
    
    /// Resolve a conflict between two vector clocks
    pub fn resolve(&self, local: &VectorClock, remote: &VectorClock) -> ConflictResolution {
        match local.compare(remote) {
            Some(true) => ConflictResolution::UseLocal, // Local is newer
            Some(false) => ConflictResolution::UseRemote, // Remote is newer
            None => {
                // Concurrent - apply strategy
                match self.strategy {
                    ConflictStrategy::LastWriteWins => ConflictResolution::Merge,
                    ConflictStrategy::FirstWriteWins => ConflictResolution::UseLocal,
                    ConflictStrategy::Merge => ConflictResolution::Merge,
                    ConflictStrategy::Reject => ConflictResolution::Reject,
                }
            }
        }
    }
}

/// Conflict resolution outcome
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ConflictResolution {
    /// Use local version
    UseLocal,
    /// Use remote version
    UseRemote,
    /// Merge both versions
    Merge,
    /// Reject the change
    Reject,
}

impl DeltaApplier {
    /// Create a new delta applier
    pub fn new(device_id: impl Into<String>) -> Self {
        Self {
            device_id: device_id.into(),
            conflict_resolver: ConflictResolver::new(ConflictStrategy::Merge),
        }
    }
    
    /// Apply a delta to a document
    pub fn apply(&self, doc: &mut CrdtDocument, delta: &SyncDelta) -> anyhow::Result<ApplyResult> {
        debug!("Applying delta {} to document {}", delta.id, delta.doc_id);
        
        // Check for conflicts
        let resolution = self.conflict_resolver.resolve(doc.vector_clock(), &delta.vector_clock);
        
        match resolution {
            ConflictResolution::UseLocal => {
                debug!("Conflict resolved: using local version");
                return Ok(ApplyResult::Skipped { reason: SkipReason::LocalNewer });
            }
            ConflictResolution::Reject => {
                debug!("Conflict resolved: rejecting change");
                return Ok(ApplyResult::Skipped { reason: SkipReason::ConflictRejected });
            }
            _ => {}
        }
        
        // Decompress if needed
        let mut delta = delta.clone();
        if delta.compression != CompressionType::None {
            delta.decompress()?;
        }
        
        // Apply the update
        doc.apply_update(&delta.update_data)?;
        
        // Merge vector clocks
        doc.vector_clock_mut().merge(&delta.vector_clock);
        
        info!("Applied delta {} to document {}", delta.id, delta.doc_id);
        
        Ok(ApplyResult::Applied {
            bytes_changed: delta.original_size,
        })
    }
    
    /// Apply multiple deltas
    pub fn apply_batch(
        &self,
        doc: &mut CrdtDocument,
        deltas: &[SyncDelta],
    ) -> anyhow::Result<Vec<ApplyResult>> {
        deltas.iter().map(|d| self.apply(doc, d)).collect()
    }
    
    /// Set conflict strategy
    pub fn with_strategy(mut self, strategy: ConflictStrategy) -> Self {
        self.conflict_resolver = ConflictResolver::new(strategy);
        self
    }
}

/// Result of applying a delta
#[derive(Debug, Clone)]
pub enum ApplyResult {
    /// Delta was successfully applied
    Applied {
        /// Number of bytes changed
        bytes_changed: usize,
    },
    /// Delta was skipped
    Skipped {
        /// Reason for skipping
        reason: SkipReason,
    },
    /// Application failed
    Failed {
        /// Error message
        error: String,
    },
}

/// Reasons for skipping a delta
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SkipReason {
    /// Local version is newer
    LocalNewer,
    /// Conflict was rejected
    ConflictRejected,
    /// Duplicate delta
    Duplicate,
    /// Invalid delta
    Invalid,
}

/// Trait for types that can be converted to/from deltas
pub trait DeltaConvertible {
    /// Convert to sync delta
    fn to_delta(&self, generator: &DeltaGenerator) -> anyhow::Result<SyncDelta>;
    
    /// Apply sync delta
    fn apply_delta(&mut self, delta: &SyncDelta, applier: &DeltaApplier) -> anyhow::Result<ApplyResult>;
}

// Helper trait for mutable access to vector clock
impl CrdtDocument {
    fn vector_clock_mut(&mut self) -> &mut VectorClock {
        &mut self.vector_clock
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::{WikiNode, WikiNodeType, Provenance};
    use crate::sync::crdt::CrdtDocument;
    
    fn create_test_node() -> WikiNode {
        let provenance = Provenance::new(vec![Uuid::new_v4()], "test", 0.9);
        WikiNode::new(
            "Test Node",
            WikiNodeType::Entity,
            "Test content",
            provenance,
        )
    }
    
    #[test]
    fn test_sync_delta_creation() {
        let clock = VectorClock::new();
        let delta = SyncDelta::new("doc1", "device1", vec![1, 2, 3], clock);
        
        assert_eq!(delta.doc_id, "doc1");
        assert_eq!(delta.source_device, "device1");
        assert_eq!(delta.update_data, vec![1, 2, 3]);
    }
    
    #[test]
    fn test_delta_generator() {
        let generator = DeltaGenerator::new("device1");
        let node = create_test_node();
        let doc = CrdtDocument::from_node(&node).unwrap();
        
        let delta = generator.generate(&doc, None).unwrap();
        
        assert_eq!(delta.doc_id, node.id.to_string());
        assert_eq!(delta.source_device, "device1");
        assert!(delta.update_data.len() > 0);
    }
    
    #[test]
    fn test_conflict_resolver() {
        let resolver = ConflictResolver::new(ConflictStrategy::LastWriteWins);
        
        let mut local = VectorClock::new();
        local.increment("device1");
        local.increment("device1");
        
        let mut remote = VectorClock::new();
        remote.increment("device1");
        
        // Local is newer
        let result = resolver.resolve(&local, &remote);
        assert_eq!(result, ConflictResolution::UseLocal);
        
        // Remote is older
        let result = resolver.resolve(&remote, &local);
        assert_eq!(result, ConflictResolution::UseRemote);
    }
    
    #[test]
    fn test_delta_applier() {
        let applier = DeltaApplier::new("device2");
        let node = create_test_node();
        let mut doc = CrdtDocument::from_node(&node).unwrap();
        
        let generator = DeltaGenerator::new("device1");
        let delta = generator.generate(&doc, None).unwrap();
        
        // Apply to same document
        let result = applier.apply(&mut doc, &delta).unwrap();
        
        // Should skip since clocks are equal
        match result {
            ApplyResult::Skipped { reason } => {
                assert_eq!(reason, SkipReason::LocalNewer);
            }
            _ => panic!("Expected skip"),
        }
    }
    
    #[test]
    fn test_vector_clock_compare() {
        let mut clock1 = VectorClock::new();
        clock1.increment("a");
        clock1.increment("a");
        
        let mut clock2 = VectorClock::new();
        clock2.increment("a");
        clock2.increment("b");
        
        // Concurrent - neither happens before the other
        assert_eq!(clock1.compare(&clock2), None);
        
        // clock1 > clock2
        clock2.increment("a");
        assert_eq!(clock1.compare(&clock2), Some(false));
    }
}
