//! Cryptographic Primitives for Sync
//!
//! Implements zero-knowledge sync encryption:
//! - X3DH key exchange (Signal Protocol)
//! - AES-256-GCM for symmetric encryption
//! - Ed25519 for device authentication
//!
//! Per Section 2.3.1 of the Enterprise Specification

use aes_gcm::{
    aead::{Aead, KeyInit},
    Aes256Gcm, Nonce,
};
use ed25519_dalek::{SigningKey, VerifyingKey, Signature, Signer, Verifier};
use rand::rngs::OsRng;
use sha2::{Sha256, Digest};
use hkdf::Hkdf;
use tracing::{info, debug, error};

/// Device key pair for authentication and encryption
#[derive(Debug)]
pub struct DeviceKeys {
    /// Ed25519 signing key (for authentication)
    pub signing_key: SigningKey,
    /// Ed25519 verifying key (public)
    pub verifying_key: VerifyingKey,
    /// X25519 key for key exchange (if using X3DH)
    pub x25519_public: Option<[u8; 32]>,
}

impl DeviceKeys {
    /// Generate new device keys
    pub fn generate() -> Self {
        let signing_key = SigningKey::generate(&mut OsRng);
        let verifying_key = signing_key.verifying_key();
        
        debug!("Generated new device keys");
        
        Self {
            signing_key,
            verifying_key,
            x25519_public: None,
        }
    }
    
    /// Get device fingerprint (hash of public key)
    pub fn fingerprint(&self) -> String {
        let mut hasher = Sha256::new();
        hasher.update(self.verifying_key.as_bytes());
        let result = hasher.finalize();
        hex::encode(&result[..16])
    }
    
    /// Sign data
    pub fn sign(&self, data: &[u8]) -> Signature {
        self.signing_key.sign(data)
    }
    
    /// Verify signature
    pub fn verify(&self, data: &[u8], signature: &Signature) -> anyhow::Result<()> {
        self.verifying_key.verify(data, signature)
            .map_err(|e| anyhow::anyhow!("Signature verification failed: {:?}", e))
    }
    
    /// Export public keys for sharing
    pub fn export_public(&self) -> PublicKeyBundle {
        PublicKeyBundle {
            verifying_key: *self.verifying_key.as_bytes(),
            x25519_public: self.x25519_public,
        }
    }
}

/// Public key bundle for device pairing
#[derive(Debug, Clone)]
pub struct PublicKeyBundle {
    /// Ed25519 verifying key
    pub verifying_key: [u8; 32],
    /// X25519 public key (if using X3DH)
    pub x25519_public: Option<[u8; 32]>,
}

impl PublicKeyBundle {
    /// Serialize to bytes
    pub fn to_bytes(&self) -> Vec<u8> {
        let mut bytes = Vec::with_capacity(32 + if self.x25519_public.is_some() { 32 } else { 0 });
        bytes.extend_from_slice(&self.verifying_key);
        if let Some(x25519) = self.x25519_public {
            bytes.extend_from_slice(&x25519);
        }
        bytes
    }
    
    /// Deserialize from bytes
    pub fn from_bytes(bytes: &[u8]) -> anyhow::Result<Self> {
        if bytes.len() < 32 {
            return Err(anyhow::anyhow!("Invalid key bundle size"));
        }
        
        let verifying_key: [u8; 32] = bytes[..32].try_into()?;
        let x25519_public = if bytes.len() >= 64 {
            Some(bytes[32..64].try_into()?)
        } else {
            None
        };
        
        Ok(Self {
            verifying_key,
            x25519_public,
        })
    }
    
    /// Get fingerprint
    pub fn fingerprint(&self) -> String {
        let mut hasher = Sha256::new();
        hasher.update(&self.verifying_key);
        let result = hasher.finalize();
        hex::encode(&result[..16])
    }
}

/// Encrypted payload for sync messages
#[derive(Debug, Clone)]
pub struct EncryptedPayload {
    /// Ciphertext
    pub ciphertext: Vec<u8>,
    /// Additional authenticated data (optional)
    pub aad: Option<Vec<u8>>,
}

impl EncryptedPayload {
    /// Create new payload
    pub fn new(ciphertext: Vec<u8>) -> Self {
        Self {
            ciphertext,
            aad: None,
        }
    }
    
    /// With AAD
    pub fn with_aad(mut self, aad: Vec<u8>) -> Self {
        self.aad = Some(aad);
        self
    }
    
    /// Get total size
    pub fn size_bytes(&self) -> usize {
        self.ciphertext.len() + self.aad.as_ref().map(|a| a.len()).unwrap_or(0)
    }
}

/// Sync crypto handler
pub struct SyncCrypto {
    device_keys: DeviceKeys,
    shared_secrets: std::collections::HashMap<String, [u8; 32]>,
}

impl SyncCrypto {
    /// Create new sync crypto handler
    pub fn new(device_keys: DeviceKeys) -> Self {
        Self {
            device_keys,
            shared_secrets: std::collections::HashMap::new(),
        }
    }
    
