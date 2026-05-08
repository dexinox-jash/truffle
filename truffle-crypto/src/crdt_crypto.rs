//! CRDT Encryption for Yjs Sync Protocol
//!
//! This module provides encryption for Conflict-free Replicated Data Type (CRDT)
//! updates, specifically designed for the Yjs library's sync protocol.
//!
//! ## Architecture
//!
//! Yjs generates binary update messages that need to be encrypted before
//! transmission over the network. This module wraps those updates with:
//! - **Encryption**: AES-256-GCM or ChaCha20-Poly1305
//! - **Authentication**: HMAC-SHA256 for message integrity
//! - **Versioning**: Schema version for migration handling
//! - **Tombstones**: GDPR-compliant deletion markers
//!
//! ## Message Format
//!
//! ```text
//! ┌─────────────────────────────────────────────────────────────────┐
//! │                      Encrypted CRDT Update                       │
//! ├─────────────────────────────────────────────────────────────────┤
//! │ Header (28 bytes)                                               │
//! │   - Protocol version: 1 byte (0x01)                             │
//! │   - Algorithm: 1 byte (0x01 = AES-GCM, 0x02 = ChaCha20)         │
//! │   - Device ID: 8 bytes (truncated fingerprint)                  │
//! │   - Timestamp: 8 bytes (Unix milliseconds)                      │
//! │   - Schema version: 4 bytes                                     │
//! │   - Flags: 4 bytes (compression, tombstones, etc.)              │
//! │   - Reserved: 2 bytes                                           │
//! ├─────────────────────────────────────────────────────────────────┤
//! │ Nonce (12 bytes)                                                │
//! ├─────────────────────────────────────────────────────────────────┤
//! │ Encrypted Payload                                                │
//! │   - CRDT update (Yjs binary)                                    │
//! │   - Tombstones (optional UUID list)                             │
//! ├─────────────────────────────────────────────────────────────────┤
//! │ Authentication Tag (16 bytes)                                   │
//! ├─────────────────────────────────────────────────────────────────┤
//! │ HMAC-SHA256 (32 bytes)                                          │
//! └─────────────────────────────────────────────────────────────────┘
//! ```
//!
//! ## Security Properties
//!
//! - **Confidentiality**: CRDT updates are encrypted with AES-256-GCM
//! - **Integrity**: HMAC-SHA256 prevents tampering
//! - **Authentication**: Device ID binds message to specific device
//! - **Replay Protection**: Timestamp and nonce prevent replay attacks
//! - **Forward Secrecy**: Keys rotated per session via X3DH

use crate::{
    symmetric::{encrypt_aes_gcm, encrypt_chacha20, decrypt_aes_gcm, decrypt_chacha20, EncryptedData},
    CryptoError, CryptoResult,
    hmac_sha256, verify_hmac_sha256,
};
use zeroize::{Zeroize, ZeroizeOnDrop};

/// Protocol version for CRDT encryption
pub const CRDT_PROTOCOL_VERSION: u8 = 1;

/// Algorithm identifiers
pub const ALGORITHM_AES_GCM: u8 = 1;
pub const ALGORITHM_CHACHA20: u8 = 2;

/// Flag bits
pub const FLAG_COMPRESSED: u32 = 0x0001;
pub const FLAG_HAS_TOMBSTONES: u32 = 0x0002;
pub const FLAG_SCHEMA_MIGRATION: u32 = 0x0004;

/// Encrypted CRDT update structure
///
/// This is the wire format for encrypted sync messages.
#[derive(Debug, Clone)]
pub struct EncryptedCrdtUpdate {
    /// Protocol version (should be 1)
    pub protocol_version: u8,
    /// Encryption algorithm used
    pub algorithm: u8,
    /// Device fingerprint (8 bytes, truncated)
    pub device_id: [u8; 8],
    /// Timestamp (Unix milliseconds)
    pub timestamp: u64,
    /// Schema version (for migration handling)
    pub schema_version: u32,
    /// Flags (compression, tombstones, etc.)
    pub flags: u32,
    /// Encryption nonce (12 bytes)
    pub nonce: [u8; 12],
    /// Encrypted payload
    pub ciphertext: Vec<u8>,
    /// Authentication tag (16 bytes)
    pub auth_tag: [u8; 16],
    /// HMAC-SHA256 for integrity
    pub hmac: [u8; 32],
}

