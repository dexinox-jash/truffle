//! Data Models for Truffle Core
//!
//! This module contains the core data structures that define the Truffle knowledge system:
//! - RawArtifact: Immutable capture of visual data (screenshots)
//! - WikiNode: Compiled, structured knowledge entities
//! - Schema: Ruleset for AI compilation

pub mod raw_artifact;
pub mod wiki_node;
pub mod schema;
pub mod entity;
pub mod relationship;
pub mod decision;
pub mod action_item;
pub mod meeting;

pub use raw_artifact::{RawArtifact, RawArtifactMetadata, AppContext, IngestionStatus};
pub use wiki_node::{WikiNode, WikiNodeType, WikiLink, Provenance, TemporalVectors, PrivacyClassification, EncryptionStatus};
pub use schema::{SchemaRuleset, CompilationRule, Trigger, Action, PrivacyLevel as SchemaPrivacyLevel, SchemaConfig};
pub use entity::{Entity, EntityType, EntityMetadata, EntityAlias, AliasType, PrivacyLevel};
pub use relationship::{Relationship, RelationshipDirection, RelationshipType, relation_types};
pub use decision::{Decision, DecisionStatus};
pub use action_item::{ActionItem, Priority, ActionItemStatus};
pub use meeting::{Meeting, MeetingType};

use serde::{Deserialize, Serialize};
use uuid::Uuid;
use chrono::{DateTime, Utc};

/// Content-addressable UUID generator using SHA256
/// Creates deterministic UUIDs based on content hash
pub fn content_addressable_uuid(content: &[u8]) -> Uuid {
    use sha2::{Sha256, Digest};
    
    let mut hasher = Sha256::new();
    hasher.update(content);
    let result = hasher.finalize();
    
    // Use first 16 bytes of SHA256 hash for UUIDv5-like behavior
    let bytes: [u8; 16] = result[..16].try_into().expect("SHA256 produces 32 bytes");
    Uuid::from_bytes(bytes)
}

/// Device fingerprint for privacy-preserving device identification
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub struct DeviceFingerprint(String);

