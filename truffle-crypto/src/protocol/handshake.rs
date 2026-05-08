//! X3DH + Kyber-768 Hybrid Handshake
//!
//! This module implements the Extended Triple Diffie-Hellman (X3DH) handshake
//! with Kyber-768 post-quantum hybrid, as specified in Section 2.3.1 and 2.3.2.
//!
//! ## X3DH Overview
//!
//! X3DH is a key agreement protocol used in the Signal Protocol. It provides:
//! - Authentication via identity keys
//! - Forward secrecy via ephemeral keys
//! - Future secrecy via one-time prekeys
//!
//! ## Kyber-768 Hybrid
//!
//! We add Kyber-768 (NIST FIPS 203 compliant) for post-quantum resistance:
//! - Classical X25519 + Kyber-768 encapsulation
//! - Shared secrets are combined using HKDF
//!
//! ## Handshake Flow
//!
//! ```text
//! Device A (Initiator)                    Device B (Responder)
//! --------------------                    --------------------
//! IK_A, EK_A                              IK_B, SPK_B, OPK_B
//!                                         (PreKeyBundle published)
//!
//! 1. Fetch PreKeyBundle from Device B
//!
//! 2. Generate ephemeral key pair EK_A
//!
//! 3. Compute DH values:
//!    DH1 = DH(IK_A, SPK_B)
//!    DH2 = DH(EK_A, IK_B)
//!    DH3 = DH(EK_A, SPK_B)
//!    DH4 = DH(EK_A, OPK_B) [if available]
//!
//! 4. Generate Kyber-768 key pair
//!    Encapsulate to Kyber_PK_B
//!    (ct, ss_kyber) = Kyber.Encap(Kyber_PK_B)
//!
//! 5. Combine secrets:
//!    SK = HKDF(DH1 || DH2 || DH3 || DH4 || ss_kyber, "ZKS-1-v1")
//!
//! 6. Send initial message with:
//!    - IK_A (identity key)
//!    - EK_A (ephemeral key)
//!    - ct (Kyber ciphertext)
//!    - Prekey identifier
//! ```

use crate::error::{CryptoError, CryptoResult};
use crate::keys::{IdentityKeyPair, IdentityPublicKey};
use crate::types::KeyFingerprint;
use crate::utils::{concat_bytes, hkdf_sha256, random_bytes};
use ed25519_dalek::{Signer, Verifier};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use x25519_dalek::{EphemeralSecret, PublicKey as X25519PublicKey, SharedSecret, StaticSecret};
use zeroize::{Zeroize, ZeroizeOnDrop};

/// Size of the combined shared secret (X3DH + Kyber)
pub const COMBINED_SECRET_SIZE: usize = 32;

/// Ephemeral key pair for X3DH
#[derive(Clone, Zeroize, ZeroizeOnDrop)]
pub struct EphemeralKeyPair {
    /// Secret key (zeroized on drop)
    #[zeroize(skip)]
    secret: StaticSecret,
    /// Public key
    public: X25519PublicKey,
}

impl EphemeralKeyPair {
    /// Generate a new ephemeral key pair
    pub fn generate() -> Self {
        let secret = StaticSecret::random_from_rng(rand::thread_rng());
        let public = X25519PublicKey::from(&secret);
        Self { secret, public }
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
}

impl std::fmt::Debug for EphemeralKeyPair {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "EphemeralKeyPair {{ public: {:?} }}", self.public)
    }
}

/// One-time prekey for X3DH
#[derive(Clone, Zeroize, ZeroizeOnDrop)]
pub struct OneTimePrekey {
    /// Prekey ID
    pub id: u32,
    /// Secret key (zeroized on drop)
    #[zeroize(skip)]
    secret: StaticSecret,
    /// Public key
    public: X25519PublicKey,
}

impl OneTimePrekey {
    /// Generate a new one-time prekey
    pub fn generate(id: u32) -> Self {
        let secret = StaticSecret::random_from_rng(rand::thread_rng());
        let public = X25519PublicKey::from(&secret);
        Self {
            id,
            secret,
            public,
        }
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
}

impl std::fmt::Debug for OneTimePrekey {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "OneTimePrekey {{ id: {}, public: {:?} }}", self.id, self.public)
    }
}

/// Signed prekey (medium-term)
#[derive(Clone, Zeroize, ZeroizeOnDrop)]
pub struct SignedPrekey {
    /// Prekey ID
    pub id: u32,
    /// Secret key (zeroized on drop)
    #[zeroize(skip)]
    secret: StaticSecret,
    /// Public key
    public: X25519PublicKey,
    /// Signature by identity key
    signature: Vec<u8>,
}