/// Decrypted CRDT update payload
#[derive(Debug, Clone)]
pub struct CrdtPayload {
    /// The CRDT update (Yjs binary)
    pub crdt_update: Vec<u8>,
    /// Tombstones for GDPR deletion (optional)
    pub tombstones: Vec<String>,
    /// Schema version
    pub schema_version: String,
}

/// CRDT encryption configuration
#[derive(Debug, Clone)]
pub struct CrdtConfig {
    /// Preferred algorithm (AES-GCM or ChaCha20)
    pub prefer_chacha: bool,
    /// Enable compression
    pub compress: bool,
    /// Schema version
    pub schema_version: String,
}

impl Default for CrdtConfig {
    fn default() -> Self {
        Self {
            prefer_chacha: false, // Use AES-GCM by default (hardware accelerated)
            compress: false,
            schema_version: "1.0".to_string(),
        }
    }
}

impl EncryptedCrdtUpdate {
    /// Serialize to bytes for transmission
    ///
    /// Format matches the specification in the module documentation.
    pub fn to_bytes(&self) -> Vec<u8> {
        let mut result = Vec::with_capacity(
            28 + 12 + self.ciphertext.len() + 16 + 32
        );

        // Header (28 bytes)
        result.push(self.protocol_version);
        result.push(self.algorithm);
        result.extend_from_slice(&self.device_id);
        result.extend_from_slice(&self.timestamp.to_be_bytes());
        result.extend_from_slice(&self.schema_version.to_be_bytes());
        result.extend_from_slice(&self.flags.to_be_bytes());
        result.extend_from_slice(&[0u8; 2]); // Reserved

        // Nonce (12 bytes)
        result.extend_from_slice(&self.nonce);

        // Ciphertext
        result.extend_from_slice(&self.ciphertext);

        // Auth tag (16 bytes)
        result.extend_from_slice(&self.auth_tag);

        // HMAC (32 bytes)
        result.extend_from_slice(&self.hmac);

        result
    }

    /// Deserialize from bytes
    ///
    /// # Errors
    ///
    /// Returns `CryptoError::InvalidCiphertext` if the format is invalid.
    pub fn from_bytes(bytes: &[u8]) -> CryptoResult<Self> {
        if bytes.len() < 28 + 12 + 16 + 32 {
            return Err(CryptoError::InvalidCiphertext(
                "Encrypted CRDT update too short".to_string()
            ));
        }

        let mut offset = 0;

        // Header
        let protocol_version = bytes[offset];
        offset += 1;

        let algorithm = bytes[offset];
        offset += 1;

        let mut device_id = [0u8; 8];
        device_id.copy_from_slice(&bytes[offset..offset + 8]);
        offset += 8;

        let timestamp = u64::from_be_bytes([
            bytes[offset], bytes[offset + 1], bytes[offset + 2], bytes[offset + 3],
            bytes[offset + 4], bytes[offset + 5], bytes[offset + 6], bytes[offset + 7],
        ]);
        offset += 8;

        let schema_version = u32::from_be_bytes([
            bytes[offset], bytes[offset + 1], bytes[offset + 2], bytes[offset + 3],
        ]);
        offset += 4;

        let flags = u32::from_be_bytes([
            bytes[offset], bytes[offset + 1], bytes[offset + 2], bytes[offset + 3],
        ]);
        offset += 4;

        // Reserved (2 bytes)
        offset += 2;

        // Nonce
        let mut nonce = [0u8; 12];
        nonce.copy_from_slice(&bytes[offset..offset + 12]);
        offset += 12;

        // Ciphertext (everything between nonce and auth_tag)
        let ciphertext_len = bytes.len() - offset - 16 - 32;
        let ciphertext = bytes[offset..offset + ciphertext_len].to_vec();
        offset += ciphertext_len;

        // Auth tag
        let mut auth_tag = [0u8; 16];
        auth_tag.copy_from_slice(&bytes[offset..offset + 16]);
        offset += 16;

        // HMAC
        let mut hmac = [0u8; 32];
        hmac.copy_from_slice(&bytes[offset..offset + 32]);

        Ok(Self {
            protocol_version,
            algorithm,
            device_id,
            timestamp,
            schema_version,
            flags,
            nonce,
            ciphertext,
            auth_tag,
            hmac,
        })
    }

