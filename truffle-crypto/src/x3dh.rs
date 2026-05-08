//! X3DH Key Exchange (Signal Protocol with Post-Quantum Hybrid)
//!
//! This module implements the Extended Triple Diffie-Hellman (X3DH) key agreement
//! protocol from the Signal Protocol, enhanced with Kyber-768 for post-quantum
//! security.
//!
//! ## Protocol Overview
//!
//! X3DH establishes a shared secret key between two parties who mutually
//! authenticate each other based on public keys. Each party has a long-term
//! identity key pair and a medium-term signed prekey pair.
//!
//! ## Post-Quantum Hybrid
//!
//! We combine X25519 with Kyber-768 using the following approach:
//! 1. Perform standard X3DH to get `shared_secret_x25519`
//! 2. Perform Kyber-768 encapsulation to get `shared_secret_kyber`
//! 3. Combine: `final_secret = HKDF(shared_secret_x25519 || shared_secret_kyber)`
//!
//! This provides security even if either X25519 or Kyber-768 is broken.
//!
//! ## Key Types
//!
//! - **Identity Key (IK)**: Long-term Ed25519/X25519 keypair
//! - **Signed Prekey (SPK)**: Medium-term X25519 keypair, signed by IK
//! - **One-Time Prekey (OPK)**: Ephemeral X25519 keypair, deleted after use
//! - **Kyber Key (KK)**: Post-quantum Kyber-768 keypair
//!
//! ## Security Properties
//!
//! - **Authentication**: Both parties verify each other's identity
//! - **Forward Secrecy**: Ephemeral keys provide past message protection
//! - **Future Secrecy**: Ratcheting provides future message protection
//! - **Post-Quantum**: Kyber-768 provides quantum resistance

use crate::{
    keys::{Ed25519Identity, X25519Keypair, Kyber768Keypair, hkdf_derive},
    CryptoError, CryptoResult,
};
use zeroize::{Zeroize, ZeroizeOnDrop};

/// X3DH public bundle (published to server or shared via QR code)
///
/// This contains all the public keys needed for another party to
/// initiate an X3DH handshake.
#[derive(Debug, Clone)]
pub struct X3DHPublicBundle {
    /// Ed25519 identity public key (32 bytes)
    pub identity_key: [u8; 32],
    /// X25519 signed prekey public key (32 bytes)
    pub signed_prekey: [u8; 32],
    /// Ed25519 signature of the signed prekey (64 bytes)
    pub signed_prekey_signature: [u8; 64],
    /// Optional X25519 one-time prekey public key (32 bytes)
    pub one_time_prekey: Option<[u8; 32]>,
    /// Kyber-768 public key (1184 bytes)
    pub kyber_public_key: [u8; 1184],
}

impl X3DHPublicBundle {
    /// Create a new public bundle from components
    pub fn new(
        identity_key: [u8; 32],
        signed_prekey: [u8; 32],
        signed_prekey_signature: [u8; 64],
        one_time_prekey: Option<[u8; 32]>,
        kyber_public_key: [u8; 1184],
    ) -> Self {
        Self {
            identity_key,
            signed_prekey,
            signed_prekey_signature,
            one_time_prekey,
            kyber_public_key,
        }
    }

    /// Serialize to bytes for transmission
    pub fn to_bytes(&self) -> Vec<u8> {
        let mut result = Vec::with_capacity(
            32 + 32 + 64 + 1 + 32 + 1184
        );
        result.extend_from_slice(&self.identity_key);
        result.extend_from_slice(&self.signed_prekey);
        result.extend_from_slice(&self.signed_prekey_signature);

        // One-time prekey presence flag
        if let Some(opk) = self.one_time_prekey {
            result.push(1);
            result.extend_from_slice(&opk);
        } else {
            result.push(0);
        }

        result.extend_from_slice(&self.kyber_public_key);
        result
    }

