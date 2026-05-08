//! Core types for the Truffle Crypto library
//!
//! This module defines the fundamental types used throughout the
//! cryptographic implementation. All types are designed with
//! security and zero-knowledge principles in mind.

use serde::{Deserialize, Serialize};
use std::fmt;
use zeroize::{Zeroize, ZeroizeOnDrop};

/// Device identifier (hashed fingerprint)
///
/// This is a privacy-preserving device identifier that does not
/// reveal any actual device information.
#[derive(
    Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Zeroize, ZeroizeOnDrop,
)]
pub struct DeviceId([u8; crate::DEVICE_FINGERPRINT_SIZE]);

impl DeviceId {
    /// Create a new device ID from raw bytes
    pub fn new(bytes: [u8; crate::DEVICE_FINGERPRINT_SIZE]) -> Self {
        Self(bytes)
    }

    /// Generate a random device ID (for testing)
    pub fn generate_random() -> CryptoResult<Self> {
        let mut bytes = [0u8; crate::DEVICE_FINGERPRINT_SIZE];
        getrandom::getrandom(&mut bytes)
            .map_err(|e| crate::error::CryptoError::RandomGenerationFailed(e.to_string()))?;
        Ok(Self(bytes))
    }

    /// Get the raw bytes
    pub fn as_bytes(&self) -> &[u8] {
        &self.0
    }

    /// Convert to hex string
    pub fn to_hex(&self) -> String {
        hex::encode(&self.0)
    }
}

impl fmt::Debug for DeviceId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "DeviceId({}...)", &self.to_hex()[..8])
    }
}

impl fmt::Display for DeviceId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.to_hex())
    }
}

/// Key fingerprint for identifying public keys
///
/// This is a truncated hash of a public key, used for
/// identification without revealing the full key.
#[derive(Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Zeroize, ZeroizeOnDrop)]
pub struct KeyFingerprint([u8; 16]);

impl KeyFingerprint {
    /// Create a new key fingerprint from raw bytes
    pub fn new(bytes: [u8; 16]) -> Self {
        Self(bytes)
    }

    /// Compute fingerprint from a public key
    pub fn from_public_key(key: &[u8]) -> Self {
        use sha2::{Digest, Sha256};
        let hash = Sha256::digest(key);
        let mut fingerprint = [0u8; 16];
        fingerprint.copy_from_slice(&hash[..16]);
        Self(fingerprint)
    }

    /// Get the raw bytes
    pub fn as_bytes(&self) -> &[u8] {
        &self.0
    }

    /// Convert to hex string
    pub fn to_hex(&self) -> String {
        hex::encode(&self.0)
    }
}

impl fmt::Debug for KeyFingerprint {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "KeyFingerprint({}...)", &self.to_hex()[..8])
    }
}

impl fmt::Display for KeyFingerprint {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.to_hex())
    }
}

/// Nonce for encryption operations
///
/// A 96-bit nonce used with AES-256-GCM and ChaCha20-Poly1305.
/// Each nonce MUST be unique for a given key.
#[derive(Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Zeroize, ZeroizeOnDrop)]
pub struct Nonce([u8; crate::AES256_NONCE_SIZE]);

impl Nonce {
    /// Create a new nonce from raw bytes
    pub fn new(bytes: [u8; crate::AES256_NONCE_SIZE]) -> Self {
        Self(bytes)
    }

    /// Generate a random nonce
    ///
    /// # Security
    ///
    /// This uses the OS CSPRNG. The probability of collision is
    /// negligible for 96-bit nonces when properly randomized.
    pub fn generate_random() -> CryptoResult<Self> {
        let mut bytes = [0u8; crate::AES256_NONCE_SIZE];
        getrandom::getrandom(&mut bytes)
            .map_err(|e| crate::error::CryptoError::RandomGenerationFailed(e.to_string()))?;
        Ok(Self(bytes))
    }

    /// Create a nonce from a counter (for deterministic testing)
    ///
    /// # Security
    ///
    /// This should NOT be used in production as it may lead to
    /// nonce reuse if the counter is not properly managed.
    #[cfg(test)]
    pub fn from_counter(counter: u64) -> Self {
        let mut bytes = [0u8; crate::AES256_NONCE_SIZE];
        bytes[4..12].copy_from_slice(&counter.to_be_bytes());
        Self(bytes)
    }

    /// Get the raw bytes
    pub fn as_bytes(&self) -> &[u8] {
        &self.0
    }

    /// Convert to a fixed-size array
    pub fn to_array(&self) -> [u8; crate::AES256_NONCE_SIZE] {
        self.0
    }
}

impl fmt::Debug for Nonce {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Nonce([REDACTED])")
    }
}