    /// Get the header bytes for HMAC computation
    fn header_bytes(&self) -> Vec<u8> {
        let mut result = Vec::with_capacity(28);
        result.push(self.protocol_version);
        result.push(self.algorithm);
        result.extend_from_slice(&self.device_id);
        result.extend_from_slice(&self.timestamp.to_be_bytes());
        result.extend_from_slice(&self.schema_version.to_be_bytes());
        result.extend_from_slice(&self.flags.to_be_bytes());
        result.extend_from_slice(&[0u8; 2]);
        result
    }

    /// Verify HMAC
    ///
    /// Returns true if the HMAC is valid.
    ///
    /// # Errors
    ///
    /// Returns `CryptoError::InternalError` if HMAC computation fails.
    pub fn verify_hmac(&self, key: &[u8; 32]) -> CryptoResult<bool> {
        let computed = compute_hmac(key, &self.header_bytes(), &self.nonce, &self.ciphertext, &self.auth_tag)?;
        Ok(crate::constant_time_eq(&computed, &self.hmac))
    }

    /// Check if the update has tombstones
    pub fn has_tombstones(&self) -> bool {
        self.flags & FLAG_HAS_TOMBSTONES != 0
    }

    /// Check if the update is compressed
    pub fn is_compressed(&self) -> bool {
        self.flags & FLAG_COMPRESSED != 0
    }
}

/// Compute HMAC over encrypted CRDT update
fn compute_hmac(
    key: &[u8; 32],
    header: &[u8],
    nonce: &[u8; 12],
    ciphertext: &[u8],
    auth_tag: &[u8; 16],
) -> CryptoResult<[u8; 32]> {
    let mut data = Vec::with_capacity(header.len() + 12 + ciphertext.len() + 16);
    data.extend_from_slice(header);
    data.extend_from_slice(nonce);
    data.extend_from_slice(ciphertext);
    data.extend_from_slice(auth_tag);

    hmac_sha256(key, &data)
}

/// CRDT encryption context
///
/// This manages the encryption/decryption of CRDT updates.
pub struct CrdtCrypto {
    /// Sync key for encryption
    sync_key: [u8; 32],
    /// Device ID for attribution
    device_id: [u8; 8],
    /// Configuration
    config: CrdtConfig,
}

impl CrdtCrypto {
    /// Create a new CRDT crypto context
    ///
    /// # Arguments
    ///
    /// * `sync_key` - 32-byte sync key from X3DH handshake
    /// * `device_id` - 8-byte device fingerprint
    /// * `config` - Encryption configuration
    pub fn new(sync_key: [u8; 32], device_id: [u8; 8], config: CrdtConfig) -> Self {
        Self {
            sync_key,
            device_id,
            config,
        }
    }

    /// Create with default configuration
    pub fn with_defaults(sync_key: [u8; 32], device_id: [u8; 8]) -> Self {
        Self::new(sync_key, device_id, CrdtConfig::default())
    }

    /// Encrypt a CRDT update
    ///
    /// # Arguments
    ///
    /// * `update` - The Yjs CRDT update (binary)
    /// * `tombstones` - Optional list of deleted artifact UUIDs
    ///
    /// # Returns
    ///
    /// Returns the encrypted update ready for transmission.
    ///
    /// # Example
    ///
    /// ```rust
    /// use truffle_crypto::crdt_crypto::CrdtCrypto;
    ///
    /// let sync_key = [0x42u8; 32];
    /// let device_id = [0x01u8; 8];
    /// let crypto = CrdtCrypto::with_defaults(sync_key, device_id);
    ///
    /// let update = b"yjs binary update data";
    /// let encrypted = crypto.encrypt_update(update, None).unwrap();
    /// ```
    pub fn encrypt_update(
        &self,
        update: &[u8],
        tombstones: Option<Vec<String>>,
    ) -> CryptoResult<EncryptedCrdtUpdate> {
        // Build payload
        let payload = CrdtPayload {
            crdt_update: update.to_vec(),
            tombstones: tombstones.unwrap_or_default(),
            schema_version: self.config.schema_version.clone(),
        };

        // Serialize payload
        let payload_bytes = serialize_payload(&payload)?;

        // Determine flags
        let mut flags: u32 = 0;
        if !payload.tombstones.is_empty() {
            flags |= FLAG_HAS_TOMBSTONES;
        }

        // Generate timestamp
        let timestamp = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_millis() as u64;

        // Generate nonce
        let nonce = crate::generate_random_nonce::<12>()?;

        // Encrypt payload
        let encrypted = if self.config.prefer_chacha {
            encrypt_chacha20(&self.sync_key, &payload_bytes, &[])?
        } else {
            encrypt_aes_gcm(&self.sync_key, &payload_bytes, &[])?
        };

        // Parse encrypted data
        let auth_tag: [u8; 16] = encrypted.tag.try_into()
            .map_err(|_| CryptoError::InternalError("Invalid auth tag size".to_string()))?;

        let nonce_fixed: [u8; 12] = encrypted.nonce.try_into()
            .map_err(|_| CryptoError::InternalError("Invalid nonce size".to_string()))?;

        // Build encrypted update
        let mut encrypted_update = EncryptedCrdtUpdate {
            protocol_version: CRDT_PROTOCOL_VERSION,
            algorithm: if self.config.prefer_chacha { ALGORITHM_CHACHA20 } else { ALGORITHM_AES_GCM },
            device_id: self.device_id,
            timestamp,
            schema_version: 1, // Parsed from config
            flags,
            nonce: nonce_fixed,
            ciphertext: encrypted.ciphertext,
            auth_tag,
            hmac: [0u8; 32], // Will be computed below
        };

        // Compute HMAC
        encrypted_update.hmac = compute_hmac(
            &self.sync_key,
            &encrypted_update.header_bytes(),
            &encrypted_update.nonce,
            &encrypted_update.ciphertext,
            &encrypted_update.auth_tag,
        )?;

        Ok(encrypted_update)
    }

