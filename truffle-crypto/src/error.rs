//! Error types for the Truffle Crypto library
//!
//! This module defines all error types that can occur during cryptographic
//! operations. Each error includes context to help with debugging while
//! maintaining security (no sensitive data is leaked in error messages).

use thiserror::Error;

/// Result type alias for crypto operations
pub type CryptoResult<T> = Result<T, CryptoError>;

/// Main error type for cryptographic operations
///
/// All errors are designed to be safe to display/log without
/// leaking sensitive cryptographic material.
#[derive(Error, Debug, Clone, PartialEq)]
pub enum CryptoError {
    /// Random number generation failed
    #[error("Random number generation failed: {0}")]
    RandomGenerationFailed(String),

    /// Invalid key format or size
    #[error("Invalid key: {context}")]
    InvalidKey {
        /// Context about what was invalid
        context: String,
    },

    /// Key derivation failed
    #[error("Key derivation failed: {context}")]
    KeyDerivationFailed {
        /// Context about the failure
        context: String,
    },

    /// Encryption operation failed
    #[error("Encryption failed: {context}")]
    EncryptionFailed {
        /// Context about the failure
        context: String,
    },

    /// Decryption operation failed
    #[error("Decryption failed: {context}")]
    DecryptionFailed {
        /// Context about the failure
        context: String,
    },

    /// Authentication verification failed (tampering detected)
    #[error("Authentication failed: {context}")]
    AuthenticationFailed {
        /// Context about the failure
        context: String,
    },

    /// Invalid signature
    #[error("Invalid signature: {context}")]
    InvalidSignature {
        /// Context about the failure
        context: String,
    },

    /// Handshake protocol error
    #[error("Handshake failed: {context}")]
    HandshakeFailed {
        /// Context about the failure
        context: String,
    },

    /// Device pairing error
    #[error("Pairing failed: {context}")]
    PairingFailed {
        /// Context about the failure
        context: String,
    },

    /// SAS verification failed (possible MITM)
    #[error("SAS verification failed - possible man-in-the-middle attack")]
    SasVerificationFailed,

    /// QR code generation/parsing error
    #[error("QR code error: {context}")]
    QrCodeError {
        /// Context about the error
        context: String,
    },

    /// Key storage error (Secure Enclave/TPM)
    #[error("Key storage error: {context}")]
    KeyStorageError {
        /// Context about the error
        context: String,
    },

    /// Protocol version mismatch
    #[error("Protocol version mismatch: expected {expected}, got {actual}")]
    ProtocolVersionMismatch {
        /// Expected protocol version
        expected: u8,
        /// Actual protocol version received
        actual: u8,
    },

    /// Message format error
    #[error("Invalid message format: {context}")]
    InvalidMessageFormat {
        /// Context about the error
        context: String,
    },

    /// Replay attack detected
    #[error("Replay attack detected: {context}")]
    ReplayAttackDetected {
        /// Context about the detection
        context: String,
    },

    /// Rate limit exceeded
    #[error("Rate limit exceeded: {context}")]
    RateLimitExceeded {
        /// Context about the limit
        context: String,
    },

    /// WebSocket communication error
    #[error("WebSocket error: {context}")]
    WebSocketError {
        /// Context about the error
        context: String,
    },

    /// Serialization error
    #[error("Serialization error: {context}")]
    SerializationError {
        /// Context about the error
        context: String,
    },

    /// Compliance/audit error
    #[error("Compliance error: {context}")]
    ComplianceError {
        /// Context about the error
        context: String,
    },

    /// Export operation failed
    #[error("Export failed: {context}")]
    ExportFailed {
        /// Context about the failure
        context: String,
    },

    /// Invalid state transition
    #[error("Invalid state: {context}")]
    InvalidState {
        /// Context about the invalid state
        context: String,
    },

    /// Operation timed out
    #[error("Operation timed out: {context}")]
    Timeout {
        /// Context about the timeout
        context: String,
    },

    /// Generic internal error (avoid using when possible)
    #[error("Internal error: {context}")]
    InternalError {
        /// Context about the error
        context: String,
    },
}

impl CryptoError {
    /// Create an invalid key error
    pub fn invalid_key<S: Into<String>>(context: S) -> Self {
        Self::InvalidKey {
            context: context.into(),
        }
    }

    /// Create a key derivation failed error
    pub fn key_derivation_failed<S: Into<String>>(context: S) -> Self {
        Self::KeyDerivationFailed {
            context: context.into(),
        }
    }

