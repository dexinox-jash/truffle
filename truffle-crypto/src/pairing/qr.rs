//! QR Code Generation and Parsing for Device Pairing
//!
//! This module implements QR code generation for the pairing ceremony.
//! The QR code contains all necessary information for the secondary device
//! to connect and perform the X3DH handshake.

use crate::error::{CryptoError, CryptoResult};
use crate::types::KeyFingerprint;
use crate::utils::{base64url_decode, base64url_encode};
use qrcode::{Color, QrCode, Version};
use serde::{Deserialize, Serialize};

/// QR code data structure for pairing
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct QrCodeData {
    /// QR code format version
    pub version: u8,
    /// Pairing session ID
    pub session_id: String,
    /// WebSocket relay endpoint
    pub relay_endpoint: String,
    /// One-time token for authentication
    pub one_time_token: String,
    /// Identity key fingerprint
    pub identity_fingerprint: KeyFingerprint,
    /// PreKey bundle (optional, may be fetched separately)
    #[serde(with = "serde_bytes", skip_serializing_if = "Option::is_none", default)]
    pub prekey_bundle: Option<Vec<u8>>,
}

impl QrCodeData {
    /// Create new QR code data
    pub fn new(
        session_id: String,
        relay_endpoint: String,
        one_time_token: String,
        identity_fingerprint: KeyFingerprint,
    ) -> Self {
        Self {
            version: 1,
            session_id,
            relay_endpoint,
            one_time_token,
            identity_fingerprint,
            prekey_bundle: None,
        }
    }

    /// Add PreKey bundle
    pub fn with_prekey_bundle(mut self, bundle: Vec<u8>) -> Self {
        self.prekey_bundle = Some(bundle);
        self
    }

    /// Serialize to compact string format
    ///
    /// Format: truffle://v1/{base64url-encoded-data}
    pub fn to_compact_string(&self) -> CryptoResult<String> {
        let json = serde_json::to_string(self)
            .map_err(|e| CryptoError::serialization_error(format!("QR data: {}", e)))?;

        let compressed = miniz_oxide::deflate::compress_to_vec(json.as_bytes(), 6);
        let encoded = base64url_encode(&compressed);

        Ok(format!("truffle://v1/{}", encoded))
    }

    /// Parse from compact string format
    pub fn from_compact_string(s: &str) -> CryptoResult<Self> {
        // Check prefix
        const PREFIX: &str = "truffle://v1/";
        if !s.starts_with(PREFIX) {
            return Err(CryptoError::invalid_message_format(
                "Invalid QR code format: missing prefix"
            ));
        }

        let encoded = &s[PREFIX.len()..];
        let compressed = base64url_decode(encoded)?;

        let json = miniz_oxide::inflate::decompress_to_vec(&compressed)
            .map_err(|e| CryptoError::serialization_error(format!("Decompression failed: {:?}", e)))?;

        let data: QrCodeData = serde_json::from_slice(&json)
            .map_err(|e| CryptoError::serialization_error(format!("JSON parse: {}", e)))?;

        // Validate version
        if data.version != 1 {
            return Err(CryptoError::invalid_message_format(
                format!("Unsupported QR code version: {}", data.version)
            ));
        }

        Ok(data)
    }

    /// Serialize to bytes
    pub fn to_bytes(&self) -> CryptoResult<Vec<u8>> {
        bincode::serialize(self)
            .map_err(|e| CryptoError::serialization_error(format!("QR data: {}", e)))
    }

    /// Deserialize from bytes
    pub fn from_bytes(bytes: &[u8]) -> CryptoResult<Self> {
        bincode::deserialize(bytes)
            .map_err(|e| CryptoError::serialization_error(format!("QR data: {}", e)))
    }
}

/// QR code generator
pub struct QrCodeGenerator;

impl QrCodeGenerator {
    /// Generate a QR code from pairing data
    ///
    /// Returns the QR code as an SVG string
    pub fn generate_svg(data: &QrCodeData) -> CryptoResult<String> {
        let compact = data.to_compact_string()?;

        // Generate QR code
        let code = QrCode::with_version(
            compact.as_bytes(),
            Version::Normal(crate::pairing::PAIRING_QR_VERSION),
            qrcode::EcLevel::M,
        ).map_err(|e| CryptoError::qr_code_error(format!("QR generation failed: {:?}", e)))?;

        // Render as SVG
        let svg = code.render::<svg::Color>()
            .min_dimensions(200, 200)
            .dark_color(svg::Color("#000000"))
            .light_color(svg::Color("#FFFFFF"))
            .build();

        Ok(svg)
    }

