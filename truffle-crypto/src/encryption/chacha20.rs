//! ChaCha20-Poly1305 Encryption
//!
//! This module implements ChaCha20-Poly1305 authenticated encryption as specified
//! in RFC 8439. It provides:
//! - 256-bit key strength
//! - 96-bit nonces
//! - 128-bit authentication tags
//! - Constant-time operations (side-channel resistant)
//!
//! ChaCha20-Poly1305 is preferred for mobile platforms due to its
//! constant-time implementation and resistance to timing attacks.
//!
//! # Security Considerations
//!
//! - Nonces MUST be unique for each encryption with the same key
//! - This cipher is preferred on mobile devices for side-channel resistance
//! - Software implementations are typically faster than AES on devices
//!   without AES hardware acceleration

use crate::encryption::EncryptionCipher;
use crate::error::{CryptoError, CryptoResult};
use crate::types::{EncryptedBlob, Nonce};
use chacha20poly1305::{
    aead::{Aead, KeyInit, Payload},
    ChaCha20Poly1305, Nonce as ChaChaNonce, Tag,
};

/// ChaCha20-Poly1305 cipher
pub struct ChaCha20Poly1305Cipher {
    /// The underlying ChaCha20-Poly1305 instance
    cipher: ChaCha20Poly1305,
}

impl ChaCha20Poly1305Cipher {
    /// Key size in bytes (256 bits)
    pub const KEY_SIZE: usize = 32;

    /// Nonce size in bytes (96 bits)
    pub const NONCE_SIZE: usize = 12;

    /// Tag size in bytes (128 bits)
    pub const TAG_SIZE: usize = 16;

    /// Algorithm identifier
    pub const ALGORITHM_ID: u8 = 0x02;

    /// Algorithm name
    pub const ALGORITHM_NAME: &'static str = "ChaCha20-Poly1305";

    /// Create a new ChaCha20-Poly1305 cipher
    ///
    /// # Arguments
    ///
    /// * `key` - 32-byte (256-bit) encryption key
    ///
    /// # Errors
    ///
    /// Returns an error if the key is not exactly 32 bytes
    pub fn new(key: &[u8; 32]) -> CryptoResult<Self> {
        let cipher = ChaCha20Poly1305::new_from_slice(key)
            .map_err(|e| CryptoError::invalid_key(format!("ChaCha20-Poly1305 key init failed: {:?}", e)))?;

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
        let chacha_nonce = ChaChaNonce::from_slice(nonce.as_bytes());

        let tag = self.cipher
            .encrypt_in_place_detached(
                chacha_nonce,
                associated_data,
                plaintext,
            )
            .map_err(|e| CryptoError::encryption_failed(format!("ChaCha20-Poly1305 encrypt failed: {:?}", e)))?;

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
        let chacha_nonce = ChaChaNonce::from_slice(nonce.as_bytes());
        let chacha_tag = Tag::from_slice(tag);

        self.cipher
            .decrypt_in_place_detached(
                chacha_nonce,
                associated_data,
                ciphertext,
                chacha_tag,
            )
            .map_err(|_| CryptoError::authentication_failed("ChaCha20-Poly1305 authentication failed"))?;

        Ok(())
    }
}

impl EncryptionCipher for ChaCha20Poly1305Cipher {
    fn new(key: &[u8; 32]) -> CryptoResult<Self> {
        Self::new(key)
    }

    fn encrypt(
        &self,
        plaintext: &[u8],
        nonce: &Nonce,
        associated_data: &[u8],
    ) -> CryptoResult<EncryptedBlob> {
        let chacha_nonce = ChaChaNonce::from_slice(nonce.as_bytes());

        let payload = Payload {
            msg: plaintext,
            aad: associated_data,
        };

        let ciphertext = self.cipher
            .encrypt(chacha_nonce, payload)
            .map_err(|e| CryptoError::encryption_failed(format!("ChaCha20-Poly1305 encrypt failed: {:?}", e)))?;

        Ok(EncryptedBlob::new(ciphertext, nonce.clone()))
    }

    fn decrypt(
        &self,
        ciphertext: &EncryptedBlob,
        associated_data: &[u8],
    ) -> CryptoResult<Vec<u8>> {
        let chacha_nonce = ChaChaNonce::from_slice(ciphertext.nonce().as_bytes());

        let payload = Payload {
            msg: ciphertext.ciphertext(),
            aad: associated_data,
        };

        let plaintext = self.cipher
            .decrypt(chacha_nonce, payload)
            .map_err(|_| CryptoError::authentication_failed("ChaCha20-Poly1305 authentication failed"))?;

        Ok(plaintext)
    }

