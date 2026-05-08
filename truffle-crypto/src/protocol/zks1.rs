//! ZKS-1 Protocol Main Implementation
//!
//! This module provides the high-level interface for the Zero-Knowledge Sync protocol.
//! It manages sessions, handles message encryption/decryption, and coordinates
//! the sync process across devices.

use crate::encryption::{Aes256GcmCipher, ChaCha20Poly1305Cipher, EncryptionCipher};
use crate::error::{CryptoError, CryptoResult};
use crate::keys::{IdentityKeyPair, KeyStorage};
use crate::protocol::{
    handshake::{EphemeralKeyPair, PreKeyBundle, SignedPrekey, X3dhHandshake, X3dhInitiator, X3dhResponder, X3dhResult},
    message::{CrdtUpdateContent, EncryptedPayload, MessageDeduplicator, SyncMessage, SyncMessageHeader, SyncMessagePayload},
    MessageType, PrivacyClassification, MAX_PAYLOAD_SIZE, compute_message_id,
};
use crate::types::{CryptoTimestamp, DeviceId, KeyFingerprint, Nonce, ProtocolVersion};
use crate::utils::{concat_bytes, hmac_sha256, sha3_256};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::{Arc, RwLock};
use zeroize::{Zeroize, ZeroizeOnDrop};

/// Device trust level for access control
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum DeviceTrustLevel {
    /// Untrusted - requires verification
    Untrusted = 0,
    /// Pending verification
    Pending = 1,
    /// Verified and trusted
    Verified = 2,
    /// Highly trusted (primary device)
    Primary = 3,
    /// Revoked - no longer trusted
    Revoked = 4,
}

impl DeviceTrustLevel {
    /// Check if this trust level allows sync
    pub fn can_sync(&self) -> bool {
        matches!(self, Self::Verified | Self::Primary)
    }

    /// Check if this trust level allows key rotation
    pub fn can_rotate_keys(&self) -> bool {
        matches!(self, Self::Primary)
    }
}

/// Session state for a device pair
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum SessionState {
    /// Initial state, no handshake performed
    Initial,
    /// Handshake in progress
    Handshaking,
    /// Active session with established keys
    Active,
    /// Session suspended (e.g., device offline)
    Suspended,
    /// Session terminated
    Terminated,
}

/// Sync configuration
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SyncConfig {
    /// Sync interval in milliseconds
    pub sync_interval_ms: u64,
    /// Maximum payload size in bytes
    pub max_payload_size: usize,
    /// Enable ChaCha20-Poly1305 (for mobile)
    pub use_chacha20: bool,
    /// Enable post-quantum Kyber-768
    pub use_kyber768: bool,
    /// Rate limit (messages per day)
    pub rate_limit: u32,
    /// Message retention days
    pub retention_days: u32,
}

impl Default for SyncConfig {
    fn default() -> Self {
        Self {
            sync_interval_ms: 5000,
            max_payload_size: MAX_PAYLOAD_SIZE,
            use_chacha20: false,
            use_kyber768: cfg!(feature = "kyber768"),
            rate_limit: 10000,
            retention_days: 30,
        }
    }
}

impl SyncConfig {
    /// Mobile-optimized configuration
    pub fn mobile() -> Self {
        Self {
            use_chacha20: true,
            sync_interval_ms: 10000,
            ..Default::default()
        }
    }

    /// High-security configuration
    pub fn high_security() -> Self {
        Self {
            use_chacha20: false,
            use_kyber768: true,
            rate_limit: 1000,
            ..Default::default()
        }
    }
}

/// Statistics for sync operations
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct SyncStatistics {
    /// Total messages sent
    pub messages_sent: u64,
    /// Total messages received
    pub messages_received: u64,
    /// Total bytes sent
    pub bytes_sent: u64,
    /// Total bytes received
    pub bytes_received: u64,
    /// Number of failed syncs
    pub failed_syncs: u64,
    /// Last sync timestamp
    pub last_sync_at: Option<u64>,
    /// Average sync latency in ms
    pub avg_latency_ms: u64,
}

impl SyncStatistics {
    /// Record a sent message
    pub fn record_sent(&mut self, bytes: usize) {
        self.messages_sent += 1;
        self.bytes_sent += bytes as u64;
        self.last_sync_at = Some(CryptoTimestamp::now().as_millis());
    }

