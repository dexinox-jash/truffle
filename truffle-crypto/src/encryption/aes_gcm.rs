//! AES-256-GCM Encryption
//!
//! This module implements AES-256-GCM authenticated encryption as specified
//! in NIST SP 800-38D. It provides:
//! - 256-bit key strength
//! - 96-bit nonces (random IV per message)
//! - 128-bit authentication tags
//!
//! # Security Considerations
//!
//! - Nonces MUST be unique for each encryption with the same key
//! - This implementation uses random nonces (96-bit) which provides
//!   sufficient collision resistance
//! - Keys should be rotated periodically

use crate::encryption::EncryptionCipher;
use crate::error::{CryptoError, CryptoResult};
use crate::types::{EncryptedBlob, Nonce};
use aes_gcm::{
    aead::{Aead, KeyInit, Payload},
    Aes256Gcm, Nonce as AesGcmNonce, Tag,
};

/// AES-256-GCM cipher
pub struct Aes256GcmCipher {
    /// The underlying AES-256-GCM instance
    cipher: Aes256Gcm,
}

impl Aes256GcmCipher {
    /// Key size in bytes (256 bits)
    pub const KEY_SIZE: usize = 32;

    /// Nonce size in bytes (96 bits)
    pub const NONCE_SIZE: usize = 12;

    /// Tag size in bytes (128 bits)
    pub const TAG_SIZE: usize = 16;

    /// Algorithm identifier
    pub const ALGORITHM_ID: u8 = 0x01;

    /// Algorithm name
    pub const ALGORITHM_NAME: &'static str = "AES-256-GCM";

    /// Create a new AES-256-GCM cipher
    ///
    /// # Arguments
    ///
    /// * `key` - 32-byte (256-bit) encryption key
    ///
    /// # Errors
    ///
    /// Returns an error if the key is not exactly 32 bytes
    pub fn new(key: &[u8; 32]) -> CryptoResult<Self> {
        let cipher = Aes256Gcm::new_from_slice(key)
            .map_err(|e| CryptoError::invalid_key(format!("AES-256-GCM key init failed: {:?}", e)))?;

        Ok(Self { cipher })
    }

    /// Encrypt plaintext in-place
    ///
    /// # Arguments
    ///
    /// * `plaintext` - Data to encrypt (will be overwritten with ciphertext)
    /// * `nonce` - Unique nonce
    /// * `associated_data` - Additional authenticated data
    ///
    /// # Returns
    ///
    /// The authentication tag
    pub fn encrypt_in_place(
        &self,
        plaintext: &mut [u8],
        nonce: &Nonce,
        associated_data: &[u8],
    ) -> CryptoResult<[u8; 16]> {
        let aes_nonce = AesGcmNonce::from_slice(nonce.as_bytes());

        let tag = self.cipher
            .encrypt_in_place_detached(
                aes_nonce,
                associated_data,
                plaintext,
            )
            .map_err(|e| CryptoError::encryption_failed(format!("AES-256-GCM encrypt failed: {:?}", e)))?;

        let mut tag_bytes = [0u8; 16];
        tag_bytes.copy_from_slice(tag.as_slice());
        Ok(tag_bytes)
    }

    /// Decrypt ciphertext in-place
    ///
    /// # Arguments
    ///
    /// * `ciphertext` - Data to decrypt (will be overwritten with plaintext)
    /// * `tag` - Authentication tag
    /// * `nonce` - Nonce used for encryption
    /// * `associated_data` - Additional authenticated data
    ///
    /// # Errors
    ///
    /// Returns `CryptoError::AuthenticationFailed` if authentication fails
    pub fn decrypt_in_place(
        &self,
        ciphertext: &mut [u8],
        tag: &[u8; 16],
        nonce: &Nonce,
        associated_data: &[u8],
    ) -> CryptoResult<()> {
        let aes_nonce = AesGcmNonce::from_slice(nonce.as_bytes());
        let aes_tag = Tag::from_slice(tag);

        self.cipher
            .decrypt_in_place_detached(
                aes_nonce,
                associated_data,
                ciphertext,
                aes_tag,
            )
            .map_err(|_| CryptoError::authentication_failed("AES-256-GCM authentication failed"))?;

        Ok(())
    }
}

impl EncryptionCipher for Aes256GcmCipher {
    fn new(key: &[u8; 32]) -> CryptoResult<Self> {
        Self::new(key)
    }

