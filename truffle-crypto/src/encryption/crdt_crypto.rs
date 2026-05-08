//! CRDT Encryption Wrapper
//!
//! This module provides encryption for Yjs CRDT updates, enabling
//! zero-knowledge synchronization of document state.
//!
//! The wrapper encrypts Yjs binary updates before transmission and
//! decrypts them upon receipt, ensuring the relay server cannot
//! access document content.

use crate::encryption::{Aes256GcmCipher, ChaCha20Poly1305Cipher, EncryptionCipher};
use crate::error::{CryptoError, CryptoResult};
use crate::types::{DeviceId, EncryptedBlob, Nonce, PrivacyClassification};
use crate::utils::{sha3_256, sha2_256};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use zeroize::{Zeroize, ZeroizeOnDrop};

/// Encrypted CRDT update
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct EncryptedCrdtUpdate {
    /// Update ID (hash of original update for deduplication)
    #[serde(with = "serde_bytes")]
    pub update_id: [u8; 32],
    /// Encrypted update data
    #[serde(with = "serde_bytes")]
    pub encrypted_data: Vec<u8>,
    /// Nonce used for encryption
    #[serde(with = "serde_bytes")]
    pub nonce: [u8; 12],
    /// Encryption algorithm used
    pub algorithm: u8,
    /// Document ID this update belongs to
    pub document_id: String,
    /// Timestamp
    pub timestamp: u64,
    /// Privacy classification
    pub privacy: u8,
    /// Schema version
    pub schema_version: String,
}

impl EncryptedCrdtUpdate {
    /// Create a new encrypted CRDT update
    pub fn new(
        update_id: [u8; 32],
        encrypted_data: Vec<u8>,
        nonce: [u8; 12],
        algorithm: u8,
        document_id: String,
        privacy: PrivacyClassification,
        schema_version: String,
    ) -> Self {
        let timestamp = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs();

        Self {
            update_id,
            encrypted_data,
            nonce,
            algorithm,
            document_id,
            timestamp,
            privacy: privacy as u8,
            schema_version,
        }
    }

    /// Get the privacy classification
    pub fn privacy_classification(&self) -> Option<PrivacyClassification> {
        PrivacyClassification::from_u8(self.privacy)
    }

    /// Serialize to bytes
    pub fn to_bytes(&self) -> CryptoResult<Vec<u8>> {
        bincode::serialize(self)
            .map_err(|e| CryptoError::serialization_error(format!("EncryptedCrdtUpdate: {}", e)))
    }

    /// Deserialize from bytes
    pub fn from_bytes(bytes: &[u8]) -> CryptoResult<Self> {
        bincode::deserialize(bytes)
            .map_err(|e| CryptoError::serialization_error(format!("EncryptedCrdtUpdate: {}", e)))
    }

    /// Get the size of the encrypted update
    pub fn size(&self) -> usize {
        self.encrypted_data.len()
    }
}

/// CRDT crypto wrapper for Yjs integration
pub struct CrdtCryptoWrapper {
    /// Encryption key for this document
    key: [u8; 32],
    /// Document ID
    document_id: String,
    /// Use ChaCha20-Poly1305 (mobile) or AES-256-GCM (desktop)
    use_chacha20: bool,
    /// Schema version
    schema_version: String,
    /// Privacy classification
    privacy: PrivacyClassification,
    /// Update ID cache for deduplication
    seen_updates: HashMap<[u8; 32], u64>,
    /// Maximum cache size
    max_cache_size: usize,
}

impl CrdtCryptoWrapper {
    /// Create a new CRDT crypto wrapper
    pub fn new(
        key: [u8; 32],
        document_id: String,
        use_chacha20: bool,
        schema_version: String,
        privacy: PrivacyClassification,
    ) -> Self {
        Self {
            key,
            document_id,
            use_chacha20,
            schema_version,
            privacy,
            seen_updates: HashMap::new(),
            max_cache_size: 10000,
        }
    }

    /// Create from a derived key
    pub fn from_derived_key(
        master_key: &[u8; 32],
        document_id: &str,
        use_chacha20: bool,
        schema_version: String,
        privacy: PrivacyClassification,
    ) -> CryptoResult<Self> {
        // Derive document-specific key
        let info = crate::utils::concat_bytes(&[
            b"truffle-crdt-key-v1",
            document_id.as_bytes(),
        ]);
        let derived = crate::utils::hkdf_sha256(&[], master_key, &info, 32)?;
        let mut key = [0u8; 32];
        key.copy_from_slice(&derived);

        Ok(Self::new(
            key,
            document_id.to_string(),
            use_chacha20,
            schema_version,
            privacy,
        ))
    }

