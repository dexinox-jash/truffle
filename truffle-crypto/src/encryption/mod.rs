//! Encryption Primitives Module
//!
//! This module provides symmetric encryption primitives for the ZKS-1 protocol:
//! - AES-256-GCM for desktop platforms
//! - ChaCha20-Poly1305 for mobile platforms (constant-time, side-channel resistant)
//! - CRDT encryption wrapper for Yjs integration

mod aes_gcm;
mod chacha20;
mod crdt_crypto;

pub use aes_gcm::Aes256GcmCipher;
pub use chacha20::ChaCha20Poly1305Cipher;
pub use crdt_crypto::{CrdtCryptoWrapper, EncryptedCrdtUpdate};

use crate::error::{CryptoError, CryptoResult};
use crate::types::{EncryptedBlob, Nonce};

/// Trait for symmetric encryption ciphers
pub trait EncryptionCipher {
    /// Create a new cipher with the given key
    fn new(key: &[u8; 32]) -> CryptoResult<Self>
    where
        Self: Sized;

    /// Encrypt plaintext
    ///
    /// # Arguments
    ///
    /// * `plaintext` - Data to encrypt
    /// * `nonce` - Unique nonce (must not be reused with same key)
    /// * `associated_data` - Additional authenticated data (not encrypted)
    ///
    /// # Returns
    ///
    /// Encrypted blob containing ciphertext and authentication tag
    fn encrypt(
        &self,
        plaintext: &[u8],
        nonce: &Nonce,
        associated_data: &[u8],
    ) -> CryptoResult<EncryptedBlob>;

    /// Decrypt ciphertext
    ///
    /// # Arguments
    ///
    /// * `ciphertext` - Encrypted blob from encrypt()
    /// * `associated_data` - Must match the associated_data used for encryption
    ///
    /// # Returns
    ///
    /// Decrypted plaintext
    ///
    /// # Errors
    ///
    /// Returns `CryptoError::AuthenticationFailed` if authentication fails
    /// (indicating tampering or corrupted data)
    fn decrypt(
        &self,
        ciphertext: &EncryptedBlob,
        associated_data: &[u8],
    ) -> CryptoResult<Vec<u8>>;

    /// Get the cipher's algorithm identifier
    fn algorithm_id(&self) -> u8;

    /// Get the cipher's name
    fn algorithm_name(&self) -> &'static str;
}

/// Algorithm identifiers
pub mod algorithms {
    /// AES-256-GCM
    pub const AES256_GCM: u8 = 0x01;
    /// ChaCha20-Poly1305
    pub const CHACHA20_POLY1305: u8 = 0x02;
}

/// Create a cipher from algorithm ID
pub fn create_cipher(algorithm_id: u8, key: &[u8; 32]) -> CryptoResult<Box<dyn EncryptionCipher>> {
    match algorithm_id {
        algorithms::AES256_GCM => {
            Ok(Box::new(Aes256GcmCipher::new(key)?))
        }
        algorithms::CHACHA20_POLY1305 => {
            Ok(Box::new(ChaCha20Poly1305Cipher::new(key)?))
        }
        _ => Err(CryptoError::invalid_key(
            format!("Unknown algorithm ID: {}", algorithm_id)
        )),
    }
}

/// Securely generate a random encryption key
pub fn generate_key() -> CryptoResult<[u8; 32]> {
    crate::utils::random_bytes::<32>()
}

/// Securely generate a random nonce
pub fn generate_nonce() -> CryptoResult<Nonce> {
    Nonce::generate_random()
}

/// Encrypt with algorithm auto-detection from blob
pub fn decrypt_auto(
    ciphertext: &EncryptedBlob,
    key: &[u8; 32],
    associated_data: &[u8],
) -> CryptoResult<Vec<u8>> {
    // Try AES-256-GCM first (most common)
    let cipher = Aes256GcmCipher::new(key)?;
    match cipher.decrypt(ciphertext, associated_data) {
        Ok(plaintext) => Ok(plaintext),
        Err(CryptoError::AuthenticationFailed { .. }) => {
            // Try ChaCha20-Poly1305
            let cipher = ChaCha20Poly1305Cipher::new(key)?;
            cipher.decrypt(ciphertext, associated_data)
        }
        Err(e) => Err(e),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_algorithm_ids() {
        assert_eq!(algorithms::AES256_GCM, 0x01);
        assert_eq!(algorithms::CHACHA20_POLY1305, 0x02);
    }

    #[test]
    fn test_generate_key() {
        let key1 = generate_key().unwrap();
        let key2 = generate_key().unwrap();
        assert_ne!(key1, key2);
        assert_eq!(key1.len(), 32);
    }

    #[test]
    fn test_generate_nonce() {
        let nonce1 = generate_nonce().unwrap();
        let nonce2 = generate_nonce().unwrap();
        assert_ne!(nonce1.as_bytes(), nonce2.as_bytes());
    }
}
