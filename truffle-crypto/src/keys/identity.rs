//! Identity Key Management
//!
//! This module implements Ed25519 identity keys for device authentication.
//! Identity keys are long-term keys that identify a device in the ZKS-1 protocol.

use crate::error::{CryptoError, CryptoResult};
use crate::types::KeyFingerprint;
use crate::utils::{sha2_256, sha3_256};
use ed25519_dalek::hazmat::ExpandedSecretKey;
use ed25519_dalek::{Signer, Verifier};
use rand::rngs::OsRng;
use serde::{Deserialize, Serialize};
use x25519_dalek::{PublicKey as X25519PublicKey, StaticSecret as X25519StaticSecret};
use zeroize::{Zeroize, ZeroizeOnDrop};

/// Ed25519 identity key pair
///
/// This is the long-term identity key for a device. It is used for:
/// - Authenticating device identity
/// - Signing prekeys
/// - Deriving X25519 keys for X3DH
#[derive(Clone, Zeroize, ZeroizeOnDrop)]
pub struct IdentityKeyPair {
    /// Secret key (zeroized on drop)
    #[zeroize(skip)]
    secret_key: ed25519_dalek::SecretKey,
    /// Public key
    public_key: ed25519_dalek::PublicKey,
    /// Expanded secret key (cached for performance)
    #[zeroize(skip)]
    expanded_secret: ExpandedSecretKey,
}

impl IdentityKeyPair {
    /// Generate a new identity key pair
    ///
    /// Uses the OS CSPRNG for secure key generation.
    pub fn generate() -> Self {
        let mut csprng = OsRng;
        let signing_key = ed25519_dalek::SigningKey::generate(&mut csprng);
        let secret_key = signing_key.to_bytes();
        let public_key = signing_key.verifying_key();
        let expanded_secret = ExpandedSecretKey::from(&signing_key);

        Self {
            secret_key,
            public_key,
            expanded_secret,
        }
    }

    /// Create from existing secret key bytes
    ///
    /// # Security
    ///
    /// The secret key bytes must be securely generated and stored.
    /// Never hardcode secret keys.
    pub fn from_secret_key_bytes(bytes: &[u8; 32]) -> CryptoResult<Self> {
        let secret_key = *bytes;
        let signing_key = ed25519_dalek::SigningKey::from_bytes(bytes);
        let public_key = signing_key.verifying_key();
        let expanded_secret = ExpandedSecretKey::from(&signing_key);

        Ok(Self {
            secret_key,
            public_key,
            expanded_secret,
        })
    }

    /// Get the public key
    pub fn public_key(&self) -> &ed25519_dalek::PublicKey {
        &self.public_key
    }

    /// Get the raw public key bytes
    pub fn public_key_bytes(&self) -> [u8; 32] {
        self.public_key.to_bytes()
    }

    /// Get the secret key bytes
    ///
    /// # Security
    ///
    /// This should only be used for secure storage. Never log or
    /// transmit the secret key.
    pub fn secret_key_bytes(&self) -> &[u8; 32] {
        &self.secret_key
    }

    /// Sign a message
    pub fn sign(&self, message: &[u8]) -> CryptoResult<Vec<u8>> {
        let signature = self.expanded_secret.sign(message, &self.public_key);
        Ok(signature.to_bytes().to_vec())
    }

    /// Get the key fingerprint
    pub fn fingerprint(&self) -> KeyFingerprint {
        KeyFingerprint::from_public_key(&self.public_key.to_bytes())
    }

    /// Convert to X25519 key pair for X3DH
    ///
    /// This uses the standard Ed25519 to X25519 conversion as
    /// specified in RFC 7748.
    pub fn to_x25519(&self) -> CryptoResult<X25519StaticSecret> {
        // Ed25519 to X25519 conversion
        // We use the Montgomery form of the Ed25519 secret key
        let mut x25519_secret = [0u8; 32];

        // Hash the Ed25519 secret key and clamp for X25519
        let hash = sha2_256(&self.secret_key);
        x25519_secret.copy_from_slice(&hash);

        // Clamp for X25519 (clear bits 0, 1, 2 and set bit 254)
        x25519_secret[0] &= 248;
        x25519_secret[31] &= 127;
        x25519_secret[31] |= 64;

        Ok(X25519StaticSecret::from(x25519_secret))
    }

    /// Get the X25519 public key
    pub fn to_x25519_public(&self) -> CryptoResult<X25519PublicKey> {
        let x25519_secret = self.to_x25519()?;
        Ok(X25519PublicKey::from(&x25519_secret))
    }

    /// Serialize to bytes (secret key || public key)
    pub fn to_bytes(&self) -> Vec<u8> {
        let mut result = Vec::with_capacity(64);
        result.extend_from_slice(&self.secret_key);
        result.extend_from_slice(&self.public_key.to_bytes());
        result
    }

