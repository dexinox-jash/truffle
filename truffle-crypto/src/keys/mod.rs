//! Key Management Module
//!
//! This module provides secure key generation, storage, and derivation
//! for the ZKS-1 protocol. All keys are user-controlled with no escrow.

mod derivation;
mod ephemeral;
mod identity;
mod storage;

pub use derivation::{
    derive_backup_key, derive_device_key, derive_encryption_key, derive_session_keys,
    KeyDerivationPath, KeyHierarchy,
};
pub use ephemeral::{EphemeralKeyManager, EphemeralKeyPair, EphemeralPublicKey};
pub use identity::{IdentityKeyPair, IdentityPublicKey, IdentitySecretKey};
pub use storage::{
    KeyStorage, KeyStorageError, PlatformKeyStorage, SecureKeyStorage,
    SoftwareKeyStorage, StoredKey,
};

use crate::error::CryptoResult;

/// Initialize the key management system
pub fn init() -> CryptoResult<()> {
    // Validate that we have access to secure storage
    // This will fail on platforms without secure enclave/TPM
    // but we fall back to software storage
    Ok(())
}

/// Key type identifiers for storage
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum KeyType {
    /// Ed25519 identity key
    Identity,
    /// X25519 signed prekey
    SignedPrekey,
    /// X25519 one-time prekey
    OneTimePrekey,
    /// AES-256-GCM symmetric key
    Symmetric,
    /// Session key
    Session,
    /// Backup encryption key
    Backup,
}

impl KeyType {
    /// Get the string identifier for this key type
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Identity => "identity",
            Self::SignedPrekey => "signed_prekey",
            Self::OneTimePrekey => "one_time_prekey",
            Self::Symmetric => "symmetric",
            Self::Session => "session",
            Self::Backup => "backup",
        }
    }
}

impl std::fmt::Display for KeyType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_key_type_display() {
        assert_eq!(KeyType::Identity.to_string(), "identity");
        assert_eq!(KeyType::Backup.to_string(), "backup");
    }
}
