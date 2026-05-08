//! Ephemeral Key Management
//!
//! This module manages ephemeral (short-lived) keys used for X3DH.
//! Ephemeral keys provide forward secrecy - compromise of long-term
//! keys does not compromise past communications.

use crate::error::{CryptoError, CryptoResult};
use crate::utils::random_bytes;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::time::{Duration, SystemTime, UNIX_EPOCH};
use x25519_dalek::{PublicKey as X25519PublicKey, SharedSecret, StaticSecret};
use zeroize::{Zeroize, ZeroizeOnDrop};

/// Ephemeral key pair for X3DH
///
/// These keys are short-lived and rotated frequently to provide
/// forward secrecy. Each key should only be used once.
#[derive(Clone, Zeroize, ZeroizeOnDrop)]
pub struct EphemeralKeyPair {
    /// Key ID for tracking
    pub id: u32,
    /// Secret key (zeroized on drop)
    #[zeroize(skip)]
    secret: StaticSecret,
    /// Public key
    public: X25519PublicKey,
    /// Creation timestamp
    created_at: u64,
    /// Whether this key has been used
    used: bool,
}

impl EphemeralKeyPair {
    /// Generate a new ephemeral key pair
    ///
    /// # Errors
    ///
    /// Returns `CryptoError::InternalError` if system time is unavailable.
    pub fn generate(id: u32) -> CryptoResult<Self> {
        let secret = StaticSecret::random_from_rng(rand::thread_rng());
        let public = X25519PublicKey::from(&secret);
        let created_at = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map_err(|e| CryptoError::InternalError(format!("System time error: {:?}", e)))?
            .as_secs();

        Ok(Self {
            id,
            secret,
            public,
            created_at,
            used: false,
        })
    }

    /// Get the public key
    pub fn public_key(&self) -> &X25519PublicKey {
        &self.public
    }

    /// Get the raw public key bytes
    pub fn public_key_bytes(&self) -> [u8; 32] {
        self.public.to_bytes()
    }

    /// Perform Diffie-Hellman with another public key
    pub fn diffie_hellman(&self, other: &X25519PublicKey) -> SharedSecret {
        self.secret.diffie_hellman(other)
    }

    /// Mark this key as used
    pub fn mark_used(&mut self) {
        self.used = true;
    }

    /// Check if this key has been used
    pub fn is_used(&self) -> bool {
        self.used
    }

    /// Get the age of this key in seconds
    ///
    /// # Errors
    ///
    /// Returns `CryptoError::InternalError` if system time is unavailable.
    pub fn age_secs(&self) -> CryptoResult<u64> {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map_err(|e| CryptoError::InternalError(format!("System time error: {:?}", e)))?
            .as_secs();
        Ok(now.saturating_sub(self.created_at))
    }

    /// Check if this key has expired
    ///
    /// # Errors
    ///
    /// Returns `CryptoError::InternalError` if system time is unavailable.
    pub fn is_expired(&self, max_age_secs: u64) -> CryptoResult<bool> {
        Ok(self.age_secs()? > max_age_secs)
    }

    /// Serialize to bytes
    pub fn to_bytes(&self) -> Vec<u8> {
        let mut result = Vec::with_capacity(41);
        result.extend_from_slice(&self.id.to_be_bytes());
        result.extend_from_slice(&self.secret.to_bytes());
        result.extend_from_slice(&self.created_at.to_be_bytes());
        result.push(self.used as u8);
        result
    }

    /// Deserialize from bytes
    pub fn from_bytes(bytes: &[u8]) -> CryptoResult<Self> {
        if bytes.len() < 45 {
            return Err(CryptoError::invalid_key("Ephemeral key too short"));
        }

        let id = u32::from_be_bytes([bytes[0], bytes[1], bytes[2], bytes[3]]);

        let mut secret_bytes = [0u8; 32];
        secret_bytes.copy_from_slice(&bytes[4..36]);
        let secret = StaticSecret::from(secret_bytes);
        let public = X25519PublicKey::from(&secret);

        let created_at = u64::from_be_bytes([
            bytes[36], bytes[37], bytes[38], bytes[39],
            bytes[40], bytes[41], bytes[42], bytes[43],
        ]);

        let used = bytes[44] != 0;

        Ok(Self {
            id,
            secret,
            public,
            created_at,
            used,
        })
    }
}

impl std::fmt::Debug for EphemeralKeyPair {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "EphemeralKeyPair {{ id: {}, public: {:?}, used: {}, age: {}s }}",
            self.id,
            self.public,
            self.used,
            self.age_secs()
        )
    }
}

/// Public portion of ephemeral key
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct EphemeralPublicKey {
    /// Key ID
    pub id: u32,
    /// Public key bytes
    #[serde(with = "serde_bytes")]
    pub public_key: [u8; 32],
}

impl EphemeralPublicKey {
    /// Create from an ephemeral key pair
    pub fn from_keypair(kp: &EphemeralKeyPair) -> Self {
        Self {
            id: kp.id,
            public_key: kp.public_key_bytes(),
        }
    }