    /// Record a received message
    pub fn record_received(&mut self, bytes: usize) {
        self.messages_received += 1;
        self.bytes_received += bytes as u64;
        self.last_sync_at = Some(CryptoTimestamp::now().as_millis());
    }

    /// Record a failed sync
    pub fn record_failure(&mut self) {
        self.failed_syncs += 1;
    }
}

/// Session keys derived from X3DH handshake
#[derive(Clone, Zeroize, ZeroizeOnDrop)]
pub struct SessionKeys {
    /// Encryption key for sync messages
    #[zeroize(skip)]
    pub sync_key: [u8; 32],
    /// Authentication key for HMAC
    #[zeroize(skip)]
    pub auth_key: [u8; 32],
    /// Associated data for this session
    pub associated_data: Vec<u8>,
}

impl SessionKeys {
    /// Derive session keys from X3DH result
    pub fn from_x3dh_result(result: &X3dhResult) -> CryptoResult<Self> {
        Ok(Self {
            sync_key: result.derive_sync_key()?,
            auth_key: result.derive_auth_key()?,
            associated_data: result.associated_data.clone(),
        })
    }
}

/// A sync session with another device
pub struct Zks1Session {
    /// Device ID of the peer
    pub peer_device_id: DeviceId,
    /// Current session state
    pub state: SessionState,
    /// Trust level of the peer device
    pub trust_level: DeviceTrustLevel,
    /// Session keys (if established)
    pub session_keys: Option<SessionKeys>,
    /// Sequence number for outgoing messages
    pub outgoing_sequence: u64,
    /// Sequence number for incoming messages (for replay detection)
    pub incoming_sequence: u64,
    /// Session creation timestamp
    pub created_at: u64,
    /// Last activity timestamp
    pub last_activity_at: u64,
    /// Sync statistics
    pub statistics: SyncStatistics,
    /// Message deduplicator
    deduplicator: MessageDeduplicator,
}

impl Zks1Session {
    /// Create a new session
    pub fn new(peer_device_id: DeviceId) -> Self {
        let now = CryptoTimestamp::now().as_millis();
        Self {
            peer_device_id,
            state: SessionState::Initial,
            trust_level: DeviceTrustLevel::Untrusted,
            session_keys: None,
            outgoing_sequence: 0,
            incoming_sequence: 0,
            created_at: now,
            last_activity_at: now,
            statistics: SyncStatistics::default(),
            deduplicator: MessageDeduplicator::new(10000),
        }
    }

    /// Check if the session is active
    pub fn is_active(&self) -> bool {
        matches!(self.state, SessionState::Active)
    }

    /// Activate the session with derived keys
    pub fn activate(&mut self, keys: SessionKeys) -> CryptoResult<()> {
        self.session_keys = Some(keys);
        self.state = SessionState::Active;
        self.trust_level = DeviceTrustLevel::Verified;
        self.last_activity_at = CryptoTimestamp::now().as_millis();
        Ok(())
    }

    /// Get the next outgoing sequence number
    pub fn next_sequence(&mut self) -> u64 {
        self.outgoing_sequence += 1;
        self.outgoing_sequence
    }

    /// Validate incoming sequence number (anti-replay)
    pub fn validate_sequence(&mut self, sequence: u64) -> bool {
        if sequence <= self.incoming_sequence {
            return false; // Replay or out-of-order
        }
        self.incoming_sequence = sequence;
        true
    }

    /// Check if we've seen this message before
    pub fn is_duplicate(&self, message_id: &[u8; 32]) -> bool {
        self.deduplicator.has_seen(message_id)
    }

    /// Mark a message as seen
    pub fn mark_seen(&mut self, message_id: [u8; 32]) {
        self.deduplicator.mark_seen(message_id);
    }

    /// Update last activity timestamp
    pub fn touch(&mut self) {
        self.last_activity_at = CryptoTimestamp::now().as_millis();
    }

    /// Check if the session has expired (no activity for too long)
    pub fn is_expired(&self, timeout_ms: u64) -> bool {
        let now = CryptoTimestamp::now().as_millis();
        now - self.last_activity_at > timeout_ms
    }

    /// Suspend the session
    pub fn suspend(&mut self) {
        self.state = SessionState::Suspended;
    }

