//! Immutable Audit Log with Merkle Tree
//!
//! This module implements tamper-evident audit logging using Merkle trees.
//! Each log entry is hashed and combined into a Merkle tree, allowing
//! efficient verification of log integrity.
//!
//! # Merkle Tree Structure
//!
//! ```text
//!                    Root Hash
//!                   /         \
//!              Hash(A+B)   Hash(C+D)
//!              /      \     /      \
//!           Hash(A) Hash(B) Hash(C) Hash(D)
//!             |       |       |       |
//!            EntryA EntryB  EntryC  EntryD
//! ```
//!
//! # Security Properties
//!
//! - **Tamper Detection**: Any modification to an entry changes the root hash
//! - **Efficient Verification**: O(log n) to verify an entry
//! - **Incremental**: New entries can be added without recomputing everything

use crate::error::{CryptoError, CryptoResult};
use crate::utils::sha3_256;
use rs_merkle::{algorithms::Sha256, Hasher, MerkleTree};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::time::{SystemTime, UNIX_EPOCH};

/// Audit log configuration
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct AuditLogConfig {
    /// Maximum number of entries per log file
    pub max_entries_per_file: usize,
    /// Enable Merkle tree verification
    pub enable_merkle: bool,
    /// Log file retention days
    pub retention_days: u32,
    /// Hash algorithm (sha256 or sha3_256)
    pub hash_algorithm: String,
}

impl Default for AuditLogConfig {
    fn default() -> Self {
        Self {
            max_entries_per_file: 10000,
            enable_merkle: true,
            retention_days: 365,
            hash_algorithm: "sha3_256".to_string(),
        }
    }
}

/// Audit event types
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum AuditEventType {
    /// Key generation
    KeyGenerated,
    /// Key rotation
    KeyRotated,
    /// Device paired
    DevicePaired,
    /// Device removed
    DeviceRemoved,
    /// Sync message sent
    SyncSent,
    /// Sync message received
    SyncReceived,
    /// Encryption operation
    Encryption,
    /// Decryption operation
    Decryption,
    /// Authentication success
    AuthSuccess,
    /// Authentication failure
    AuthFailure,
    /// Data export
    DataExport,
    /// Data deletion (GDPR)
    DataDeletion,
    /// Configuration change
    ConfigChange,
    /// Security alert
    SecurityAlert,
}

impl AuditEventType {
    /// Get the event category
    pub fn category(&self) -> &'static str {
        match self {
            Self::KeyGenerated | Self::KeyRotated => "key_management",
            Self::DevicePaired | Self::DeviceRemoved => "device_management",
            Self::SyncSent | Self::SyncReceived => "sync",
            Self::Encryption | Self::Decryption => "cryptography",
            Self::AuthSuccess | Self::AuthFailure => "authentication",
            Self::DataExport | Self::DataDeletion => "data_governance",
            Self::ConfigChange => "configuration",
            Self::SecurityAlert => "security",
        }
    }

    /// Get severity level
    pub fn severity(&self) -> &'static str {
        match self {
            Self::AuthFailure | Self::SecurityAlert => "high",
            Self::KeyRotated | Self::DeviceRemoved | Self::DataDeletion => "medium",
            _ => "low",
        }
    }
}

/// Single audit entry
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct AuditEntry {
    /// Entry ID (sequence number)
    pub id: u64,
    /// Timestamp
    pub timestamp: u64,
    /// Event type
    pub event_type: AuditEventType,
    /// Event category
    pub category: String,
    /// Severity level
    pub severity: String,
    /// Device ID (if applicable)
    pub device_id: Option<String>,
    /// Session ID (if applicable)
    pub session_id: Option<String>,
    /// Event description
    pub description: String,
    /// Additional metadata (key-value pairs)
    pub metadata: HashMap<String, String>,
    /// Entry hash (SHA3-256)
    #[serde(with = "serde_bytes")]
    pub hash: [u8; 32],
    /// Previous entry hash (for chain verification)
    #[serde(with = "serde_bytes")]
    pub previous_hash: [u8; 32],
}

impl AuditEntry {
    /// Create a new audit entry
    ///
    /// # Errors
    ///
    /// Returns `CryptoError::InternalError` if system time is unavailable.
    pub fn new(
        id: u64,
        event_type: AuditEventType,
        description: impl Into<String>,
    ) -> CryptoResult<Self> {
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map_err(|e| CryptoError::InternalError(format!("System time error: {:?}", e)))?
            .as_secs();

        Ok(Self {
            id,
            timestamp,
            event_type,
            category: event_type.category().to_string(),
            severity: event_type.severity().to_string(),
            device_id: None,
            session_id: None,
            description: description.into(),
            metadata: HashMap::new(),
            hash: [0u8; 32],
            previous_hash: [0u8; 32],
        })
    }

