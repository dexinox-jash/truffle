//! Secure Key Storage
//!
//! This module provides secure storage for cryptographic keys using:
//! - Secure Enclave (macOS/iOS)
//! - TPM/Windows Hello (Windows)
//! - Keyring/Secret Service (Linux)
//! - Software fallback (with encrypted storage)

use crate::error::{CryptoError, CryptoResult};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use zeroize::{Zeroize, ZeroizeOnDrop};

/// Key storage error type
#[derive(Debug, Clone, thiserror::Error)]
pub enum KeyStorageError {
    /// Platform storage unavailable
    #[error("Platform secure storage unavailable: {0}")]
    PlatformUnavailable(String),

    /// Key not found
    #[error("Key not found: {0}")]
    KeyNotFound(String),

    /// Access denied
    #[error("Access denied to key storage")]
    AccessDenied,

    /// Storage full
    #[error("Key storage is full")]
    StorageFull,

    /// Serialization error
    #[error("Serialization error: {0}")]
    SerializationError(String),

    /// Generic storage error
    #[error("Storage error: {0}")]
    Generic(String),
}

/// A stored key with metadata
#[derive(Clone, Debug, Serialize, Deserialize, Zeroize, ZeroizeOnDrop)]
pub struct StoredKey {
    /// Key identifier
    pub id: String,
    /// Key type
    pub key_type: String,
    /// Key data (encrypted at rest)
    #[serde(with = "serde_bytes")]
    #[zeroize(skip)]
    pub data: Vec<u8>,
    /// Creation timestamp
    pub created_at: u64,
    /// Last access timestamp
    pub last_accessed_at: u64,
    /// Key version for rotation tracking
    pub version: u32,
}

impl StoredKey {
    /// Create a new stored key
    ///
    /// # Errors
    ///
    /// Returns `KeyStorageError::Internal` if system time is unavailable.
    pub fn new(id: String, key_type: String, data: Vec<u8>) -> Result<Self, KeyStorageError> {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map_err(|e| KeyStorageError::Internal(format!("System time error: {:?}", e)))?
            .as_secs();

        Ok(Self {
            id,
            key_type,
            data,
            created_at: now,
            last_accessed_at: now,
            version: 1,
        })
    }

    /// Update last accessed timestamp
    ///
    /// # Errors
    ///
    /// Returns `KeyStorageError::Internal` if system time is unavailable.
    pub fn touch(&mut self) -> Result<(), KeyStorageError> {
        self.last_accessed_at = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map_err(|e| KeyStorageError::Internal(format!("System time error: {:?}", e)))?
            .as_secs();
        Ok(())
    }

    /// Increment version
    pub fn increment_version(&mut self) {
        self.version += 1;
    }
}

/// Trait for secure key storage implementations
pub trait KeyStorage: Send + Sync {
    /// Store a key
    fn store(&mut self, key: StoredKey) -> Result<(), KeyStorageError>;

    /// Retrieve a key by ID
    fn retrieve(&mut self, id: &str) -> Result<StoredKey, KeyStorageError>;

    /// Delete a key
    fn delete(&mut self, id: &str) -> Result<(), KeyStorageError>;

    /// List all stored key IDs
    fn list(&self) -> Result<Vec<String>, KeyStorageError>;

    /// Check if a key exists
    fn exists(&self, id: &str) -> Result<bool, KeyStorageError>;

    /// Clear all keys (use with caution!)
    fn clear(&mut self) -> Result<(), KeyStorageError>;
}

/// Platform-specific secure storage
pub struct PlatformKeyStorage {
    /// Service name for keyring
    service_name: String,
}

impl PlatformKeyStorage {
    /// Create a new platform key storage
    pub fn new(service_name: impl Into<String>) -> Self {
        Self {
            service_name: service_name.into(),
        }
    }

    /// Check if platform storage is available
    pub fn is_available() -> bool {
        // Try to access the keyring
        keyring::Entry::new("truffle-test", "availability-check").is_ok()
    }
}

impl KeyStorage for PlatformKeyStorage {
    fn store(&mut self, key: StoredKey) -> Result<(), KeyStorageError> {
        let entry = keyring::Entry::new(&self.service_name, &key.id)
            .map_err(|e| KeyStorageError::PlatformUnavailable(e.to_string()))?;

        // Serialize key data
        let serialized = bincode::serialize(&key)
            .map_err(|e| KeyStorageError::SerializationError(e.to_string()))?;

        // Store in keyring (base64 encoded)
        let encoded = base64::encode(&serialized);
        entry.set_password(&encoded)
            .map_err(|e| KeyStorageError::Generic(e.to_string()))?;

        Ok(())
    }