impl DeviceFingerprint {
    /// Create a device fingerprint from device info using HMAC-SHA256
    pub fn new(device_info: &str, salt: &[u8]) -> Self {
        use hmac::{Hmac, Mac};
        use sha2::Sha256;
        
        type HmacSha256 = Hmac<Sha256>;
        
        let mut mac = HmacSha256::new_from_slice(salt)
            .expect("HMAC can take key of any size");
        mac.update(device_info.as_bytes());
        let result = mac.finalize();
        let bytes = result.into_bytes();
        
        // Use first 16 bytes as hex string
        Self(hex::encode(&bytes[..16]))
    }
    
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl std::fmt::Display for DeviceFingerprint {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// Geohash for privacy-safe location (4 chars = ~20km precision)
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Geohash(String);

impl Geohash {
    pub fn new(latitude: f64, longitude: f64) -> Self {
        // 4-character geohash = ~20km precision (privacy-safe)
        Self(encode_geohash(latitude, longitude, 4))
    }
    
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

/// Simple geohash encoder (base32)
fn encode_geohash(lat: f64, lon: f64, precision: usize) -> String {
    const BASE32: &[u8] = b"0123456789bcdefghjkmnpqrstuvwxyz";
    
    let mut lat_range = (-90.0, 90.0);
    let mut lon_range = (-180.0, 180.0);
    let mut geohash = String::with_capacity(precision);
    let mut bits = 0u8;
    let mut bits_count = 0;
    let mut is_even = true;
    
    while geohash.len() < precision {
        if is_even {
            // Refine longitude
            let mid = (lon_range.0 + lon_range.1) / 2.0;
            if lon >= mid {
                bits = (bits << 1) | 1;
                lon_range.0 = mid;
            } else {
                bits <<= 1;
                lon_range.1 = mid;
            }
        } else {
            // Refine latitude
            let mid = (lat_range.0 + lat_range.1) / 2.0;
            if lat >= mid {
                bits = (bits << 1) | 1;
                lat_range.0 = mid;
            } else {
                bits <<= 1;
                lat_range.1 = mid;
            }
        }
        
        is_even = !is_even;
        bits_count += 1;
        
        if bits_count == 5 {
            geohash.push(BASE32[bits as usize] as char);
            bits = 0;
            bits_count = 0;
        }
    }
    
    geohash
}

/// CRDT Vector Clock for conflict resolution
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
pub struct VectorClock {
    /// Map of device_id -> Lamport timestamp
    pub clocks: std::collections::HashMap<String, u64>,
}

impl VectorClock {
    pub fn new() -> Self {
        Self {
            clocks: std::collections::HashMap::new(),
        }
    }
    
    /// Increment clock for a device
    pub fn increment(&mut self, device_id: &str) -> u64 {
        let entry = self.clocks.entry(device_id.to_string()).or_insert(0);
        *entry += 1;
        *entry
    }
    
    /// Merge two vector clocks (takes maximum of each entry)
    pub fn merge(&mut self, other: &VectorClock) {
        for (device_id, timestamp) in &other.clocks {
            let entry = self.clocks.entry(device_id.clone()).or_insert(0);
            *entry = (*entry).max(*timestamp);
        }
    }
    
    /// Compare two vector clocks
    /// Returns: Some(true) if self > other, Some(false) if self < other, None if concurrent
    pub fn compare(&self, other: &VectorClock) -> Option<bool> {
        let mut self_greater = false;
        let mut other_greater = false;
        
        // Check all keys from both clocks
        let all_keys: std::collections::HashSet<_> = self
            .clocks
            .keys()
            .chain(other.clocks.keys())
            .collect();
        
        for key in all_keys {
            let self_val = self.clocks.get(key).copied().unwrap_or(0);
            let other_val = other.clocks.get(key).copied().unwrap_or(0);
            
            if self_val > other_val {
                self_greater = true;
            } else if other_val > self_val {
                other_greater = true;
            }
        }
        
        match (self_greater, other_greater) {
            (true, false) => Some(true),
            (false, true) => Some(false),
            _ => None, // Equal or concurrent
        }
    }
}

/// Binary pointer to local filesystem storage
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct BlobPointer {
    /// Absolute path to the file (platform-specific)
    pub path: std::path::PathBuf,
    /// SHA256 hash of file content for integrity
    pub content_hash: String,
}

impl BlobPointer {
    pub fn new(path: std::path::PathBuf, content: &[u8]) -> Self {
        use sha2::{Sha256, Digest};
        
        let mut hasher = Sha256::new();
        hasher.update(content);
        let hash = hex::encode(hasher.finalize());
        
        Self {
            path,
            content_hash: hash,
        }
    }
    
    /// Verify file integrity
    pub fn verify(&self) -> anyhow::Result<bool> {
        use sha2::{Sha256, Digest};
        use std::io::Read;
        
        let mut file = std::fs::File::open(&self.path)?;
        let mut hasher = Sha256::new();
        let mut buffer = [0u8; 8192];
        
        loop {
            let bytes_read = file.read(&mut buffer)?;
            if bytes_read == 0 {
                break;
            }
            hasher.update(&buffer[..bytes_read]);
        }
        
        let computed_hash = hex::encode(hasher.finalize());
        Ok(computed_hash == self.content_hash)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_content_addressable_uuid() {
        let content1 = b"test content";
        let content2 = b"test content";
        let content3 = b"different content";
        
        let uuid1 = content_addressable_uuid(content1);
        let uuid2 = content_addressable_uuid(content2);
        let uuid3 = content_addressable_uuid(content3);
        
        assert_eq!(uuid1, uuid2, "Same content should produce same UUID");
        assert_ne!(uuid1, uuid3, "Different content should produce different UUID");
    }
    
    #[test]
    fn test_device_fingerprint() {
        let salt = b"test_salt";
        let fp1 = DeviceFingerprint::new("device123", salt);
        let fp2 = DeviceFingerprint::new("device123", salt);
        let fp3 = DeviceFingerprint::new("device456", salt);
        
        assert_eq!(fp1, fp2, "Same device info should produce same fingerprint");
        assert_ne!(fp1, fp3, "Different device info should produce different fingerprint");
    }
    
    #[test]
    fn test_geohash() {
        let geohash = Geohash::new(40.7128, -74.0060); // NYC
        assert_eq!(geohash.as_str().len(), 4);
        
        // Same approximate location should produce same geohash
        let geohash2 = Geohash::new(40.7129, -74.0061);
        assert_eq!(geohash.as_str(), geohash2.as_str());
    }
    
    #[test]
    fn test_vector_clock() {
        let mut clock1 = VectorClock::new();
        let mut clock2 = VectorClock::new();
        
        clock1.increment("device_a");
        clock1.increment("device_a");
        clock2.increment("device_b");
        
        // Concurrent - neither happens before the other
        assert_eq!(clock1.compare(&clock2), None);
        
        // Merge and verify
        clock1.merge(&clock2);
        assert_eq!(clock1.clocks.get("device_a"), Some(&2));
        assert_eq!(clock1.clocks.get("device_b"), Some(&1));
    }
}