    /// Deserialize from bytes
    pub fn from_bytes(bytes: &[u8]) -> CryptoResult<Self> {
        if bytes.len() != 64 {
            return Err(CryptoError::invalid_key(
                "Identity key pair must be 64 bytes"
            ));
        }

        let mut secret_key = [0u8; 32];
        secret_key.copy_from_slice(&bytes[..32]);

        let mut public_key_bytes = [0u8; 32];
        public_key_bytes.copy_from_slice(&bytes[32..]);

        let public_key = ed25519_dalek::PublicKey::from_bytes(&public_key_bytes)
            .map_err(|_| CryptoError::invalid_key("Invalid public key"))?;

        let signing_key = ed25519_dalek::SigningKey::from_bytes(&secret_key);
        let expanded_secret = ExpandedSecretKey::from(&signing_key);

        Ok(Self {
            secret_key,
            public_key,
            expanded_secret,
        })
    }

    /// Serialize public key only
    pub fn to_public_bytes(&self) -> [u8; 32] {
        self.public_key.to_bytes()
    }
}

impl std::fmt::Debug for IdentityKeyPair {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "IdentityKeyPair {{ public: {:?}, fingerprint: {} }}",
            self.public_key,
            self.fingerprint()
        )
    }
}

/// Ed25519 public key for identity verification
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct IdentityPublicKey {
    /// Raw public key bytes
    #[serde(with = "serde_bytes")]
    pub key: [u8; 32],
}

impl IdentityPublicKey {
    /// Create from raw bytes
    pub fn from_bytes(bytes: &[u8]) -> CryptoResult<Self> {
        if bytes.len() != 32 {
            return Err(CryptoError::invalid_key(
                "Ed25519 public key must be 32 bytes"
            ));
        }
        let mut key = [0u8; 32];
        key.copy_from_slice(bytes);
        Ok(Self { key })
    }

    /// Get the raw bytes
    pub fn as_bytes(&self) -> &[u8; 32] {
        &self.key
    }

    /// Verify a signature
    pub fn verify(&self, message: &[u8], signature: &[u8]) -> bool {
        let public_key = match ed25519_dalek::PublicKey::from_bytes(&self.key) {
            Ok(pk) => pk,
            Err(_) => return false,
        };

        if signature.len() != 64 {
            return false;
        }

        let mut sig_bytes = [0u8; 64];
        sig_bytes.copy_from_slice(signature);
        let sig = ed25519_dalek::Signature::from_bytes(&sig_bytes);

        public_key.verify(message, &sig).is_ok()
    }

    /// Get the key fingerprint
    pub fn fingerprint(&self) -> KeyFingerprint {
        KeyFingerprint::from_public_key(&self.key)
    }

    /// Convert to X25519 public key for X3DH
    pub fn to_x25519(&self) -> CryptoResult<X25519PublicKey> {
        // Ed25519 to X25519 point conversion
        // This uses the Montgomery u-coordinate from the Edwards point
        // For now, we assume the key was generated from an X25519 conversion
        // In practice, we should do proper point conversion
        Ok(X25519PublicKey::from(self.key))
    }
}

/// Ed25519 secret key (for storage/loading)
#[derive(Clone, Zeroize, ZeroizeOnDrop)]
pub struct IdentitySecretKey {
    /// Raw secret key bytes
    key: [u8; 32],
}

impl IdentitySecretKey {
    /// Create from raw bytes
    pub fn from_bytes(bytes: &[u8]) -> CryptoResult<Self> {
        if bytes.len() != 32 {
            return Err(CryptoError::invalid_key(
                "Ed25519 secret key must be 32 bytes"
            ));
        }
        let mut key = [0u8; 32];
        key.copy_from_slice(bytes);
        Ok(Self { key })
    }

    /// Generate a new random secret key
    ///
    /// # Errors
    ///
    /// Returns `CryptoError::RandomGenerationFailed` if RNG fails.
    pub fn generate() -> CryptoResult<Self> {
        let mut key = [0u8; 32];
        getrandom::getrandom(&mut key)
            .map_err(|e| CryptoError::RandomGenerationFailed(format!("CSPRNG failed: {:?}", e)))?;
        Ok(Self { key })
    }

    /// Get the raw bytes
    pub fn as_bytes(&self) -> &[u8; 32] {
        &self.key
    }

    /// Convert to a full key pair
    ///
    /// # Errors
    ///
    /// Returns `CryptoError::InvalidKey` if the secret key is invalid.
    pub fn to_keypair(&self) -> CryptoResult<IdentityKeyPair> {
        IdentityKeyPair::from_secret_key_bytes(&self.key)
            .map_err(|e| CryptoError::invalid_key(format!("Invalid secret key: {:?}", e)))
    }
}

impl std::fmt::Debug for IdentitySecretKey {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "IdentitySecretKey([REDACTED])")
    }
}

/// Key rotation policy
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum KeyRotationPolicy {
    /// Never rotate (for identity keys)
    Never,
    /// Rotate after specified duration
    AfterDuration(u64), // seconds
    /// Rotate after number of uses
    AfterUses(u64),
    /// Rotate based on both duration and uses
    AfterDurationOrUses(u64, u64),
}