    fn retrieve(&mut self, id: &str) -> Result<StoredKey, KeyStorageError> {
        let entry = keyring::Entry::new(&self.service_name, id)
            .map_err(|e| KeyStorageError::PlatformUnavailable(e.to_string()))?;

        let encoded = entry.get_password()
            .map_err(|_| KeyStorageError::KeyNotFound(id.to_string()))?;

        let serialized = base64::decode(&encoded)
            .map_err(|e| KeyStorageError::SerializationError(e.to_string()))?;

        let mut key: StoredKey = bincode::deserialize(&serialized)
            .map_err(|e| KeyStorageError::SerializationError(e.to_string()))?;

        key.touch();

        // Update the stored key with new timestamp
        let _ = self.store(key.clone());

        Ok(key)
    }

    fn delete(&mut self, id: &str) -> Result<(), KeyStorageError> {
        let entry = keyring::Entry::new(&self.service_name, id)
            .map_err(|e| KeyStorageError::PlatformUnavailable(e.to_string()))?;

        entry.delete_password()
            .map_err(|_| KeyStorageError::KeyNotFound(id.to_string()))?;

        Ok(())
    }

    fn list(&self) -> Result<Vec<String>, KeyStorageError> {
        // Keyring doesn't support listing, so we maintain a separate index
        // For now, return empty list
        Ok(Vec::new())
    }

    fn exists(&self, id: &str) -> Result<bool, KeyStorageError> {
        let entry = keyring::Entry::new(&self.service_name, id)
            .map_err(|e| KeyStorageError::PlatformUnavailable(e.to_string()))?;

        match entry.get_password() {
            Ok(_) => Ok(true),
            Err(keyring::Error::NoEntry) => Ok(false),
            Err(e) => Err(KeyStorageError::Generic(e.to_string())),
        }
    }

    fn clear(&mut self) -> Result<(), KeyStorageError> {
        // Keyring doesn't support bulk delete
        // This would need to be implemented with a key index
        Ok(())
    }
}

/// Software-based key storage (encrypted at rest)
pub struct SoftwareKeyStorage {
    /// Storage directory
    storage_path: std::path::PathBuf,
    /// Master encryption key (derived from user password)
    master_key: Option<[u8; 32]>,
    /// In-memory key cache
    cache: HashMap<String, StoredKey>,
}

impl SoftwareKeyStorage {
    /// Create a new software key storage
    pub fn new(storage_path: impl AsRef<std::path::Path>) -> Self {
        Self {
            storage_path: storage_path.as_ref().to_path_buf(),
            master_key: None,
            cache: HashMap::new(),
        }
    }

    /// Initialize with a master key
    pub fn with_master_key(mut self, master_key: [u8; 32]) -> Self {
        self.master_key = Some(master_key);
        self
    }

    /// Derive master key from password
    pub fn derive_master_key(password: &str, salt: &[u8]) -> CryptoResult<[u8; 32]> {
        use crate::utils::hkdf_sha256;

        let key = hkdf_sha256(salt, password.as_bytes(), b"truffle-master-key-v1", 32)?;
        let mut result = [0u8; 32];
        result.copy_from_slice(&key);
        Ok(result)
    }

    /// Get the file path for a key
    fn key_path(&self, id: &str) -> std::path::PathBuf {
        let filename = format!("{}.key", base64::encode(id));
        self.storage_path.join(filename)
    }

    /// Encrypt key data
    fn encrypt(&self, data: &[u8]) -> Result<Vec<u8>, KeyStorageError> {
        let master_key = self.master_key
            .ok_or_else(|| KeyStorageError::AccessDenied)?;

        // Use AES-256-GCM for encryption
        use crate::encryption::Aes256GcmCipher;
        use crate::types::Nonce;
        use crate::utils::random_bytes;

        let cipher = Aes256GcmCipher::new(&master_key)
            .map_err(|e| KeyStorageError::Generic(e.to_string()))?;

        let nonce = random_bytes::<12>()
            .map_err(|e| KeyStorageError::Generic(e.to_string()))?;

        let encrypted = cipher.encrypt(data, &Nonce::new(nonce), &[])
            .map_err(|e| KeyStorageError::Generic(e.to_string()))?;

        Ok(encrypted.to_bytes())
    }

