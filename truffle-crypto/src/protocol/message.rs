//! Sync Message Structure
//!
//! This module implements the SyncMessage structure as defined in
//! Section 2.3.3 of the Truffle specification.
//!
//! ```text
//! SyncMessage {
//!     header: {
//!         protocol_version: 1;
//!         device_id: DeviceFingerprint;
//!         timestamp: UnixMs;
//!         nonce: Uint8Array[12];
//!     };
//!     payload: AES256GCM_Encrypted({
//!         crdt_update: Uint8Array; // Yjs binary diff
//!         schema_version: string; // For migration handling
//!         deleted_artifacts?: UUIDv4[]; // Tombstones for GDPR deletion
//!     });
//!     mac: HMAC_SHA256; // Authenticated encryption
//! }
//! ```

use crate::error::{CryptoError, CryptoResult};
use crate::types::{
    CryptoTimestamp, DeviceId, EncryptedBlob, MessageType, Nonce, PrivacyClassification,
    ProtocolVersion,
};
use crate::utils::{concat_bytes, hmac_sha256, sha3_256};
use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use zeroize::{Zeroize, ZeroizeOnDrop};

/// Maximum size of a CRDT update in bytes (512 KB)
const MAX_CRDT_UPDATE_SIZE: usize = 512 * 1024;

/// Maximum schema version string length
const MAX_SCHEMA_VERSION_LEN: usize = 32;

/// Sync message header containing metadata
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct SyncMessageHeader {
    /// Protocol version
    pub protocol_version: u8,
    /// Device that sent this message
    pub device_id: DeviceId,
    /// Timestamp when message was created
    pub timestamp: u64,
    /// Unique nonce for this message
    #[serde(with = "serde_bytes")]
    pub nonce: [u8; 12],
    /// Message type
    pub message_type: u8,
    /// Sequence number for ordering
    pub sequence_number: u64,
}

impl SyncMessageHeader {
    /// Create a new message header
    pub fn new(
        device_id: DeviceId,
        message_type: MessageType,
        sequence_number: u64,
    ) -> CryptoResult<Self> {
        let nonce = Nonce::generate_random()?;
        Ok(Self {
            protocol_version: ProtocolVersion::CURRENT.as_u8(),
            device_id,
            timestamp: CryptoTimestamp::now().as_millis(),
            nonce: nonce.to_array(),
            message_type: message_type as u8,
            sequence_number,
        })
    }

    /// Serialize the header to bytes
    pub fn to_bytes(&self) -> CryptoResult<Vec<u8>> {
        bincode::serialize(self)
            .map_err(|e| CryptoError::serialization_error(format!("Header: {}", e)))
    }

    /// Deserialize header from bytes
    pub fn from_bytes(bytes: &[u8]) -> CryptoResult<Self> {
        bincode::deserialize(bytes)
            .map_err(|e| CryptoError::serialization_error(format!("Header: {}", e)))
    }

    /// Validate the header
    pub fn validate(&self) -> CryptoResult<()> {
        // Check protocol version
        if self.protocol_version != ProtocolVersion::CURRENT.as_u8() {
            return Err(CryptoError::ProtocolVersionMismatch {
                expected: ProtocolVersion::CURRENT.as_u8(),
                actual: self.protocol_version,
            });
        }

        // Check timestamp is reasonable (not in future, not too old)
        let now = CryptoTimestamp::now().as_millis();
        if self.timestamp > now + 60000 {
            // More than 1 minute in future
            return Err(CryptoError::invalid_message_format(
                "Timestamp is in the future",
            ));
        }
        if now - self.timestamp > 86400000 {
            // More than 24 hours old
            return Err(CryptoError::invalid_message_format("Timestamp is too old"));
        }

        // Check message type is valid
        if MessageType::from_u8(self.message_type).is_none() {
            return Err(CryptoError::invalid_message_format("Invalid message type"));
        }

        Ok(())
    }

    /// Get the message type
    pub fn message_type(&self) -> Option<MessageType> {
        MessageType::from_u8(self.message_type)
    }

    /// Compute a unique message ID for deduplication
    pub fn message_id(&self) -> [u8; 32] {
        let data = concat_bytes(&[
            self.device_id.as_bytes(),
            &self.timestamp.to_be_bytes(),
            &self.nonce,
            &self.sequence_number.to_be_bytes(),
        ]);
        sha3_256(&data)
    }
}

/// Content of a CRDT update message
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct CrdtUpdateContent {
    /// Yjs binary update data
    #[serde(with = "serde_bytes")]
    pub crdt_update: Vec<u8>,
    /// Schema version for migration handling
    pub schema_version: String,
    /// Tombstones for deleted artifacts (GDPR compliance)
    #[serde(default)]
    pub deleted_artifacts: Vec<TombstoneEntry>,
    /// Privacy classification of this update
    pub privacy_classification: u8,
}