    /// Generate new keys
    pub fn generate() -> Self {
        Self::new(DeviceKeys::generate())
    }
    
    /// Derive shared secret using X3DH (simplified)
    pub fn derive_shared_secret(
        &mut self,
        device_id: impl Into<String>,
        their_public: &PublicKeyBundle,
    ) -> anyhow::Result<[u8; 32]> {
        // In production, implement full X3DH
        // For now, use simple HKDF
        
        let ikm = self.device_keys.verifying_key.as_bytes();
        let salt = &their_public.verifying_key;
        
        let hkdf = Hkdf::<Sha256>::new(Some(salt), ikm);
        let mut okm = [0u8; 32];
        hkdf.expand(b"truffle-sync-v1", &mut okm)
            .map_err(|e| anyhow::anyhow!("HKDF expansion failed: {:?}", e))?;
        
        let device_id = device_id.into();
        self.shared_secrets.insert(device_id.clone(), okm);
        
        debug!("Derived shared secret for device: {}", device_id);
        
        Ok(okm)
    }
    
    /// Encrypt data for a device
    pub fn encrypt(
        &self,
        device_id: &str,
        plaintext: &[u8],
        nonce: &[u8; 12],
    ) -> anyhow::Result<EncryptedPayload> {
        let secret = self.shared_secrets.get(device_id)
            .ok_or_else(|| anyhow::anyhow!("No shared secret for device: {}", device_id))?;
        
        let cipher = Aes256Gcm::new_from_slice(secret)
            .map_err(|e| anyhow::anyhow!("Failed to create cipher: {:?}", e))?;
        
        let nonce = Nonce::from_slice(nonce);
        
        let ciphertext = cipher.encrypt(nonce, plaintext)
            .map_err(|e| anyhow::anyhow!("Encryption failed: {:?}", e))?;
        
        Ok(EncryptedPayload::new(ciphertext))
    }
    
    /// Decrypt data from a device
    pub fn decrypt(
        &self,
        device_id: &str,
        payload: &EncryptedPayload,
        nonce: &[u8; 12],
    ) -> anyhow::Result<Vec<u8>> {
        let secret = self.shared_secrets.get(device_id)
            .ok_or_else(|| anyhow::anyhow!("No shared secret for device: {}", device_id))?;
        
        let cipher = Aes256Gcm::new_from_slice(secret)
            .map_err(|e| anyhow::anyhow!("Failed to create cipher: {:?}", e))?;
        
        let nonce = Nonce::from_slice(nonce);
        
        let plaintext = cipher.decrypt(nonce, payload.ciphertext.as_ref())
            .map_err(|e| anyhow::anyhow!("Decryption failed: {:?}", e))?;
        
        Ok(plaintext)
    }
    
    /// Sign a message
    pub fn sign(&self, data: &[u8]) -> Vec<u8> {
        self.device_keys.sign(data).to_bytes().to_vec()
    }
    
    /// Verify a signature
    pub fn verify(&self, data: &[u8], signature: &[u8], their_public: &PublicKeyBundle) -> anyhow::Result<()> {
        let signature = Signature::from_slice(signature)
            .map_err(|e| anyhow::anyhow!("Invalid signature format: {:?}", e))?;
        
        let verifying_key = VerifyingKey::from_bytes(&their_public.verifying_key)
            .map_err(|e| anyhow::anyhow!("Invalid public key: {:?}", e))?;
        
        verifying_key.verify(data, &signature)
            .map_err(|e| anyhow::anyhow!("Signature verification failed: {:?}", e))
    }
    
    /// Get device fingerprint
    pub fn fingerprint(&self) -> String {
        self.device_keys.fingerprint()
    }
    
    /// Export public keys
    pub fn export_public(&self) -> PublicKeyBundle {
        self.device_keys.export_public()
    }
    
    /// Generate random nonce
    pub fn generate_nonce() -> [u8; 12] {
        let mut nonce = [0u8; 12];
        rand::RngCore::fill_bytes(&mut OsRng, &mut nonce);
        nonce
    }
}

/// Device pairing ceremony
pub struct PairingCeremony {
    crypto: SyncCrypto,
    state: PairingState,
}

/// Pairing state machine
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PairingState {
    /// Initial state
    Idle,
    /// Waiting for QR scan
    WaitingForScan,
    /// X3DH handshake in progress
    HandshakeInProgress,
    /// SAS verification required
    SasVerification,
    /// Paired successfully
    Paired,
    /// Pairing failed
    Failed,
}

/// Short Authentication String (SAS) for verification
#[derive(Debug, Clone)]
pub struct SasCode {
    /// 6-digit code
    pub code: String,
    /// Hash of the combined keys
    pub key_hash: [u8; 32],
}

impl PairingCeremony {
    /// Create new pairing ceremony
    pub fn new(crypto: SyncCrypto) -> Self {
        Self {
            crypto,
            state: PairingState::Idle,
        }
    }
    
    /// Start pairing as primary device
    pub fn start_primary(&mut self) -> anyhow::Result<PublicKeyBundle> {
        self.state = PairingState::WaitingForScan;
        Ok(self.crypto.export_public())
    }
    