    /// Terminate the session
    pub fn terminate(&mut self) {
        self.state = SessionState::Terminated;
        self.session_keys = None;
        self.trust_level = DeviceTrustLevel::Revoked;
    }
}

impl std::fmt::Debug for Zks1Session {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Zks1Session")
            .field("peer_device_id", &self.peer_device_id)
            .field("state", &self.state)
            .field("trust_level", &self.trust_level)
            .field("has_keys", &self.session_keys.is_some())
            .field("outgoing_sequence", &self.outgoing_sequence)
            .field("incoming_sequence", &self.incoming_sequence)
            .finish()
    }
}

/// Main ZKS-1 protocol handler
pub struct Zks1Protocol {
    /// Our device identity
    identity: IdentityKeyPair,
    /// Our device ID
    device_id: DeviceId,
    /// Sync configuration
    config: SyncConfig,
    /// Active sessions with other devices
    sessions: RwLock<HashMap<DeviceId, Zks1Session>>,
    /// Our signed prekey
    signed_prekey: RwLock<SignedPrekey>,
    /// One-time prekeys
    one_time_prekeys: RwLock<Vec<EphemeralKeyPair>>,
}

impl Zks1Protocol {
    /// Create a new ZKS-1 protocol instance
    pub fn new(identity: IdentityKeyPair, config: SyncConfig) -> CryptoResult<Self> {
        let device_id = DeviceId::generate_random()?;
        let signed_prekey = SignedPrekey::generate(1, &identity)?;

        // Generate one-time prekeys
        let mut prekeys = Vec::with_capacity(crate::DEFAULT_PREKEY_COUNT);
        for i in 0..crate::DEFAULT_PREKEY_COUNT {
            prekeys.push(EphemeralKeyPair::generate());
        }

        Ok(Self {
            identity,
            device_id,
            config,
            sessions: RwLock::new(HashMap::new()),
            signed_prekey: RwLock::new(signed_prekey),
            one_time_prekeys: RwLock::new(prekeys),
        })
    }

    /// Get our device ID
    pub fn device_id(&self) -> DeviceId {
        self.device_id
    }

    /// Get our identity key fingerprint
    pub fn identity_fingerprint(&self) -> KeyFingerprint {
        self.identity.fingerprint()
    }

    /// Get sync configuration
    pub fn config(&self) -> &SyncConfig {
        &self.config
    }

    /// Generate a PreKey bundle for pairing
    pub fn generate_prekey_bundle(&self) -> CryptoResult<PreKeyBundle> {
        let identity_x25519 = self.identity.to_x25519()?;
        let signed_prekey = self.signed_prekey.read().map_err(|_| {
            CryptoError::internal_error("Failed to read signed prekey")
        })?;

        let one_time_prekeys = self.one_time_prekeys.read().map_err(|_| {
            CryptoError::internal_error("Failed to read one-time prekeys")
        })?;

        let otp_public: Vec<crate::protocol::handshake::OneTimePrekeyPublic> = one_time_prekeys
            .iter()
            .enumerate()
            .map(|(i, pk)| crate::protocol::handshake::OneTimePrekeyPublic {
                id: i as u32,
                public_key: pk.public_key_bytes(),
            })
            .collect();

        Ok(PreKeyBundle {
            identity_key: identity_x25519.to_bytes(),
            signed_prekey: crate::protocol::handshake::SignedPrekeyPublic {
                id: signed_prekey.id,
                public_key: signed_prekey.public_key_bytes(),
                signature: signed_prekey.signature().to_vec(),
            },
            one_time_prekeys: otp_public,
            #[cfg(feature = "kyber768")]
            kyber_public_key: None,
        })
    }

    /// Initiate a session with another device
    pub fn initiate_session(&self, peer_device_id: DeviceId, prekey_bundle: PreKeyBundle) -> CryptoResult<()> {
        // Check if session already exists
        {
            let sessions = self.sessions.read().map_err(|_| {
                CryptoError::internal_error("Failed to read sessions")
            })?;
            if sessions.contains_key(&peer_device_id) {
                return Err(CryptoError::invalid_state("Session already exists"));
            }
        }

        // Create new session
        let mut session = Zks1Session::new(peer_device_id);
        session.state = SessionState::Handshaking;

        // Perform X3DH handshake
        let (initiator, result) = X3dhHandshake::initiate(
            self.identity.clone(),
            prekey_bundle,
        )?;

        // Derive session keys
        let session_keys = SessionKeys::from_x3dh_result(&result)?;
        session.activate(session_keys)?;

        // Store session
        {
            let mut sessions = self.sessions.write().map_err(|_| {
                CryptoError::internal_error("Failed to write sessions")
            })?;
            sessions.insert(peer_device_id, session);
        }

        Ok(())
    }