impl CrdtUpdateContent {
    /// Create a new CRDT update content
    pub fn new(
        crdt_update: Vec<u8>,
        schema_version: String,
        privacy_classification: PrivacyClassification,
    ) -> CryptoResult<Self> {
        // Validate size limits
        if crdt_update.len() > MAX_CRDT_UPDATE_SIZE {
            return Err(CryptoError::invalid_message_format(
                "CRDT update exceeds maximum size",
            ));
        }

        if schema_version.len() > MAX_SCHEMA_VERSION_LEN {
            return Err(CryptoError::invalid_message_format(
                "Schema version string too long",
            ));
        }

        Ok(Self {
            crdt_update,
            schema_version,
            deleted_artifacts: Vec::new(),
            privacy_classification: privacy_classification as u8,
        })
    }

    /// Add a tombstone entry for GDPR deletion
    pub fn add_tombstone(&mut self, artifact_id: [u8; 16], deleted_at: u64) {
        self.deleted_artifacts.push(TombstoneEntry {
            artifact_id,
            deleted_at,
        });
    }

    /// Get privacy classification
    pub fn privacy_classification(&self) -> Option<PrivacyClassification> {
        PrivacyClassification::from_u8(self.privacy_classification)
    }

    /// Serialize to bytes
    pub fn to_bytes(&self) -> CryptoResult<Vec<u8>> {
        bincode::serialize(self)
            .map_err(|e| CryptoError::serialization_error(format!("CrdtContent: {}", e)))
    }

    /// Deserialize from bytes
    pub fn from_bytes(bytes: &[u8]) -> CryptoResult<Self> {
        bincode::deserialize(bytes)
            .map_err(|e| CryptoError::serialization_error(format!("CrdtContent: {}", e)))
    }
}

/// Tombstone entry for GDPR deletion tracking
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct TombstoneEntry {
    /// ID of the deleted artifact
    #[serde(with = "serde_bytes")]
    pub artifact_id: [u8; 16],
    /// Timestamp when deletion occurred
    pub deleted_at: u64,
}

/// Payload of a sync message (encrypted)
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct SyncMessagePayload {
    /// Encrypted content
    #[serde(with = "serde_bytes")]
    pub encrypted_content: Vec<u8>,
    /// Authentication tag (for AEAD ciphers, included in ciphertext)
    #[serde(with = "serde_bytes")]
    pub auth_tag: Vec<u8>,
    /// Nonce used for encryption
    #[serde(with = "serde_bytes")]
    pub nonce: [u8; 12],
}

impl SyncMessagePayload {
    /// Create a new payload from encrypted content
    pub fn new(encrypted_content: Vec<u8>, nonce: [u8; 12]) -> Self {
        // For AEAD ciphers, auth tag is at the end of ciphertext
        let content_len = encrypted_content.len().saturating_sub(16);
        let auth_tag = encrypted_content[content_len..].to_vec();
        let content = encrypted_content[..content_len].to_vec();

        Self {
            encrypted_content: content,
            auth_tag,
            nonce,
        }
    }

    /// Get the full ciphertext including auth tag
    pub fn full_ciphertext(&self) -> Vec<u8> {
        let mut result = self.encrypted_content.clone();
        result.extend_from_slice(&self.auth_tag);
        result
    }

    /// Serialize to bytes
    pub fn to_bytes(&self) -> CryptoResult<Vec<u8>> {
        bincode::serialize(self)
            .map_err(|e| CryptoError::serialization_error(format!("Payload: {}", e)))
    }

    /// Deserialize from bytes
    pub fn from_bytes(bytes: &[u8]) -> CryptoResult<Self> {
        bincode::deserialize(bytes)
            .map_err(|e| CryptoError::serialization_error(format!("Payload: {}", e)))
    }
}

/// Encrypted payload wrapper with metadata
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct EncryptedPayload {
    /// Ciphertext
    #[serde(with = "serde_bytes")]
    pub ciphertext: Vec<u8>,
    /// Nonce
    #[serde(with = "serde_bytes")]
    pub nonce: [u8; 12],
    /// Encryption algorithm identifier
    pub algorithm: u8,
}

impl EncryptedPayload {
    /// Algorithm identifiers
    pub const ALGORITHM_AES256_GCM: u8 = 0x01;
    pub const ALGORITHM_CHACHA20_POLY1305: u8 = 0x02;

    /// Create a new encrypted payload
    pub fn new(ciphertext: Vec<u8>, nonce: [u8; 12], algorithm: u8) -> Self {
        Self {
            ciphertext,
            nonce,
            algorithm,
        }
    }

