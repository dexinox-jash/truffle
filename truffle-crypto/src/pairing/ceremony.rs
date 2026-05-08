//! Pairing Ceremony Implementation
//!
//! This module implements the device pairing ceremony flow.
//! The ceremony ensures secure key exchange with MITM protection.

use crate::error::{CryptoError, CryptoResult};
use crate::keys::IdentityKeyPair;
use crate::pairing::qr::QrCodeData;
use crate::pairing::verification::SasGenerator;
use crate::protocol::handshake::{PreKeyBundle, X3dhHandshake, X3dhResult};
use crate::types::{DeviceId, KeyFingerprint};
use crate::utils::{random_bytes, sha3_256};
use serde::{Deserialize, Serialize};
use std::time::{Duration, Instant};

/// Configuration for the pairing ceremony
#[derive(Clone, Debug)]
pub struct PairingConfig {
    /// WebSocket relay endpoint
    pub relay_endpoint: String,
    /// Pairing timeout
    pub timeout: Duration,
    /// Enable SAS verification
    pub enable_sas: bool,
    /// SAS digit count
    pub sas_digits: usize,
}

impl Default for PairingConfig {
    fn default() -> Self {
        Self {
            relay_endpoint: crate::pairing::DEFAULT_RELAY_ENDPOINT.to_string(),
            timeout: Duration::from_secs(crate::pairing::DEFAULT_PAIRING_TIMEOUT_SECS),
            enable_sas: true,
            sas_digits: crate::SAS_DIGIT_COUNT,
        }
    }
}

/// State of the pairing ceremony
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PairingState {
    /// Initial state
    Initial,
    /// QR code generated/displayed
    QrDisplayed,
    /// QR code scanned
    QrScanned,
    /// Handshake in progress
    Handshaking,
    /// SAS verification pending
    SasPending,
    /// SAS verified
    SasVerified,
    /// Pairing complete
    Complete,
    /// Pairing failed
    Failed,
    /// Pairing timed out
    TimedOut,
}

impl PairingState {
    /// Check if pairing is in progress
    pub fn is_in_progress(&self) -> bool {
        matches!(
            self,
            Self::QrDisplayed | Self::QrScanned | Self::Handshaking | Self::SasPending
        )
    }

    /// Check if pairing is complete
    pub fn is_complete(&self) -> bool {
        matches!(self, Self::Complete)
    }

    /// Check if pairing failed
    pub fn is_failed(&self) -> bool {
        matches!(self, Self::Failed | Self::TimedOut)
    }
}

/// Pairing session data
#[derive(Clone, Debug)]
pub struct PairingSession {
    /// Session ID
    pub session_id: String,
    /// Current state
    pub state: PairingState,
    /// Primary device ID
    pub primary_device_id: Option<DeviceId>,
    /// Secondary device ID
    pub secondary_device_id: Option<DeviceId>,
    /// Primary device fingerprint
    pub primary_fingerprint: Option<KeyFingerprint>,
    /// Secondary device fingerprint
    pub secondary_fingerprint: Option<KeyFingerprint>,
    /// SAS code (for verification)
    pub sas_code: Option<String>,
    /// X3DH result
    pub x3dh_result: Option<X3dhResult>,
    /// Session start time
    pub started_at: Instant,
    /// Session timeout
    pub timeout: Duration,
}

impl PairingSession {
    /// Create a new pairing session
    pub fn new(session_id: String, timeout: Duration) -> Self {
        Self {
            session_id,
            state: PairingState::Initial,
            primary_device_id: None,
            secondary_device_id: None,
            primary_fingerprint: None,
            secondary_fingerprint: None,
            sas_code: None,
            x3dh_result: None,
            started_at: Instant::now(),
            timeout,
        }
    }

    /// Check if the session has timed out
    pub fn is_timed_out(&self) -> bool {
        self.started_at.elapsed() > self.timeout
    }

    /// Check if the session is still valid
    pub fn is_valid(&self) -> bool {
        !self.is_timed_out() && !self.state.is_failed()
    }