    /// Generate a QR code as a PNG image
    #[cfg(feature = "image")]
    pub fn generate_png(data: &QrCodeData, size: u32) -> CryptoResult<Vec<u8>> {
        use image::{Luma, ImageBuffer};

        let compact = data.to_compact_string()?;

        let code = QrCode::with_version(
            compact.as_bytes(),
            Version::Normal(crate::pairing::PAIRING_QR_VERSION),
            qrcode::EcLevel::M,
        ).map_err(|e| CryptoError::qr_code_error(format!("QR generation failed: {:?}", e)))?;

        let image = code.render::<Luma<u8>>()
            .min_dimensions(size, size)
            .build();

        let mut png_data = Vec::new();
        image.write_to(&mut std::io::Cursor::new(&mut png_data), image::ImageFormat::Png)
            .map_err(|e| CryptoError::qr_code_error(format!("PNG encoding failed: {}", e)))?;

        Ok(png_data)
    }

    /// Generate a QR code as ASCII art (for terminal display)
    pub fn generate_ascii(data: &QrCodeData) -> CryptoResult<String> {
        let compact = data.to_compact_string()?;

        let code = QrCode::with_version(
            compact.as_bytes(),
            Version::Normal(crate::pairing::PAIRING_QR_VERSION),
            qrcode::EcLevel::M,
        ).map_err(|e| CryptoError::qr_code_error(format!("QR generation failed: {:?}", e)))?;

        let string = code.render()
            .quiet_zone(false)
            .module_dimensions(2, 1)
            .build();

        Ok(string)
    }
}

/// QR code parser
pub struct QrCodeParser;

impl QrCodeParser {
    /// Parse QR code data from a string
    pub fn parse(s: &str) -> CryptoResult<QrCodeData> {
        // Try compact format first
        if s.starts_with("truffle://") {
            return QrCodeData::from_compact_string(s);
        }

        // Try base64url encoded
        if let Ok(decoded) = base64url_decode(s) {
            if let Ok(data) = QrCodeData::from_bytes(&decoded) {
                return Ok(data);
            }
        }

        // Try JSON
        if s.starts_with('{') {
            let data: QrCodeData = serde_json::from_str(s)
                .map_err(|e| CryptoError::serialization_error(format!("JSON parse: {}", e)))?;
            return Ok(data);
        }

        Err(CryptoError::invalid_message_format(
            "Unrecognized QR code format"
        ))
    }

    /// Parse from raw QR code bytes (e.g., from camera)
    pub fn parse_from_bytes(bytes: &[u8]) -> CryptoResult<QrCodeData> {
        // Try to decode as UTF-8 string first
        let s = std::str::from_utf8(bytes)
            .map_err(|_| CryptoError::invalid_message_format("Invalid UTF-8 in QR code"))?;

        Self::parse(s.trim())
    }
}

/// QR code scanner interface (placeholder for actual implementation)
pub trait QrCodeScanner {
    /// Start scanning
    fn start(&mut self) -> CryptoResult<()>;

    /// Stop scanning
    fn stop(&mut self) -> CryptoResult<()>;

    /// Check if a code was detected
    fn detected(&self) -> bool;

    /// Get the detected code
    fn get_code(&mut self) -> Option<String>;
}

/// Mock QR code scanner for testing
pub struct MockQrCodeScanner {
    codes: Vec<String>,
    index: usize,
    running: bool,
}

impl MockQrCodeScanner {
    /// Create a new mock scanner with predefined codes
    pub fn new(codes: Vec<String>) -> Self {
        Self {
            codes,
            index: 0,
            running: false,
        }
    }
}

impl QrCodeScanner for MockQrCodeScanner {
    fn start(&mut self) -> CryptoResult<()> {
        self.running = true;
        Ok(())
    }

    fn stop(&mut self) -> CryptoResult<()> {
        self.running = false;
        Ok(())
    }

    fn detected(&self) -> bool {
        self.running && self.index < self.codes.len()
    }

