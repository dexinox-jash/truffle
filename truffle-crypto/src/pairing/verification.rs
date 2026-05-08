//! SAS (Short Authentication String) Verification
//!
//! This module implements the 6-digit SAS verification as specified
//! in Section 2.3.2 of the Truffle specification.
//!
//! SAS verification protects against man-in-the-middle attacks by
//! requiring users to compare a short code on both devices.

use crate::error::{CryptoError, CryptoResult};
use crate::utils::hkdf_sha256;
use constant_time_eq::constant_time_eq;

/// Default SAS digit count
pub const DEFAULT_SAS_DIGITS: usize = 6;

/// Maximum SAS digit count
pub const MAX_SAS_DIGITS: usize = 8;

/// Minimum SAS digit count
pub const MIN_SAS_DIGITS: usize = 4;

/// SAS generator
pub struct SasGenerator;

impl SasGenerator {
    /// Generate a SAS code from shared secrets
    ///
    /// # Arguments
    ///
    /// * `shared_secret` - The shared secret from X3DH
    /// * `device_a` - Bytes identifying device A
    /// * `device_b` - Bytes identifying device B
    /// * `digit_count` - Number of digits (4-8)
    ///
    /// # Returns
    ///
    /// A string of digits (e.g., "123456")
    pub fn generate(
        shared_secret: &[u8],
        device_a: &[u8],
        device_b: &[u8],
        digit_count: usize,
    ) -> String {
        // Validate digit count
        let digits = digit_count.clamp(MIN_SAS_DIGITS, MAX_SAS_DIGITS);

        // Sort device identifiers for consistent ordering
        let (first, second) = if device_a <= device_b {
            (device_a, device_b)
        } else {
            (device_b, device_a)
        };

        // Derive SAS bytes using HKDF
        let info = crate::utils::concat_bytes(&[
            b"truffle-sas-v1",
            first,
            second,
        ]);

        let sas_bytes = hkdf_sha256(&[], shared_secret, &info, 4)
            .unwrap_or_default();

        // Convert to a number
        let sas_value = u32::from_be_bytes([
            sas_bytes[0],
            sas_bytes[1],
            sas_bytes[2],
            sas_bytes[3],
        ]);

        // Format as digits
        let max_value = 10u32.pow(digits as u32);
        let formatted = format!("{:0digits$}", sas_value % max_value, digits = digits);

        formatted
    }

    /// Generate a 6-digit SAS (default)
    pub fn generate_6_digit(
        shared_secret: &[u8],
        device_a: &[u8],
        device_b: &[u8],
    ) -> String {
        Self::generate(shared_secret, device_a, device_b, 6)
    }

    /// Generate SAS with custom digit count
    pub fn generate_with_digits(
        shared_secret: &[u8],
        device_a: &[u8],
        device_b: &[u8],
        digits: usize,
    ) -> String {
        Self::generate(shared_secret, device_a, device_b, digits)
    }
}

/// SAS verifier
pub struct SasVerifier;

impl SasVerifier {
    /// Verify a SAS code in constant time
    ///
    /// This uses constant-time comparison to prevent timing attacks.
    ///
    /// # Arguments
    ///
    /// * `expected` - The expected SAS code
    /// * `provided` - The provided SAS code to verify
    ///
    /// # Returns
    ///
    /// `true` if the codes match, `false` otherwise
    pub fn verify(expected: &str, provided: &str) -> bool {
        // Check length first (not secret, so not timing-sensitive)
        if expected.len() != provided.len() {
            return false;
        }

        // Constant-time comparison
        constant_time_eq(expected.as_bytes(), provided.as_bytes())
    }

    /// Verify and validate a SAS code
    ///
    /// Also checks that the code contains only digits.
    pub fn verify_valid(expected: &str, provided: &str) -> CryptoResult<bool> {
        // Validate that codes contain only digits
        if !Self::is_valid_sas(expected) {
            return Err(CryptoError::invalid_message_format(
                "Expected SAS contains non-digit characters"
            ));
        }

        if !Self::is_valid_sas(provided) {
            return Err(CryptoError::invalid_message_format(
                "Provided SAS contains non-digit characters"
            ));
        }

        Ok(Self::verify(expected, provided))
    }

    /// Check if a string is a valid SAS code
    pub fn is_valid_sas(s: &str) -> bool {
        let len = s.len();
        if len < MIN_SAS_DIGITS || len > MAX_SAS_DIGITS {
            return false;
        }

        s.chars().all(|c| c.is_ascii_digit())
    }