    fn algorithm_id(&self) -> u8 {
        Self::ALGORITHM_ID
    }

    fn algorithm_name(&self) -> &'static str {
        Self::ALGORITHM_NAME
    }
}

impl std::fmt::Debug for ChaCha20Poly1305Cipher {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "ChaCha20Poly1305Cipher {{ algorithm: {} }}", Self::ALGORITHM_NAME)
    }
}

/// Encrypt a message with ChaCha20-Poly1305 (convenience function)
pub fn encrypt(
    key: &[u8; 32],
    plaintext: &[u8],
    nonce: &Nonce,
    associated_data: &[u8],
) -> CryptoResult<EncryptedBlob> {
    let cipher = ChaCha20Poly1305Cipher::new(key)?;
    cipher.encrypt(plaintext, nonce, associated_data)
}

/// Decrypt a message with ChaCha20-Poly1305 (convenience function)
pub fn decrypt(
    key: &[u8; 32],
    ciphertext: &EncryptedBlob,
    associated_data: &[u8],
) -> CryptoResult<Vec<u8>> {
    let cipher = ChaCha20Poly1305Cipher::new(key)?;
    cipher.decrypt(ciphertext, associated_data)
}

/// Benchmark comparison between AES-256-GCM and ChaCha20-Poly1305
#[cfg(test)]
pub mod benchmarks {
    use super::*;
    use std::time::Instant;