    /// Deserialize from bytes
    pub fn from_bytes(bytes: &[u8]) -> CryptoResult<Self> {
        if bytes.len() < 32 + 32 + 64 + 1 + 1184 {
            return Err(CryptoError::InvalidCiphertext(
                "Public bundle too short".to_string()
            ));
        }

        let mut offset = 0;

        let mut identity_key = [0u8; 32];
        identity_key.copy_from_slice(&bytes[offset..offset + 32]);
        offset += 32;

        let mut signed_prekey = [0u8; 32];
        signed_prekey.copy_from_slice(&bytes[offset..offset + 32]);
        offset += 32;

        let mut signed_prekey_signature = [0u8; 64];
        signed_prekey_signature.copy_from_slice(&bytes[offset..offset + 64]);
        offset += 64;

        let has_opk = bytes[offset] != 0;
        offset += 1;

        let one_time_prekey = if has_opk {
            let mut opk = [0u8; 32];
            opk.copy_from_slice(&bytes[offset..offset + 32]);
            offset += 32;
            Some(opk)
        } else {
            None
        };

        let mut kyber_public_key = [0u8; 1184];
        kyber_public_key.copy_from_slice(&bytes[offset..offset + 1184]);

        Ok(Self {
            identity_key,
            signed_prekey,
            signed_prekey_signature,
            one_time_prekey,
            kyber_public_key,
        })
    }

    /// Verify the signed prekey signature
    pub fn verify_signature(&self) -> bool {
        let identity = match Ed25519Identity::from_bytes(&self.identity_key, &[0u8; 32]) {
            Ok(id) => id,
            Err(_) => return false,
        };

        identity.verify(&self.signed_prekey, &self.signed_prekey_signature)
    }
}

/// X3DH private bundle (kept secret on device)
///
/// This contains all the secret keys needed to respond to an X3DH handshake.
#[derive(Clone)]
pub struct X3DHPrivateBundle {
    /// Ed25519 identity secret key (32 bytes)
    pub identity_secret: [u8; 32],
    /// X25519 signed prekey secret key (32 bytes)
    pub signed_prekey_secret: [u8; 32],
    /// Optional X25519 one-time prekey secret key (32 bytes)
    pub one_time_prekey_secret: Option<[u8; 32]>,
    /// Kyber-768 secret key (2400 bytes)
    pub kyber_secret_key: [u8; 2400],
}

impl X3DHPrivateBundle {
    /// Create a new private bundle from components
    pub fn new(
        identity_secret: [u8; 32],
        signed_prekey_secret: [u8; 32],
        one_time_prekey_secret: Option<[u8; 32]>,
        kyber_secret_key: [u8; 2400],
    ) -> Self {
        Self {
            identity_secret,
            signed_prekey_secret,
            one_time_prekey_secret,
            kyber_secret_key,
        }
    }
}

impl Zeroize for X3DHPrivateBundle {
    fn zeroize(&mut self) {
        self.identity_secret.zeroize();
        self.signed_prekey_secret.zeroize();
        if let Some(ref mut opk) = self.one_time_prekey_secret {
            opk.zeroize();
        }
        self.kyber_secret_key.zeroize();
    }
}

impl ZeroizeOnDrop for X3DHPrivateBundle {}

/// X3DH key pair bundle (both public and private components)
pub struct X3DHKeyBundle {
    /// Public components
    pub public: X3DHPublicBundle,
    /// Private components
    pub private: X3DHPrivateBundle,
}

impl X3DHKeyBundle {
    /// Generate a new X3DH key bundle
    ///
    /// This generates all the necessary keys for X3DH:
    /// - Ed25519 identity key
    /// - X25519 signed prekey (signed by identity key)
    /// - Optional X25519 one-time prekey
    /// - Kyber-768 keypair
    ///
    /// # Errors
    ///
    /// Returns `CryptoError::RngFailed` if key generation fails.
    pub fn generate() -> CryptoResult<Self> {
        // Generate identity key
        let identity = Ed25519Identity::generate()?;

        // Generate signed prekey
        let signed_prekey = X25519Keypair::generate()?;

        // Sign the signed prekey with identity key
        let signed_prekey_signature = identity.sign(&signed_prekey.public_key);

        // Generate one-time prekey (optional but recommended)
        let one_time_prekey = X25519Keypair::generate()?;

        // Generate Kyber-768 keypair
        let kyber = Kyber768Keypair::generate()?;

        let public = X3DHPublicBundle::new(
            identity.public_key,
            signed_prekey.public_key,
            signed_prekey_signature,
            Some(one_time_prekey.public_key),
            kyber.public_key,
        );

        let private = X3DHPrivateBundle::new(
            identity.secret_key,
            signed_prekey.secret_key,
            Some(one_time_prekey.secret_key),
            kyber.secret_key,
        );

        Ok(Self { public, private })
    }