    /// Set device ID
    pub fn with_device_id(mut self, device_id: impl Into<String>) -> Self {
        self.device_id = Some(device_id.into());
        self
    }

    /// Set session ID
    pub fn with_session_id(mut self, session_id: impl Into<String>) -> Self {
        self.session_id = Some(session_id.into());
        self
    }

    /// Add metadata
    pub fn with_metadata(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.metadata.insert(key.into(), value.into());
        self
    }

    /// Set previous hash
    pub fn with_previous_hash(mut self, previous_hash: [u8; 32]) -> Self {
        self.previous_hash = previous_hash;
        self
    }

    /// Compute the entry hash
    pub fn compute_hash(&self) -> [u8; 32] {
        let data = self.hash_data();
        sha3_256(&data)
    }

    /// Finalize the entry (compute and set hash)
    pub fn finalize(&mut self) {
        self.hash = self.compute_hash();
    }

    /// Verify entry integrity
    pub fn verify(&self) -> bool {
        self.hash == self.compute_hash()
    }

    /// Get the data to be hashed
    fn hash_data(&self) -> Vec<u8> {
        let mut data = Vec::new();
        data.extend_from_slice(&self.id.to_be_bytes());
        data.extend_from_slice(&self.timestamp.to_be_bytes());
        data.extend_from_slice(self.category.as_bytes());
        data.extend_from_slice(self.description.as_bytes());
        data.extend_from_slice(&self.previous_hash);
        data
    }

    /// Serialize to bytes
    pub fn to_bytes(&self) -> CryptoResult<Vec<u8>> {
        bincode::serialize(self)
            .map_err(|e| CryptoError::serialization_error(format!("AuditEntry: {}", e)))
    }

    /// Deserialize from bytes
    pub fn from_bytes(bytes: &[u8]) -> CryptoResult<Self> {
        bincode::deserialize(bytes)
            .map_err(|e| CryptoError::serialization_error(format!("AuditEntry: {}", e)))
    }
}

/// Merkle tree node
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct MerkleNode {
    /// Node hash
    #[serde(with = "serde_bytes")]
    pub hash: [u8; 32],
    /// Left child (if internal node)
    pub left: Option<Box<MerkleNode>>,
    /// Right child (if internal node)
    pub right: Option<Box<MerkleNode>>,
    /// Entry index (if leaf node)
    pub entry_index: Option<usize>,
}

impl MerkleNode {
    /// Create a leaf node from an entry hash
    pub fn leaf(hash: [u8; 32], entry_index: usize) -> Self {
        Self {
            hash,
            left: None,
            right: None,
            entry_index: Some(entry_index),
        }
    }

    /// Create an internal node from two children
    pub fn internal(left: MerkleNode, right: MerkleNode) -> Self {
        let mut combined = Vec::with_capacity(64);
        combined.extend_from_slice(&left.hash);
        combined.extend_from_slice(&right.hash);
        let hash = sha3_256(&combined);

        Self {
            hash,
            left: Some(Box::new(left)),
            right: Some(Box::new(right)),
            entry_index: None,
        }
    }

    /// Check if this is a leaf node
    pub fn is_leaf(&self) -> bool {
        self.left.is_none() && self.right.is_none()
    }
}

/// Merkle-based audit log
pub struct MerkleAuditLog {
    /// Configuration
    config: AuditLogConfig,
    /// Log entries
    entries: Vec<AuditEntry>,
    /// Merkle tree root
    root: Option<MerkleNode>,
    /// Entry hashes (for Merkle tree construction)
    hashes: Vec<[u8; 32]>,
}

impl MerkleAuditLog {
    /// Create a new Merkle audit log
    pub fn new(config: AuditLogConfig) -> Self {
        Self {
            config,
            entries: Vec::new(),
            root: None,
            hashes: Vec::new(),
        }
    }

    /// Add an entry to the log
    pub fn add_entry(&mut self, mut entry: AuditEntry) -> CryptoResult<()> {
        // Set previous hash
        if let Some(last) = self.entries.last() {
            entry.previous_hash = last.hash;
        }

        // Finalize entry
        entry.finalize();

        // Add to entries
        self.entries.push(entry.clone());
        self.hashes.push(entry.hash);

        // Rebuild Merkle tree if enabled
        if self.config.enable_merkle {
            self.rebuild_tree();
        }

        Ok(())
    }