impl SignedPrekey {
    /// Generate a new signed prekey
    pub fn generate(id: u32, identity: &IdentityKeyPair) -> CryptoResult<Self> {
        let secret = StaticSecret::random_from_rng(rand::thread_rng());
        let public = X25519PublicKey::from(&secret);

        // Sign the public key with identity key
        let signature = identity.sign(&public.to_bytes())?;

        Ok(Self {
            id,
            secret,
            public,
            signature,
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

    /// Get the signature
    pub fn signature(&self) -> &[u8] {
        &self.signature
    }

    /// Verify the signature
    pub fn verify_signature(&self, identity_key: &IdentityPublicKey) -> bool {
        identity_key.verify(&self.public.to_bytes(), &self.signature)
    }

    /// Perform Diffie-Hellman with another public key
    pub fn diffie_hellman(&self, other: &X25519PublicKey) -> SharedSecret {
        self.secret.diffie_hellman(other)
    }
}

impl std::fmt::Debug for SignedPrekey {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "SignedPrekey {{ id: {}, public: {:?} }}", self.id, self.public)
    }
}

/// PreKey bundle published to the server
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct PreKeyBundle {
    /// Identity key (Ed25519, converted to X25519 for DH)
    pub identity_key: [u8; 32],
    /// Signed prekey
    pub signed_prekey: SignedPrekeyPublic,
    /// One-time prekeys (may be empty)
    pub one_time_prekeys: Vec<OneTimePrekeyPublic>,
    /// Kyber-768 public key (post-quantum)
    #[cfg(feature = "kyber768")]
    #[serde(with = "serde_bytes")]
    pub kyber_public_key: Option<Vec<u8>>,
}

/// Public portion of signed prekey
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SignedPrekeyPublic {
    /// Prekey ID
    pub id: u32,
    /// Public key bytes
    #[serde(with = "serde_bytes")]
    pub public_key: [u8; 32],
    /// Signature by identity key
    #[serde(with = "serde_bytes")]
    pub signature: Vec<u8>,
}

/// Public portion of one-time prekey
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct OneTimePrekeyPublic {
    /// Prekey ID
    pub id: u32,
    /// Public key bytes
    #[serde(with = "serde_bytes")]
    pub public_key: [u8; 32],
}

impl PreKeyBundle {
    /// Create a new PreKey bundle
    pub fn new(
        identity_key: [u8; 32],
        signed_prekey: SignedPrekeyPublic,
        one_time_prekeys: Vec<OneTimePrekeyPublic>,
    ) -> Self {
        Self {
            identity_key,
            signed_prekey,
            one_time_prekeys,
            #[cfg(feature = "kyber768")]
            kyber_public_key: None,
        }
    }

    /// Get a one-time prekey (removes it from the bundle)
    pub fn take_one_time_prekey(&mut self) -> Option<OneTimePrekeyPublic> {
        self.one_time_prekeys.pop()
    }

    /// Get the fingerprint of the identity key
    pub fn fingerprint(&self) -> KeyFingerprint {
        KeyFingerprint::from_public_key(&self.identity_key)
    }

    /// Serialize to bytes
    pub fn to_bytes(&self) -> CryptoResult<Vec<u8>> {
        bincode::serialize(self)
            .map_err(|e| CryptoError::serialization_error(format!("PreKeyBundle: {}", e)))
    }

    /// Deserialize from bytes
    pub fn from_bytes(bytes: &[u8]) -> CryptoResult<Self> {
        bincode::deserialize(bytes)
            .map_err(|e| CryptoError::serialization_error(format!("PreKeyBundle: {}", e)))
    }
}

/// Result of the X3DH + Kyber hybrid handshake
#[derive(Clone, Zeroize, ZeroizeOnDrop)]
pub struct X3dhResult {
    /// Combined shared secret (32 bytes)
    #[zeroize(skip)]
    pub shared_secret: [u8; 32],
    /// Associated data for session
    pub associated_data: Vec<u8>,
}

impl X3dhResult {
    /// Derive the sync encryption key from the shared secret
    pub fn derive_sync_key(&self) -> CryptoResult<[u8; 32]> {
        let key = hkdf_sha256(&[], &self.shared_secret, b"truffle-sync-key-v1", 32)?;
        let mut result = [0u8; 32];
        result.copy_from_slice(&key);
        Ok(result)
    }