    /// Format a SAS code for display (adds spaces for readability)
    ///
    /// Example: "123456" -> "123 456"
    pub fn format_for_display(sas: &str) -> String {
        match sas.len() {
            4 => format!("{} {}", &sas[0..2], &sas[2..4]),
            6 => format!("{} {}", &sas[0..3], &sas[3..6]),
            8 => format!("{} {} {}", &sas[0..2], &sas[2..5], &sas[5..8]),
            _ => sas.to_string(),
        }
    }

    /// Compare two SAS codes and return detailed result
    pub fn compare(expected: &str, provided: &str) -> SasComparisonResult {
        if expected.len() != provided.len() {
            return SasComparisonResult::LengthMismatch;
        }

        if !Self::is_valid_sas(expected) || !Self::is_valid_sas(provided) {
            return SasComparisonResult::InvalidFormat;
        }

        if Self::verify(expected, provided) {
            SasComparisonResult::Match
        } else {
            // Count matching digits (for debugging, not security)
            let matches = expected.chars()
                .zip(provided.chars())
                .filter(|(a, b)| a == b)
                .count();

            SasComparisonResult::Mismatch { matching_digits: matches }
        }
    }
}

/// Result of SAS comparison
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SasComparisonResult {
    /// SAS codes match
    Match,
    /// SAS codes don't match
    Mismatch { matching_digits: usize },
    /// Length mismatch
    LengthMismatch,
    /// Invalid format (non-digit characters)
    InvalidFormat,
}

impl SasComparisonResult {
    /// Check if the comparison was successful
    pub fn is_match(&self) -> bool {
        matches!(self, Self::Match)
    }

    /// Check if the comparison failed
    pub fn is_mismatch(&self) -> bool {
        !self.is_match()
    }

    /// Get a user-friendly message
    pub fn message(&self) -> &'static str {
        match self {
            Self::Match => "SAS codes match! Devices are paired securely.",
            Self::Mismatch { .. } => "SAS codes do not match. Possible man-in-the-middle attack!",
            Self::LengthMismatch => "SAS codes have different lengths.",
            Self::InvalidFormat => "SAS code contains invalid characters.",
        }
    }
}

/// SAS display helper for UI
pub struct SasDisplay;

impl SasDisplay {
    /// Get emoji representation of SAS digits
    ///
    /// This can help users compare codes more easily.
    pub fn to_emojis(sas: &str) -> Vec<&'static str> {
        const DIGIT_EMOJIS: &[&str] = &[
            "0️⃣", "1️⃣", "2️⃣", "3️⃣", "4️⃣",
            "5️⃣", "6️⃣", "7️⃣", "8️⃣", "9️⃣",
        ];

