//! # Truffle Crypto - Zero-Knowledge Security Layer
//!
//! This crate provides the cryptographic primitives for Project Truffle's
//! zero-knowledge architecture. All operations are designed with the following
//! principles:
//!
//! - **Zero-Knowledge**: Infrastructure operators maintain mathematical inability
//!   to decrypt user content.
//! - **Post-Quantum Security**: Hybrid X3DH with Kyber-768 for future-proofing.
//! - **Constant-Time Operations**: All cryptographic operations resist timing attacks.
//! - **No Key Escrow**: All keys are user-controlled; no backdoors exist.
//!
//! ## Architecture
//!
//! ```text
//! ┌─────────────────────────────────────────────────────────────────┐
//! │                    TRUFFLE CRYPTO LAYER                          │
//! ├─────────────────────────────────────────────────────────────────┤
//! │  Symmetric        │  Asymmetric        │  Key Management        │
//! │  ─────────        │  ──────────        │  ─────────────         │
//! │  AES-256-GCM      │  X3DH (Signal)     │  Secure Enclave        │
//! │  ChaCha20-Poly1305│  Kyber-768 (PQ)    │  HKDF-SHA256           │
//! │  HMAC-SHA256      │  Ed25519/X25519    │  Zeroize-on-drop       │
//! └─────────────────────────────────────────────────────────────────┘
//! ```
//!
//! ## Security Guarantees
//!
//! 1. **Confidentiality**: AES-256-GCM with unique nonces per message.
//! 2. **Authentication**: Ed25519 signatures for device identity.
//! 3. **Integrity**: HMAC-SHA256 for message authentication.
//! 4. **Forward Secrecy**: Ephemeral X25519 keys per session.
//! 5. **Post-Quantum**: Kyber-768 for quantum-resistant key exchange.

#![warn(missing_docs)]
#![warn(unsafe_code)]
// Relax these for now during initial compilation; will re-enable in Phase 2
// #![deny(clippy::unwrap_used)]
// #![deny(clippy::expect_used)]

use zeroize::{Zeroize, ZeroizeOnDrop};

/// Re-export commonly used types
pub use aes_gcm;
pub use chacha20poly1305;
pub use ed25519_dalek;
pub use x25519_dalek;

// ─── Core infrastructure modules ───────────────────────────────────────
/// Error types for cryptographic operations
pub mod error;

/// Common types (DeviceId, KeyFingerprint, etc.)
pub mod types;

/// Utility functions (hashing, HKDF, encoding, etc.)
pub mod utils;

// ─── Cryptographic primitive modules ───────────────────────────────────
/// Symmetric encryption primitives (AES-256-GCM, ChaCha20-Poly1305)
pub mod symmetric;

/// Key generation, derivation, and management
pub mod keys;

/// X3DH key exchange (Signal Protocol with post-quantum hybrid)
pub mod x3dh;

/// Device pairing ceremony with QR codes and SAS verification
pub mod pairing;

/// CRDT encryption for Yjs sync protocol
pub mod crdt_crypto;

/// Export functionality for 5-minute data liberation
pub mod export;

// ─── Higher-level modules ──────────────────────────────────────────────
/// Encryption wrappers (AES-GCM, ChaCha20, CRDT encryption)
pub mod encryption;

/// ZKS-1 protocol implementation (handshake, messaging)
pub mod protocol;

/// Compliance and audit modules (GDPR, SOC2)
pub mod compliance;

// ─── Re-exports for convenience ────────────────────────────────────────
pub use error::{CryptoError, CryptoResult};

// ─── Protocol constants ────────────────────────────────────────────────

/// Version of the ZKS (Zero-Knowledge Sync) protocol
pub const ZKS_PROTOCOL_VERSION: u8 = 1;

/// Size of AES-256 keys in bytes
pub const AES_KEY_SIZE: usize = 32;

/// Size of ChaCha20-Poly1305 keys in bytes
pub const CHACHA_KEY_SIZE: usize = 32;

/// Size of AES-GCM nonces in bytes (96 bits)
pub const AES_NONCE_SIZE: usize = 12;

/// Size of ChaCha20-Poly1305 nonces in bytes (96 bits)
pub const CHACHA_NONCE_SIZE: usize = 12;

/// Size of HMAC-SHA256 output in bytes
pub const HMAC_SIZE: usize = 32;

/// Size of Ed25519 public keys in bytes
pub const ED25519_PUBLIC_KEY_SIZE: usize = 32;

/// Size of Ed25519 secret keys in bytes
pub const ED25519_SECRET_KEY_SIZE: usize = 32;

/// Size of X25519 public keys in bytes
pub const X25519_PUBLIC_KEY_SIZE: usize = 32;

/// Size of X25519 secret keys in bytes
pub const X25519_SECRET_KEY_SIZE: usize = 32;

/// Size of Kyber-768 public keys in bytes
pub const KYBER_PUBLIC_KEY_SIZE: usize = 1184;

/// Size of Kyber-768 secret keys in bytes
pub const KYBER_SECRET_KEY_SIZE: usize = 2400;

/// Size of Kyber-768 ciphertext in bytes
pub const KYBER_CIPHERTEXT_SIZE: usize = 1088;

/// Size of Kyber-768 shared secret in bytes
pub const KYBER_SHARED_SECRET_SIZE: usize = 32;

/// Maximum age of ephemeral keys in seconds (7 days)
pub const MAX_EPHEMERAL_KEY_AGE: u64 = 7 * 24 * 60 * 60;