    fn get_code(&mut self) -> Option<String> {
        if self.detected() {
            let code = self.codes[self.index].clone();
            self.index += 1;
            Some(code)
        } else {
            None
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_qr_code_data_creation() {
        let data = QrCodeData::new(
            "session-123".to_string(),
            "wss://relay.truffle.io".to_string(),
            "token-abc".to_string(),
            KeyFingerprint::from_public_key(&[0u8; 32]),
        );

        assert_eq!(data.version, 1);
        assert_eq!(data.session_id, "session-123");
    }

    #[test]
    fn test_qr_code_data_with_prekey() {
        let data = QrCodeData::new(
            "session-123".to_string(),
            "wss://relay.truffle.io".to_string(),
            "token-abc".to_string(),
            KeyFingerprint::from_public_key(&[0u8; 32]),
        ).with_prekey_bundle(vec![1, 2, 3, 4]);

        assert_eq!(data.prekey_bundle, Some(vec![1, 2, 3, 4]));
    }

    #[test]
    fn test_qr_code_compact_string_roundtrip() {
        let original = QrCodeData::new(
            "session-123".to_string(),
            "wss://relay.truffle.io".to_string(),
            "token-abc".to_string(),
            KeyFingerprint::from_public_key(&[0u8; 32]),
        );

        let compact = original.to_compact_string().unwrap();
        assert!(compact.starts_with("truffle://v1/"));

        let recovered = QrCodeData::from_compact_string(&compact).unwrap();
        assert_eq!(original.session_id, recovered.session_id);
        assert_eq!(original.relay_endpoint, recovered.relay_endpoint);
        assert_eq!(original.identity_fingerprint, recovered.identity_fingerprint);
    }

    #[test]
    fn test_qr_code_bytes_roundtrip() {
        let original = QrCodeData::new(
            "session-123".to_string(),
            "wss://relay.truffle.io".to_string(),
            "token-abc".to_string(),
            KeyFingerprint::from_public_key(&[0u8; 32]),
        );

        let bytes = original.to_bytes().unwrap();
        let recovered = QrCodeData::from_bytes(&bytes).unwrap();

        assert_eq!(original, recovered);
    }

    #[test]
    fn test_qr_code_generator_svg() {
        let data = QrCodeData::new(
            "session-123".to_string(),
            "wss://relay.truffle.io".to_string(),
            "token-abc".to_string(),
            KeyFingerprint::from_public_key(&[0u8; 32]),
        );

        let svg = QrCodeGenerator::generate_svg(&data).unwrap();
        assert!(svg.contains("<svg"));
        assert!(svg.contains("</svg>"));
    }

    #[test]
    fn test_qr_code_generator_ascii() {
        let data = QrCodeData::new(
            "session-123".to_string(),
            "wss://relay.truffle.io".to_string(),
            "token-abc".to_string(),
            KeyFingerprint::from_public_key(&[0u8; 32]),
        );

        let ascii = QrCodeGenerator::generate_ascii(&data).unwrap();
        assert!(!ascii.is_empty());
        // ASCII QR codes use '#' and ' ' characters
        assert!(ascii.contains('#') || ascii.contains('█'));
    }

    #[test]
    fn test_qr_code_parser_compact() {
        let data = QrCodeData::new(
            "session-123".to_string(),
            "wss://relay.truffle.io".to_string(),
            "token-abc".to_string(),
            KeyFingerprint::from_public_key(&[0u8; 32]),
        );

        let compact = data.to_compact_string().unwrap();
        let parsed = QrCodeParser::parse(&compact).unwrap();

        assert_eq!(data.session_id, parsed.session_id);
    }

    #[test]
    fn test_qr_code_parser_json() {
        let data = QrCodeData::new(
            "session-123".to_string(),
            "wss://relay.truffle.io".to_string(),
            "token-abc".to_string(),
            KeyFingerprint::from_public_key(&[0u8; 32]),
        );

        let json = serde_json::to_string(&data).unwrap();
        let parsed = QrCodeParser::parse(&json).unwrap();

        assert_eq!(data.session_id, parsed.session_id);
    }

    #[test]
    fn test_qr_code_parser_invalid() {
        let result = QrCodeParser::parse("invalid data");
        assert!(result.is_err());
    }

    #[test]
    fn test_mock_qr_scanner() {
        let codes = vec![
            "truffle://v1/test1".to_string(),
            "truffle://v1/test2".to_string(),
        ];

        let mut scanner = MockQrCodeScanner::new(codes);

        assert!(!scanner.detected());
        scanner.start().unwrap();
        assert!(scanner.detected());

        let code1 = scanner.get_code();
        assert!(code1.is_some());
        assert_eq!(code1.unwrap(), "truffle://v1/test1");

        let code2 = scanner.get_code();
        assert!(code2.is_some());
        assert_eq!(code2.unwrap(), "truffle://v1/test2");

        assert!(!scanner.detected());
    }
}