    fn encrypt(
        &self,
        plaintext: &[u8],
        nonce: &Nonce,
        associated_data: &[u8],
    ) -> CryptoResult<EncryptedBlob> {
        let aes_nonce = AesGcmNonce::from_slice(nonce.as_bytes());

        let payload = Payload {
            msg: plaintext,
            aad: associated_data,
        };

        let ciphertext = self.cipher
            .encrypt(aes_nonce, payload)
            .map_err(|e| CryptoError::encryption_failed(format!("AES-256-GCM encrypt failed: {:?}", e)))?;

        Ok(EncryptedBlob::new(ciphertext, nonce.clone()))
    }

    fn decrypt(
        &self,
        ciphertext: &EncryptedBlob,
        associated_data: &[u8],
    ) -> CryptoResult<Vec<u8>> {
        let aes_nonce = AesGcmNonce::from_slice(ciphertext.nonce().as_bytes());

        let payload = Payload {
            msg: ciphertext.ciphertext(),
            aad: associated_data,
        };

        let plaintext = self.cipher
            .decrypt(aes_nonce, payload)
            .map_err(|_| CryptoError::authentication_failed("AES-256-GCM authentication failed"))?;

        Ok(plaintext)
    }

    fn algorithm_id(&self) -> u8 {
        Self::ALGORITHM_ID
    }

    fn algorithm_name(&self) -> &'static str {
        Self::ALGORITHM_NAME
    }
}

impl std::fmt::Debug for Aes256GcmCipher {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Aes256GcmCipher {{ algorithm: {} }}", Self::ALGORITHM_NAME)
    }
}

/// Encrypt a message with AES-256-GCM (convenience function)
pub fn encrypt(
    key: &[u8; 32],
    plaintext: &[u8],
    nonce: &Nonce,
    associated_data: &[u8],
) -> CryptoResult<EncryptedBlob> {
    let cipher = Aes256GcmCipher::new(key)?;
    cipher.encrypt(plaintext, nonce, associated_data)
}