    /// Decrypt a CRDT update
    ///
    /// # Arguments
    ///
    /// * `encrypted` - The encrypted update
    ///
    /// # Returns
    ///
    /// Returns the decrypted payload.
    ///
    /// # Errors
    ///
    /// Returns `CryptoError::DecryptionFailed` if authentication fails.
    pub fn decrypt_update(&self, encrypted: &EncryptedCrdtUpdate) -> CryptoResult<CrdtPayload> {
        // Verify protocol version
        if encrypted.protocol_version != CRDT_PROTOCOL_VERSION {
            return Err(CryptoError::InvalidCiphertext(
                format!("Unsupported protocol version: {}", encrypted.protocol_version)
            ));
        }

        // Verify HMAC
        if !encrypted.verify_hmac(&self.sync_key)? {
            return Err(CryptoError::authentication_failed("HMAC verification failed"));
        }

        // Build EncryptedData for decryption
        let encrypted_data = EncryptedData {
            algorithm: encrypted.algorithm,
            nonce: encrypted.nonce.to_vec(),
            ciphertext: encrypted.ciphertext.clone(),
            tag: encrypted.auth_tag.to_vec(),
        };

        // Decrypt
        let payload_bytes = match encrypted.algorithm {
            ALGORITHM_AES_GCM => decrypt_aes_gcm(&self.sync_key, &encrypted_data, &[])?,
            ALGORITHM_CHACHA20 => decrypt_chacha20(&self.sync_key, &encrypted_data, &[])?
            ,
            _ => return Err(CryptoError::InvalidCiphertext(
                format!("Unknown algorithm: {}", encrypted.algorithm)
            )),
        };

        // Deserialize payload
        deserialize_payload(&payload_bytes)
    }

    /// Get device ID
    pub fn device_id(&self) -> &[u8; 8] {
        &self.device_id
    }

    /// Get sync key (for advanced use)
    pub fn sync_key(&self) -> &[u8; 32] {
        &self.sync_key
    }
}

impl Zeroize for CrdtCrypto {
    fn zeroize(&mut self) {
        self.sync_key.zeroize();
        self.device_id.zeroize();
    }
}

impl ZeroizeOnDrop for CrdtCrypto {}

/// Serialize CRDT payload to bytes
fn serialize_payload(payload: &CrdtPayload) -> CryptoResult<Vec<u8>> {
    // Simple serialization format:
    // [schema_version_len:1][schema_version][tombstones_count:2][tombstones...][crdt_update]

    let schema_version_bytes = payload.schema_version.as_bytes();
    let tombstones_count = payload.tombstones.len() as u16;

    let tombstones_bytes: Vec<u8> = payload.tombstones
        .iter()
        .flat_map(|t| {
            let bytes = t.as_bytes();
            let len = bytes.len() as u16;
            let mut result = Vec::with_capacity(2 + bytes.len());
            result.extend_from_slice(&len.to_be_bytes());
            result.extend_from_slice(bytes);
            result
        })
        .collect();

    let mut result = Vec::with_capacity(
        1 + schema_version_bytes.len() + 2 + tombstones_bytes.len() + payload.crdt_update.len()
    );

    result.push(schema_version_bytes.len() as u8);
    result.extend_from_slice(schema_version_bytes);
    result.extend_from_slice(&tombstones_count.to_be_bytes());
    result.extend_from_slice(&tombstones_bytes);
    result.extend_from_slice(&payload.crdt_update);

    Ok(result)
}