    /// Generate a new bundle without one-time prekey
    ///
    /// This is useful when one-time prekeys are exhausted.
    pub fn generate_without_opk() -> CryptoResult<Self> {
        // Generate identity key
        let identity = Ed25519Identity::generate()?;

        // Generate signed prekey
        let signed_prekey = X25519Keypair::generate()?;

        // Sign the signed prekey with identity key
        let signed_prekey_signature = identity.sign(&signed_prekey.public_key);

        // Generate Kyber-768 keypair
        let kyber = Kyber768Keypair::generate()?;

        let public = X3DHPublicBundle::new(
            identity.public_key,
            signed_prekey.public_key,
            signed_prekey_signature,
            None,
            kyber.public_key,
        );

        let private = X3DHPrivateBundle::new(
            identity.secret_key,
            signed_prekey.secret_key,
            None,
            kyber.secret_key,
        );

        Ok(Self { public, private })
    }
}

/// X3DH handshake result
#[derive(Debug, Clone)]
pub struct X3DHResult {
    /// The derived shared secret (32 bytes)
    pub shared_secret: [u8; 32],
    /// Associated data for the session (identity keys concatenated)
    pub associated_data: Vec<u8>,
    /// Kyber ciphertext (for responder to decrypt)
    pub kyber_ciphertext: Option<Vec<u8>>,
}

/// X3DH initiator (Alice) state
///
/// Alice initiates the handshake by sending her ephemeral public key
/// and performing the X3DH calculations.
pub struct X3DHInitiator {
    /// Alice's identity keypair
    identity_key: X25519Keypair,
    /// Alice's ephemeral keypair (generated for this handshake)
    ephemeral_key: X25519Keypair,
}

impl X3DHInitiator {
    /// Create a new initiator with the given identity key
    ///
    /// Generates a fresh ephemeral key for this handshake.
    pub fn new(identity_key: X25519Keypair) -> CryptoResult<Self> {
        let ephemeral_key = X25519Keypair::generate()?;
        Ok(Self {
            identity_key,
            ephemeral_key,
        })
    }

