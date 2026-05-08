//! ZKS-1 Protocol Implementation
//!
//! This module implements the Zero-Knowledge Sync protocol (ZKS-1)
//! as specified in Section 2.3 of the Truffle specification.
//!
//! ## Protocol Overview
//!
//! ZKS-1 provides:
//! - End-to-end encryption using X3DH + Kyber-768 hybrid
//! - Forward secrecy via ephemeral keys
//! - Post-quantum resistance via Kyber-768
//! - CRDT synchronization with encrypted payloads
//!
//! ## Message Flow
//!
//! ```text
//! Device A                    Relay                    Device B
//!    |                          |                          |
//!    |-- X3DH Handshake ------>|                          |
//!    |                          |<-- X3DH Handshake ------|
//!    |                          |                          |
//!    |-- Encrypted CRDT ------>|                          |
//!    |                          |-- Encrypted CRDT ------->|
//!    |                          |                          |
//!    |<-- Ack -----------------|                          |
//!    |                          |<-- Ack ------------------|
//! ```

mod handshake;
mod message;
mod zks1;

pub use handshake::{
    X3dhHandshake, X3dhInitiator, X3dhResponder, X3dhResult, PreKeyBundle,
    KyberHybridResult, EphemeralKeyPair,
};
pub use message::{
    SyncMessage, SyncMessageHeader, SyncMessagePayload, EncryptedPayload,
    MessageAuthenticator, CrdtUpdateContent, TombstoneEntry,
};
pub use zks1::{
    Zks1Protocol, Zks1Session, SessionState, SyncConfig,
    DeviceTrustLevel, SyncStatistics,
};

use crate::error::CryptoResult;
use crate::types::{DeviceId, ProtocolVersion};

/// Maximum size of a sync message payload (1 MB)
pub const MAX_PAYLOAD_SIZE: usize = 1024 * 1024;

/// Maximum number of tombstones per message
pub const MAX_TOMBSTONES: usize = 1000;

/// Default sync interval in milliseconds
pub const DEFAULT_SYNC_INTERVAL_MS: u64 = 5000;

/// Validate that a protocol version is supported
pub fn validate_protocol_version(version: u8) -> CryptoResult<()> {
    if version != crate::ZKS1_PROTOCOL_VERSION {
        return Err(crate::error::CryptoError::ProtocolVersionMismatch {
            expected: crate::ZKS1_PROTOCOL_VERSION,
            actual: version,
        });
    }
    Ok(())
}

/// Compute message ID for deduplication
pub fn compute_message_id(
    device_id: &DeviceId,
    timestamp: u64,
    nonce: &[u8],
) -> [u8; 32] {
    use crate::utils::sha3_256;
    let data = crate::utils::concat_bytes(&[
        device_id.as_bytes(),
        &timestamp.to_be_bytes(),
        nonce,
    ]);
    sha3_256(&data)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validate_protocol_version() {
        assert!(validate_protocol_version(crate::ZKS1_PROTOCOL_VERSION).is_ok());
        assert!(validate_protocol_version(99).is_err());
    }

    #[test]
    fn test_compute_message_id() {
        let device_id = DeviceId::generate_random().unwrap();
        let timestamp = 1234567890u64;
        let nonce = b"test_nonce";

        let id1 = compute_message_id(&device_id, timestamp, nonce);
        let id2 = compute_message_id(&device_id, timestamp, nonce);
        assert_eq!(id1, id2);

        let id3 = compute_message_id(&device_id, timestamp + 1, nonce);
        assert_ne!(id1, id3);
    }
}
