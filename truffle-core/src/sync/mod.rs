//! Sync Module - CRDT-based synchronization
//!
//! Implements zero-knowledge sync using Yjs/Yrs CRDTs:
//! - YATA algorithm for conflict-free merging
//! - Encrypted delta generation
//! - Device pairing and key exchange
//!
//! Per Section 2.3 of the Enterprise Specification

pub mod crdt;
pub mod delta;
pub mod crypto;

pub use crdt::{CrdtDocument, CrdtManager, SyncDocument};
pub use delta::{DeltaGenerator, DeltaApplier, SyncDelta};
pub use crypto::{SyncCrypto, DeviceKeys, EncryptedPayload};

use crate::models::{WikiNode, VectorClock};
use uuid::Uuid;

/// Sync configuration
#[derive(Debug, Clone)]
pub struct SyncConfig {
    /// Device ID (hashed fingerprint)
    pub device_id: String,
    /// Enable encryption
    pub enable_encryption: bool,
    /// Sync interval (seconds)
    pub sync_interval_secs: u64,
    /// Maximum delta size (bytes)
    pub max_delta_size: usize,
    /// Enable compression
    pub enable_compression: bool,
}

impl Default for SyncConfig {
    fn default() -> Self {
        Self {
            device_id: Uuid::new_v4().to_string(),
            enable_encryption: true,
            sync_interval_secs: 300, // 5 minutes
            max_delta_size: 1024 * 1024, // 1MB
            enable_compression: true,
        }
    }
}

/// Sync state for a device
#[derive(Debug, Clone)]
pub struct DeviceSyncState {
    /// Device ID
    pub device_id: String,
    /// Last sync timestamp
    pub last_sync_at: Option<chrono::DateTime<chrono::Utc>>,
    /// Vector clock
    pub vector_clock: VectorClock,
    /// Pending changes count
    pub pending_changes: usize,
}

/// Sync message for transport
#[derive(Debug, Clone)]
pub struct SyncMessage {
    /// Protocol version
    pub protocol_version: u8,
    /// Sender device ID
    pub device_id: String,
    /// Timestamp
    pub timestamp: chrono::DateTime<chrono::Utc>,
    /// Nonce for encryption
    pub nonce: [u8; 12],
    /// Encrypted payload
    pub payload: EncryptedPayload,
    /// HMAC for authentication
    pub mac: Vec<u8>,
}

impl SyncMessage {
    pub const CURRENT_VERSION: u8 = 1;
    
    /// Create a new sync message
    pub fn new(device_id: String, payload: EncryptedPayload, nonce: [u8; 12]) -> Self {
        Self {
            protocol_version: Self::CURRENT_VERSION,
            device_id,
            timestamp: chrono::Utc::now(),
            nonce,
            payload,
            mac: Vec::new(),
        }
    }
    
    /// Calculate message size
    pub fn size_bytes(&self) -> usize {
        std::mem::size_of::<u8>() + // version
        self.device_id.len() +
        std::mem::size_of::<i64>() + // timestamp
        12 + // nonce
        self.payload.ciphertext.len() +
        self.mac.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_sync_config() {
        let config = SyncConfig::default();
        assert!(config.enable_encryption);
        assert_eq!(config.sync_interval_secs, 300);
    }
    
    #[test]
    fn test_sync_message() {
        let payload = EncryptedPayload {
            ciphertext: vec![1, 2, 3],
            aad: None,
        };
        let msg = SyncMessage::new("device1".to_string(), payload, [0u8; 12]);
        
        assert_eq!(msg.protocol_version, 1);
        assert_eq!(msg.device_id, "device1");
    }
}