        sas.chars()
            .filter_map(|c| c.to_digit(10))
            .map(|d| DIGIT_EMOJIS[d as usize])
            .collect()
    }

    /// Format SAS with emoji representation
    pub fn format_with_emojis(sas: &str) -> String {
        let emojis = Self::to_emojis(sas);
        emojis.join(" ")
    }

    /// Get spoken representation for accessibility
    pub fn to_spoken(sas: &str) -> String {
        sas.chars()
            .map(|c| match c {
                '0' => "zero",
                '1' => "one",
                '2' => "two",
                '3' => "three",
                '4' => "four",
                '5' => "five",
                '6' => "six",
                '7' => "seven",
                '8' => "eight",
                '9' => "nine",
                _ => "",
            })
            .collect::<Vec<_>>()
            .join(" ")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sas_generation() {
        let shared_secret = b"test shared secret";
        let device_a = b"device_a";
        let device_b = b"device_b";

        let sas = SasGenerator::generate(shared_secret, device_a, device_b, 6);

        assert_eq!(sas.len(), 6);
        assert!(SasVerifier::is_valid_sas(&sas));

        // Same inputs should produce same SAS
        let sas2 = SasGenerator::generate(shared_secret, device_a, device_b, 6);
        assert_eq!(sas, sas2);

        // Different inputs should produce different SAS
        let sas3 = SasGenerator::generate(b"different", device_a, device_b, 6);
        assert_ne!(sas, sas3);
    }

    #[test]
    fn test_sas_generation_different_lengths() {
        let shared_secret = b"test";
        let device_a = b"a";
        let device_b = b"b";

        let sas4 = SasGenerator::generate(shared_secret, device_a, device_b, 4);
        assert_eq!(sas4.len(), 4);

        let sas6 = SasGenerator::generate(shared_secret, device_a, device_b, 6);
        assert_eq!(sas6.len(), 6);

        let sas8 = SasGenerator::generate(shared_secret, device_a, device_b, 8);
        assert_eq!(sas8.len(), 8);

        // Out of range should be clamped
        let sas_too_small = SasGenerator::generate(shared_secret, device_a, device_b, 2);
        assert_eq!(sas_too_small.len(), MIN_SAS_DIGITS);

        let sas_too_large = SasGenerator::generate(shared_secret, device_a, device_b, 10);
        assert_eq!(sas_too_large.len(), MAX_SAS_DIGITS);
    }

    #[test]
    fn test_sas_device_order_independence() {
        let shared_secret = b"test";
        let device_a = b"device_a";
        let device_b = b"device_b";

        // Order of devices should not matter
        let sas1 = SasGenerator::generate(shared_secret, device_a, device_b, 6);
        let sas2 = SasGenerator::generate(shared_secret, device_b, device_a, 6);
        assert_eq!(sas1, sas2);
    }

    #[test]
    fn test_sas_verification() {
        let expected = "123456";
        let provided = "123456";

        assert!(SasVerifier::verify(expected, provided));
        assert!(!SasVerifier::verify(expected, "654321"));
        assert!(!SasVerifier::verify(expected, "12345"));
    }

    #[test]
    fn test_sas_verification_constant_time() {
        // These should all take roughly the same time
        assert!(!SasVerifier::verify("123456", "000000"));
        assert!(!SasVerifier::verify("123456", "123455"));
        assert!(!SasVerifier::verify("123456", "023456"));
    }

    #[test]
    fn test_sas_validation() {
        assert!(SasVerifier::is_valid_sas("123456"));
        assert!(SasVerifier::is_valid_sas("1234"));
        assert!(SasVerifier::is_valid_sas("12345678"));

        assert!(!SasVerifier::is_valid_sas("123")); // Too short
        assert!(!SasVerifier::is_valid_sas("123456789")); // Too long
        assert!(!SasVerifier::is_valid_sas("12345a")); // Non-digit
        assert!(!SasVerifier::is_valid_sas("")); // Empty
    }

    #[test]
    fn test_sas_format_for_display() {
        assert_eq!(SasVerifier::format_for_display("1234"), "12 34");
        assert_eq!(SasVerifier::format_for_display("123456"), "123 456");
        assert_eq!(SasVerifier::format_for_display("12345678"), "12 345 678");
    }

    #[test]
    fn test_sas_comparison() {
        assert!(SasVerifier::compare("123456", "123456").is_match());
        assert!(SasVerifier::compare("123456", "654321").is_mismatch());

        let result = SasVerifier::compare("123456", "123455");
        match result {
            SasComparisonResult::Mismatch { matching_digits } => {
                assert_eq!(matching_digits, 5);
            }
            _ => panic!("Expected Mismatch"),
        }

        assert_eq!(
            SasVerifier::compare("123456", "12345"),
            SasComparisonResult::LengthMismatch
        );

        assert_eq!(
            SasVerifier::compare("12345a", "12345a"),
            SasComparisonResult::InvalidFormat
        );
    }

    #[test]
    fn test_sas_display_emojis() {
        let emojis = SasDisplay::to_emojis("123");
        assert_eq!(emojis.len(), 3);
        assert_eq!(emojis[0], "1️⃣");
        assert_eq!(emojis[1], "2️⃣");
        assert_eq!(emojis[2], "3️⃣");

        let formatted = SasDisplay::format_with_emojis("42");
        assert_eq!(formatted, "4️⃣ 2️⃣");
    }

    #[test]
    fn test_sas_display_spoken() {
        let spoken = SasDisplay::to_spoken("123");
        assert_eq!(spoken, "one two three");
    }

    #[test]
    fn test_sas_comparison_result_messages() {
        assert!(SasComparisonResult::Match.is_match());
        assert!(!SasComparisonResult::Mismatch { matching_digits: 0 }.is_match());

        assert!(SasComparisonResult::Mismatch { matching_digits: 0 }.message().contains("not match"));
        assert!(SasComparisonResult::Match.message().contains("match"));
    }
}