/// Default number of pre-keys to maintain
pub const DEFAULT_PREKEY_COUNT: usize = 100;

/// Threshold below which new pre-keys are generated
pub const PREKEY_THRESHOLD: usize = 20;

// ─── Top-level convenience functions ───────────────────────────────────

/// Securely zeroizes sensitive data when dropped
///
/// This trait is automatically implemented for all types that implement
/// `ZeroizeOnDrop`, ensuring sensitive cryptographic material is cleared
/// from memory when no longer needed.
pub trait SecureZeroize: Zeroize + ZeroizeOnDrop {}

impl<T> SecureZeroize for T where T: Zeroize + ZeroizeOnDrop {}

/// Constant-time comparison of two byte slices
///
/// Returns true if the slices are equal, false otherwise.
/// This function runs in constant time regardless of the input,
/// preventing timing attacks.
pub fn constant_time_eq(a: &[u8], b: &[u8]) -> bool {
    use subtle::ConstantTimeEq;
    a.ct_eq(b).into()
}

/// Securely generate random bytes using the OS CSPRNG
///
/// This function uses `getrandom` which delegates to the operating
/// system's cryptographically secure random number generator.
///
/// # Errors
///
/// Returns `CryptoError::RandomGenerationFailed` if the OS RNG fails.
pub fn secure_random_bytes(buf: &mut [u8]) -> CryptoResult<()> {
    use rand::RngCore;
    let mut rng = rand::thread_rng();
    rng.try_fill_bytes(buf)
        .map_err(|e| CryptoError::RandomGenerationFailed(e.to_string()))
}

/// Generate a cryptographically secure random 32-byte key
///
/// # Errors
///
/// Returns error if the OS RNG fails.
pub fn generate_random_key() -> CryptoResult<[u8; 32]> {
    let mut key = [0u8; 32];
    secure_random_bytes(&mut key)?;
    Ok(key)
}

/// Generate a cryptographically secure random nonce
///
/// # Errors
///
/// Returns error if the OS RNG fails.
pub fn generate_random_nonce<const N: usize>() -> CryptoResult<[u8; N]> {
    let mut nonce = [0u8; N];
    secure_random_bytes(&mut nonce)?;
    Ok(nonce)
}

/// Hash data using SHA3-256
///
/// Returns the 32-byte hash digest.
pub fn sha3_256_hash(data: &[u8]) -> [u8; 32] {
    use sha3::{Digest, Sha3_256};
    let mut hasher = Sha3_256::new();
    hasher.update(data);
    hasher.finalize().into()
}

/// Hash data using SHA2-256
///
/// Returns the 32-byte hash digest.
pub fn sha2_256_hash(data: &[u8]) -> [u8; 32] {
    use sha2::{Digest, Sha256};
    let mut hasher = Sha256::new();
    hasher.update(data);
    hasher.finalize().into()
}

/// Compute HMAC-SHA256
///
/// Returns the 32-byte HMAC tag.
///
/// # Errors
///
/// Returns `CryptoError::InternalError` if HMAC computation fails (extremely unlikely).
pub fn hmac_sha256(key: &[u8], data: &[u8]) -> CryptoResult<[u8; 32]> {
    use hmac::{Hmac, Mac};
    use sha2::Sha256;

    type HmacSha256 = Hmac<Sha256>;

    let mut mac = HmacSha256::new_from_slice(key)
        .map_err(|e| CryptoError::InternalError(format!("HMAC initialization failed: {:?}", e)))?;
    mac.update(data);
    let result = mac.finalize();
    let bytes: [u8; 32] = result.into_bytes().into();
    Ok(bytes)
}

/// Verify HMAC-SHA256 in constant time
///
/// Returns true if the computed HMAC matches the provided tag.
/// This comparison is done in constant time to prevent timing attacks.
///
/// # Errors
///
/// Returns `CryptoError::InternalError` if HMAC computation fails (extremely unlikely).
pub fn verify_hmac_sha256(key: &[u8], data: &[u8], tag: &[u8; 32]) -> CryptoResult<bool> {
    let computed = hmac_sha256(key, data)?;
    Ok(constant_time_eq(&computed, tag))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_constant_time_eq() {
        let a = [1u8, 2, 3, 4];
        let b = [1u8, 2, 3, 4];
        let c = [1u8, 2, 3, 5];

        assert!(constant_time_eq(&a, &b));
        assert!(!constant_time_eq(&a, &c));
    }

    #[test]
    fn test_secure_random_bytes() {
        let mut buf1 = [0u8; 32];
        let mut buf2 = [0u8; 32];

        secure_random_bytes(&mut buf1).unwrap();
        secure_random_bytes(&mut buf2).unwrap();

        // Probability of collision is negligible
        assert_ne!(buf1, buf2);
    }

    #[test]
    fn test_sha3_256_hash() {
        let data = b"hello world";
        let hash1 = sha3_256_hash(data);
        let hash2 = sha3_256_hash(data);

        assert_eq!(hash1, hash2);
        assert_eq!(hash1.len(), 32);
    }

    #[test]
    fn test_hmac_sha256() {
        let key = b"secret key";
        let data = b"message to authenticate";

        let tag1 = hmac_sha256(key, data);
        let tag2 = hmac_sha256(key, data);

        assert_eq!(tag1, tag2);
        assert!(verify_hmac_sha256(key, data, &tag1));

        // Different data should produce different tags
        let tag3 = hmac_sha256(key, b"different message");
        assert_ne!(tag1, tag3);
    }
}