    /// Encrypt a Yjs update
    ///
    /// # Arguments
    ///
    /// * `update` - Raw Yjs binary update
    ///
    /// # Returns
    ///
    /// Encrypted CRDT update ready for transmission
    pub fn encrypt(&mut self, update: &[u8]) -> CryptoResult<EncryptedCrdtUpdate> {
        // Compute update ID for deduplication
        let update_id = sha3_256(update);

        // Check if we've seen this update
        if self.seen_updates.contains_key(&update_id) {
            return Err(CryptoError::invalid_state("Update already processed"));
        }

        // Generate nonce
        let nonce = Nonce::generate_random()?;

        // Encrypt based on algorithm preference
        let (encrypted_data, algorithm) = if self.use_chacha20 {
            let cipher = ChaCha20Poly1305Cipher::new(&self.key)?;
            let blob = cipher.encrypt(update, &nonce, &[])?;
            (blob.to_bytes(), ChaCha20Poly1305Cipher::ALGORITHM_ID)
        } else {
            let cipher = Aes256GcmCipher::new(&self.key)?;
            let blob = cipher.encrypt(update, &nonce, &[])?;
            (blob.to_bytes(), Aes256GcmCipher::ALGORITHM_ID)
        };

        // Cache update ID
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs();
        self.add_to_cache(update_id, now);

        Ok(EncryptedCrdtUpdate::new(
            update_id,
            encrypted_data,
            nonce.to_array(),
            algorithm,
            self.document_id.clone(),
            self.privacy,
            self.schema_version.clone(),
        ))
    }

    /// Decrypt a Yjs update
    ///
    /// # Arguments
    ///
    /// * `encrypted` - Encrypted CRDT update
    ///
    /// # Returns
    ///
    /// Raw Yjs binary update
    pub fn decrypt(&mut self, encrypted: &EncryptedCrdtUpdate) -> CryptoResult<Vec<u8>> {
        // Verify document ID
        if encrypted.document_id != self.document_id {
            return Err(CryptoError::invalid_message_format(
                "Document ID mismatch"
            ));
        }

        // Check if we've seen this update
        if self.seen_updates.contains_key(&encrypted.update_id) {
            return Err(CryptoError::invalid_state("Update already processed"));
        }

        // Create cipher based on algorithm
        let nonce = Nonce::new(encrypted.nonce);
        let blob = EncryptedBlob::from_bytes(&encrypted.encrypted_data)?;

        let plaintext = match encrypted.algorithm {
            0x01 => {
                let cipher = Aes256GcmCipher::new(&self.key)?;
                cipher.decrypt(&blob, &[])?
            }
            0x02 => {
                let cipher = ChaCha20Poly1305Cipher::new(&self.key)?;
                cipher.decrypt(&blob, &[])?
            }
            _ => return Err(CryptoError::invalid_key("Unknown encryption algorithm")),
        };

        // Verify update ID (detect tampering)
        let computed_id = sha3_256(&plaintext);
        if computed_id != encrypted.update_id {
            return Err(CryptoError::authentication_failed(
                "Update ID mismatch - possible tampering"
            ));
        }

        // Cache update ID
        self.add_to_cache(encrypted.update_id, encrypted.timestamp);

        Ok(plaintext)
    }

    /// Add update ID to cache with cleanup
    fn add_to_cache(&mut self, update_id: [u8; 32], timestamp: u64) {
        // Cleanup if cache is full
        if self.seen_updates.len() >= self.max_cache_size {
            self.cleanup_cache();
        }

        self.seen_updates.insert(update_id, timestamp);
    }

    /// Clean up old entries from cache
    fn cleanup_cache(&mut self) {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs();

        // Remove entries older than 1 hour
        let cutoff = now - 3600;
        self.seen_updates.retain(|_, &mut ts| ts > cutoff);

        // If still too full, remove oldest entries
        if self.seen_updates.len() >= self.max_cache_size {
            let mut entries: Vec<_> = self.seen_updates.iter().collect();
            entries.sort_by_key(|&(_, ts)| ts);

            let to_remove = entries.len() - self.max_cache_size / 2;
            for (id, _) in entries.into_iter().take(to_remove) {
                self.seen_updates.remove(id);
            }
        }
    }

    /// Check if an update has been seen
    pub fn has_seen(&self, update_id: &[u8; 32]) -> bool {
        self.seen_updates.contains_key(update_id)
    }

    /// Get cache size
    pub fn cache_size(&self) -> usize {
        self.seen_updates.len()
    }

    /// Clear the cache
    pub fn clear_cache(&mut self) {
        self.seen_updates.clear();
    }

    /// Rotate the encryption key
    pub fn rotate_key(&mut self, new_key: [u8; 32]) {
        self.key = new_key;
        self.clear_cache();
    }

    /// Get document ID
    pub fn document_id(&self) -> &str {
        &self.document_id
    }

