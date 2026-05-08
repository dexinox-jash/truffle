//! Symmetric Encryption Primitives
//!
//! This module provides authenticated encryption using:
//! - **AES-256-GCM**: Hardware-accelerated on most modern CPUs
//! - **ChaCha20-Poly1305**: Constant-time, side-channel resistant
//!
//! Both algorithms provide:
//! - **Confidentiality**: 256-bit security level
//! - **Authentication**: Built-in MAC prevents tampering
//! - **Nonce-misuse resistance**: Unique nonces enforced
//!
//! ## Security Considerations
//!
//! 1. **Never reuse nonces** with the same key - this destroys security
//! 2. **Always verify authentication tags** before decrypting
//! 3. **Use ChaCha20-Poly1305 on mobile** for side-channel resistance
//! 4. **Use AES-256-GCM on desktop** for hardware acceleration

use crate::{CryptoError, CryptoResult, AES_KEY_SIZE, AES_NONCE_SIZE, CHACHA_KEY_SIZE, CHACHA_NONCE_SIZE};
use zeroize::{Zeroize, ZeroizeOnDrop};

/// Encrypted data structure with all necessary components
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EncryptedData {
    /// Algorithm identifier (1 = AES-256-GCM, 2 = ChaCha20-Poly1305)
    pub algorithm: u8,
    /// Nonce/IV used for encryption
    pub nonce: Vec<u8>,
    /// Ciphertext
    pub ciphertext: Vec<u8>,
    /// Authentication tag
    pub tag: Vec<u8>,
}

impl EncryptedData {
    /// Serialize to bytes for storage/transmission
    ///
    /// Format: [algorithm:1][nonce_len:1][nonce:N][tag_len:1][tag:T][ciphertext:...]
    pub fn to_bytes(&self) -> Vec<u8> {
        let mut result = Vec::with_capacity(
            1 + 1 + self.nonce.len() + 1 + self.tag.len() + self.ciphertext.len()
        );
        result.push(self.algorithm);
        result.push(self.nonce.len() as u8);
        result.extend_from_slice(&self.nonce);
        result.push(self.tag.len() as u8);
        result.extend_from_slice(&self.tag);
        result.extend_from_slice(&self.ciphertext);
        result
    }

    /// Deserialize from bytes
    ///
    /// # Errors
    ///
    /// Returns `CryptoError::InvalidCiphertext` if the format is invalid.
    pub fn from_bytes(bytes: &[u8]) -> CryptoResult<Self> {
        if bytes.len() < 4 {
            return Err(CryptoError::InvalidCiphertext(
                "Data too short".to_string()
            ));
        }

        let algorithm = bytes[0];
        let nonce_len = bytes[1] as usize;

        if bytes.len() < 3 + nonce_len {
            return Err(CryptoError::InvalidCiphertext(
                "Invalid nonce length".to_string()
            ));
        }

        let nonce = bytes[2..2 + nonce_len].to_vec();
        let tag_len = bytes[2 + nonce_len] as usize;

        if bytes.len() < 3 + nonce_len + 1 + tag_len {
            return Err(CryptoError::InvalidCiphertext(
                "Invalid tag length".to_string()
            ));
        }

        let tag_start = 3 + nonce_len;
        let tag = bytes[tag_start..tag_start + tag_len].to_vec();
        let ciphertext = bytes[tag_start + tag_len..].to_vec();

        Ok(EncryptedData {
            algorithm,
            nonce,
            ciphertext,
            tag,
        })
    }
}

/// AES-256-GCM encryption key
///
/// This type ensures the key is properly sized and securely zeroized when dropped.
#[derive(Clone)]
pub struct Aes256Key([u8; AES_KEY_SIZE]);

impl Aes256Key {
    /// Create a new AES-256 key from bytes
    ///
    /// # Errors
    ///
    /// Returns `CryptoError::InvalidKeySize` if the input is not 32 bytes.
    pub fn new(bytes: &[u8]) -> CryptoResult<Self> {
        if bytes.len() != AES_KEY_SIZE {
            return Err(CryptoError::InvalidKeySize {
                expected: AES_KEY_SIZE,
                actual: bytes.len(),
            });
        }
        let mut key = [0u8; AES_KEY_SIZE];
        key.copy_from_slice(bytes);
        Ok(Self(key))
    }