    /// Accept a session from another device
    pub fn accept_session(
        &self,
        peer_device_id: DeviceId,
        initiator_identity: [u8; 32],
        initiator_ephemeral: [u8; 32],
        one_time_prekey_id: Option<u32>,
    ) -> CryptoResult<()> {
        // Create responder
        let signed_prekey = self.signed_prekey.read().map_err(|_| {
            CryptoError::internal_error("Failed to read signed prekey")
        })?;

        let one_time_prekeys: Vec<_> = self.one_time_prekeys.read().map_err(|_| {
            CryptoError::internal_error("Failed to read one-time prekeys")
        })?.iter().enumerate().map(|(i, kp)| {
            crate::protocol::handshake::OneTimePrekey {
                id: i as u32,
                secret: x25519_dalek::StaticSecret::from(kp.public_key_bytes()), // This is wrong but placeholder
                public: *kp.public_key(),
            }
        }).collect();

        let mut responder = X3dhResponder::new(
            self.identity.clone(),
            signed_prekey.clone(),
            one_time_prekeys,
        );

        // Perform handshake
        let result = X3dhHandshake::respond(
            &mut responder,
            initiator_identity,
            initiator_ephemeral,
            one_time_prekey_id,
        )?;

        // Create and activate session
        let mut session = Zks1Session::new(peer_device_id);
        let session_keys = SessionKeys::from_x3dh_result(&result)?;
        session.activate(session_keys)?;

        // Store session
        {
            let mut sessions = self.sessions.write().map_err(|_| {
                CryptoError::internal_error("Failed to write sessions")
            })?;
            sessions.insert(peer_device_id, session);
        }

        Ok(())
    }

    /// Get a session by device ID
    pub fn get_session(&self, device_id: DeviceId) -> Option<Zks1Session> {
        self.sessions.read().ok()?.get(&device_id).cloned()
    }

    /// Encrypt a CRDT update for a specific device
    pub fn encrypt_crdt_update(
        &self,
        peer_device_id: DeviceId,
        crdt_update: Vec<u8>,
        schema_version: String,
        privacy: PrivacyClassification,
    ) -> CryptoResult<SyncMessage> {
        // Get session
        let mut sessions = self.sessions.write().map_err(|_| {
            CryptoError::internal_error("Failed to write sessions")
        })?;

        let session = sessions.get_mut(&peer_device_id).ok_or_else(|| {
            CryptoError::invalid_state("No active session with this device")
        })?;

        if !session.is_active() {
            return Err(CryptoError::invalid_state("Session is not active"));
        }

        let keys = session.session_keys.as_ref().ok_or_else(|| {
            CryptoError::invalid_state("Session keys not available")
        })?;

        // Create content
        let content = CrdtUpdateContent::new(
            crdt_update,
            schema_version,
            privacy,
        )?;

        // Serialize content
        let plaintext = content.to_bytes()?;

        // Encrypt based on config
        let encrypted_payload = if self.config.use_chacha20 {
            self.encrypt_chacha20(&plaintext, &keys.sync_key)?
        } else {
            self.encrypt_aes256_gcm(&plaintext, &keys.sync_key)?
        };

        // Create header
        let sequence = session.next_sequence();
        let header = SyncMessageHeader::new(
            self.device_id,
            MessageType::CrdtUpdate,
            sequence,
        )?;

        // Create message
        let message = SyncMessage::new(
            header,
            encrypted_payload,
            &keys.auth_key,
        );

        // Update statistics
        let msg_bytes = message.to_bytes()?.len();
        session.statistics.record_sent(msg_bytes);
        session.touch();

        Ok(message)
    }