    /// Get schema version
    pub fn schema_version(&self) -> &str {
        &self.schema_version
    }

    /// Update schema version
    pub fn set_schema_version(&mut self, version: String) {
        self.schema_version = version;
    }
}

impl Zeroize for CrdtCryptoWrapper {
    fn zeroize(&mut self) {
        self.key.zeroize();
    }
}

impl ZeroizeOnDrop for CrdtCryptoWrapper {}

/// Batch encrypt multiple updates
pub fn batch_encrypt(
    wrapper: &mut CrdtCryptoWrapper,
    updates: Vec<Vec<u8>>,
) -> CryptoResult<Vec<EncryptedCrdtUpdate>> {
    updates.into_iter()
        .map(|update| wrapper.encrypt(&update))
        .collect()
}

/// Batch decrypt multiple updates
pub fn batch_decrypt(
    wrapper: &mut CrdtCryptoWrapper,
    encrypted: Vec<EncryptedCrdtUpdate>,
) -> CryptoResult<Vec<Vec<u8>>> {
    encrypted.into_iter()
        .map(|e| wrapper.decrypt(&e))
        .collect()
}

/// Document key manager for multiple documents
pub struct DocumentKeyManager {
    /// Master key
    master_key: [u8; 32],
    /// Document wrappers
    wrappers: HashMap<String, CrdtCryptoWrapper>,
    /// Default schema version
    default_schema_version: String,
    /// Default privacy classification
    default_privacy: PrivacyClassification,
    /// Use ChaCha20 by default
    use_chacha20: bool,
}

impl DocumentKeyManager {
    /// Create a new document key manager
    pub fn new(
        master_key: [u8; 32],
        default_schema_version: String,
        default_privacy: PrivacyClassification,
        use_chacha20: bool,
    ) -> Self {
        Self {
            master_key,
            wrappers: HashMap::new(),
            default_schema_version,
            default_privacy,
            use_chacha20,
        }
    }

    /// Get or create a wrapper for a document
    pub fn get_or_create_wrapper(&mut self, document_id: &str) -> CryptoResult<&mut CrdtCryptoWrapper> {
        if !self.wrappers.contains_key(document_id) {
            let wrapper = CrdtCryptoWrapper::from_derived_key(
                &self.master_key,
                document_id,
                self.use_chacha20,
                self.default_schema_version.clone(),
                self.default_privacy,
            )?;
            self.wrappers.insert(document_id.to_string(), wrapper);
        }

        Ok(self.wrappers.get_mut(document_id).unwrap())
    }

    /// Get a wrapper for a document (if exists)
    pub fn get_wrapper(&self, document_id: &str) -> Option<&CrdtCryptoWrapper> {
        self.wrappers.get(document_id)
    }

    /// Remove a document wrapper
    pub fn remove_wrapper(&mut self, document_id: &str) {
        self.wrappers.remove(document_id);
    }

    /// List document IDs
    pub fn list_documents(&self) -> Vec<&String> {
        self.wrappers.keys().collect()
    }

    /// Rotate master key (re-derive all document keys)
    pub fn rotate_master_key(&mut self, new_master_key: [u8; 32]) -> CryptoResult<()> {
        self.master_key = new_master_key;

        // Re-derive all document keys
        let document_ids: Vec<String> = self.wrappers.keys().cloned().collect();
        for document_id in document_ids {
            let wrapper = CrdtCryptoWrapper::from_derived_key(
                &self.master_key,
                &document_id,
                self.use_chacha20,
                self.default_schema_version.clone(),
                self.default_privacy,
            )?;
            self.wrappers.insert(document_id, wrapper);
        }

        Ok(())
    }
}

impl Zeroize for DocumentKeyManager {
    fn zeroize(&mut self) {
        self.master_key.zeroize();
        for wrapper in self.wrappers.values_mut() {
            wrapper.zeroize();
        }
    }
}

impl ZeroizeOnDrop for DocumentKeyManager {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_encrypted_crdt_update_creation() {
        let update = EncryptedCrdtUpdate::new(
            [0u8; 32],
            vec![1, 2, 3, 4],
            [0u8; 12],
            0x01,
            "doc-123".to_string(),
            PrivacyClassification::Personal,
            "1.0.0".to_string(),
        );

        assert_eq!(update.document_id, "doc-123");
        assert_eq!(update.algorithm, 0x01);
    }

    #[test]
    fn test_crdt_crypto_wrapper_creation() {
        let key = [0x42; 32];
        let wrapper = CrdtCryptoWrapper::new(
            key,
            "doc-123".to_string(),
            false,
            "1.0.0".to_string(),
            PrivacyClassification::Personal,
        );

        assert_eq!(wrapper.document_id(), "doc-123");
        assert_eq!(wrapper.cache_size(), 0);
    }