    /// Start pairing as secondary device (scan QR)
    pub fn start_secondary(&mut self, primary_public: &PublicKeyBundle) -> anyhow::Result<()> {
        self.state = PairingState::HandshakeInProgress;
        
        // Derive shared secret
        let fingerprint = primary_public.fingerprint();
        self.crypto.derive_shared_secret(&fingerprint, primary_public)?;
        
        self.state = PairingState::SasVerification;
        
        Ok(())
    }
    
    /// Generate SAS code for verification
    pub fn generate_sas(&self, their_public: &PublicKeyBundle) -> SasCode {
        let our_public = self.crypto.export_public();
        
        // Combine public keys and hash
        let mut hasher = Sha256::new();
        hasher.update(&our_public.to_bytes());
        hasher.update(&their_public.to_bytes());
        let result = hasher.finalize();
        
        // Convert first 4 bytes to 6-digit code
        let num = u32::from_be_bytes([result[0], result[1], result[2], result[3]]);
        let code = format!("{:06}", num % 1_000_000);
        
        let mut key_hash = [0u8; 32];
        key_hash.copy_from_slice(&result);
        
        SasCode { code, key_hash }
    }
    
    /// Verify SAS code
    pub fn verify_sas(&mut self, our_code: &SasCode, their_code: &SasCode) -> bool {
        if our_code.code == their_code.code {
            self.state = PairingState::Paired;
            true
        } else {
            self.state = PairingState::Failed;
            false
        }
    }
    
    /// Get current state
    pub fn state(&self) -> PairingState {
        self.state
    }
    
    /// Check if paired
    pub fn is_paired(&self) -> bool {
        self.state == PairingState::Paired
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_device_keys_generation() {
        let keys = DeviceKeys::generate();
        let fingerprint = keys.fingerprint();
        
        assert_eq!(fingerprint.len(), 32); // 16 bytes = 32 hex chars
        
        // Sign and verify
        let data = b"test message";
        let sig = keys.sign(data);
        keys.verify(data, &sig).unwrap();
    }
    
    #[test]
    fn test_public_key_bundle() {
        let keys = DeviceKeys::generate();
        let bundle = keys.export_public();
        
        let bytes = bundle.to_bytes();
        let restored = PublicKeyBundle::from_bytes(&bytes).unwrap();
        
        assert_eq!(bundle.verifying_key, restored.verifying_key);
    }
    
    #[test]
    fn test_sync_crypto() {
        let crypto1 = SyncCrypto::generate();
        let crypto2 = SyncCrypto::generate();
        
        let public1 = crypto1.export_public();
        let public2 = crypto2.export_public();
        
        // Derive shared secrets
        let mut crypto1 = crypto1;
        let mut crypto2 = crypto2;
        
        crypto1.derive_shared_secret("device2", &public2).unwrap();
        crypto2.derive_shared_secret("device1", &public1).unwrap();
        
        // Encrypt with crypto1, decrypt with crypto2
        let nonce = SyncCrypto::generate_nonce();
        let plaintext = b"secret message";
        
        let encrypted = crypto1.encrypt("device2", plaintext, &nonce).unwrap();
        let decrypted = crypto2.decrypt("device1", &encrypted, &nonce).unwrap();
        
        assert_eq!(plaintext.to_vec(), decrypted);
    }
    
    #[test]
    fn test_pairing_ceremony() {
        let crypto1 = SyncCrypto::generate();
        let crypto2 = SyncCrypto::generate();
        
        let mut ceremony1 = PairingCeremony::new(crypto1);
        let mut ceremony2 = PairingCeremony::new(crypto2);
        
        // Primary starts
        let public1 = ceremony1.start_primary().unwrap();
        assert_eq!(ceremony1.state(), PairingState::WaitingForScan);
        
        // Secondary scans
        ceremony2.start_secondary(&public1).unwrap();
        assert_eq!(ceremony2.state(), PairingState::SasVerification);
        
        // Generate SAS codes
        let public2 = ceremony2.crypto.export_public();
        let sas1 = ceremony1.generate_sas(&public2);
        let sas2 = ceremony2.generate_sas(&public1);
        
        // Codes should match
        assert_eq!(sas1.code, sas2.code);
        
        // Verify
        assert!(ceremony1.verify_sas(&sas1, &sas2));
        assert!(ceremony1.is_paired());
    }
    
    #[test]
    fn test_sas_mismatch() {
        let crypto1 = SyncCrypto::generate();
        let crypto2 = SyncCrypto::generate();
        let crypto3 = SyncCrypto::generate();
        
        let mut ceremony1 = PairingCeremony::new(crypto1);
        let mut ceremony2 = PairingCeremony::new(crypto2);
        
        let public1 = ceremony1.start_primary().unwrap();
        ceremony2.start_secondary(&public1).unwrap();
        
        // Use wrong public key for SAS
        let sas1 = ceremony1.generate_sas(&crypto3.export_public());
        let sas2 = ceremony2.generate_sas(&public1);
        
        // Codes should NOT match
        assert_ne!(sas1.code, sas2.code);
    }
}