    /// Generate a random AES-256 key
    ///
    /// # Errors
    ///
    /// Returns `CryptoError::RngFailed` if random generation fails.
    pub fn generate() -> CryptoResult<Self> {
        let key = crate::generate_random_key()?;
        Ok(Self(key))
    }

    /// Get the key bytes
    pub fn as_bytes(&self) -> &[u8; AES_KEY_SIZE] {
        &self.0
    }

    /// Convert to a ChaCha20 key (same size, different algorithm)
    pub fn to_chacha_key(&self) -> ChaCha20Key {
        ChaCha20Key(self.0)
    }
}

impl Zeroize for Aes256Key {
    fn zeroize(&mut self) {
        self.0.zeroize();
    }
}

impl ZeroizeOnDrop for Aes256Key {}

/// ChaCha20-Poly1305 encryption key
///
/// This type ensures the key is properly sized and securely zeroized when dropped.
#[derive(Clone)]
pub struct ChaCha20Key([u8; CHACHA_KEY_SIZE]);

impl ChaCha20Key {
    /// Create a new ChaCha20 key from bytes
    ///
    /// # Errors
    ///
    /// Returns `CryptoError::InvalidKeySize` if the input is not 32 bytes.
    pub fn new(bytes: &[u8]) -> CryptoResult<Self> {
        if bytes.len() != CHACHA_KEY_SIZE {
            return Err(CryptoError::InvalidKeySize {
                expected: CHACHA_KEY_SIZE,
                actual: bytes.len(),
            });
        }
        let mut key = [0u8; CHACHA_KEY_SIZE];
        key.copy_from_slice(bytes);
        Ok(Self(key))
    }

    /// Generate a random ChaCha20 key
    ///
    /// # Errors
    ///
    /// Returns `CryptoError::RngFailed` if random generation fails.
    pub fn generate() -> CryptoResult<Self> {
        let key = crate::generate_random_key()?;
        Ok(Self(key))
    }

    /// Get the key bytes
    pub fn as_bytes(&self) -> &[u8; CHACHA_KEY_SIZE] {
        &self.0
    }

    /// Convert to an AES-256 key (same size, different algorithm)
    pub fn to_aes_key(&self) -> Aes256Key {
        Aes256Key(self.0)
    }
}

impl Zeroize for ChaCha20Key {
    fn zeroize(&mut self) {
        self.0.zeroize();
    }
}

impl ZeroizeOnDrop for ChaCha20Key {}

/// Encrypt data using AES-256-GCM
///
/// # Arguments
///
/// * `key` - 32-byte encryption key
/// * `plaintext` - Data to encrypt
/// * `associated_data` - Additional authenticated data (not encrypted, but integrity-protected)
///
/// # Returns
///
/// Returns the encrypted data structure containing nonce, ciphertext, and authentication tag.
///
/// # Errors
///
/// Returns `CryptoError::EncryptionFailed` if encryption fails.
///
/// # Example
///
/// ```rust
/// use truffle_crypto::symmetric::{encrypt_aes_gcm, decrypt_aes_gcm};
///
/// let key = [0u8; 32]; // Use a proper random key in production!
/// let plaintext = b"Hello, World!";
/// let aad = b"additional data";
///
/// let encrypted = encrypt_aes_gcm(&key, plaintext, aad).unwrap();
/// let decrypted = decrypt_aes_gcm(&key, &encrypted, aad).unwrap();
///
/// assert_eq!(decrypted, plaintext);
/// ```
pub fn encrypt_aes_gcm(
    key: &[u8; 32],
    plaintext: &[u8],
    associated_data: &[u8],
) -> CryptoResult<EncryptedData> {
    use aes_gcm::{
        aead::{Aead, KeyInit, Payload},
        Aes256Gcm, Nonce,
    };

    // Generate a random nonce
    let nonce_bytes = crate::generate_random_nonce::<AES_NONCE_SIZE>()?;
    let nonce = Nonce::from_slice(&nonce_bytes);

    // Create cipher instance
    let cipher = Aes256Gcm::new_from_slice(key)
        .map_err(|e| CryptoError::EncryptionFailed(format!("Key init failed: {:?}", e)))?;

    // Encrypt with associated data
    let payload = Payload {
        msg: plaintext,
        aad: associated_data,
    };

    let ciphertext = cipher
        .encrypt(nonce, payload)
        .map_err(|e| CryptoError::EncryptionFailed(format!("Encryption failed: {:?}", e)))?;

    // AES-GCM appends the 16-byte tag to the ciphertext
    let (ct, tag) = ciphertext.split_at(ciphertext.len() - 16);

    Ok(EncryptedData {
        algorithm: 1,
        nonce: nonce_bytes.to_vec(),
        ciphertext: ct.to_vec(),
        tag: tag.to_vec(),
    })
}