    /// Serialize to bytes
    pub fn to_bytes(&self) -> CryptoResult<Vec<u8>> {
        bincode::serialize(self)
            .map_err(|e| CryptoError::serialization_error(format!("EncryptedPayload: {}", e)))
    }

    /// Deserialize from bytes
    pub fn from_bytes(bytes: &[u8]) -> CryptoResult<Self> {
        bincode::deserialize(bytes)
            .map_err(|e| CryptoError::serialization_error(format!("EncryptedPayload: {}", e)))
    }
}

/// Message authenticator using HMAC-SHA256
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq, Zeroize, ZeroizeOnDrop)]
pub struct MessageAuthenticator {
    /// HMAC value
    #[serde(with = "serde_bytes")]
    pub hmac: [u8; 32],
}

impl MessageAuthenticator {
    /// Create a new authenticator by computing HMAC
    pub fn compute(key: &[u8], header: &SyncMessageHeader, payload: &EncryptedPayload) -> Self {
        let header_bytes = header.to_bytes().unwrap_or_default();
        let payload_bytes = payload.to_bytes().unwrap_or_default();
        let data = concat_bytes(&[&header_bytes, &payload_bytes]);
        let hmac = hmac_sha256(key, &data);
        Self { hmac }
    }

    /// Verify the authenticator
    pub fn verify(
        &self,
        key: &[u8],
        header: &SyncMessageHeader,
        payload: &EncryptedPayload,
    ) -> bool {
        let expected = Self::compute(key, header, payload);
        crate::utils::secure_compare(&self.hmac, &expected.hmac)
    }
}

/// Complete sync message structure
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct SyncMessage {
    /// Message header
    pub header: SyncMessageHeader,
    /// Encrypted payload
    pub payload: EncryptedPayload,
    /// Message authenticator (HMAC-SHA256)
    pub authenticator: MessageAuthenticator,
}

impl SyncMessage {
    /// Create a new sync message
    pub fn new(
        header: SyncMessageHeader,
        payload: EncryptedPayload,
        auth_key: &[u8],
    ) -> Self {
        let authenticator = MessageAuthenticator::compute(auth_key, &header, &payload);
        Self {
            header,
            payload,
            authenticator,
        }
    }

    /// Verify the message authenticator
    pub fn verify_authenticator(&self, auth_key: &[u8]) -> bool {
        self.authenticator.verify(auth_key, &self.header, &self.payload)
    }

    /// Validate the entire message
    pub fn validate(&self, auth_key: &[u8]) -> CryptoResult<()> {
        // Validate header
        self.header.validate()?;

        // Verify authenticator
        if !self.verify_authenticator(auth_key) {
            return Err(CryptoError::authentication_failed(
                "Message authenticator verification failed",
            ));
        }

        Ok(())
    }

    /// Serialize to bytes for transmission
    pub fn to_bytes(&self) -> CryptoResult<Vec<u8>> {
        bincode::serialize(self)
            .map_err(|e| CryptoError::serialization_error(format!("SyncMessage: {}", e)))
    }

    /// Deserialize from bytes
    pub fn from_bytes(bytes: &[u8]) -> CryptoResult<Self> {
        bincode::deserialize(bytes)
            .map_err(|e| CryptoError::serialization_error(format!("SyncMessage: {}", e)))
    }

    /// Get the message ID for deduplication
    pub fn message_id(&self) -> [u8; 32] {
        self.header.message_id()
    }

    /// Get the message type
    pub fn message_type(&self) -> Option<MessageType> {
        self.header.message_type()
    }

    /// Check if this message contains tombstones (GDPR deletion)
    pub fn has_tombstones(&self) -> bool {
        // This would require decrypting the payload to know for sure
        // For now, we return false and let the caller decrypt
        false
    }
}

/// Builder for constructing sync messages
pub struct SyncMessageBuilder {
    device_id: Option<DeviceId>,
    message_type: Option<MessageType>,
    sequence_number: Option<u64>,
    content: Option<CrdtUpdateContent>,
}

impl SyncMessageBuilder {
    /// Create a new builder
    pub fn new() -> Self {
        Self {
            device_id: None,
            message_type: None,
            sequence_number: None,
            content: None,
        }
    }

    /// Set the device ID
    pub fn device_id(mut self, device_id: DeviceId) -> Self {
        self.device_id = Some(device_id);
        self
    }

    /// Set the message type
    pub fn message_type(mut self, message_type: MessageType) -> Self {
        self.message_type = Some(message_type);
        self
    }

    /// Set the sequence number
    pub fn sequence_number(mut self, sequence_number: u64) -> Self {
        self.sequence_number = Some(sequence_number);
        self
    }

    /// Set the content
    pub fn content(mut self, content: CrdtUpdateContent) -> Self {
        self.content = Some(content);
        self
    }