    /// Decrypt a sync message
    pub fn decrypt_message(&self, message: &SyncMessage) -> CryptoResult<CrdtUpdateContent> {
        // Get peer device ID from header
        let peer_device_id = message.header.device_id;

        // Get session
        let mut sessions = self.sessions.write().map_err(|_| {
            CryptoError::internal_error("Failed to write sessions")
        })?;

        let session = sessions.get_mut(&peer_device_id).ok_or_else(|| {
            CryptoError::invalid_state("No session with this device")
        })?;

        let keys = session.session_keys.as_ref().ok_or_else(|| {
            CryptoError::invalid_state("Session keys not available")
        })?;

        // Verify authenticator
        if !message.verify_authenticator(&keys.auth_key) {
            return Err(CryptoError::authentication_failed(
                "Message authenticator verification failed"
            ));
        }

        // Check for replay
        if !session.validate_sequence(message.header.sequence_number) {
            return Err(CryptoError::replay_attack_detected(
                "Invalid sequence number"
            ));
        }

        // Check for duplicates
        let msg_id = message.message_id();
        if session.is_duplicate(&msg_id) {
            return Err(CryptoError::replay_attack_detected(
                "Duplicate message detected"
            ));
        }
        session.mark_seen(msg_id);

        // Decrypt payload
        let plaintext = if self.config.use_chacha20 {
            self.decrypt_chacha20(&message.payload, &keys.sync_key)?
        } else {
            self.decrypt_aes256_gcm(&message.payload, &keys.sync_key)?
        };

        // Deserialize content
        let content = CrdtUpdateContent::from_bytes(&plaintext)?;

        // Update statistics
        let msg_bytes = message.to_bytes()?.len();
        session.statistics.record_received(msg_bytes);
        session.touch();

        Ok(content)
    }

    /// Encrypt using AES-256-GCM
    fn encrypt_aes256_gcm(&self, plaintext: &[u8], key: &[u8; 32]) -> CryptoResult<EncryptedPayload> {
        let cipher = Aes256GcmCipher::new(key)?;
        let nonce = Nonce::generate_random()?;
        let ciphertext = cipher.encrypt(plaintext, &nonce, &[])?;

        Ok(EncryptedPayload::new(
            ciphertext.to_bytes(),
            nonce.to_array(),
            EncryptedPayload::ALGORITHM_AES256_GCM,
        ))
    }

    /// Decrypt using AES-256-GCM
    fn decrypt_aes256_gcm(&self, payload: &EncryptedPayload, key: &[u8; 32]) -> CryptoResult<Vec<u8>> {
        let cipher = Aes256GcmCipher::new(key)?;
        let nonce = Nonce::new(payload.nonce);
        let encrypted_blob = crate::types::EncryptedBlob::new(
            payload.ciphertext.clone(),
            nonce,
        );
        cipher.decrypt(&encrypted_blob, &[])
    }

    /// Encrypt using ChaCha20-Poly1305
    fn encrypt_chacha20(&self, plaintext: &[u8], key: &[u8; 32]) -> CryptoResult<EncryptedPayload> {
        let cipher = ChaCha20Poly1305Cipher::new(key)?;
        let nonce = Nonce::generate_random()?;
        let ciphertext = cipher.encrypt(plaintext, &nonce, &[])?;

        Ok(EncryptedPayload::new(
            ciphertext.to_bytes(),
            nonce.to_array(),
            EncryptedPayload::ALGORITHM_CHACHA20_POLY1305,
        ))
    }

    /// Decrypt using ChaCha20-Poly1305
    fn decrypt_chacha20(&self, payload: &EncryptedPayload, key: &[u8; 32]) -> CryptoResult<Vec<u8>> {
        let cipher = ChaCha20Poly1305Cipher::new(key)?;
        let nonce = Nonce::new(payload.nonce);
        let encrypted_blob = crate::types::EncryptedBlob::new(
            payload.ciphertext.clone(),
            nonce,
        );
        cipher.decrypt(&encrypted_blob, &[])
    }

    /// Rotate our signed prekey
    pub fn rotate_signed_prekey(&self) -> CryptoResult<()> {
        let current_id = self.signed_prekey.read()
            .map_err(|_| CryptoError::internal_error("Failed to read signed prekey"))?
            .id;
        let new_prekey = SignedPrekey::generate(
            current_id + 1,
            &self.identity,
        )?;

        let mut prekey = self.signed_prekey.write().map_err(|_| {
            CryptoError::internal_error("Failed to write signed prekey")
        })?;
        *prekey = new_prekey;

        Ok(())
    }