    #[test]
    fn test_crdt_encrypt_decrypt() {
        let key = [0x42; 32];
        let mut wrapper = CrdtCryptoWrapper::new(
            key,
            "doc-123".to_string(),
            false,
            "1.0.0".to_string(),
            PrivacyClassification::Personal,
        );

        let update = b"test yjs update data";

        // Encrypt
        let encrypted = wrapper.encrypt(update).unwrap();
        assert!(!encrypted.encrypted_data.is_empty());
        assert_eq!(encrypted.document_id, "doc-123");

        // Decrypt
        let decrypted = wrapper.decrypt(&encrypted).unwrap();
        assert_eq!(decrypted, update);
    }

    #[test]
    fn test_crdt_deduplication() {
        let key = [0x42; 32];
        let mut wrapper = CrdtCryptoWrapper::new(
            key,
            "doc-123".to_string(),
            false,
            "1.0.0".to_string(),
            PrivacyClassification::Personal,
        );

        let update = b"test update";

        // First encryption should succeed
        let encrypted = wrapper.encrypt(update).unwrap();
        assert_eq!(wrapper.cache_size(), 1);

        // Second encryption of same data should fail
        let result = wrapper.encrypt(update);
        assert!(result.is_err());
    }

    #[test]
    fn test_crdt_document_id_mismatch() {
        let key = [0x42; 32];
        let mut wrapper = CrdtCryptoWrapper::new(
            key,
            "doc-123".to_string(),
            false,
            "1.0.0".to_string(),
            PrivacyClassification::Personal,
        );

        let update = b"test update";
        let mut encrypted = wrapper.encrypt(update).unwrap();

        // Tamper with document ID
        encrypted.document_id = "doc-456".to_string();

        let result = wrapper.decrypt(&encrypted);
        assert!(result.is_err());
    }

    #[test]
    fn test_crdt_chacha20() {
        let key = [0x42; 32];
        let mut wrapper = CrdtCryptoWrapper::new(
            key,
            "doc-123".to_string(),
            true, // Use ChaCha20
            "1.0.0".to_string(),
            PrivacyClassification::Personal,
        );

        let update = b"test update with chacha20";

        let encrypted = wrapper.encrypt(update).unwrap();
        assert_eq!(encrypted.algorithm, 0x02); // ChaCha20

        let decrypted = wrapper.decrypt(&encrypted).unwrap();
        assert_eq!(decrypted, update);
    }

    #[test]
    fn test_document_key_manager() {
        let master_key = [0x42; 32];
        let mut manager = DocumentKeyManager::new(
            master_key,
            "1.0.0".to_string(),
            PrivacyClassification::Personal,
            false,
        );

        // Get wrapper for document
        let wrapper = manager.get_or_create_wrapper("doc-1").unwrap();
        assert_eq!(wrapper.document_id(), "doc-1");

        // Get same wrapper again
        let wrapper2 = manager.get_or_create_wrapper("doc-1").unwrap();
        assert_eq!(wrapper2.document_id(), "doc-1");

        // List documents
        let docs = manager.list_documents();
        assert_eq!(docs.len(), 1);
        assert_eq!(docs[0], "doc-1");
    }

    #[test]
    fn test_batch_encrypt_decrypt() {
        let key = [0x42; 32];
        let mut wrapper = CrdtCryptoWrapper::new(
            key,
            "doc-123".to_string(),
            false,
            "1.0.0".to_string(),
            PrivacyClassification::Personal,
        );

        let updates = vec![
            b"update 1".to_vec(),
            b"update 2".to_vec(),
            b"update 3".to_vec(),
        ];

        let encrypted = batch_encrypt(&mut wrapper, updates.clone()).unwrap();
        assert_eq!(encrypted.len(), 3);

        // Need fresh wrapper for decryption (to avoid deduplication)
        let mut wrapper2 = CrdtCryptoWrapper::new(
            key,
            "doc-123".to_string(),
            false,
            "1.0.0".to_string(),
            PrivacyClassification::Personal,
        );

        let decrypted = batch_decrypt(&mut wrapper2, encrypted).unwrap();
        assert_eq!(decrypted, updates);
    }

    #[test]
    fn test_encrypted_crdt_serialization() {
        let original = EncryptedCrdtUpdate::new(
            [1u8; 32],
            vec![2, 3, 4],
            [5u8; 12],
            0x01,
            "doc-123".to_string(),
            PrivacyClassification::Sensitive,
            "2.0.0".to_string(),
        );

        let bytes = original.to_bytes().unwrap();
        let recovered = EncryptedCrdtUpdate::from_bytes(&bytes).unwrap();

        assert_eq!(original.update_id, recovered.update_id);
        assert_eq!(original.encrypted_data, recovered.encrypted_data);
        assert_eq!(original.document_id, recovered.document_id);
    }
}