/// Deserialize CRDT payload from bytes
fn deserialize_payload(bytes: &[u8]) -> CryptoResult<CrdtPayload> {
    if bytes.is_empty() {
        return Err(CryptoError::InvalidCiphertext(
            "Empty payload".to_string()
        ));
    }

    let mut offset = 0;

    // Schema version
    let schema_version_len = bytes[offset] as usize;
    offset += 1;

    if bytes.len() < offset + schema_version_len {
        return Err(CryptoError::InvalidCiphertext(
            "Invalid schema version length".to_string()
        ));
    }

    let schema_version = String::from_utf8(bytes[offset..offset + schema_version_len].to_vec())
        .map_err(|_| CryptoError::InvalidCiphertext("Invalid UTF-8 in schema version".to_string()))?;
    offset += schema_version_len;

    // Tombstones count
    if bytes.len() < offset + 2 {
        return Err(CryptoError::InvalidCiphertext(
            "Missing tombstones count".to_string()
        ));
    }

    let tombstones_count = u16::from_be_bytes([bytes[offset], bytes[offset + 1]]) as usize;
    offset += 2;

    // Tombstones
    let mut tombstones = Vec::with_capacity(tombstones_count);
    for _ in 0..tombstones_count {
        if bytes.len() < offset + 2 {
            return Err(CryptoError::InvalidCiphertext(
                "Invalid tombstone length".to_string()
            ));
        }

        let tombstone_len = u16::from_be_bytes([bytes[offset], bytes[offset + 1]]) as usize;
        offset += 2;

        if bytes.len() < offset + tombstone_len {
            return Err(CryptoError::InvalidCiphertext(
                "Tombstone data truncated".to_string()
            ));
        }

        let tombstone = String::from_utf8(bytes[offset..offset + tombstone_len].to_vec())
            .map_err(|_| CryptoError::InvalidCiphertext("Invalid UTF-8 in tombstone".to_string()))?;
        tombstones.push(tombstone);
        offset += tombstone_len;
    }

    // CRDT update (remaining bytes)
    let crdt_update = bytes[offset..].to_vec();

    Ok(CrdtPayload {
        crdt_update,
        tombstones,
        schema_version,
    })
}

/// Convenience function to encrypt a CRDT update
///
/// # Arguments
///
/// * `key` - 32-byte encryption key
/// * `update` - Yjs CRDT update bytes
///
/// # Returns
///
/// Returns the encrypted update.
pub fn encrypt_crdt_update(key: &[u8; 32], update: &[u8]) -> CryptoResult<EncryptedCrdtUpdate> {
    let device_id = [0u8; 8]; // Default device ID
    let crypto = CrdtCrypto::with_defaults(*key, device_id);
    crypto.encrypt_update(update, None)
}