/// Encrypted blob with authentication tag
///
/// This type represents ciphertext that includes the authentication
/// tag for AEAD ciphers.
#[derive(Clone, PartialEq, Eq, Serialize, Deserialize, Zeroize, ZeroizeOnDrop)]
pub struct EncryptedBlob {
    /// The ciphertext (includes authentication tag for AEAD)
    ciphertext: Vec<u8>,
    /// The nonce used for encryption
    nonce: Nonce,
}

impl EncryptedBlob {
    /// Create a new encrypted blob
    pub fn new(ciphertext: Vec<u8>, nonce: Nonce) -> Self {
        Self { ciphertext, nonce }
    }

    /// Get the ciphertext (including auth tag)
    pub fn ciphertext(&self) -> &[u8] {
        &self.ciphertext
    }

    /// Get the nonce
    pub fn nonce(&self) -> &Nonce {
        &self.nonce
    }

    /// Get the total size in bytes
    pub fn len(&self) -> usize {
        self.ciphertext.len()
    }

    /// Check if empty
    pub fn is_empty(&self) -> bool {
        self.ciphertext.is_empty()
    }

    /// Serialize to bytes (ciphertext || nonce)
    pub fn to_bytes(&self) -> Vec<u8> {
        let mut result = self.ciphertext.clone();
        result.extend_from_slice(&self.nonce.0);
        result
    }

    /// Deserialize from bytes
    pub fn from_bytes(bytes: &[u8]) -> crate::error::CryptoResult<Self> {
        if bytes.len() < crate::AES256_NONCE_SIZE {
            return Err(crate::error::CryptoError::invalid_message_format(
                "Encrypted blob too short",
            ));
        }

        let ciphertext_len = bytes.len() - crate::AES256_NONCE_SIZE;
        let ciphertext = bytes[..ciphertext_len].to_vec();
        let mut nonce_bytes = [0u8; crate::AES256_NONCE_SIZE];
        nonce_bytes.copy_from_slice(&bytes[ciphertext_len..]);

        Ok(Self {
            ciphertext,
            nonce: Nonce::new(nonce_bytes),
        })
    }
}

impl fmt::Debug for EncryptedBlob {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "EncryptedBlob {{ len: {}, nonce: {:?} }}",
            self.ciphertext.len(),
            self.nonce
        )
    }
}

/// Timestamp for cryptographic operations
///
/// Uses Unix milliseconds for compact representation.
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct CryptoTimestamp(u64);

impl CryptoTimestamp {
    /// Create a new timestamp from Unix milliseconds
    pub fn new(millis: u64) -> Self {
        Self(millis)
    }

    /// Get the current timestamp
    pub fn now() -> Self {
        use std::time::{SystemTime, UNIX_EPOCH};
        let millis = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("System time before Unix epoch")
            .as_millis() as u64;
        Self(millis)
    }

    /// Get the Unix milliseconds
    pub fn as_millis(&self) -> u64 {
        self.0
    }

    /// Check if this timestamp is older than the given duration
    pub fn is_older_than(&self, duration_ms: u64) -> bool {
        let now = Self::now().0;
        now.saturating_sub(self.0) > duration_ms
    }
}

impl fmt::Debug for CryptoTimestamp {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "CryptoTimestamp({})", self.0)
    }
}

/// Protocol version identifier
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct ProtocolVersion(u8);

impl ProtocolVersion {
    /// Current protocol version
    pub const CURRENT: Self = Self(crate::ZKS1_PROTOCOL_VERSION);

    /// Create a new protocol version
    pub fn new(version: u8) -> Self {
        Self(version)
    }

    /// Get the version number
    pub fn as_u8(&self) -> u8 {
        self.0
    }

    /// Check if this version is supported
    pub fn is_supported(&self) -> bool {
        self.0 == crate::ZKS1_PROTOCOL_VERSION
    }
}

impl fmt::Debug for ProtocolVersion {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "ProtocolVersion({})", self.0)
    }
}

/// Message type for sync protocol
#[derive(Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[repr(u8)]
pub enum MessageType {
    /// CRDT update message
    CrdtUpdate = 0x01,
    /// Key rotation message
    KeyRotation = 0x02,
    /// Device removal message
    DeviceRemoval = 0x03,
    /// Schema migration message
    SchemaMigration = 0x04,
    /// Ping/keepalive
    Ping = 0x05,
    /// Acknowledgment
    Ack = 0x06,
}

impl MessageType {
    /// Convert from u8
    pub fn from_u8(value: u8) -> Option<Self> {
        match value {
            0x01 => Some(Self::CrdtUpdate),
            0x02 => Some(Self::KeyRotation),
            0x03 => Some(Self::DeviceRemoval),
            0x04 => Some(Self::SchemaMigration),
            0x05 => Some(Self::Ping),
            0x06 => Some(Self::Ack),
            _ => None,
        }
    }
}