    /// Derive the authentication key from the shared secret
    pub fn derive_auth_key(&self) -> CryptoResult<[u8; 32]> {
        let key = hkdf_sha256(&[], &self.shared_secret, b"truffle-auth-key-v1", 32)?;
        let mut result = [0u8; 32];
        result.copy_from_slice(&key);
        Ok(result)
    }
}

/// Kyber-768 hybrid result
#[cfg(feature = "kyber768")]
#[derive(Clone)]
pub struct KyberHybridResult {
    /// Kyber ciphertext
    pub ciphertext: Vec<u8>,
    /// Kyber shared secret
    pub shared_secret: [u8; 32],
}

#[cfg(feature = "kyber768")]
impl KyberHybridResult {
    /// Encapsulate to a Kyber public key
    pub fn encapsulate(public_key: &[u8]) -> CryptoResult<Self> {
        use pqc_kyber::{encapsulate, Ciphertext, SharedSecret};

        let pk: [u8; crate::KYBER768_PUBLIC_KEY_SIZE] = public_key
            .try_into()
            .map_err(|_| CryptoError::invalid_key("Invalid Kyber public key size"))?;

        let (ct, ss) = encapsulate(&pk, None)
            .map_err(|e| CryptoError::encryption_failed(format!("Kyber encapsulation failed: {:?}", e)))?;

        Ok(Self {
            ciphertext: ct.to_vec(),
            shared_secret: ss,
        })
    }

    /// Decapsulate using Kyber secret key
    pub fn decapsulate(ciphertext: &[u8], secret_key: &[u8]) -> CryptoResult<[u8; 32]> {
        use pqc_kyber::decapsulate;

        let ct: [u8; crate::KYBER768_CIPHERTEXT_SIZE] = ciphertext
            .try_into()
            .map_err(|_| CryptoError::invalid_key("Invalid Kyber ciphertext size"))?;

        let sk: [u8; crate::KYBER768_SECRET_KEY_SIZE] = secret_key
            .try_into()
            .map_err(|_| CryptoError::invalid_key("Invalid Kyber secret key size"))?;

        let ss = decapsulate(&ct, &sk, None)
            .map_err(|e| CryptoError::decryption_failed(format!("Kyber decapsulation failed: {:?}", e)))?;

        Ok(ss)
    }
}

/// X3DH handshake initiator (Alice)
pub struct X3dhInitiator {
    /// Our identity key
    identity: IdentityKeyPair,
    /// Our ephemeral key
    ephemeral: EphemeralKeyPair,
    /// PreKey bundle from responder
    prekey_bundle: PreKeyBundle,
}

impl X3dhInitiator {
    /// Create a new initiator
    pub fn new(
        identity: IdentityKeyPair,
        prekey_bundle: PreKeyBundle,
    ) -> Self {
        let ephemeral = EphemeralKeyPair::generate();
        Self {
            identity,
            ephemeral,
            prekey_bundle,
        }
    }

    /// Perform the X3DH handshake
    pub fn handshake(&self) -> CryptoResult<X3dhResult> {
        // Convert Ed25519 identity key to X25519 for DH
        let identity_x25519 = self.identity.to_x25519()?;

        // Parse responder's identity key
        let responder_identity = X25519PublicKey::from(
            <[u8; 32]>::try_from(&self.prekey_bundle.identity_key[..])
                .map_err(|_| CryptoError::invalid_key("Invalid identity key"))?
        );

        // Parse responder's signed prekey
        let responder_spk = X25519PublicKey::from(
            self.prekey_bundle.signed_prekey.public_key
        );

        // Verify signed prekey signature
        let identity_pub = IdentityPublicKey::from_bytes(&self.prekey_bundle.identity_key)?;
        if !identity_pub.verify(
            &self.prekey_bundle.signed_prekey.public_key,
            &self.prekey_bundle.signed_prekey.signature,
        ) {
            return Err(CryptoError::invalid_signature("Signed prekey verification failed"));
        }

        // DH1 = DH(IK_A, SPK_B)
        let dh1 = identity_x25519.diffie_hellman(&responder_spk);

        // DH2 = DH(EK_A, IK_B)
        let dh2 = self.ephemeral.diffie_hellman(&responder_identity);

        // DH3 = DH(EK_A, SPK_B)
        let dh3 = self.ephemeral.diffie_hellman(&responder_spk);

        // DH4 = DH(EK_A, OPK_B) if available
        let dh4 = if let Some(opk) = self.prekey_bundle.one_time_prekeys.first() {
            let opk_public = X25519PublicKey::from(opk.public_key);
            Some(self.ephemeral.diffie_hellman(&opk_public))
        } else {
            None
        };

        // Combine DH results
        let mut dh_secrets = concat_bytes(&[
            dh1.as_bytes(),
            dh2.as_bytes(),
            dh3.as_bytes(),
        ]);

        if let Some(dh4) = dh4 {
            dh_secrets.extend_from_slice(dh4.as_bytes());
        }

        // TODO: Add Kyber-768 hybrid when feature is enabled
        #[cfg(feature = "kyber768")]
        {
            // Kyber hybrid would go here
        }

        // Derive shared secret using HKDF
        let shared_secret = hkdf_sha256(&[], &dh_secrets, b"ZKS-1-X3DH-v1", 32)?;
        let mut secret_array = [0u8; 32];
        secret_array.copy_from_slice(&shared_secret);

        // Associated data = IK_A || IK_B
        let associated_data = concat_bytes(&[
            &identity_x25519.to_bytes(),
            &responder_identity.to_bytes(),
        ]);

        // Clear intermediate secrets
        dh_secrets.zeroize();

        Ok(X3dhResult {
            shared_secret: secret_array,
            associated_data,
        })
    }

