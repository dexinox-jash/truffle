//! Key Derivation Functions
//!
//! This module implements HKDF-SHA256 based key derivation for
//! deriving multiple keys from a single master secret.

use crate::error::{CryptoError, CryptoResult};
use crate::types::DeviceId;
use crate::utils::hkdf_sha256;
use zeroize::{Zeroize, ZeroizeOnDrop};

/// Key derivation path for hierarchical key derivation
///
/// Implements a BIP32-like path for deriving keys from a master secret.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct KeyDerivationPath {
    /// Path components
    components: Vec<u32>,
}

impl KeyDerivationPath {
    /// Create a new derivation path
    pub fn new(components: Vec<u32>) -> Self {
        Self { components }
    }

    /// Create a path from a string (e.g., "m/44'/0'/0'/0/0")
    pub fn from_string(path: &str) -> CryptoResult<Self> {
        let components: Result<Vec<u32>, _> = path
            .split('/')
            .filter(|s| !s.is_empty() && s != &"m")
            .map(|s| {
                let hardened = s.ends_with('\'');
                let num_str = s.trim_end_matches('\'');
                let num: u32 = num_str.parse()
                    .map_err(|_| CryptoError::invalid_key("Invalid path component"))?;
                Ok(if hardened { num | 0x80000000 } else { num })
            })
            .collect();

        Ok(Self::new(components?))
    }

    /// Create a device-specific path
    pub fn for_device(device_id: &DeviceId, purpose: u32) -> Self {
        let device_hash = crate::utils::sha2_256(device_id.as_bytes());
        let device_component = u32::from_be_bytes([device_hash[0], device_hash[1], device_hash[2], device_hash[3]]);

        Self::new(vec![purpose, device_component])
    }

    /// Create a sync key path
    pub fn sync(device_id: &DeviceId) -> Self {
        Self::for_device(device_id, 1)
    }

    /// Create a backup key path
    pub fn backup(device_id: &DeviceId) -> Self {
        Self::for_device(device_id, 2)
    }

    /// Create an encryption key path
    pub fn encryption(device_id: &DeviceId) -> Self {
        Self::for_device(device_id, 3)
    }

    /// Get the path components
    pub fn components(&self) -> &[u32] {
        &self.components
    }

    /// Serialize to bytes for HKDF info
    pub fn to_bytes(&self) -> Vec<u8> {
        let mut result = Vec::with_capacity(self.components.len() * 4);
        for component in &self.components {
            result.extend_from_slice(&component.to_be_bytes());
        }
        result
    }
}

impl std::fmt::Display for KeyDerivationPath {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "m")?;
        for component in &self.components {
            if component & 0x80000000 != 0 {
                write!(f, "/{}'", component & !0x80000000)?;
            } else {
                write!(f, "/{}", component)?;
            }
        }
        Ok(())
    }
}

/// Key hierarchy for organized key derivation
#[derive(Clone, Debug)]
pub struct KeyHierarchy {
    /// Master secret
    master_secret: [u8; 32],
    /// Salt for HKDF
    salt: [u8; 32],
}

impl KeyHierarchy {
    /// Create a new key hierarchy from a master secret
    pub fn new(master_secret: [u8; 32], salt: [u8; 32]) -> Self {
        Self {
            master_secret,
            salt,
        }
    }

    /// Generate a random hierarchy
    pub fn generate_random() -> CryptoResult<Self> {
        let master_secret = crate::utils::random_bytes::<32>()?;
        let salt = crate::utils::random_bytes::<32>()?;
        Ok(Self::new(master_secret, salt))
    }

    /// Derive a key at the given path
    pub fn derive_key(&self, path: &KeyDerivationPath, output_len: usize) -> CryptoResult<Vec<u8>> {
        let info = path.to_bytes();
        hkdf_sha256(&self.salt, &self.master_secret, &info, output_len)
    }

    /// Derive a 256-bit key at the given path
    pub fn derive_key_256(&self, path: &KeyDerivationPath) -> CryptoResult<[u8; 32]> {
        let key = self.derive_key(path, 32)?;
        let mut result = [0u8; 32];
        result.copy_from_slice(&key);
        Ok(result)
    }

    /// Derive a child hierarchy
    pub fn derive_child(&self, index: u32) -> CryptoResult<Self> {
        let path = KeyDerivationPath::new(vec![index]);
        let child_secret = self.derive_key_256(&path)?;
        let child_salt = self.derive_key(&KeyDerivationPath::new(vec![index, 0xFFFFFFFF]), 32)?;
        let mut salt = [0u8; 32];
        salt.copy_from_slice(&child_salt);

        Ok(Self::new(child_secret, salt))
    }