/// Decrypt data using AES-256-GCM
///
/// # Arguments
///
/// * `key` - 32-byte encryption key
/// * `encrypted` - Encrypted data structure
/// * `associated_data` - Additional authenticated data (must match encryption)
///
/// # Returns
///
/// Returns the decrypted plaintext on success.
///
/// # Errors
///
/// Returns `CryptoError::DecryptionFailed` if authentication fails.
pub fn decrypt_aes_gcm(
    key: &[u8; 32],
    encrypted: &EncryptedData,
    associated_data: &[u8],
) -> CryptoResult<Vec<u8>> {
    use aes_gcm::{
        aead::{Aead, KeyInit, Payload},
        Aes256Gcm, Nonce,
    };

    if encrypted.algorithm != 1 {
        return Err(CryptoError::InvalidCiphertext(
            "Wrong algorithm: expected AES-256-GCM".to_string()
        ));
    }

    if encrypted.nonce.len() != AES_NONCE_SIZE {
        return Err(CryptoError::InvalidNonceSize {
            expected: AES_NONCE_SIZE,
            actual: encrypted.nonce.len(),
        });
    }

    let nonce = Nonce::from_slice(&encrypted.nonce);

    // Create cipher instance
    let cipher = Aes256Gcm::new_from_slice(key)
        .map_err(|_| CryptoError::DecryptionFailed)?;

    // Reconstruct ciphertext with tag
    let mut ct_with_tag = encrypted.ciphertext.clone();
    ct_with_tag.extend_from_slice(&encrypted.tag);

    // Decrypt with associated data
    let payload = Payload {
        msg: &ct_with_tag,
        aad: associated_data,
    };

    cipher
        .decrypt(nonce, payload)
        .map_err(|_| CryptoError::DecryptionFailed)
}

/// Encrypt data using ChaCha20-Poly1305
///
/// ChaCha20-Poly1305 is preferred on mobile devices and platforms without
/// AES hardware acceleration due to its constant-time implementation.
///
/// # Arguments
///
/// * `key` - 32-byte encryption key
/// * `plaintext` - Data to encrypt
/// * `associated_data` - Additional authenticated data (not encrypted, but integrity-protected)
///
/// # Returns
///
/// Returns the encrypted data structure containing nonce, ciphertext, and authentication tag.
///
/// # Errors
///
/// Returns `CryptoError::EncryptionFailed` if encryption fails.
pub fn encrypt_chacha20(
    key: &[u8; 32],
    plaintext: &[u8],
    associated_data: &[u8],
) -> CryptoResult<EncryptedData> {
    use chacha20poly1305::{
        aead::{Aead, KeyInit, Payload},
        ChaCha20Poly1305, Nonce,
    };

    // Generate a random nonce
    let nonce_bytes = crate::generate_random_nonce::<CHACHA_NONCE_SIZE>()?;
    let nonce = Nonce::from_slice(&nonce_bytes);

    // Create cipher instance
    let cipher = ChaCha20Poly1305::new_from_slice(key)
        .map_err(|e| CryptoError::EncryptionFailed(format!("Key init failed: {:?}", e)))?;

    // Encrypt with associated data
    let payload = Payload {
        msg: plaintext,
        aad: associated_data,
    };

    let ciphertext = cipher
        .encrypt(nonce, payload)
        .map_err(|e| CryptoError::EncryptionFailed(format!("Encryption failed: {:?}", e)))?;

    // ChaCha20-Poly1305 appends the 16-byte tag to the ciphertext
    let (ct, tag) = ciphertext.split_at(ciphertext.len() - 16);

    Ok(EncryptedData {
        algorithm: 2,
        nonce: nonce_bytes.to_vec(),
        ciphertext: ct.to_vec(),
        tag: tag.to_vec(),
    })
}