    /// Create an encryption failed error
    pub fn encryption_failed<S: Into<String>>(context: S) -> Self {
        Self::EncryptionFailed {
            context: context.into(),
        }
    }

    /// Create a decryption failed error
    pub fn decryption_failed<S: Into<String>>(context: S) -> Self {
        Self::DecryptionFailed {
            context: context.into(),
        }
    }

    /// Create an authentication failed error
    pub fn authentication_failed<S: Into<String>>(context: S) -> Self {
        Self::AuthenticationFailed {
            context: context.into(),
        }
    }

    /// Create an invalid signature error
    pub fn invalid_signature<S: Into<String>>(context: S) -> Self {
        Self::InvalidSignature {
            context: context.into(),
        }
    }

    /// Create a handshake failed error
    pub fn handshake_failed<S: Into<String>>(context: S) -> Self {
        Self::HandshakeFailed {
            context: context.into(),
        }
    }

    /// Create a pairing failed error
    pub fn pairing_failed<S: Into<String>>(context: S) -> Self {
        Self::PairingFailed {
            context: context.into(),
        }
    }

    /// Create a QR code error
    pub fn qr_code_error<S: Into<String>>(context: S) -> Self {
        Self::QrCodeError {
            context: context.into(),
        }
    }

    /// Create a key storage error
    pub fn key_storage_error<S: Into<String>>(context: S) -> Self {
        Self::KeyStorageError {
            context: context.into(),
        }
    }

    /// Create an invalid message format error
    pub fn invalid_message_format<S: Into<String>>(context: S) -> Self {
        Self::InvalidMessageFormat {
            context: context.into(),
        }
    }

    /// Create a replay attack detected error
    pub fn replay_attack_detected<S: Into<String>>(context: S) -> Self {
        Self::ReplayAttackDetected {
            context: context.into(),
        }
    }

    /// Create a rate limit exceeded error
    pub fn rate_limit_exceeded<S: Into<String>>(context: S) -> Self {
        Self::RateLimitExceeded {
            context: context.into(),
        }
    }

    /// Create a WebSocket error
    pub fn websocket_error<S: Into<String>>(context: S) -> Self {
        Self::WebSocketError {
            context: context.into(),
        }
    }

    /// Create a serialization error
    pub fn serialization_error<S: Into<String>>(context: S) -> Self {
        Self::SerializationError {
            context: context.into(),
        }
    }

    /// Create a compliance error
    pub fn compliance_error<S: Into<String>>(context: S) -> Self {
        Self::ComplianceError {
            context: context.into(),
        }
    }

    /// Create an export failed error
    pub fn export_failed<S: Into<String>>(context: S) -> Self {
        Self::ExportFailed {
            context: context.into(),
        }
    }

    /// Create an invalid state error
    pub fn invalid_state<S: Into<String>>(context: S) -> Self {
        Self::InvalidState {
            context: context.into(),
        }
    }

    /// Create a timeout error
    pub fn timeout<S: Into<String>>(context: S) -> Self {
        Self::Timeout {
            context: context.into(),
        }
    }

    /// Create an internal error
    pub fn internal_error<S: Into<String>>(context: S) -> Self {
        Self::InternalError {
            context: context.into(),
        }
    }

    /// Returns true if this error indicates a potential security attack
    pub fn is_security_attack(&self) -> bool {
        matches!(
            self,
            Self::AuthenticationFailed { .. }
                | Self::InvalidSignature { .. }
                | Self::ReplayAttackDetected { .. }
                | Self::SasVerificationFailed
        )
    }

    /// Returns true if this error is retryable
    pub fn is_retryable(&self) -> bool {
        matches!(
            self,
            Self::RandomGenerationFailed { .. }
                | Self::WebSocketError { .. }
                | Self::Timeout { .. }
                | Self::RateLimitExceeded { .. }
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_creation() {
        let err = CryptoError::invalid_key("test key invalid");
        assert!(err.to_string().contains("test key invalid"));
    }

    #[test]
    fn test_is_security_attack() {
        assert!(CryptoError::authentication_failed("test").is_security_attack());
        assert!(CryptoError::invalid_signature("test").is_security_attack());
        assert!(CryptoError::replay_attack_detected("test").is_security_attack());
        assert!(CryptoError::SasVerificationFailed.is_security_attack());
        assert!(!CryptoError::invalid_key("test").is_security_attack());
    }

    #[test]
    fn test_is_retryable() {
        assert!(CryptoError::timeout("test").is_retryable());
        assert!(CryptoError::rate_limit_exceeded("test").is_retryable());
        assert!(!CryptoError::invalid_key("test").is_retryable());
    }
}