    /// Transition to a new state
    pub fn transition(&mut self, new_state: PairingState) -> CryptoResult<()> {
        // Validate state transition
        match (&self.state, &new_state) {
            (PairingState::Initial, PairingState::QrDisplayed) => Ok(()),
            (PairingState::QrDisplayed, PairingState::QrScanned) => Ok(()),
            (PairingState::QrScanned, PairingState::Handshaking) => Ok(()),
            (PairingState::Handshaking, PairingState::SasPending) => Ok(()),
            (PairingState::SasPending, PairingState::SasVerified) => Ok(()),
            (PairingState::SasVerified, PairingState::Complete) => Ok(()),
            (_, PairingState::Failed) => Ok(()),
            (_, PairingState::TimedOut) => Ok(()),
            _ => Err(CryptoError::invalid_state(
                format!("Invalid state transition: {:?} -> {:?}", self.state, new_state)
            )),
        }?;

        self.state = new_state;
        Ok(())
    }

    /// Generate SAS code from shared secret
    pub fn generate_sas(&mut self) -> CryptoResult<String> {
        let result = self.x3dh_result.as_ref()
            .ok_or_else(|| CryptoError::invalid_state("No X3DH result available"))?;

        let primary_fp = self.primary_fingerprint.as_ref()
            .ok_or_else(|| CryptoError::invalid_state("No primary fingerprint"))?;

        let secondary_fp = self.secondary_fingerprint.as_ref()
            .ok_or_else(|| CryptoError::invalid_state("No secondary fingerprint"))?;

        let sas = SasGenerator::generate(
            &result.shared_secret,
            primary_fp.as_bytes(),
            secondary_fp.as_bytes(),
            crate::SAS_DIGIT_COUNT,
        );

        self.sas_code = Some(sas.clone());
        Ok(sas)
    }
}

/// Primary device in the pairing ceremony
pub struct PrimaryDevice {
    /// Our identity key
    identity: IdentityKeyPair,
    /// Our device ID
    device_id: DeviceId,
    /// Pairing configuration
    config: PairingConfig,
    /// Pairing session
    session: Option<PairingSession>,
    /// One-time token for this pairing
    one_time_token: String,
}

impl PrimaryDevice {
    /// Create a new primary device for pairing
    pub fn new(identity: IdentityKeyPair, config: PairingConfig) -> CryptoResult<Self> {
        let device_id = DeviceId::generate_random()?;

        // Generate one-time token
        let token_bytes = random_bytes::<16>()?;
        let one_time_token = base64::encode(&token_bytes);

        Ok(Self {
            identity,
            device_id,
            config,
            session: None,
            one_time_token,
        })
    }

    /// Generate QR code data for pairing
    pub fn generate_qr_data(&mut self) -> CryptoResult<QrCodeData> {
        // Generate session ID
        let session_id = format!("truffle-{}", hex::encode(random_bytes::<8>()?));

        // Create session
        let session = PairingSession::new(
            session_id.clone(),
            self.config.timeout,
        );
        self.session = Some(session);

        // Generate PreKey bundle
        let prekey_bundle = self.generate_prekey_bundle()?;

        // Create QR code data
        let qr_data = QrCodeData {
            version: 1,
            session_id,
            relay_endpoint: self.config.relay_endpoint.clone(),
            one_time_token: self.one_time_token.clone(),
            identity_fingerprint: self.identity.fingerprint(),
            prekey_bundle: Some(prekey_bundle.to_bytes()?),
        };

        // Update session state
        if let Some(ref mut session) = self.session {
            session.transition(PairingState::QrDisplayed)?;
            session.primary_device_id = Some(self.device_id);
            session.primary_fingerprint = Some(self.identity.fingerprint());
        }

        Ok(qr_data)
    }

    /// Generate PreKey bundle
    fn generate_prekey_bundle(&self) -> CryptoResult<PreKeyBundle> {
        let x25519_public = self.identity.to_x25519_public()?;

        // Generate signed prekey
        let signed_prekey_secret = x25519_dalek::StaticSecret::random_from_rng(rand::thread_rng());
        let signed_prekey_public = x25519_dalek::PublicKey::from(&signed_prekey_secret);

        // Sign the prekey
        let signature = self.identity.sign(&signed_prekey_public.to_bytes())?;

        // Generate one-time prekeys
        let mut one_time_prekeys = Vec::new();
        for i in 0..5 {
            let secret = x25519_dalek::StaticSecret::random_from_rng(rand::thread_rng());
            let public = x25519_dalek::PublicKey::from(&secret);
            one_time_prekeys.push(crate::protocol::handshake::OneTimePrekeyPublic {
                id: i,
                public_key: public.to_bytes(),
            });
        }

        Ok(PreKeyBundle {
            identity_key: x25519_public.to_bytes(),
            signed_prekey: crate::protocol::handshake::SignedPrekeyPublic {
                id: 1,
                public_key: signed_prekey_public.to_bytes(),
                signature,
            },
            one_time_prekeys,
            #[cfg(feature = "kyber768")]
            kyber_public_key: None,
        })
    }