impl fmt::Debug for MessageType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::CrdtUpdate => write!(f, "CrdtUpdate"),
            Self::KeyRotation => write!(f, "KeyRotation"),
            Self::DeviceRemoval => write!(f, "DeviceRemoval"),
            Self::SchemaMigration => write!(f, "SchemaMigration"),
            Self::Ping => write!(f, "Ping"),
            Self::Ack => write!(f, "Ack"),
        }
    }
}

/// Privacy classification for data
#[derive(Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[repr(u8)]
pub enum PrivacyClassification {
    /// Public data (no encryption required)
    Public = 0x00,
    /// Personal data (standard encryption)
    Personal = 0x01,
    /// Sensitive data (enhanced encryption)
    Sensitive = 0x02,
    /// Financial data (maximum encryption)
    Financial = 0x03,
}

impl PrivacyClassification {
    /// Convert from u8
    pub fn from_u8(value: u8) -> Option<Self> {
        match value {
            0x00 => Some(Self::Public),
            0x01 => Some(Self::Personal),
            0x02 => Some(Self::Sensitive),
            0x03 => Some(Self::Financial),
            _ => None,
        }
    }

    /// Check if encryption is required for this classification
    pub fn requires_encryption(&self) -> bool {
        !matches!(self, Self::Public)
    }
}

impl fmt::Debug for PrivacyClassification {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Public => write!(f, "Public"),
            Self::Personal => write!(f, "Personal"),
            Self::Sensitive => write!(f, "Sensitive"),
            Self::Financial => write!(f, "Financial"),
        }
    }
}

/// Key usage type for derivation
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum KeyUsage {
    /// Symmetric encryption key
    Encryption,
    /// Authentication/MAC key
    Authentication,
    /// Key derivation key
    KeyDerivation,
    /// Session key
    Session,
    /// Backup key
    Backup,
}

impl KeyUsage {
    /// Get the context string for HKDF
    pub fn context(&self) -> &'static [u8] {
        match self {
            Self::Encryption => b"truffle-crypto-encryption-v1",
            Self::Authentication => b"truffle-crypto-auth-v1",
            Self::KeyDerivation => b"truffle-crypto-kdf-v1",
            Self::Session => b"truffle-crypto-session-v1",
            Self::Backup => b"truffle-crypto-backup-v1",
        }
    }
}

impl fmt::Debug for KeyUsage {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Encryption => write!(f, "Encryption"),
            Self::Authentication => write!(f, "Authentication"),
            Self::KeyDerivation => write!(f, "KeyDerivation"),
            Self::Session => write!(f, "Session"),
            Self::Backup => write!(f, "Backup"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_device_id_generation() {
        let id1 = DeviceId::generate_random().unwrap();
        let id2 = DeviceId::generate_random().unwrap();
        assert_ne!(id1, id2);
    }

    #[test]
    fn test_key_fingerprint() {
        let key = b"test public key data";
        let fp = KeyFingerprint::from_public_key(key);
        assert_eq!(fp.as_bytes().len(), 16);
    }

    #[test]
    fn test_nonce_generation() {
        let nonce1 = Nonce::generate_random().unwrap();
        let nonce2 = Nonce::generate_random().unwrap();
        assert_ne!(nonce1, nonce2);
    }

    #[test]
    fn test_encrypted_blob_serialization() {
        let blob = EncryptedBlob::new(
            vec![1, 2, 3, 4, 5],
            Nonce::generate_random().unwrap(),
        );
        let bytes = blob.to_bytes();
        let recovered = EncryptedBlob::from_bytes(&bytes).unwrap();
        assert_eq!(blob.ciphertext, recovered.ciphertext);
    }

    #[test]
    fn test_timestamp_now() {
        let ts = CryptoTimestamp::now();
        assert!(ts.as_millis() > 0);
    }

    #[test]
    fn test_protocol_version() {
        assert!(ProtocolVersion::CURRENT.is_supported());
        assert!(!ProtocolVersion::new(99).is_supported());
    }

    #[test]
    fn test_message_type_roundtrip() {
        assert_eq!(MessageType::from_u8(0x01), Some(MessageType::CrdtUpdate));
        assert_eq!(MessageType::from_u8(0x99), None);
    }

    #[test]
    fn test_privacy_classification() {
        assert!(!PrivacyClassification::Public.requires_encryption());
        assert!(PrivacyClassification::Personal.requires_encryption());
        assert!(PrivacyClassification::Financial.requires_encryption());
    }
}