    /// Get the X25519 public key
    pub fn to_x25519(&self) -> X25519PublicKey {
        X25519PublicKey::from(self.public_key)
    }
}

/// Manager for ephemeral keys
///
/// Handles generation, storage, rotation, and cleanup of ephemeral keys.
pub struct EphemeralKeyManager {
    /// Active ephemeral keys
    keys: HashMap<u32, EphemeralKeyPair>,
    /// Next key ID
    next_id: u32,
    /// Maximum key age in seconds
    max_age_secs: u64,
    /// Target number of keys to maintain
    target_key_count: usize,
    /// Minimum keys before regeneration
    min_key_threshold: usize,
}

impl EphemeralKeyManager {
    /// Create a new ephemeral key manager
    pub fn new(max_age_secs: u64, target_key_count: usize, min_key_threshold: usize) -> Self {
        Self {
            keys: HashMap::new(),
            next_id: 1,
            max_age_secs,
            target_key_count,
            min_key_threshold,
        }
    }

    /// Create with default settings
    pub fn default() -> Self {
        Self::new(
            crate::MAX_EPHEMERAL_KEY_AGE, // 7 days
            crate::DEFAULT_PREKEY_COUNT,  // 100 keys
            crate::PREKEY_THRESHOLD,      // 20 keys
        )
    }

    /// Initialize with a batch of keys
    pub fn initialize(&mut self) {
        for _ in 0..self.target_key_count {
            self.generate_key();
        }
    }

    /// Generate a new ephemeral key
    pub fn generate_key(&mut self) -> u32 {
        let id = self.next_id;
        self.next_id = self.next_id.wrapping_add(1);

        let key = EphemeralKeyPair::generate(id);
        self.keys.insert(id, key);

        id
    }

    /// Get an unused key
    pub fn get_unused_key(&mut self) -> Option<&EphemeralKeyPair> {
        // First, try to find an unused key
        for (id, key) in &self.keys {
            if !key.is_used() && !key.is_expired(self.max_age_secs) {
                return self.keys.get(id);
            }
        }

        // If we're below threshold, generate more
        if self.keys.len() < self.min_key_threshold {
            self.replenish_keys();
        }

        // Try again
        for (id, key) in &self.keys {
            if !key.is_used() && !key.is_expired(self.max_age_secs) {
                return self.keys.get(id);
            }
        }

        None
    }

    /// Take (remove) an unused key
    pub fn take_unused_key(&mut self) -> Option<EphemeralKeyPair> {
        // Find an unused key
        let key_id = self.keys.iter()
            .find(|(_, key)| !key.is_used() && !key.is_expired(self.max_age_secs))
            .map(|(id, _)| *id);

        if let Some(id) = key_id {
            let mut key = self.keys.remove(&id)?;
            key.mark_used();
            return Some(key);
        }

        // If we're below threshold, generate more
        if self.keys.len() < self.min_key_threshold {
            self.replenish_keys();
        }

        // Try again
        let key_id = self.keys.iter()
            .find(|(_, key)| !key.is_used() && !key.is_expired(self.max_age_secs))
            .map(|(id, _)| *id);

        key_id.and_then(|id| {
            let mut key = self.keys.remove(&id)?;
            key.mark_used();
            Some(key)
        })
    }

    /// Mark a key as used by ID
    pub fn mark_used(&mut self, id: u32) {
        if let Some(key) = self.keys.get_mut(&id) {
            key.mark_used();
        }
    }

    /// Replenish keys to target count
    pub fn replenish_keys(&mut self) {
        while self.keys.len() < self.target_key_count {
            self.generate_key();
        }
    }

    /// Clean up expired and used keys
    pub fn cleanup(&mut self) -> usize {
        let to_remove: Vec<u32> = self.keys
            .iter()
            .filter(|(_, key)| {
                key.is_used() || key.is_expired(self.max_age_secs)
            })
            .map(|(id, _)| *id)
            .collect();

        let count = to_remove.len();
        for id in to_remove {
            self.keys.remove(&id);
        }

        count
    }

    /// Get the number of available (unused, unexpired) keys
    pub fn available_count(&self) -> usize {
        self.keys
            .values()
            .filter(|key| !key.is_used() && !key.is_expired(self.max_age_secs))
            .count()
    }

    /// Get the total number of keys
    pub fn total_count(&self) -> usize {
        self.keys.len()
    }

    /// Get all public keys
    pub fn get_public_keys(&self) -> Vec<EphemeralPublicKey> {
        self.keys
            .values()
            .filter(|key| !key.is_used() && !key.is_expired(self.max_age_secs))
            .map(EphemeralPublicKey::from_keypair)
            .collect()
    }

    /// Get a key by ID
    pub fn get_key(&self, id: u32) -> Option<&EphemeralKeyPair> {
        self.keys.get(&id)
    }

    /// Rotate all keys (invalidate and regenerate)
    pub fn rotate_all(&mut self) {
        self.keys.clear();
        self.initialize();
    }