    /// Complete the pairing after SAS verification
    pub fn complete_pairing(&mut self, sas_verified: bool) -> CryptoResult<X3dhResult> {
        let session = self.session.as_mut()
            .ok_or_else(|| CryptoError::invalid_state("No active pairing session"))?;

        if !sas_verified {
            session.transition(PairingState::Failed)?;
            return Err(CryptoError::SasVerificationFailed);
        }

        session.transition(PairingState::SasVerified)?;
        session.transition(PairingState::Complete)?;

        let result = session.x3dh_result.clone()
            .ok_or_else(|| CryptoError::invalid_state("No X3DH result available"))?;

        Ok(result)
    }

    /// Get the current session state
    pub fn session_state(&self) -> Option<PairingState> {
        self.session.as_ref().map(|s| s.state)
    }

    /// Get the SAS code
    pub fn sas_code(&self) -> Option<&str> {
        self.session.as_ref().and_then(|s| s.sas_code.as_deref())
    }
}

/// Secondary device in the pairing ceremony
pub struct SecondaryDevice {
    /// Our identity key
    identity: IdentityKeyPair,
    /// Our device ID
    device_id: DeviceId,
    /// Pairing configuration
    config: PairingConfig,
    /// Pairing session
    session: Option<PairingSession>,
}

impl SecondaryDevice {
    /// Create a new secondary device for pairing
    pub fn new(identity: IdentityKeyPair, config: PairingConfig) -> CryptoResult<Self> {
        let device_id = DeviceId::generate_random()?;

        Ok(Self {
            identity,
            device_id,
            config,
            session: None,
        })
    }

    /// Process QR code and initiate pairing
    pub fn process_qr_data(&mut self, qr_data: &QrCodeData) -> CryptoResult<()> {
        // Validate QR code version
        if qr_data.version != 1 {
            return Err(CryptoError::invalid_message_format("Unsupported QR code version"));
        }

        // Create session
        let mut session = PairingSession::new(
            qr_data.session_id.clone(),
            self.config.timeout,
        );

        session.transition(PairingState::QrScanned)?;
        session.secondary_device_id = Some(self.device_id);
        session.secondary_fingerprint = Some(self.identity.fingerprint());
        session.primary_fingerprint = Some(qr_data.identity_fingerprint);

        self.session = Some(session);

        Ok(())
    }

    /// Perform X3DH handshake
    pub fn perform_handshake(&mut self, prekey_bundle: PreKeyBundle) -> CryptoResult<()> {
        let session = self.session.as_mut()
            .ok_or_else(|| CryptoError::invalid_state("No active pairing session"))?;

        session.transition(PairingState::Handshaking)?;

        // Perform X3DH handshake
        let (_initiator, result) = X3dhHandshake::initiate(
            self.identity.clone(),
            prekey_bundle,
        )?;

        session.x3dh_result = Some(result);

        // Transition to SAS pending if SAS is enabled
        if self.config.enable_sas {
            session.transition(PairingState::SasPending)?;
            session.generate_sas()?;
        } else {
            session.transition(PairingState::Complete)?;
        }

        Ok(())
    }

    /// Verify SAS code
    pub fn verify_sas(&mut self, expected_sas: &str) -> CryptoResult<bool> {
        let session = self.session.as_mut()
            .ok_or_else(|| CryptoError::invalid_state("No active pairing session"))?;

        if session.state != PairingState::SasPending {
            return Err(CryptoError::invalid_state("Not in SAS verification state"));
        }

        let actual_sas = session.sas_code.as_ref()
            .ok_or_else(|| CryptoError::invalid_state("No SAS code generated"))?;

        let verified = crate::utils::secure_compare(actual_sas.as_bytes(), expected_sas.as_bytes());

        if verified {
            session.transition(PairingState::SasVerified)?;
            session.transition(PairingState::Complete)?;
        } else {
            session.transition(PairingState::Failed)?;
        }

        Ok(verified)
    }

    /// Get the X3DH result
    pub fn get_x3dh_result(&self) -> Option<X3dhResult> {
        self.session.as_ref().and_then(|s| s.x3dh_result.clone())
    }