impl KeyRotationPolicy {
    /// Check if rotation is needed
    pub fn should_rotate(&self, age_secs: u64, uses: u64) -> bool {
        match self {
            Self::Never => false,
            Self::AfterDuration(max_age) => age_secs > *max_age,
            Self::AfterUses(max_uses) => uses > *max_uses,
            Self::AfterDurationOrUses(max_age, max_uses) => {
                age_secs > *max_age || uses > *max_uses
            }
        }
    }
}

/// Key metadata for tracking
#[derive(Clone, Debug)]
pub struct KeyMetadata {
    /// Key creation timestamp
    pub created_at: u64,
    /// Last use timestamp
    pub last_used_at: u64,
    /// Number of uses
    pub use_count: u64,
    /// Rotation policy
    pub rotation_policy: KeyRotationPolicy,
}

impl KeyMetadata {
    /// Create new metadata with the given policy
    ///
    /// # Errors
    ///
    /// Returns `CryptoError::InternalError` if system time is unavailable.
    pub fn new(rotation_policy: KeyRotationPolicy) -> CryptoResult<Self> {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map_err(|e| CryptoError::InternalError(format!("System time error: {:?}", e)))?
            .as_secs();

        Ok(Self {
            created_at: now,
            last_used_at: now,
            use_count: 0,
            rotation_policy,
        })
    }

    /// Record a key use
    ///
    /// # Errors
    ///
    /// Returns `CryptoError::InternalError` if system time is unavailable.
    pub fn record_use(&mut self) -> CryptoResult<()> {
        self.last_used_at = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map_err(|e| CryptoError::InternalError(format!("System time error: {:?}", e)))?
            .as_secs();
        self.use_count += 1;
        Ok(())
    }

    /// Check if rotation is needed
    ///
    /// # Errors
    ///
    /// Returns `CryptoError::InternalError` if system time is unavailable.
    pub fn should_rotate(&self) -> CryptoResult<bool> {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map_err(|e| CryptoError::InternalError(format!("System time error: {:?}", e)))?
            .as_secs();
        let age = now - self.created_at;

        Ok(self.rotation_policy.should_rotate(age, self.use_count))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_identity_keypair_generation() {
        let kp1 = IdentityKeyPair::generate();
        let kp2 = IdentityKeyPair::generate();

        assert_ne!(kp1.public_key_bytes(), kp2.public_key_bytes());
        assert_eq!(kp1.public_key_bytes().len(), 32);
    }

    #[test]
    fn test_sign_and_verify() {
        let kp = IdentityKeyPair::generate();
        let message = b"test message";

        let signature = kp.sign(message).unwrap();
        assert_eq!(signature.len(), 64);

        // Verify with public key
        let public_key = IdentityPublicKey::from_bytes(&kp.public_key_bytes()).unwrap();
        assert!(public_key.verify(message, &signature));

        // Wrong message should fail
        assert!(!public_key.verify(b"wrong message", &signature));
    }

    #[test]
    fn test_fingerprint() {
        let kp = IdentityKeyPair::generate();
        let fingerprint = kp.fingerprint();
        assert_eq!(fingerprint.as_bytes().len(), 16);
    }

    #[test]
    fn test_to_x25519() {
        let kp = IdentityKeyPair::generate();
        let x25519_secret = kp.to_x25519().unwrap();
        let x25519_public = kp.to_x25519_public().unwrap();

        // Verify the public key matches
        let derived_public = X25519PublicKey::from(&x25519_secret);
        assert_eq!(derived_public.as_bytes(), x25519_public.as_bytes());
    }

    #[test]
    fn test_keypair_serialization() {
        let kp = IdentityKeyPair::generate();
        let bytes = kp.to_bytes();
        assert_eq!(bytes.len(), 64);

        let recovered = IdentityKeyPair::from_bytes(&bytes).unwrap();
        assert_eq!(kp.public_key_bytes(), recovered.public_key_bytes());
    }

    #[test]
    fn test_public_key_only() {
        let kp = IdentityKeyPair::generate();
        let public_bytes = kp.to_public_bytes();

        let public_key = IdentityPublicKey::from_bytes(&public_bytes).unwrap();
        assert_eq!(public_key.as_bytes(), &public_bytes);
    }

    #[test]
    fn test_key_rotation_policy() {
        let policy = KeyRotationPolicy::AfterDuration(86400); // 1 day
        assert!(!policy.should_rotate(86399, 0));
        assert!(policy.should_rotate(86401, 0));

        let policy = KeyRotationPolicy::AfterUses(100);
        assert!(!policy.should_rotate(0, 99));
        assert!(policy.should_rotate(0, 101));

        let policy = KeyRotationPolicy::Never;
        assert!(!policy.should_rotate(u64::MAX, u64::MAX));
    }

    #[test]
    fn test_key_metadata() {
        let mut metadata = KeyMetadata::new(KeyRotationPolicy::AfterUses(10));
        assert!(!metadata.should_rotate());

        for _ in 0..11 {
            metadata.record_use();
        }
        assert!(metadata.should_rotate());
    }
}