/// Decrypt data using ChaCha20-Poly1305
///
/// # Arguments
///
/// * `key` - 32-byte encryption key
/// * `encrypted` - Encrypted data structure
/// * `associated_data` - Additional authenticated data (must match encryption)
///
/// # Returns
///
/// Returns the decrypted plaintext on success.
///
/// # Errors
///
/// Returns `CryptoError::DecryptionFailed` if authentication fails.
pub fn decrypt_chacha20(
    key: &[u8; 32],
    encrypted: &EncryptedData,
    associated_data: &[u8],
) -> CryptoResult<Vec<u8>> {
    use chacha20poly1305::{
        aead::{Aead, KeyInit, Payload},
        ChaCha20Poly1305, Nonce,
    };

    if encrypted.algorithm != 2 {
        return Err(CryptoError::InvalidCiphertext(
            "Wrong algorithm: expected ChaCha20-Poly1305".to_string()
        ));
    }

    if encrypted.nonce.len() != CHACHA_NONCE_SIZE {
        return Err(CryptoError::InvalidNonceSize {
            expected: CHACHA_NONCE_SIZE,
            actual: encrypted.nonce.len(),
        });
    }

    let nonce = Nonce::from_slice(&encrypted.nonce);

    // Create cipher instance
    let cipher = ChaCha20Poly1305::new_from_slice(key)
        .map_err(|_| CryptoError::DecryptionFailed)?;

    // Reconstruct ciphertext with tag
    let mut ct_with_tag = encrypted.ciphertext.clone();
    ct_with_tag.extend_from_slice(&encrypted.tag);

    // Decrypt with associated data
    let payload = Payload {
        msg: &ct_with_tag,
        aad: associated_data,
    };

    cipher
        .decrypt(nonce, payload)
        .map_err(|_| CryptoError::DecryptionFailed)
}

/// Convenience function to encrypt with algorithm auto-detection
///
/// On desktop platforms, uses AES-256-GCM for hardware acceleration.
/// On mobile platforms, uses ChaCha20-Poly1305 for side-channel resistance.
///
/// # Arguments
///
/// * `key` - 32-byte encryption key
/// * `plaintext` - Data to encrypt
/// * `associated_data` - Additional authenticated data
/// * `prefer_chacha` - If true, use ChaCha20-Poly1305; otherwise use AES-256-GCM
///
/// # Returns
///
/// Returns the encrypted data structure.
pub fn encrypt_auto(
    key: &[u8; 32],
    plaintext: &[u8],
    associated_data: &[u8],
    prefer_chacha: bool,
) -> CryptoResult<EncryptedData> {
    if prefer_chacha {
        encrypt_chacha20(key, plaintext, associated_data)
    } else {
        encrypt_aes_gcm(key, plaintext, associated_data)
    }
}