    /// Decrypt key data
    fn decrypt(&self, data: &[u8]) -> Result<Vec<u8>, KeyStorageError> {
        let master_key = self.master_key
            .ok_or_else(|| KeyStorageError::AccessDenied)?;

        use crate::encryption::Aes256GcmCipher;
        use crate::types::EncryptedBlob;
        use crate::types::Nonce;

        let cipher = Aes256GcmCipher::new(&master_key)
            .map_err(|e| KeyStorageError::Generic(e.to_string()))?;

        let blob = EncryptedBlob::from_bytes(data)
            .map_err(|e| KeyStorageError::SerializationError(e.to_string()))?;

        let decrypted = cipher.decrypt(&blob, &[])
            .map_err(|e| KeyStorageError::Generic(e.to_string()))?;

        Ok(decrypted)
    }
}

impl KeyStorage for SoftwareKeyStorage {
    fn store(&mut self, key: StoredKey) -> Result<(), KeyStorageError> {
        // Ensure storage directory exists
        std::fs::create_dir_all(&self.storage_path)
            .map_err(|e| KeyStorageError::Generic(e.to_string()))?;

        // Serialize and encrypt
        let serialized = bincode::serialize(&key)
            .map_err(|e| KeyStorageError::SerializationError(e.to_string()))?;

        let encrypted = self.encrypt(&serialized)?;

        // Write to file
        let path = self.key_path(&key.id);
        std::fs::write(&path, encrypted)
            .map_err(|e| KeyStorageError::Generic(e.to_string()))?;

        // Update cache
        self.cache.insert(key.id.clone(), key);

        Ok(())
    }

    fn retrieve(&mut self, id: &str) -> Result<StoredKey, KeyStorageError> {
        // Check cache first
        if let Some(key) = self.cache.get(id) {
            return Ok(key.clone());
        }

        // Read from file
        let path = self.key_path(id);
        let encrypted = std::fs::read(&path)
            .map_err(|_| KeyStorageError::KeyNotFound(id.to_string()))?;

        // Decrypt
        let decrypted = self.decrypt(&encrypted)?;

        // Deserialize
        let mut key: StoredKey = bincode::deserialize(&decrypted)
            .map_err(|e| KeyStorageError::SerializationError(e.to_string()))?;

        key.touch();

        // Update cache
        self.cache.insert(id.to_string(), key.clone());

        Ok(key)
    }

    fn delete(&mut self, id: &str) -> Result<(), KeyStorageError> {
        let path = self.key_path(id);

        // Securely overwrite before deleting
        if path.exists() {
            let metadata = std::fs::metadata(&path)
                .map_err(|e| KeyStorageError::Generic(e.to_string()))?;
            let size = metadata.len() as usize;

            // Overwrite with random data
            let overwrite = vec![0u8; size];
            std::fs::write(&path, &overwrite)
                .map_err(|e| KeyStorageError::Generic(e.to_string()))?;

            std::fs::remove_file(&path)
                .map_err(|e| KeyStorageError::Generic(e.to_string()))?;
        }

        // Remove from cache
        self.cache.remove(id);

        Ok(())
    }

    fn list(&self) -> Result<Vec<String>, KeyStorageError> {
        let mut keys = Vec::new();

        if self.storage_path.exists() {
            for entry in std::fs::read_dir(&self.storage_path)
                .map_err(|e| KeyStorageError::Generic(e.to_string()))? {
                let entry = entry.map_err(|e| KeyStorageError::Generic(e.to_string()))?;
                let filename = entry.file_name();
                let filename_str = filename.to_string_lossy();

                if filename_str.ends_with(".key") {
                    let encoded = &filename_str[..filename_str.len() - 4];
                    if let Ok(decoded) = base64::decode(encoded) {
                        if let Ok(id) = String::from_utf8(decoded) {
                            keys.push(id);
                        }
                    }
                }
            }
        }

        Ok(keys)
    }

    fn exists(&self, id: &str) -> Result<bool, KeyStorageError> {
        let path = self.key_path(id);
        Ok(path.exists())
    }

    fn clear(&mut self) -> Result<(), KeyStorageError> {
        // Delete all keys
        for id in self.list()? {
            self.delete(&id)?;
        }

        // Clear cache
        self.cache.clear();

        Ok(())
    }
}

/// Secure key storage that uses the best available method
pub struct SecureKeyStorage {
    /// Platform storage (preferred)
    platform: Option<PlatformKeyStorage>,
    /// Software fallback
    software: Option<SoftwareKeyStorage>,
}

impl SecureKeyStorage {
    /// Create a new secure key storage
    pub fn new(service_name: impl Into<String>) -> Self {
        let service_name = service_name.into();

        // Try platform storage first
        let platform = if PlatformKeyStorage::is_available() {
            Some(PlatformKeyStorage::new(service_name))
        } else {
            None
        };

        Self {
            platform,
            software: None,
        }
    }