    /// Initiate X3DH handshake with Bob's public bundle
    ///
    /// This performs the X3DH calculations from Alice's perspective:
    /// - DH1: Alice's identity key + Bob's signed prekey
    /// - DH2: Alice's ephemeral key + Bob's identity key
    /// - DH3: Alice's ephemeral key + Bob's signed prekey
    /// - DH4: Alice's ephemeral key + Bob's one-time prekey (if available)
    /// - Kyber: Encapsulate to Bob's Kyber public key
    ///
    /// # Arguments
    ///
    /// * `bob_bundle` - Bob's X3DH public bundle
    ///
    /// # Returns
    ///
    /// Returns the X3DH result containing the shared secret and associated data.
    pub fn initiate(&self, bob_bundle: &X3DHPublicBundle) -> CryptoResult<X3DHResult> {
        // Verify Bob's signed prekey
        if !bob_bundle.verify_signature() {
            return Err(CryptoError::SignatureVerificationFailed);
        }

        // DH1: Alice's IK + Bob's SPK
        let dh1 = self.identity_key.diffie_hellman(&bob_bundle.signed_prekey)?;

        // DH2: Alice's EK + Bob's IK
        let dh2 = self.ephemeral_key.diffie_hellman(&bob_bundle.identity_key)?;

        // DH3: Alice's EK + Bob's SPK
        let dh3 = self.ephemeral_key.diffie_hellman(&bob_bundle.signed_prekey)?;

        // DH4: Alice's EK + Bob's OPK (if available)
        let dh4 = if let Some(opk) = bob_bundle.one_time_prekey {
            Some(self.ephemeral_key.diffie_hellman(&opk)?)
        } else {
            None
        };

        // Kyber encapsulation
        let kyber = Kyber768Keypair::from_bytes(&bob_bundle.kyber_public_key, &[0u8; 2400]);
        let (kyber_ciphertext, kyber_shared) = kyber.encapsulate()?;

        // Combine all DH results and Kyber shared secret
        let mut kdf_input = Vec::with_capacity(32 * 4 + 32);
        kdf_input.extend_from_slice(&dh1);
        kdf_input.extend_from_slice(&dh2);
        kdf_input.extend_from_slice(&dh3);
        if let Some(ref dh4_val) = dh4 {
            kdf_input.extend_from_slice(dh4_val);
        }
        kdf_input.extend_from_slice(&kyber_shared);

        // Derive final shared secret
        let derived = hkdf_derive(
            &kdf_input,
            Some(b"truffle-x3dh-v1"),
            b"x3dh-shared-secret",
            32,
        )?;

        let mut shared_secret = [0u8; 32];
        shared_secret.copy_from_slice(&derived);

        // Associated data: IK_A || IK_B
        let mut associated_data = Vec::with_capacity(64);
        associated_data.extend_from_slice(&self.identity_key.public_key);
        associated_data.extend_from_slice(&bob_bundle.identity_key);

        Ok(X3DHResult {
            shared_secret,
            associated_data,
            kyber_ciphertext: Some(kyber_ciphertext),
        })
    }

    /// Get the ephemeral public key to send to Bob
    pub fn ephemeral_public(&self) -> [u8; 32] {
        self.ephemeral_key.public_key
    }
}

/// X3DH responder (Bob) state
///
/// Bob responds to Alice's handshake using his private keys.
pub struct X3DHResponder {
    /// Bob's private bundle
    private_bundle: X3DHPrivateBundle,
}

impl X3DHResponder {
    /// Create a new responder with the given private bundle
    pub fn new(private_bundle: X3DHPrivateBundle) -> Self {
        Self { private_bundle }
    }