/// Convenience function to decrypt with algorithm auto-detection
///
/// The algorithm is determined from the encrypted data structure.
///
/// # Arguments
///
/// * `key` - 32-byte encryption key
/// * `encrypted` - Encrypted data structure
/// * `associated_data` - Additional authenticated data
///
/// # Returns
///
/// Returns the decrypted plaintext.
pub fn decrypt_auto(
    key: &[u8; 32],
    encrypted: &EncryptedData,
    associated_data: &[u8],
) -> CryptoResult<Vec<u8>> {
    match encrypted.algorithm {
        1 => decrypt_aes_gcm(key, encrypted, associated_data),
        2 => decrypt_chacha20(key, encrypted, associated_data),
        _ => Err(CryptoError::InvalidCiphertext(
            format!("Unknown algorithm: {}", encrypted.algorithm)
        )),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_aes_gcm_roundtrip() {
        let key = [0x42u8; 32];
        let plaintext = b"Hello, World! This is a test message.";
        let aad = b"additional authenticated data";

        let encrypted = encrypt_aes_gcm(&key, plaintext, aad).unwrap();
        let decrypted = decrypt_aes_gcm(&key, &encrypted, aad).unwrap();

        assert_eq!(decrypted, plaintext.as_slice());
    }

    #[test]
    fn test_chacha20_roundtrip() {
        let key = [0x42u8; 32];
        let plaintext = b"Hello, World! This is a test message.";
        let aad = b"additional authenticated data";

        let encrypted = encrypt_chacha20(&key, plaintext, aad).unwrap();
        let decrypted = decrypt_chacha20(&key, &encrypted, aad).unwrap();

        assert_eq!(decrypted, plaintext.as_slice());
    }

    #[test]
    fn test_aes_gcm_wrong_key() {
        let key1 = [0x42u8; 32];
        let key2 = [0x43u8; 32];
        let plaintext = b"secret message";
        let aad = b"";

        let encrypted = encrypt_aes_gcm(&key1, plaintext, aad).unwrap();
        let result = decrypt_aes_gcm(&key2, &encrypted, aad);

        assert!(result.is_err());
    }

    #[test]
    fn test_aes_gcm_tampered_ciphertext() {
        let key = [0x42u8; 32];
        let plaintext = b"secret message";
        let aad = b"";

        let mut encrypted = encrypt_aes_gcm(&key, plaintext, aad).unwrap();
        encrypted.ciphertext[0] ^= 0xFF; // Flip bits

        let result = decrypt_aes_gcm(&key, &encrypted, aad);
        assert!(result.is_err());
    }

    #[test]
    fn test_aes_gcm_wrong_aad() {
        let key = [0x42u8; 32];
        let plaintext = b"secret message";
        let aad1 = b"correct aad";
        let aad2 = b"wrong aad";

        let encrypted = encrypt_aes_gcm(&key, plaintext, aad1).unwrap();
        let result = decrypt_aes_gcm(&key, &encrypted, aad2);

        assert!(result.is_err());
    }

    #[test]
    fn test_encrypted_data_serialization() {
        let key = [0x42u8; 32];
        let plaintext = b"test message";
        let aad = b"";

        let encrypted = encrypt_aes_gcm(&key, plaintext, aad).unwrap();
        let bytes = encrypted.to_bytes();
        let deserialized = EncryptedData::from_bytes(&bytes).unwrap();

        assert_eq!(encrypted, deserialized);

        let decrypted = decrypt_aes_gcm(&key, &deserialized, aad).unwrap();
        assert_eq!(decrypted, plaintext.as_slice());
    }

    #[test]
    fn test_auto_encrypt_decrypt() {
        let key = [0x42u8; 32];
        let plaintext = b"test message for auto detection";
        let aad = b"aad";

        // Test AES path
        let encrypted_aes = encrypt_auto(&key, plaintext, aad, false).unwrap();
        assert_eq!(encrypted_aes.algorithm, 1);
        let decrypted_aes = decrypt_auto(&key, &encrypted_aes, aad).unwrap();
        assert_eq!(decrypted_aes, plaintext.as_slice());

        // Test ChaCha path
        let encrypted_chacha = encrypt_auto(&key, plaintext, aad, true).unwrap();
        assert_eq!(encrypted_chacha.algorithm, 2);
        let decrypted_chacha = decrypt_auto(&key, &encrypted_chacha, aad).unwrap();
        assert_eq!(decrypted_chacha, plaintext.as_slice());
    }

    #[test]
    fn test_key_zeroize() {
        let mut key = Aes256Key::generate().unwrap();
        let bytes_before = key.as_bytes().clone();
        assert_ne!(bytes_before, [0u8; 32]);

        key.zeroize();
        // After zeroize, the key should be all zeros
        assert_eq!(key.as_bytes(), &[0u8; 32]);
    }

    #[test]
    fn test_empty_plaintext() {
        let key = [0x42u8; 32];
        let plaintext = b"";
        let aad = b"";

        let encrypted = encrypt_aes_gcm(&key, plaintext, aad).unwrap();
        let decrypted = decrypt_aes_gcm(&key, &encrypted, aad).unwrap();

        assert_eq!(decrypted, plaintext.as_slice());
    }

    #[test]
    fn test_large_plaintext() {
        let key = [0x42u8; 32];
        let plaintext = vec![0xABu8; 1024 * 1024]; // 1MB
        let aad = b"large data test";

        let encrypted = encrypt_aes_gcm(&key, &plaintext, aad).unwrap();
        let decrypted = decrypt_aes_gcm(&key, &encrypted, aad).unwrap();

        assert_eq!(decrypted, plaintext);
    }
}