    /// Run a simple benchmark
    pub fn benchmark_both_ciphers(data_size: usize, iterations: usize) {
        let key = [0x42; 32];
        let plaintext = vec![0xABu8; data_size];
        let nonce = Nonce::generate_random().unwrap();

        // Benchmark AES-256-GCM
        let aes_cipher = crate::encryption::Aes256GcmCipher::new(&key).unwrap();
        let start = Instant::now();
        for _ in 0..iterations {
            let encrypted = aes_cipher.encrypt(&plaintext, &nonce, &[]).unwrap();
            let _ = aes_cipher.decrypt(&encrypted, &[]).unwrap();
        }
        let aes_time = start.elapsed();

        // Benchmark ChaCha20-Poly1305
        let chacha_cipher = ChaCha20Poly1305Cipher::new(&key).unwrap();
        let start = Instant::now();
        for _ in 0..iterations {
            let encrypted = chacha_cipher.encrypt(&plaintext, &nonce, &[]).unwrap();
            let _ = chacha_cipher.decrypt(&encrypted, &[]).unwrap();
        }
        let chacha_time = start.elapsed();

        println!("Data size: {} bytes, Iterations: {}", data_size, iterations);
        println!("AES-256-GCM: {:?}", aes_time);
        println!("ChaCha20-Poly1305: {:?}", chacha_time);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_chacha20_poly1305_constants() {
        assert_eq!(ChaCha20Poly1305Cipher::KEY_SIZE, 32);
        assert_eq!(ChaCha20Poly1305Cipher::NONCE_SIZE, 12);
        assert_eq!(ChaCha20Poly1305Cipher::TAG_SIZE, 16);
        assert_eq!(ChaCha20Poly1305Cipher::ALGORITHM_ID, 0x02);
    }

    #[test]
    fn test_chacha20_poly1305_encryption() {
        let key = [0x42; 32];
        let plaintext = b"Hello, World!";
        let nonce = Nonce::generate_random().unwrap();
        let associated_data = b"additional data";

        let cipher = ChaCha20Poly1305Cipher::new(&key).unwrap();

        // Encrypt
        let encrypted = cipher.encrypt(plaintext, &nonce, associated_data).unwrap();
        assert!(!encrypted.ciphertext().is_empty());

        // Decrypt
        let decrypted = cipher.decrypt(&encrypted, associated_data).unwrap();
        assert_eq!(decrypted, plaintext);
    }

    #[test]
    fn test_chacha20_poly1305_wrong_key() {
        let key1 = [0x42; 32];
        let key2 = [0x43; 32];
        let plaintext = b"Hello, World!";
        let nonce = Nonce::generate_random().unwrap();

        let cipher1 = ChaCha20Poly1305Cipher::new(&key1).unwrap();
        let encrypted = cipher1.encrypt(plaintext, &nonce, &[]).unwrap();

        let cipher2 = ChaCha20Poly1305Cipher::new(&key2).unwrap();
        let result = cipher2.decrypt(&encrypted, &[]);

        assert!(matches!(result, Err(CryptoError::AuthenticationFailed { .. })));
    }

    #[test]
    fn test_chacha20_poly1305_tampered_ciphertext() {
        let key = [0x42; 32];
        let plaintext = b"Hello, World!";
        let nonce = Nonce::generate_random().unwrap();

        let cipher = ChaCha20Poly1305Cipher::new(&key).unwrap();
        let mut encrypted = cipher.encrypt(plaintext, &nonce, &[]).unwrap();

        // Tamper with ciphertext
        let mut ciphertext = encrypted.ciphertext().to_vec();
        ciphertext[0] ^= 0xFF;
        encrypted = EncryptedBlob::new(ciphertext, nonce);

        let result = cipher.decrypt(&encrypted, &[]);
        assert!(matches!(result, Err(CryptoError::AuthenticationFailed { .. })));
    }

    #[test]
    fn test_chacha20_poly1305_wrong_associated_data() {
        let key = [0x42; 32];
        let plaintext = b"Hello, World!";
        let nonce = Nonce::generate_random().unwrap();

        let cipher = ChaCha20Poly1305Cipher::new(&key).unwrap();
        let encrypted = cipher.encrypt(plaintext, &nonce, b"correct aad").unwrap();

        let result = cipher.decrypt(&encrypted, b"wrong aad");
        assert!(matches!(result, Err(CryptoError::AuthenticationFailed { .. })));
    }

    #[test]
    fn test_chacha20_poly1305_empty_plaintext() {
        let key = [0x42; 32];
        let plaintext = b"";
        let nonce = Nonce::generate_random().unwrap();

        let cipher = ChaCha20Poly1305Cipher::new(&key).unwrap();
        let encrypted = cipher.encrypt(plaintext, &nonce, &[]).unwrap();

        // Empty plaintext still produces a tag
        assert_eq!(encrypted.ciphertext().len(), 16); // Just the tag

        let decrypted = cipher.decrypt(&encrypted, &[]).unwrap();
        assert_eq!(decrypted, plaintext);
    }

    #[test]
    fn test_chacha20_poly1305_large_plaintext() {
        let key = [0x42; 32];
        let plaintext = vec![0xABu8; 1024 * 1024]; // 1 MB
        let nonce = Nonce::generate_random().unwrap();

        let cipher = ChaCha20Poly1305Cipher::new(&key).unwrap();
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

        let cipher = ChaCha20Poly1305Cipher::new(&key).unwrap();
        let tag = cipher.encrypt_in_place(&mut plaintext, &nonce, &[]).unwrap();

        // After in-place encryption, plaintext buffer contains ciphertext
        assert_ne!(plaintext, original);

        // Decrypt in-place
        cipher.decrypt_in_place(&mut plaintext, &tag, &nonce, &[]).unwrap();

        // After in-place decryption, should be back to original
        assert_eq!(plaintext, original);
    }

    #[test]
    fn test_aes_vs_chacha_interoperability() {
        // Verify that AES and ChaCha produce different ciphertexts for same input
        let key = [0x42; 32];
        let plaintext = b"Test message";
        let nonce = Nonce::generate_random().unwrap();

        let aes_cipher = crate::encryption::Aes256GcmCipher::new(&key).unwrap();
        let chacha_cipher = ChaCha20Poly1305Cipher::new(&key).unwrap();

        let aes_encrypted = aes_cipher.encrypt(plaintext, &nonce, &[]).unwrap();
        let chacha_encrypted = chacha_cipher.encrypt(plaintext, &nonce, &[]).unwrap();

        // Ciphertexts should be different
        assert_ne!(aes_encrypted.ciphertext(), chacha_encrypted.ciphertext());

        // But both should decrypt correctly
        let aes_decrypted = aes_cipher.decrypt(&aes_encrypted, &[]).unwrap();
        let chacha_decrypted = chacha_cipher.decrypt(&chacha_encrypted, &[]).unwrap();

        assert_eq!(aes_decrypted, plaintext);
        assert_eq!(chacha_decrypted, plaintext);
    }
}