    /// Derive device-specific hierarchy
    pub fn derive_for_device(&self, device_id: &DeviceId) -> CryptoResult<Self> {
        let device_hash = crate::utils::sha2_256(device_id.as_bytes());
        let index = u32::from_be_bytes([device_hash[0], device_hash[1], device_hash[2], device_hash[3]]);
        self.derive_child(index)
    }
}

impl Zeroize for KeyHierarchy {
    fn zeroize(&mut self) {
        self.master_secret.zeroize();
        self.salt.zeroize();
    }
}

impl ZeroizeOnDrop for KeyHierarchy {}

/// Derive a sync encryption key from a shared secret
pub fn derive_encryption_key(shared_secret: &[u8], context: &[u8]) -> CryptoResult<[u8; 32]> {
    let info = crate::utils::concat_bytes(&[b"truffle-encryption-v1", context]);
    let key = hkdf_sha256(&[], shared_secret, &info, 32)?;
    let mut result = [0u8; 32];
    result.copy_from_slice(&key);
    Ok(result)
}

/// Derive an authentication key from a shared secret
pub fn derive_auth_key(shared_secret: &[u8], context: &[u8]) -> CryptoResult<[u8; 32]> {
    let info = crate::utils::concat_bytes(&[b"truffle-authentication-v1", context]);
    let key = hkdf_sha256(&[], shared_secret, &info, 32)?;
    let mut result = [0u8; 32];
    result.copy_from_slice(&key);
    Ok(result)
}

/// Derive session keys (encryption + authentication) from a shared secret
pub fn derive_session_keys(shared_secret: &[u8], context: &[u8]) -> CryptoResult<SessionKeys> {
    let encryption_key = derive_encryption_key(shared_secret, context)?;
    let auth_key = derive_auth_key(shared_secret, context)?;

    Ok(SessionKeys {
        encryption_key,
        auth_key,
    })
}

/// Derive a device-specific key
pub fn derive_device_key(
    master_secret: &[u8],
    device_id: &DeviceId,
    purpose: &str,
) -> CryptoResult<[u8; 32]> {
    let info = crate::utils::concat_bytes(&[
        b"truffle-device-key-v1",
        device_id.as_bytes(),
        purpose.as_bytes(),
    ]);
    let key = hkdf_sha256(&[], master_secret, &info, 32)?;
    let mut result = [0u8; 32];
    result.copy_from_slice(&key);
    Ok(result)
}

/// Derive a backup encryption key
pub fn derive_backup_key(master_secret: &[u8], backup_id: &[u8]) -> CryptoResult<[u8; 32]> {
    let info = crate::utils::concat_bytes(&[b"truffle-backup-v1", backup_id]);
    let key = hkdf_sha256(&[], master_secret, &info, 32)?;
    let mut result = [0u8; 32];
    result.copy_from_slice(&key);
    Ok(result)
}

/// Derive a recovery key from a mnemonic seed
pub fn derive_recovery_key(mnemonic_seed: &[u8]) -> CryptoResult<[u8; 32]> {
    let key = hkdf_sha256(&[], mnemonic_seed, b"truffle-recovery-v1", 32)?;
    let mut result = [0u8; 32];
    result.copy_from_slice(&key);
    Ok(result)
}

/// Session keys derived from a shared secret
#[derive(Clone, Zeroize, ZeroizeOnDrop)]
pub struct SessionKeys {
    /// Encryption key
    #[zeroize(skip)]
    pub encryption_key: [u8; 32],
    /// Authentication key
    #[zeroize(skip)]
    pub auth_key: [u8; 32],
}

impl SessionKeys {
    /// Create new session keys
    pub fn new(encryption_key: [u8; 32], auth_key: [u8; 32]) -> Self {
        Self {
            encryption_key,
            auth_key,
        }
    }

    /// Derive from a shared secret
    pub fn from_shared_secret(shared_secret: &[u8], context: &[u8]) -> CryptoResult<Self> {
        derive_session_keys(shared_secret, context)
    }
}

/// Key derivation parameters for different use cases
#[derive(Clone, Debug)]
pub struct DerivationParams {
    /// Salt for HKDF
    pub salt: Vec<u8>,
    /// Info/context for HKDF
    pub info: Vec<u8>,
    /// Output length
    pub output_len: usize,
}