    /// Serialize all keys to bytes
    pub fn to_bytes(&self) -> Vec<u8> {
        let mut result = Vec::new();

        // Write count
        result.extend_from_slice(&(self.keys.len() as u32).to_be_bytes());

        // Write each key
        for key in self.keys.values() {
            let key_bytes = key.to_bytes();
            result.extend_from_slice(&(key_bytes.len() as u32).to_be_bytes());
            result.extend_from_slice(&key_bytes);
        }

        // Write next_id
        result.extend_from_slice(&self.next_id.to_be_bytes());

        result
    }

    /// Deserialize from bytes
    pub fn from_bytes(bytes: &[u8]) -> CryptoResult<Self> {
        if bytes.len() < 8 {
            return Err(CryptoError::invalid_key("Ephemeral key manager data too short"));
        }

        let count = u32::from_be_bytes([bytes[0], bytes[1], bytes[2], bytes[3]]) as usize;
        let mut keys = HashMap::new();
        let mut offset = 4;

        for _ in 0..count {
            if offset + 4 > bytes.len() {
                return Err(CryptoError::invalid_key("Truncated key data"));
            }

            let key_len = u32::from_be_bytes([
                bytes[offset], bytes[offset + 1],
                bytes[offset + 2], bytes[offset + 3],
            ]) as usize;
            offset += 4;

            if offset + key_len > bytes.len() {
                return Err(CryptoError::invalid_key("Truncated key data"));
            }

            let key = EphemeralKeyPair::from_bytes(&bytes[offset..offset + key_len])?;
            keys.insert(key.id, key);
            offset += key_len;
        }

        if offset + 4 > bytes.len() {
            return Err(CryptoError::invalid_key("Missing next_id"));
        }

        let next_id = u32::from_be_bytes([
            bytes[offset], bytes[offset + 1],
            bytes[offset + 2], bytes[offset + 3],
        ]);

        Ok(Self {
            keys,
            next_id,
            max_age_secs: crate::MAX_EPHEMERAL_KEY_AGE,
            target_key_count: crate::DEFAULT_PREKEY_COUNT,
            min_key_threshold: crate::PREKEY_THRESHOLD,
        })
    }
}

impl std::fmt::Debug for EphemeralKeyManager {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("EphemeralKeyManager")
            .field("total_keys", &self.total_count())
            .field("available_keys", &self.available_count())
            .field("target_count", &self.target_key_count)
            .finish()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ephemeral_keypair_generation() {
        let kp = EphemeralKeyPair::generate(1);
        assert_eq!(kp.id, 1);
        assert!(!kp.is_used());
        assert_eq!(kp.public_key_bytes().len(), 32);
    }

    #[test]
    fn test_ephemeral_key_usage() {
        let mut kp = EphemeralKeyPair::generate(1);
        assert!(!kp.is_used());

        kp.mark_used();
        assert!(kp.is_used());
    }

    #[test]
    fn test_key_manager_initialization() {
        let mut manager = EphemeralKeyManager::default();
        manager.initialize();

        assert_eq!(manager.total_count(), crate::DEFAULT_PREKEY_COUNT);
        assert_eq!(manager.available_count(), crate::DEFAULT_PREKEY_COUNT);
    }

    #[test]
    fn test_take_unused_key() {
        let mut manager = EphemeralKeyManager::default();
        manager.initialize();

        let initial_available = manager.available_count();
        let key = manager.take_unused_key();
        assert!(key.is_some());
        assert_eq!(manager.available_count(), initial_available - 1);
    }

    #[test]
    fn test_key_replenishment() {
        let mut manager = EphemeralKeyManager::new(86400, 10, 5);
        manager.initialize();

        // Take most keys
        for _ in 0..6 {
            manager.take_unused_key();
        }

        assert!(manager.available_count() < 5);

        // Replenish
        manager.replenish_keys();
        assert_eq!(manager.total_count(), 10);
    }

    #[test]
    fn test_cleanup() {
        let mut manager = EphemeralKeyManager::new(0, 10, 5); // 0 max age for instant expiry
        manager.initialize();

        // All keys should be expired immediately
        let cleaned = manager.cleanup();
        assert_eq!(cleaned, 10);
        assert_eq!(manager.total_count(), 0);
    }

    #[test]
    fn test_key_manager_serialization() {
        let mut manager = EphemeralKeyManager::default();
        manager.initialize();

        let bytes = manager.to_bytes();
        let recovered = EphemeralKeyManager::from_bytes(&bytes).unwrap();

        assert_eq!(recovered.total_count(), manager.total_count());
    }

    #[test]
    fn test_diffie_hellman() {
        let kp1 = EphemeralKeyPair::generate(1);
        let kp2 = EphemeralKeyPair::generate(2);

        let shared1 = kp1.diffie_hellman(kp2.public_key());
        let shared2 = kp2.diffie_hellman(kp1.public_key());

        assert_eq!(shared1.as_bytes(), shared2.as_bytes());
    }
}