    /// Respond to X3DH handshake from Alice
    ///
    /// This performs the X3DH calculations from Bob's perspective:
    /// - DH1: Bob's SPK + Alice's IK
    /// - DH2: Bob's IK + Alice's EK
    /// - DH3: Bob's SPK + Alice's EK
    /// - DH4: Bob's OPK + Alice's EK (if available)
    /// - Kyber: Decapsulate Alice's ciphertext
    ///
    /// # Arguments
    ///
    /// * `alice_identity_key` - Alice's X25519 identity public key
    /// * `alice_ephemeral_key` - Alice's X25519 ephemeral public key
    /// * `kyber_ciphertext` - Alice's Kyber ciphertext (optional for PQ hybrid)
    ///
    /// # Returns
    ///
    /// Returns the X3DH result containing the shared secret and associated data.
    pub fn respond(
        &self,
        alice_identity_key: [u8; 32],
        alice_ephemeral_key: [u8; 32],
        kyber_ciphertext: Option<&[u8]>,
    ) -> CryptoResult<X3DHResult> {
        // Create keypair objects from secrets
        let identity_key = X25519Keypair::from_bytes(
            &alice_identity_key, // We don't know our own public key here, but it's not needed for DH
            &self.private_bundle.identity_secret,
        );

        let signed_prekey = X25519Keypair::from_bytes(
            &[0u8; 32], // Not needed for DH
            &self.private_bundle.signed_prekey_secret,
        );

        // DH1: Bob's SPK + Alice's IK
        let dh1 = signed_prekey.diffie_hellman(&alice_identity_key)?;

        // DH2: Bob's IK + Alice's EK
        let dh2 = identity_key.diffie_hellman(&alice_ephemeral_key)?;

        // DH3: Bob's SPK + Alice's EK
        let dh3 = signed_prekey.diffie_hellman(&alice_ephemeral_key)?;

        // DH4: Bob's OPK + Alice's EK (if available)
        let dh4 = if let Some(opk_secret) = self.private_bundle.one_time_prekey_secret {
            let opk = X25519Keypair::from_bytes(&[0u8; 32], &opk_secret);
            Some(opk.diffie_hellman(&alice_ephemeral_key)?)
        } else {
            None
        };

        // Kyber decapsulation (if ciphertext provided)
        let kyber_shared = if let Some(ct) = kyber_ciphertext {
            let kyber = Kyber768Keypair::from_bytes(&[0u8; 1184], &self.private_bundle.kyber_secret_key);
            kyber.decapsulate(ct)?
        } else {
            [0u8; 32]
        };

        // Combine all DH results and Kyber shared secret
        let mut kdf_input = Vec::with_capacity(32 * 4 + 32);
        kdf_input.extend_from_slice(&dh1);
        kdf_input.extend_from_slice(&dh2);
        kdf_input.extend_from_slice(&dh3);
        if let Some(ref dh4_val) = dh4 {
            kdf_input.extend_from_slice(dh4_val);
        }
        kdf_input.extend_from_slice(&kyber_shared);

        // Derive final shared secret
        let derived = hkdf_derive(
            &kdf_input,
            Some(b"truffle-x3dh-v1"),
            b"x3dh-shared-secret",
            32,
        )?;

        let mut shared_secret = [0u8; 32];
        shared_secret.copy_from_slice(&derived);

        // Associated data: IK_A || IK_B
        let mut associated_data = Vec::with_capacity(64);
        associated_data.extend_from_slice(&alice_identity_key);
        associated_data.extend_from_slice(&[0u8; 32]); // Our identity key (we need to get this from elsewhere)

        Ok(X3DHResult {
            shared_secret,
            associated_data,
            kyber_ciphertext: kyber_ciphertext.map(|c| c.to_vec()),
        })
    }

    /// Respond to X3DH handshake with full information
    ///
    /// This variant includes Bob's identity public key for proper AD calculation.
    pub fn respond_full(
        &self,
        bob_identity_public: [u8; 32],
        alice_identity_key: [u8; 32],
        alice_ephemeral_key: [u8; 32],
        kyber_ciphertext: Option<&[u8]>,
    ) -> CryptoResult<X3DHResult> {
        let mut result = self.respond(alice_identity_key, alice_ephemeral_key, kyber_ciphertext)?;

        // Update associated data with correct Bob identity
        result.associated_data.clear();
        result.associated_data.extend_from_slice(&alice_identity_key);
        result.associated_data.extend_from_slice(&bob_identity_public);

        Ok(result)
    }
}

/// Convenience function to perform X3DH initiation
///
/// # Arguments
///
/// * `identity_key` - Alice's X25519 identity keypair
/// * `bob_bundle` - Bob's X3DH public bundle
///
/// # Returns
///
/// Returns the X3DH result and Alice's ephemeral public key.
pub fn x3dh_initiate(
    identity_key: &X25519Keypair,
    bob_bundle: &X3DHPublicBundle,
) -> CryptoResult<(X3DHResult, [u8; 32])> {
    let initiator = X3DHInitiator::new(identity_key.clone())?;
    let result = initiator.initiate(bob_bundle)?;
    let ephemeral_public = initiator.ephemeral_public();

    Ok((result, ephemeral_public))
}