impl DerivationParams {
    /// Create params for sync key derivation
    pub fn for_sync(device_id: &DeviceId) -> Self {
        Self {
            salt: vec![],
            info: crate::utils::concat_bytes(&[b"truffle-sync", device_id.as_bytes()]),
            output_len: 32,
        }
    }

    /// Create params for backup key derivation
    pub fn for_backup(backup_id: &[u8]) -> Self {
        Self {
            salt: vec![],
            info: crate::utils::concat_bytes(&[b"truffle-backup", backup_id]),
            output_len: 32,
        }
    }

    /// Create params for encryption key derivation
    pub fn for_encryption(context: &[u8]) -> Self {
        Self {
            salt: vec![],
            info: crate::utils::concat_bytes(&[b"truffle-encryption", context]),
            output_len: 32,
        }
    }

    /// Derive a key using these params
    pub fn derive(&self, ikm: &[u8]) -> CryptoResult<Vec<u8>> {
        hkdf_sha256(&self.salt, ikm, &self.info, self.output_len)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_key_derivation_path() {
        let path = KeyDerivationPath::from_string("m/44'/0'/0'/0/0").unwrap();
        assert_eq!(path.components().len(), 5);
        assert_eq!(path.components()[0], 44 | 0x80000000);

        let path_str = path.to_string();
        assert!(path_str.contains("44'"));
    }

    #[test]
    fn test_device_specific_path() {
        let device_id = DeviceId::generate_random().unwrap();
        let path = KeyDerivationPath::for_device(&device_id, 1);
        assert_eq!(path.components().len(), 2);
        assert_eq!(path.components()[0], 1);
    }

    #[test]
    fn test_key_hierarchy() {
        let hierarchy = KeyHierarchy::generate_random().unwrap();
        let path = KeyDerivationPath::new(vec![1, 2, 3]);

        let key1 = hierarchy.derive_key_256(&path).unwrap();
        let key2 = hierarchy.derive_key_256(&path).unwrap();
        assert_eq!(key1, key2);

        let different_path = KeyDerivationPath::new(vec![1, 2, 4]);
        let key3 = hierarchy.derive_key_256(&different_path).unwrap();
        assert_ne!(key1, key3);
    }

    #[test]
    fn test_derive_child_hierarchy() {
        let parent = KeyHierarchy::generate_random().unwrap();
        let child = parent.derive_child(1).unwrap();

        let path = KeyDerivationPath::new(vec![0]);
        let parent_key = parent.derive_key_256(&path).unwrap();
        let child_key = child.derive_key_256(&path).unwrap();

        assert_ne!(parent_key, child_key);
    }

    #[test]
    fn test_derive_encryption_key() {
        let shared_secret = b"test shared secret";
        let context = b"test context";

        let key1 = derive_encryption_key(shared_secret, context).unwrap();
        let key2 = derive_encryption_key(shared_secret, context).unwrap();
        assert_eq!(key1, key2);

        let key3 = derive_encryption_key(b"different", context).unwrap();
        assert_ne!(key1, key3);
    }

    #[test]
    fn test_derive_session_keys() {
        let shared_secret = b"test shared secret";
        let context = b"test context";

        let keys = derive_session_keys(shared_secret, context).unwrap();
        assert_eq!(keys.encryption_key.len(), 32);
        assert_eq!(keys.auth_key.len(), 32);
        assert_ne!(keys.encryption_key, keys.auth_key);
    }

    #[test]
    fn test_derive_device_key() {
        let master_secret = b"master secret";
        let device_id = DeviceId::generate_random().unwrap();

        let key1 = derive_device_key(master_secret, &device_id, "sync").unwrap();
        let key2 = derive_device_key(master_secret, &device_id, "sync").unwrap();
        assert_eq!(key1, key2);

        let key3 = derive_device_key(master_secret, &device_id, "backup").unwrap();
        assert_ne!(key1, key3);
    }

    #[test]
    fn test_derive_backup_key() {
        let master_secret = b"master secret";
        let backup_id = b"backup-001";

        let key1 = derive_backup_key(master_secret, backup_id).unwrap();
        let key2 = derive_backup_key(master_secret, backup_id).unwrap();
        assert_eq!(key1, key2);

        let key3 = derive_backup_key(master_secret, b"backup-002").unwrap();
        assert_ne!(key1, key3);
    }

    #[test]
    fn test_derivation_params() {
        let device_id = DeviceId::generate_random().unwrap();
        let params = DerivationParams::for_sync(&device_id);

        let ikm = b"input key material";
        let key1 = params.derive(ikm).unwrap();
        let key2 = params.derive(ikm).unwrap();
        assert_eq!(key1, key2);
    }
}