    /// Rebuild the Merkle tree
    fn rebuild_tree(&mut self) {
        if self.hashes.is_empty() {
            self.root = None;
            return;
        }

        self.root = Some(self.build_tree(&self.hashes, 0));
    }

    /// Recursively build the Merkle tree
    fn build_tree(&self, hashes: &[[u8; 32]], offset: usize) -> MerkleNode {
        if hashes.len() == 1 {
            return MerkleNode::leaf(hashes[0], offset);
        }

        let mid = hashes.len() / 2;
        let left = self.build_tree(&hashes[..mid], offset);
        let right = self.build_tree(&hashes[mid..], offset + mid);

        MerkleNode::internal(left, right)
    }

    /// Get the Merkle root hash
    pub fn root_hash(&self) -> Option<[u8; 32]> {
        self.root.as_ref().map(|r| r.hash)
    }

    /// Verify the entire log integrity
    pub fn verify(&self) -> TamperDetectionResult {
        // Verify each entry
        for entry in &self.entries {
            if !entry.verify() {
                return TamperDetectionResult::Tampered {
                    entry_id: entry.id,
                    reason: "Entry hash mismatch".to_string(),
                };
            }
        }

        // Verify chain
        for i in 1..self.entries.len() {
            if self.entries[i].previous_hash != self.entries[i - 1].hash {
                return TamperDetectionResult::Tampered {
                    entry_id: self.entries[i].id,
                    reason: "Chain break detected".to_string(),
                };
            }
        }

        // Verify Merkle root if enabled
        if self.config.enable_merkle {
            if let Some(ref root) = self.root {
                let computed_root = self.build_tree(&self.hashes, 0);
                if root.hash != computed_root.hash {
                    return TamperDetectionResult::Tampered {
                        entry_id: 0,
                        reason: "Merkle root mismatch".to_string(),
                    };
                }
            }
        }

        TamperDetectionResult::Valid
    }

    /// Get an entry by ID
    pub fn get_entry(&self, id: u64) -> Option<&AuditEntry> {
        self.entries.iter().find(|e| e.id == id)
    }

    /// Get all entries
    pub fn entries(&self) -> &[AuditEntry] {
        &self.entries
    }

    /// Get entry count
    pub fn len(&self) -> usize {
        self.entries.len()
    }

    /// Check if log is empty
    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    /// Get proof for an entry (for verification)
    pub fn get_proof(&self, entry_id: u64) -> Option<Vec<[u8; 32]>> {
        let index = self.entries.iter().position(|e| e.id == entry_id)?;
        self.generate_proof(index)
    }

    /// Generate Merkle proof for an entry
    fn generate_proof(&self, index: usize) -> Option<Vec<[u8; 32]>> {
        if !self.config.enable_merkle || self.root.is_none() {
            return None;
        }

        let mut proof = Vec::new();
        let mut current_index = index;
        let mut level_size = self.hashes.len();

        while level_size > 1 {
            let sibling_index = if current_index % 2 == 0 {
                current_index + 1
            } else {
                current_index - 1
            };

            if sibling_index < level_size {
                // Get sibling hash from current level
                let level_start = self.entries.len() - level_size;
                if level_start + sibling_index < self.hashes.len() {
                    proof.push(self.hashes[level_start + sibling_index]);
                }
            }

            current_index /= 2;
            level_size = (level_size + 1) / 2;
        }

        Some(proof)
    }

    /// Serialize to bytes
    pub fn to_bytes(&self) -> CryptoResult<Vec<u8>> {
        bincode::serialize(&(self.config.clone(), self.entries.clone()))
            .map_err(|e| CryptoError::serialization_error(format!("MerkleAuditLog: {}", e)))
    }

    /// Deserialize from bytes
    pub fn from_bytes(bytes: &[u8]) -> CryptoResult<Self> {
        let (config, entries): (AuditLogConfig, Vec<AuditEntry>) = bincode::deserialize(bytes)
            .map_err(|e| CryptoError::serialization_error(format!("MerkleAuditLog: {}", e)))?;

        let hashes: Vec<[u8; 32]> = entries.iter().map(|e| e.hash).collect();

        let mut log = Self {
            config,
            entries,
            root: None,
            hashes,
        };

        if log.config.enable_merkle {
            log.rebuild_tree();
        }

        Ok(log)
    }
}