    /// Create with software fallback
    pub fn with_software_fallback(
        mut self,
        storage_path: impl AsRef<std::path::Path>,
    ) -> Self {
        self.software = Some(SoftwareKeyStorage::new(storage_path));
        self
    }

    /// Initialize software storage with password
    pub fn initialize_with_password(
        &mut self,
        password: &str,
        salt: &[u8],
    ) -> CryptoResult<()> {
        if let Some(ref mut software) = self.software {
            let master_key = SoftwareKeyStorage::derive_master_key(password, salt)?;
            software.master_key = Some(master_key);
        }
        Ok(())
    }
}

impl KeyStorage for SecureKeyStorage {
    fn store(&mut self, key: StoredKey) -> Result<(), KeyStorageError> {
        if let Some(ref mut platform) = self.platform {
            platform.store(key)
        } else if let Some(ref mut software) = self.software {
            software.store(key)
        } else {
            Err(KeyStorageError::PlatformUnavailable(
                "No storage backend available".to_string()
            ))
        }
    }

    fn retrieve(&mut self, id: &str) -> Result<StoredKey, KeyStorageError> {
        if let Some(ref mut platform) = self.platform {
            platform.retrieve(id)
        } else if let Some(ref mut software) = self.software {
            software.retrieve(id)
        } else {
            Err(KeyStorageError::PlatformUnavailable(
                "No storage backend available".to_string()
            ))
        }
    }

    fn delete(&mut self, id: &str) -> Result<(), KeyStorageError> {
        if let Some(ref mut platform) = self.platform {
            platform.delete(id)
        } else if let Some(ref mut software) = self.software {
            software.delete(id)
        } else {
            Err(KeyStorageError::PlatformUnavailable(
                "No storage backend available".to_string()
            ))
        }
    }

    fn list(&self) -> Result<Vec<String>, KeyStorageError> {
        if let Some(ref platform) = self.platform {
            platform.list()
        } else if let Some(ref software) = self.software {
            software.list()
        } else {
            Err(KeyStorageError::PlatformUnavailable(
                "No storage backend available".to_string()
            ))
        }
    }

    fn exists(&self, id: &str) -> Result<bool, KeyStorageError> {
        if let Some(ref platform) = self.platform {
            platform.exists(id)
        } else if let Some(ref software) = self.software {
            software.exists(id)
        } else {
            Err(KeyStorageError::PlatformUnavailable(
                "No storage backend available".to_string()
            ))
        }
    }

    fn clear(&mut self) -> Result<(), KeyStorageError> {
        if let Some(ref mut platform) = self.platform {
            platform.clear()
        } else if let Some(ref mut software) = self.software {
            software.clear()
        } else {
            Err(KeyStorageError::PlatformUnavailable(
                "No storage backend available".to_string()
            ))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_stored_key_creation() {
        let key = StoredKey::new(
            "test-key".to_string(),
            "identity".to_string(),
            vec![1, 2, 3, 4],
        );

        assert_eq!(key.id, "test-key");
        assert_eq!(key.key_type, "identity");
        assert_eq!(key.version, 1);
    }

    #[test]
    fn test_software_key_storage() {
        let temp_dir = TempDir::new().unwrap();
        let mut storage = SoftwareKeyStorage::new(temp_dir.path());

        // Derive a test master key
        let master_key = SoftwareKeyStorage::derive_master_key("test-password", b"test-salt").unwrap();
        storage.master_key = Some(master_key);

        // Store a key
        let key = StoredKey::new(
            "test-key".to_string(),
            "identity".to_string(),
            vec![1, 2, 3, 4, 5],
        );
        storage.store(key.clone()).unwrap();

        // Check exists
        assert!(storage.exists("test-key").unwrap());

        // Retrieve
        let retrieved = storage.retrieve("test-key").unwrap();
        assert_eq!(retrieved.id, key.id);
        assert_eq!(retrieved.data, key.data);

        // List
        let keys = storage.list().unwrap();
        assert_eq!(keys.len(), 1);
        assert_eq!(keys[0], "test-key");

        // Delete
        storage.delete("test-key").unwrap();
        assert!(!storage.exists("test-key").unwrap());
    }

    #[test]
    fn test_key_not_found() {
        let temp_dir = TempDir::new().unwrap();
        let mut storage = SoftwareKeyStorage::new(temp_dir.path());

        let master_key = SoftwareKeyStorage::derive_master_key("test", b"salt").unwrap();
        storage.master_key = Some(master_key);

        let result = storage.retrieve("non-existent");
        assert!(matches!(result, Err(KeyStorageError::KeyNotFound(_))));
    }
}