    /// Get the current session state
    pub fn session_state(&self) -> Option<PairingState> {
        self.session.as_ref().map(|s| s.state)
    }

    /// Get the SAS code
    pub fn sas_code(&self) -> Option<&str> {
        self.session.as_ref().and_then(|s| s.sas_code.as_deref())
    }
}

/// High-level pairing ceremony coordinator
pub struct PairingCeremony {
    /// Configuration
    config: PairingConfig,
    /// Active sessions
    sessions: std::collections::HashMap<String, PairingSession>,
}

impl PairingCeremony {
    /// Create a new pairing ceremony
    pub fn new(config: PairingConfig) -> Self {
        Self {
            config,
            sessions: std::collections::HashMap::new(),
        }
    }

    /// Create a new session
    pub fn create_session(&mut self) -> CryptoResult<String> {
        let session_id = format!("truffle-{}", hex::encode(random_bytes::<8>()?));
        let session = PairingSession::new(session_id.clone(), self.config.timeout);
        self.sessions.insert(session_id.clone(), session);
        Ok(session_id)
    }

    /// Get a session by ID
    pub fn get_session(&self, session_id: &str) -> Option<&PairingSession> {
        self.sessions.get(session_id)
    }

    /// Get a mutable session by ID
    pub fn get_session_mut(&mut self, session_id: &str) -> Option<&mut PairingSession> {
        self.sessions.get_mut(session_id)
    }

    /// Clean up expired sessions
    pub fn cleanup_sessions(&mut self) -> usize {
        let expired: Vec<String> = self.sessions
            .iter()
            .filter(|(_, session)| !session.is_valid())
            .map(|(id, _)| id.clone())
            .collect();

        for id in &expired {
            self.sessions.remove(id);
        }

        expired.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pairing_config_default() {
        let config = PairingConfig::default();
        assert_eq!(config.timeout, Duration::from_secs(300));
        assert!(config.enable_sas);
        assert_eq!(config.sas_digits, 6);
    }

    #[test]
    fn test_pairing_state_transitions() {
        let mut session = PairingSession::new("test".to_string(), Duration::from_secs(300));

        assert!(session.transition(PairingState::QrDisplayed).is_ok());
        assert!(session.transition(PairingState::QrScanned).is_ok());
        assert!(session.transition(PairingState::Handshaking).is_ok());
        assert!(session.transition(PairingState::SasPending).is_ok());
        assert!(session.transition(PairingState::SasVerified).is_ok());
        assert!(session.transition(PairingState::Complete).is_ok());
    }

    #[test]
    fn test_invalid_state_transition() {
        let mut session = PairingSession::new("test".to_string(), Duration::from_secs(300));

        // Cannot go directly to Complete from Initial
        assert!(session.transition(PairingState::Complete).is_err());
    }

    #[test]
    fn test_primary_device_qr_generation() {
        let identity = IdentityKeyPair::generate();
        let config = PairingConfig::default();
        let mut primary = PrimaryDevice::new(identity, config).unwrap();

        let qr_data = primary.generate_qr_data().unwrap();
        assert_eq!(qr_data.version, 1);
        assert!(!qr_data.session_id.is_empty());

        assert_eq!(primary.session_state(), Some(PairingState::QrDisplayed));
    }

    #[test]
    fn test_secondary_device_qr_processing() {
        let identity = IdentityKeyPair::generate();
        let config = PairingConfig::default();
        let mut secondary = SecondaryDevice::new(identity, config).unwrap();

        let qr_data = QrCodeData {
            version: 1,
            session_id: "test-session".to_string(),
            relay_endpoint: "wss://test".to_string(),
            one_time_token: "token".to_string(),
            identity_fingerprint: KeyFingerprint::from_public_key(&[0u8; 32]),
            prekey_bundle: None,
        };

        secondary.process_qr_data(&qr_data).unwrap();
        assert_eq!(secondary.session_state(), Some(PairingState::QrScanned));
    }

    #[test]
    fn test_pairing_ceremony() {
        let config = PairingConfig::default();
        let mut ceremony = PairingCeremony::new(config);

        let session_id = ceremony.create_session().unwrap();
        assert!(ceremony.get_session(&session_id).is_some());

        // Cleanup should not remove valid session
        assert_eq!(ceremony.cleanup_sessions(), 0);
        assert!(ceremony.get_session(&session_id).is_some());
    }
}
