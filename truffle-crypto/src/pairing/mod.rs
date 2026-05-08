//! Device Pairing Module
//!
//! This module implements the device pairing ceremony as specified
//! in Section 2.3.2 of the Truffle specification.
//!
//! ## Pairing Flow
//!
//! 1. **Primary Device** generates Ed25519 identity key + X25519 ephemeral keys
//! 2. **QR Code Generation** contains public_key_fingerprint + websocket_endpoint + one_time_token
//! 3. **Secondary Device** scans, generates own keys, performs X3DH handshake via WebSocket relay
//! 4. **Shared Secret Derivation** using HKDF-SHA256
//! 5. **SAS Verification** 6-digit comparison on both screens (prevent MITM)

mod ceremony;
mod qr;
mod verification;
mod websocket;

pub use ceremony::{
    PairingCeremony, PairingConfig, PairingSession, PairingState,
    PrimaryDevice, SecondaryDevice,
};
pub use qr::{QrCodeData, QrCodeGenerator, QrCodeParser};
pub use verification::{SasGenerator, SasVerifier};
pub use websocket::{PairingWebSocket, RelayClient, WebSocketConfig};

use crate::error::CryptoResult;

/// Default WebSocket relay endpoint
pub const DEFAULT_RELAY_ENDPOINT: &str = "wss://relay.truffle.io/v1/pairing";

/// Default pairing timeout in seconds
pub const DEFAULT_PAIRING_TIMEOUT_SECS: u64 = 300; // 5 minutes

/// QR code version for pairing
pub const PAIRING_QR_VERSION: i16 = 10;

/// Initialize the pairing module
pub fn init() -> CryptoResult<()> {
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_constants() {
        assert_eq!(DEFAULT_PAIRING_TIMEOUT_SECS, 300);
    }
}