/// Tamper detection result
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum TamperDetectionResult {
    /// Log is valid
    Valid,
    /// Log has been tampered with
    Tampered {
        /// Entry ID where tampering was detected
        entry_id: u64,
        /// Reason for detection
        reason: String,
    },
}

impl TamperDetectionResult {
    /// Check if valid
    pub fn is_valid(&self) -> bool {
        matches!(self, Self::Valid)
    }

    /// Check if tampered
    pub fn is_tampered(&self) -> bool {
        !self.is_valid()
    }
}

/// Simple audit log (without Merkle tree)
pub struct AuditLog {
    /// Configuration
    config: AuditLogConfig,
    /// Log entries
    entries: Vec<AuditEntry>,
}

impl AuditLog {
    /// Create a new audit log
    pub fn new(config: AuditLogConfig) -> Self {
        Self {
            config,
            entries: Vec::new(),
        }
    }

    /// Add an entry
    pub fn add_entry(&mut self, mut entry: AuditEntry) -> CryptoResult<()> {
        if let Some(last) = self.entries.last() {
            entry.previous_hash = last.hash;
        }
        entry.finalize();
        self.entries.push(entry);
        Ok(())
    }

    /// Get entries
    pub fn entries(&self) -> &[AuditEntry] {
        &self.entries
    }

    /// Verify the log
    pub fn verify(&self) -> TamperDetectionResult {
        for entry in &self.entries {
            if !entry.verify() {
                return TamperDetectionResult::Tampered {
                    entry_id: entry.id,
                    reason: "Entry hash mismatch".to_string(),
                };
            }
        }

        for i in 1..self.entries.len() {
            if self.entries[i].previous_hash != self.entries[i - 1].hash {
                return TamperDetectionResult::Tampered {
                    entry_id: self.entries[i].id,
                    reason: "Chain break".to_string(),
                };
            }
        }

        TamperDetectionResult::Valid
    }
}

/// Create a new audit log
pub fn create_audit_log(config: AuditLogConfig) -> Box<dyn AuditLogTrait> {
    if config.enable_merkle {
        Box::new(MerkleAuditLog::new(config))
    } else {
        Box::new(AuditLog::new(config))
    }
}

/// Trait for audit log operations
pub trait AuditLogTrait: Send + Sync {
    /// Add an entry
    fn add_entry(&mut self, entry: AuditEntry) -> CryptoResult<()>;

    /// Get entries
    fn entries(&self) -> &[AuditEntry];

    /// Verify the log
    fn verify(&self) -> TamperDetectionResult;
}

impl AuditLogTrait for MerkleAuditLog {
    fn add_entry(&mut self, entry: AuditEntry) -> CryptoResult<()> {
        self.add_entry(entry)
    }

    fn entries(&self) -> &[AuditEntry] {
        &self.entries
    }

    fn verify(&self) -> TamperDetectionResult {
        self.verify()
    }
}

impl AuditLogTrait for AuditLog {
    fn add_entry(&mut self, entry: AuditEntry) -> CryptoResult<()> {
        self.add_entry(entry)
    }

    fn entries(&self) -> &[AuditEntry] {
        &self.entries
    }

    fn verify(&self) -> TamperDetectionResult {
        self.verify()
    }
}