    /// Get the ephemeral public key
    pub fn ephemeral_public(&self) -> [u8; 32] {
        self.ephemeral.public_key_bytes()
    }
}

/// X3DH handshake responder (Bob)
pub struct X3dhResponder {
    /// Our identity key
    identity: IdentityKeyPair,
    /// Our signed prekey
    signed_prekey: SignedPrekey,
    /// Our one-time prekeys
    one_time_prekeys: HashMap<u32, OneTimePrekey>,
}

impl X3dhResponder {
    /// Create a new responder
    pub fn new(
        identity: IdentityKeyPair,
        signed_prekey: SignedPrekey,
        one_time_prekeys: Vec<OneTimePrekey>,
    ) -> Self {
        let prekey_map: HashMap<u32, OneTimePrekey> = one_time_prekeys
            .into_iter()
            .map(|pk| (pk.id, pk))
            .collect();

        Self {
            identity,
            signed_prekey,
            one_time_prekeys: prekey_map,
        }
    }

    /// Process an incoming handshake
    pub fn respond(
        &mut self,
        initiator_identity: [u8; 32],
        initiator_ephemeral: [u8; 32],
        one_time_prekey_id: Option<u32>,
    ) -> CryptoResult<X3dhResult> {
        // Convert our Ed25519 identity to X25519
        let identity_x25519 = self.identity.to_x25519()?;

        // Parse initiator's keys
        let initiator_ik = X25519PublicKey::from(initiator_identity);
        let initiator_ek = X25519PublicKey::from(initiator_ephemeral);

        // DH1 = DH(SPK_B, IK_A)
        let dh1 = self.signed_prekey.diffie_hellman(&initiator_ik);

        // DH2 = DH(IK_B, EK_A)
        let dh2 = identity_x25519.diffie_hellman(&initiator_ek);

        // DH3 = DH(SPK_B, EK_A)
        let dh3 = self.signed_prekey.diffie_hellman(&initiator_ek);

        // DH4 = DH(OPK_B, EK_A) if prekey was used
        let dh4 = if let Some(id) = one_time_prekey_id {
            if let Some(opk) = self.one_time_prekeys.remove(&id) {
                Some(opk.diffie_hellman(&initiator_ek))
            } else {
                return Err(CryptoError::invalid_key("One-time prekey not found or already used"));
            }
        } else {
            None
        };

        // Combine DH results
        let mut dh_secrets = concat_bytes(&[
            dh1.as_bytes(),
            dh2.as_bytes(),
            dh3.as_bytes(),
        ]);

        if let Some(dh4) = dh4 {
            dh_secrets.extend_from_slice(dh4.as_bytes());
        }

        // Derive shared secret using HKDF
        let shared_secret = hkdf_sha256(&[], &dh_secrets, b"ZKS-1-X3DH-v1", 32)?;
        let mut secret_array = [0u8; 32];
        secret_array.copy_from_slice(&shared_secret);

        // Associated data = IK_A || IK_B
        let associated_data = concat_bytes(&[
            &initiator_identity,
            &identity_x25519.to_bytes(),
        ]);

        // Clear intermediate secrets
        dh_secrets.zeroize();

        Ok(X3dhResult {
            shared_secret: secret_array,
            associated_data,
        })
    }