/// Decrypt a message with AES-256-GCM (convenience function)
pub fn decrypt(
    key: &[u8; 32],
    ciphertext: &EncryptedBlob,
    associated_data: &[u8],
) -> CryptoResult<Vec<u8>> {
    let cipher = Aes256GcmCipher::new(key)?;
    cipher.decrypt(ciphertext, associated_data)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_aes256_gcm_constants() {
        assert_eq!(Aes256GcmCipher::KEY_SIZE, 32);
        assert_eq!(Aes256GcmCipher::NONCE_SIZE, 12);
        assert_eq!(Aes256GcmCipher::TAG_SIZE, 16);
        assert_eq!(Aes256GcmCipher::ALGORITHM_ID, 0x01);
    }

    #[test]
    fn test_aes256_gcm_encryption() {
        let key = [0x42; 32];
        let plaintext = b"Hello, World!";
        let nonce = Nonce::generate_random().unwrap();
        let associated_data = b"additional data";

        let cipher = Aes256GcmCipher::new(&key).unwrap();

        // Encrypt
        let encrypted = cipher.encrypt(plaintext, &nonce, associated_data).unwrap();
        assert!(!encrypted.ciphertext().is_empty());

        // Decrypt
        let decrypted = cipher.decrypt(&encrypted, associated_data).unwrap();
        assert_eq!(decrypted, plaintext);
    }

    #[test]
    fn test_aes256_gcm_wrong_key() {
        let key1 = [0x42; 32];
        let key2 = [0x43; 32];
        let plaintext = b"Hello, World!";
        let nonce = Nonce::generate_random().unwrap();

        let cipher1 = Aes256GcmCipher::new(&key1).unwrap();
        let encrypted = cipher1.encrypt(plaintext, &nonce, &[]).unwrap();

        let cipher2 = Aes256GcmCipher::new(&key2).unwrap();
        let result = cipher2.decrypt(&encrypted, &[]);

        assert!(matches!(result, Err(CryptoError::AuthenticationFailed { .. })));
    }

    #[test]
    fn test_aes256_gcm_tampered_ciphertext() {
        let key = [0x42; 32];
        let plaintext = b"Hello, World!";
        let nonce = Nonce::generate_random().unwrap();

        let cipher = Aes256GcmCipher::new(&key).unwrap();
        let mut encrypted = cipher.encrypt(plaintext, &nonce, &[]).unwrap();

        // Tamper with ciphertext
        let mut ciphertext = encrypted.ciphertext().to_vec();
        ciphertext[0] ^= 0xFF;
        encrypted = EncryptedBlob::new(ciphertext, nonce);

        let result = cipher.decrypt(&encrypted, &[]);
        assert!(matches!(result, Err(CryptoError::AuthenticationFailed { .. })));
    }

    #[test]
    fn test_aes256_gcm_wrong_associated_data() {
        let key = [0x42; 32];
        let plaintext = b"Hello, World!";
        let nonce = Nonce::generate_random().unwrap();

        let cipher = Aes256GcmCipher::new(&key).unwrap();
        let encrypted = cipher.encrypt(plaintext, &nonce, b"correct aad").unwrap();

        let result = cipher.decrypt(&encrypted, b"wrong aad");
        assert!(matches!(result, Err(CryptoError::AuthenticationFailed { .. })));
    }

    #[test]
    fn test_aes256_gcm_nonce_reuse() {
        let key = [0x42; 32];
        let plaintext1 = b"Message 1";
        let plaintext2 = b"Message 2";
        let nonce = Nonce::generate_random().unwrap();

        let cipher = Aes256GcmCipher::new(&key).unwrap();

        // First encryption
        let encrypted1 = cipher.encrypt(plaintext1, &nonce, &[]).unwrap();
        let decrypted1 = cipher.decrypt(&encrypted1, &[]).unwrap();
        assert_eq!(decrypted1, plaintext1);

        // Second encryption with same nonce (should still work for decryption)
        let encrypted2 = cipher.encrypt(plaintext2, &nonce, &[]).unwrap();
        let decrypted2 = cipher.decrypt(&encrypted2, &[]).unwrap();
        assert_eq!(decrypted2, plaintext2);

        // Note: In practice, nonce reuse is a security vulnerability
        // This test just verifies the cipher works correctly
    }

    #[test]
    fn test_aes256_gcm_empty_plaintext() {
        let key = [0x42; 32];
        let plaintext = b"";
        let nonce = Nonce::generate_random().unwrap();

        let cipher = Aes256GcmCipher::new(&key).unwrap();
        let encrypted = cipher.encrypt(plaintext, &nonce, &[]).unwrap();

        // Empty plaintext still produces a tag
        assert_eq!(encrypted.ciphertext().len(), 16); // Just the tag

        let decrypted = cipher.decrypt(&encrypted, &[]).unwrap();
        assert_eq!(decrypted, plaintext);
    }

    #[test]
    fn test_aes256_gcm_large_plaintext() {
        let key = [0x42; 32];
        let plaintext = vec![0xABu8; 1024 * 1024]; // 1 MB
        let nonce = Nonce::generate_random().unwrap();

        let cipher = Aes256GcmCipher::new(&key).unwrap();
        let encrypted = cipher.encrypt(&plaintext, &nonce, &[]).unwrap();

        assert_eq!(encrypted.ciphertext().len(), plaintext.len() + 16);

        let decrypted = cipher.decrypt(&encrypted, &[]).unwrap();
        assert_eq!(decrypted, plaintext);
    }

    #[test]
    fn test_encrypt_decrypt_convenience_functions() {
        let key = [0x42; 32];
        let plaintext = b"Test message";
        let nonce = Nonce::generate_random().unwrap();

        let encrypted = encrypt(&key, plaintext, &nonce, &[]).unwrap();
        let decrypted = decrypt(&key, &encrypted, &[]).unwrap();

        assert_eq!(decrypted, plaintext);
    }

    #[test]
    fn test_in_place_encryption() {
        let key = [0x42; 32];
        let mut plaintext = b"Hello, World!".to_vec();
        let original = plaintext.clone();
        let nonce = Nonce::generate_random().unwrap();

        let cipher = Aes256GcmCipher::new(&key).unwrap();
        let tag = cipher.encrypt_in_place(&mut plaintext, &nonce, &[]).unwrap();

        // After in-place encryption, plaintext buffer contains ciphertext
        assert_ne!(plaintext, original);

        // Decrypt in-place
        cipher.decrypt_in_place(&mut plaintext, &tag, &nonce, &[]).unwrap();

        // After in-place decryption, should be back to original
        assert_eq!(plaintext, original);
    }
}