    /// Build the header
    pub fn build_header(&self) -> CryptoResult<SyncMessageHeader> {
        let device_id = self.device_id.ok_or_else(|| {
            CryptoError::invalid_message_format("Device ID not set")
        })?;
        let message_type = self.message_type.ok_or_else(|| {
            CryptoError::invalid_message_format("Message type not set")
        })?;
        let sequence_number = self.sequence_number.unwrap_or(0);

        SyncMessageHeader::new(device_id, message_type, sequence_number)
    }
}

impl Default for SyncMessageBuilder {
    fn default() -> Self {
        Self::new()
    }
}

/// Message deduplication cache
pub struct MessageDeduplicator {
    seen_ids: HashSet<[u8; 32]>,
    max_size: usize,
}

impl MessageDeduplicator {
    /// Create a new deduplicator with the given max size
    pub fn new(max_size: usize) -> Self {
        Self {
            seen_ids: HashSet::with_capacity(max_size),
            max_size,
        }
    }

    /// Check if a message ID has been seen
    pub fn has_seen(&self, message_id: &[u8; 32]) -> bool {
        self.seen_ids.contains(message_id)
    }

    /// Mark a message ID as seen
    pub fn mark_seen(&mut self, message_id: [u8; 32]) {
        // If at capacity, clear and start fresh (simple strategy)
        if self.seen_ids.len() >= self.max_size {
            self.seen_ids.clear();
        }
        self.seen_ids.insert(message_id);
    }

    /// Clear all seen message IDs
    pub fn clear(&mut self) {
        self.seen_ids.clear();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sync_message_header() {
        let device_id = DeviceId::generate_random().unwrap();
        let header = SyncMessageHeader::new(device_id, MessageType::CrdtUpdate, 1).unwrap();

        assert_eq!(header.protocol_version, ProtocolVersion::CURRENT.as_u8());
        assert_eq!(header.message_type, MessageType::CrdtUpdate as u8);
        assert_eq!(header.sequence_number, 1);
        assert!(header.validate().is_ok());
    }

    #[test]
    fn test_crdt_update_content() {
        let content = CrdtUpdateContent::new(
            vec![1, 2, 3, 4, 5],
            "1.0.0".to_string(),
            PrivacyClassification::Personal,
        )
        .unwrap();

        assert_eq!(content.crdt_update, vec![1, 2, 3, 4, 5]);
        assert_eq!(content.schema_version, "1.0.0");
        assert!(content.deleted_artifacts.is_empty());
    }

    #[test]
    fn test_tombstone_entry() {
        let mut content = CrdtUpdateContent::new(
            vec![1, 2, 3],
            "1.0.0".to_string(),
            PrivacyClassification::Personal,
        )
        .unwrap();

        content.add_tombstone([0u8; 16], 1234567890);
        assert_eq!(content.deleted_artifacts.len(), 1);
    }

    #[test]
    fn test_message_authenticator() {
        let key = b"test authentication key";
        let device_id = DeviceId::generate_random().unwrap();
        let header = SyncMessageHeader::new(device_id, MessageType::CrdtUpdate, 1).unwrap();
        let payload = EncryptedPayload::new(vec![1, 2, 3], [0u8; 12], EncryptedPayload::ALGORITHM_AES256_GCM);

        let auth = MessageAuthenticator::compute(key, &header, &payload);
        assert!(auth.verify(key, &header, &payload));
        assert!(!auth.verify(b"wrong key", &header, &payload));
    }

    #[test]
    fn test_sync_message_roundtrip() {
        let device_id = DeviceId::generate_random().unwrap();
        let header = SyncMessageHeader::new(device_id, MessageType::CrdtUpdate, 1).unwrap();
        let payload = EncryptedPayload::new(vec![1, 2, 3], [0u8; 12], EncryptedPayload::ALGORITHM_AES256_GCM);
        let auth_key = b"test auth key";

        let message = SyncMessage::new(header, payload, auth_key);
        let bytes = message.to_bytes().unwrap();
        let recovered = SyncMessage::from_bytes(&bytes).unwrap();

        assert_eq!(message.header.device_id, recovered.header.device_id);
        assert!(recovered.verify_authenticator(auth_key));
    }

    #[test]
    fn test_message_deduplicator() {
        let mut dedup = MessageDeduplicator::new(100);
        let id = [1u8; 32];

        assert!(!dedup.has_seen(&id));
        dedup.mark_seen(id);
        assert!(dedup.has_seen(&id));
    }

    #[test]
    fn test_header_validation_future_timestamp() {
        let device_id = DeviceId::generate_random().unwrap();
        let mut header = SyncMessageHeader::new(device_id, MessageType::CrdtUpdate, 1).unwrap();
        header.timestamp = CryptoTimestamp::now().as_millis() + 120000; // 2 minutes in future

        assert!(header.validate().is_err());
    }
}