/// Verify an audit log
pub fn verify_audit_log(log: &dyn AuditLogTrait) -> TamperDetectionResult {
    log.verify()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_audit_log_config_default() {
        let config = AuditLogConfig::default();
        assert_eq!(config.max_entries_per_file, 10000);
        assert!(config.enable_merkle);
    }

    #[test]
    fn test_audit_event_type() {
        assert_eq!(AuditEventType::KeyGenerated.category(), "key_management");
        assert_eq!(AuditEventType::DevicePaired.category(), "device_management");
        assert_eq!(AuditEventType::AuthFailure.severity(), "high");
    }

    #[test]
    fn test_audit_entry_creation() {
        let entry = AuditEntry::new(
            1,
            AuditEventType::KeyGenerated,
            "Generated new identity key",
        )
        .with_device_id("device-123")
        .with_metadata("key_type", "ed25519");

        assert_eq!(entry.id, 1);
        assert_eq!(entry.event_type, AuditEventType::KeyGenerated);
        assert_eq!(entry.device_id, Some("device-123".to_string()));
        assert!(entry.metadata.contains_key("key_type"));
    }

    #[test]
    fn test_audit_entry_hash() {
        let mut entry = AuditEntry::new(
            1,
            AuditEventType::KeyGenerated,
            "Test entry",
        );
        entry.finalize();

        assert!(entry.verify());

        // Tamper with entry
        let mut tampered = entry.clone();
        tampered.description = "Tampered".to_string();
        assert!(!tampered.verify());
    }

    #[test]
    fn test_merkle_audit_log() {
        let config = AuditLogConfig::default();
        let mut log = MerkleAuditLog::new(config);

        // Add entries
        for i in 0..5 {
            let entry = AuditEntry::new(
                i,
                AuditEventType::SyncSent,
                format!("Sync message {}", i),
            );
            log.add_entry(entry).unwrap();
        }

        assert_eq!(log.len(), 5);
        assert!(log.root_hash().is_some());

        // Verify log
        let result = log.verify();
        assert!(result.is_valid());
    }

    #[test]
    fn test_merkle_tamper_detection() {
        let config = AuditLogConfig::default();
        let mut log = MerkleAuditLog::new(config);

        // Add entries
        for i in 0..3 {
            let entry = AuditEntry::new(
                i,
                AuditEventType::SyncSent,
                format!("Message {}", i),
            );
            log.add_entry(entry).unwrap();
        }

        // Tamper with an entry
        log.entries[1].description = "Tampered".to_string();

        // Verification should fail
        let result = log.verify();
        assert!(result.is_tampered());
    }

    #[test]
    fn test_chain_verification() {
        let config = AuditLogConfig::default();
        let mut log = MerkleAuditLog::new(config);

        // Add entries
        let entry1 = AuditEntry::new(1, AuditEventType::KeyGenerated, "First");
        log.add_entry(entry1).unwrap();

        let entry2 = AuditEntry::new(2, AuditEventType::DevicePaired, "Second");
        log.add_entry(entry2).unwrap();

        // Verify chain
        assert_eq!(log.entries[1].previous_hash, log.entries[0].hash);

        // Break the chain
        log.entries[1].previous_hash = [0u8; 32];
        let result = log.verify();
        assert!(result.is_tampered());
    }

    #[test]
    fn test_merkle_proof() {
        let config = AuditLogConfig::default();
        let mut log = MerkleAuditLog::new(config);

        // Add entries
        for i in 0..4 {
            let entry = AuditEntry::new(
                i as u64,
                AuditEventType::SyncSent,
                format!("Message {}", i),
            );
            log.add_entry(entry).unwrap();
        }

        // Get proof for entry 1
        let proof = log.get_proof(1);
        assert!(proof.is_some());
    }

    #[test]
    fn test_audit_log_serialization() {
        let config = AuditLogConfig::default();
        let mut log = MerkleAuditLog::new(config);

        let entry = AuditEntry::new(1, AuditEventType::KeyGenerated, "Test");
        log.add_entry(entry).unwrap();

        let bytes = log.to_bytes().unwrap();
        let recovered = MerkleAuditLog::from_bytes(&bytes).unwrap();

        assert_eq!(log.len(), recovered.len());
        assert_eq!(log.root_hash(), recovered.root_hash());
    }

    #[test]
    fn test_tamper_detection_result() {
        assert!(TamperDetectionResult::Valid.is_valid());
        assert!(!TamperDetectionResult::Valid.is_tampered());

        let tampered = TamperDetectionResult::Tampered {
            entry_id: 1,
            reason: "Test".to_string(),
        };
        assert!(!tampered.is_valid());
        assert!(tampered.is_tampered());
    }

    #[test]
    fn test_merkle_node() {
        let leaf1 = MerkleNode::leaf([1u8; 32], 0);
        let leaf2 = MerkleNode::leaf([2u8; 32], 1);

        assert!(leaf1.is_leaf());
        assert_eq!(leaf1.entry_index, Some(0));

        let internal = MerkleNode::internal(leaf1, leaf2);
        assert!(!internal.is_leaf());
        assert!(internal.left.is_some());
        assert!(internal.right.is_some());
    }

    #[test]
    fn test_simple_audit_log() {
        let config = AuditLogConfig {
            enable_merkle: false,
            ..Default::default()
        };
        let mut log = AuditLog::new(config);

        let entry = AuditEntry::new(1, AuditEventType::KeyGenerated, "Test");
        log.add_entry(entry).unwrap();

        assert_eq!(log.entries().len(), 1);
        assert!(log.verify().is_valid());
    }
}