/// Convenience function to perform X3DH response
///
/// # Arguments
///
/// * `private_bundle` - Bob's X3DH private bundle
/// * `bob_identity_public` - Bob's X25519 identity public key
/// * `alice_identity_key` - Alice's X25519 identity public key
/// * `alice_ephemeral_key` - Alice's X25519 ephemeral public key
/// * `kyber_ciphertext` - Alice's Kyber ciphertext (optional)
///
/// # Returns
///
/// Returns the X3DH result.
pub fn x3dh_respond(
    private_bundle: &X3DHPrivateBundle,
    bob_identity_public: [u8; 32],
    alice_identity_key: [u8; 32],
    alice_ephemeral_key: [u8; 32],
    kyber_ciphertext: Option<&[u8]>,
) -> CryptoResult<X3DHResult> {
    let responder = X3DHResponder::new(private_bundle.clone());
    responder.respond_full(
        bob_identity_public,
        alice_identity_key,
        alice_ephemeral_key,
        kyber_ciphertext,
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_x3dh_bundle_generation() {
        let bundle = X3DHKeyBundle::generate().unwrap();

        // Verify the signed prekey signature
        assert!(bundle.public.verify_signature());

        // Check that one-time prekey is present
        assert!(bundle.public.one_time_prekey.is_some());
    }

    #[test]
    fn test_x3dh_full_handshake() {
        // Generate Alice's keys
        let alice_identity = X25519Keypair::generate().unwrap();

        // Generate Bob's bundle
        let bob_bundle = X3DHKeyBundle::generate().unwrap();

        // Alice initiates
        let (alice_result, alice_ephemeral) = x3dh_initiate(&alice_identity, &bob_bundle.public).unwrap();

        // Bob responds
        let bob_result = x3dh_respond(
            &bob_bundle.private,
            bob_bundle.public.identity_key,
            alice_identity.public_key,
            alice_ephemeral,
            alice_result.kyber_ciphertext.as_deref(),
        ).unwrap();

        // Both should have the same shared secret
        assert_eq!(alice_result.shared_secret, bob_result.shared_secret);
    }

    #[test]
    fn test_x3dh_without_one_time_prekey() {
        // Generate Alice's keys
        let alice_identity = X25519Keypair::generate().unwrap();

        // Generate Bob's bundle without OPK
        let bob_bundle = X3DHKeyBundle::generate_without_opk().unwrap();

        // Verify no OPK
        assert!(bob_bundle.public.one_time_prekey.is_none());

        // Alice initiates
        let (alice_result, alice_ephemeral) = x3dh_initiate(&alice_identity, &bob_bundle.public).unwrap();

        // Bob responds
        let bob_result = x3dh_respond(
            &bob_bundle.private,
            bob_bundle.public.identity_key,
            alice_identity.public_key,
            alice_ephemeral,
            alice_result.kyber_ciphertext.as_deref(),
        ).unwrap();

        // Both should have the same shared secret
        assert_eq!(alice_result.shared_secret, bob_result.shared_secret);
    }

    #[test]
    fn test_x3dh_bundle_serialization() {
        let bundle = X3DHKeyBundle::generate().unwrap();
        let bytes = bundle.public.to_bytes();
        let deserialized = X3DHPublicBundle::from_bytes(&bytes).unwrap();

        assert_eq!(bundle.public.identity_key, deserialized.identity_key);
        assert_eq!(bundle.public.signed_prekey, deserialized.signed_prekey);
        assert_eq!(bundle.public.kyber_public_key, deserialized.kyber_public_key);
    }

    #[test]
    fn test_invalid_signature_fails() {
        // Generate Alice's keys
        let alice_identity = X25519Keypair::generate().unwrap();

        // Generate Bob's bundle
        let mut bob_bundle = X3DHKeyBundle::generate().unwrap();

        // Corrupt the signature
        bob_bundle.public.signed_prekey_signature[0] ^= 0xFF;

        // Alice's initiation should fail
        let result = x3dh_initiate(&alice_identity, &bob_bundle.public);
        assert!(result.is_err());
    }

    #[test]
    fn test_x3dh_result_has_kyber() {
        let alice_identity = X25519Keypair::generate().unwrap();
        let bob_bundle = X3DHKeyBundle::generate().unwrap();

        let (result, _) = x3dh_initiate(&alice_identity, &bob_bundle.public).unwrap();

        // Should have Kyber ciphertext
        assert!(result.kyber_ciphertext.is_some());
        assert!(!result.kyber_ciphertext.unwrap().is_empty());
    }
}