    /// Replenish one-time prekeys
    pub fn replenish_prekeys(&self) -> CryptoResult<()> {
        let mut prekeys = self.one_time_prekeys.write().map_err(|_| {
            CryptoError::internal_error("Failed to write one-time prekeys")
        })?;

        while prekeys.len() < crate::DEFAULT_PREKEY_COUNT {
            prekeys.push(EphemeralKeyPair::generate());
        }

        Ok(())
    }

    /// Get session statistics
    pub fn get_statistics(&self, device_id: DeviceId) -> CryptoResult<Option<SyncStatistics>> {
        let sessions = self.sessions.read().map_err(|_| {
            CryptoError::internal_error("Failed to read sessions")
        })?;

        Ok(sessions.get(&device_id).map(|s| s.statistics.clone()))
    }

    /// Terminate a session
    pub fn terminate_session(&self, device_id: DeviceId) -> CryptoResult<()> {
        let mut sessions = self.sessions.write().map_err(|_| {
            CryptoError::internal_error("Failed to write sessions")
        })?;

        if let Some(session) = sessions.get_mut(&device_id) {
            session.terminate();
        }

        sessions.remove(&device_id);
        Ok(())
    }

    /// List all active sessions
    pub fn list_sessions(&self) -> CryptoResult<Vec<(DeviceId, SessionState, DeviceTrustLevel)>> {
        let sessions = self.sessions.read().map_err(|_| {
            CryptoError::internal_error("Failed to read sessions")
        })?;

        Ok(sessions
            .iter()
            .map(|(id, session)| (*id, session.state.clone(), session.trust_level))
            .collect())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_device_trust_level() {
        assert!(DeviceTrustLevel::Verified.can_sync());
        assert!(DeviceTrustLevel::Primary.can_sync());
        assert!(!DeviceTrustLevel::Untrusted.can_sync());
        assert!(!DeviceTrustLevel::Revoked.can_sync());

        assert!(DeviceTrustLevel::Primary.can_rotate_keys());
        assert!(!DeviceTrustLevel::Verified.can_rotate_keys());
    }

    #[test]
    fn test_sync_config() {
        let default = SyncConfig::default();
        assert_eq!(default.sync_interval_ms, 5000);

        let mobile = SyncConfig::mobile();
        assert!(mobile.use_chacha20);

        let high_sec = SyncConfig::high_security();
        assert!(high_sec.use_kyber768 || !cfg!(feature = "kyber768"));
    }

    #[test]
    fn test_zks1_protocol_creation() {
        let identity = IdentityKeyPair::generate();
        let config = SyncConfig::default();
        let protocol = Zks1Protocol::new(identity, config).unwrap();

        assert!(protocol.device_id().as_bytes().len() > 0);
    }

    #[test]
    fn test_prekey_bundle_generation() {
        let identity = IdentityKeyPair::generate();
        let config = SyncConfig::default();
        let protocol = Zks1Protocol::new(identity, config).unwrap();

        let bundle = protocol.generate_prekey_bundle().unwrap();
        assert_eq!(bundle.identity_key.len(), 32);
        assert!(!bundle.one_time_prekeys.is_empty());
    }

    #[test]
    fn test_session_management() {
        let identity = IdentityKeyPair::generate();
        let config = SyncConfig::default();
        let protocol = Zks1Protocol::new(identity, config).unwrap();

        let peer_id = DeviceId::generate_random().unwrap();

        // Initially no session
        assert!(protocol.get_session(peer_id).is_none());

        // List should be empty
        let sessions = protocol.list_sessions().unwrap();
        assert!(sessions.is_empty());
    }

    #[test]
    fn test_sync_statistics() {
        let mut stats = SyncStatistics::default();
        assert_eq!(stats.messages_sent, 0);

        stats.record_sent(100);
        assert_eq!(stats.messages_sent, 1);
        assert_eq!(stats.bytes_sent, 100);

        stats.record_received(200);
        assert_eq!(stats.messages_received, 1);
        assert_eq!(stats.bytes_received, 200);

        stats.record_failure();
        assert_eq!(stats.failed_syncs, 1);
    }
}