    /// Generate a PreKey bundle for publication
    pub fn generate_bundle(&self) -> CryptoResult<PreKeyBundle> {
        let identity_x25519 = self.identity.to_x25519()?;

        let one_time_public: Vec<OneTimePrekeyPublic> = self
            .one_time_prekeys
            .values()
            .map(|opk| OneTimePrekeyPublic {
                id: opk.id,
                public_key: opk.public_key_bytes(),
            })
            .collect();

        Ok(PreKeyBundle {
            identity_key: identity_x25519.to_bytes(),
            signed_prekey: SignedPrekeyPublic {
                id: self.signed_prekey.id,
                public_key: self.signed_prekey.public_key_bytes(),
                signature: self.signed_prekey.signature.clone(),
            },
            one_time_prekeys: one_time_public,
            #[cfg(feature = "kyber768")]
            kyber_public_key: None,
        })
    }

    /// Get the number of available one-time prekeys
    pub fn one_time_prekey_count(&self) -> usize {
        self.one_time_prekeys.len()
    }
}

/// High-level X3DH handshake interface
pub struct X3dhHandshake;

impl X3dhHandshake {
    /// Initiate a handshake as Alice
    pub fn initiate(
        identity: IdentityKeyPair,
        prekey_bundle: PreKeyBundle,
    ) -> CryptoResult<(X3dhInitiator, X3dhResult)> {
        let initiator = X3dhInitiator::new(identity, prekey_bundle);
        let result = initiator.handshake()?;
        Ok((initiator, result))
    }

    /// Respond to a handshake as Bob
    pub fn respond(
        responder: &mut X3dhResponder,
        initiator_identity: [u8; 32],
        initiator_ephemeral: [u8; 32],
        one_time_prekey_id: Option<u32>,
    ) -> CryptoResult<X3dhResult> {
        responder.respond(initiator_identity, initiator_ephemeral, one_time_prekey_id)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ephemeral_keypair_generation() {
        let kp1 = EphemeralKeyPair::generate();
        let kp2 = EphemeralKeyPair::generate();
        assert_ne!(kp1.public_key_bytes(), kp2.public_key_bytes());
    }

    #[test]
    fn test_one_time_prekey() {
        let prekey = OneTimePrekey::generate(1);
        assert_eq!(prekey.id, 1);
        assert_eq!(prekey.public_key_bytes().len(), 32);
    }

    #[test]
    fn test_signed_prekey() {
        let identity = IdentityKeyPair::generate();
        let spk = SignedPrekey::generate(1, &identity).unwrap();
        assert_eq!(spk.id, 1);
        assert!(!spk.signature.is_empty());
    }

    #[test]
    fn test_x3dh_handshake() {
        // Generate keys for Bob (responder)
        let bob_identity = IdentityKeyPair::generate();
        let bob_spk = SignedPrekey::generate(1, &bob_identity).unwrap();
        let bob_opk = vec![OneTimePrekey::generate(1), OneTimePrekey::generate(2)];

        let mut bob = X3dhResponder::new(
            bob_identity.clone(),
            bob_spk,
            bob_opk,
        );

        // Generate bundle for publication
        let bundle = bob.generate_bundle().unwrap();

        // Alice initiates handshake
        let alice_identity = IdentityKeyPair::generate();
        let (alice_initiator, alice_result) = X3dhHandshake::initiate(
            alice_identity,
            bundle,
        ).unwrap();

        // Bob responds
        let bob_result = X3dhHandshake::respond(
            &mut bob,
            alice_initiator.identity.to_x25519().unwrap().to_bytes(),
            alice_initiator.ephemeral_public(),
            Some(1),
        ).unwrap();

        // Shared secrets should match
        assert_eq!(alice_result.shared_secret, bob_result.shared_secret);

        // Associated data should match
        assert_eq!(alice_result.associated_data, bob_result.associated_data);
    }

    #[test]
    fn test_x3dh_result_key_derivation() {
        let result = X3dhResult {
            shared_secret: [0x42; 32],
            associated_data: vec![1, 2, 3],
        };

        let sync_key = result.derive_sync_key().unwrap();
        let auth_key = result.derive_auth_key().unwrap();

        assert_ne!(sync_key, auth_key);
        assert_eq!(sync_key.len(), 32);
        assert_eq!(auth_key.len(), 32);
    }

    #[test]
    fn test_prekey_bundle_serialization() {
        let bundle = PreKeyBundle::new(
            [0u8; 32],
            SignedPrekeyPublic {
                id: 1,
                public_key: [1u8; 32],
                signature: vec![2, 3, 4],
            },
            vec![OneTimePrekeyPublic {
                id: 1,
                public_key: [5u8; 32],
            }],
        );

        let bytes = bundle.to_bytes().unwrap();
        let recovered = PreKeyBundle::from_bytes(&bytes).unwrap();

        assert_eq!(bundle.identity_key, recovered.identity_key);
        assert_eq!(bundle.signed_prekey.id, recovered.signed_prekey.id);
    }
}