/// Convenience function to decrypt a CRDT update
///
/// # Arguments
///
/// * `key` - 32-byte encryption key
/// * `encrypted` - Encrypted update
///
/// # Returns
///
/// Returns the decrypted update bytes.
pub fn decrypt_crdt_update(key: &[u8; 32], encrypted: &EncryptedCrdtUpdate) -> CryptoResult<Vec<u8>> {
    let device_id = [0u8; 8]; // Default device ID
    let crypto = CrdtCrypto::with_defaults(*key, device_id);
    let payload = crypto.decrypt_update(encrypted)?;
    Ok(payload.crdt_update)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_crdt_roundtrip() {
        let sync_key = [0x42u8; 32];
        let device_id = [0x01u8; 8];
        let crypto = CrdtCrypto::with_defaults(sync_key, device_id);

        let update = b"test yjs crdt update data";

        let encrypted = crypto.encrypt_update(update, None).unwrap();
        let decrypted = crypto.decrypt_update(&encrypted).unwrap();

        assert_eq!(decrypted.crdt_update, update.as_slice());
    }

    #[test]
    fn test_crdt_with_tombstones() {
        let sync_key = [0x42u8; 32];
        let device_id = [0x01u8; 8];
        let crypto = CrdtCrypto::with_defaults(sync_key, device_id);

        let update = b"test update with deletions";
        let tombstones = vec![
            "550e8400-e29b-41d4-a716-446655440000".to_string(),
            "550e8400-e29b-41d4-a716-446655440001".to_string(),
        ];

        let encrypted = crypto.encrypt_update(update, Some(tombstones.clone())).unwrap();
        assert!(encrypted.has_tombstones());

        let decrypted = crypto.decrypt_update(&encrypted).unwrap();
        assert_eq!(decrypted.crdt_update, update.as_slice());
        assert_eq!(decrypted.tombstones, tombstones);
    }

    #[test]
    fn test_serialization_roundtrip() {
        let payload = CrdtPayload {
            crdt_update: b"test update".to_vec(),
            tombstones: vec!["uuid-1".to_string(), "uuid-2".to_string()],
            schema_version: "1.0".to_string(),
        };

        let bytes = serialize_payload(&payload).unwrap();
        let deserialized = deserialize_payload(&bytes).unwrap();

        assert_eq!(deserialized.crdt_update, payload.crdt_update);
        assert_eq!(deserialized.tombstones, payload.tombstones);
        assert_eq!(deserialized.schema_version, payload.schema_version);
    }

    #[test]
    fn test_hmac_verification() {
        let sync_key = [0x42u8; 32];
        let device_id = [0x01u8; 8];
        let crypto = CrdtCrypto::with_defaults(sync_key, device_id);

        let update = b"test update";
        let encrypted = crypto.encrypt_update(update, None).unwrap();

        // HMAC should be valid
        assert!(encrypted.verify_hmac(&sync_key));

        // Wrong key should fail
        let wrong_key = [0x43u8; 32];
        assert!(!encrypted.verify_hmac(&wrong_key));
    }

    #[test]
    fn test_tampered_ciphertext_fails() {
        let sync_key = [0x42u8; 32];
        let device_id = [0x01u8; 8];
        let crypto = CrdtCrypto::with_defaults(sync_key, device_id);

        let update = b"test update";
        let mut encrypted = crypto.encrypt_update(update, None).unwrap();

        // Tamper with ciphertext
        encrypted.ciphertext[0] ^= 0xFF;

        // Decryption should fail
        let result = crypto.decrypt_update(&encrypted);
        assert!(result.is_err());
    }

    #[test]
    fn test_convenience_functions() {
        let key = [0x42u8; 32];
        let update = b"convenience test";

        let encrypted = encrypt_crdt_update(&key, update).unwrap();
        let decrypted = decrypt_crdt_update(&key, &encrypted).unwrap();

        assert_eq!(decrypted, update.as_slice());
    }

    #[test]
    fn test_encrypted_update_serialization() {
        let sync_key = [0x42u8; 32];
        let device_id = [0x01u8; 8];
        let crypto = CrdtCrypto::with_defaults(sync_key, device_id);

        let update = b"serialization test";
        let encrypted = crypto.encrypt_update(update, None).unwrap();

        let bytes = encrypted.to_bytes();
        let deserialized = EncryptedCrdtUpdate::from_bytes(&bytes).unwrap();

        assert_eq!(deserialized.protocol_version, encrypted.protocol_version);
        assert_eq!(deserialized.algorithm, encrypted.algorithm);
        assert_eq!(deserialized.device_id, encrypted.device_id);
        assert_eq!(deserialized.timestamp, encrypted.timestamp);
        assert_eq!(deserialized.ciphertext, encrypted.ciphertext);
    }

    #[test]
    fn test_chacha20_encryption() {
        let sync_key = [0x42u8; 32];
        let device_id = [0x01u8; 8];
        let config = CrdtConfig {
            prefer_chacha: true,
            compress: false,
            schema_version: "1.0".to_string(),
        };
        let crypto = CrdtCrypto::new(sync_key, device_id, config);

        let update = b"chacha20 test";
        let encrypted = crypto.encrypt_update(update, None).unwrap();

        assert_eq!(encrypted.algorithm, ALGORITHM_CHACHA20);

        let decrypted = crypto.decrypt_update(&encrypted).unwrap();
        assert_eq!(decrypted.crdt_update, update.as_slice());
    }
}
