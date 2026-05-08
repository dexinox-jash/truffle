//! Utility functions for cryptographic operations
//!
//! This module provides helper functions that are used across
//! the cryptographic implementation.

use crate::error::{CryptoError, CryptoResult};
use constant_time_eq::constant_time_eq;
use sha2::{Digest, Sha256};
use zeroize::Zeroize;

/// Securely compare two byte arrays in constant time
///
/// This prevents timing attacks by ensuring the comparison
/// takes the same amount of time regardless of where the
/// arrays differ.
pub fn secure_compare(a: &[u8], b: &[u8]) -> bool {
    constant_time_eq(a, b)
}

/// Hash data using SHA3-256
///
/// Used for content addressing and blob identification.
pub fn sha3_256(data: &[u8]) -> [u8; 32] {
    use sha3::Sha3_256;
    let mut hasher = Sha3_256::new();
    hasher.update(data);
    let result = hasher.finalize();
    let mut output = [0u8; 32];
    output.copy_from_slice(&result);
    output
}

/// Hash data using SHA2-256
///
/// Used for general-purpose hashing.
pub fn sha2_256(data: &[u8]) -> [u8; 32] {
    let mut hasher = Sha256::new();
    hasher.update(data);
    let result = hasher.finalize();
    let mut output = [0u8; 32];
    output.copy_from_slice(&result);
    output
}

/// Compute HMAC-SHA256
///
/// Used for message authentication.
///
/// # Errors
///
/// Returns `CryptoError::InternalError` if HMAC computation fails.
pub fn hmac_sha256(key: &[u8], data: &[u8]) -> CryptoResult<[u8; 32]> {
    use hmac::Mac;
    type HmacSha256 = hmac::Hmac<Sha256>;

    let mut mac = HmacSha256::new_from_slice(key)
        .map_err(|e| CryptoError::InternalError(format!("HMAC initialization failed: {:?}", e)))?;
    mac.update(data);
    let result = mac.finalize();
    let mut output = [0u8; 32];
    output.copy_from_slice(&result.into_bytes());
    Ok(output)
}

/// Verify HMAC-SHA256 in constant time
///
/// Returns true if the HMAC matches, false otherwise.
/// Uses constant-time comparison to prevent timing attacks.
///
/// # Errors
///
/// Returns `CryptoError::InternalError` if HMAC computation fails.
pub fn verify_hmac_sha256(key: &[u8], data: &[u8], expected: &[u8]) -> CryptoResult<bool> {
    let computed = hmac_sha256(key, data)?;
    Ok(secure_compare(&computed, expected))
}

/// Derive a key using HKDF-SHA256
///
/// This implements the HKDF extract-then-expand pattern
/// as specified in RFC 5869.
pub fn hkdf_sha256(
    salt: &[u8],
    ikm: &[u8],
    info: &[u8],
    output_len: usize,
) -> CryptoResult<Vec<u8>> {
    use hkdf::Hkdf;

    let hk = Hkdf::<Sha256>::new(Some(salt), ikm);
    let mut okm = vec![0u8; output_len];
    hk.expand(info, &mut okm)
        .map_err(|e| CryptoError::key_derivation_failed(format!("HKDF failed: {}", e)))?;
    Ok(okm)
}

/// Derive multiple keys from a single IKM using HKDF
///
/// This is more efficient than calling HKDF multiple times.
pub fn hkdf_sha256_derive_multiple(
    salt: &[u8],
    ikm: &[u8],
    contexts: &[&[u8]],
) -> CryptoResult<Vec<Vec<u8>>> {
    use hkdf::Hkdf;

    let hk = Hkdf::<Sha256>::new(Some(salt), ikm);
    let mut results = Vec::with_capacity(contexts.len());

    for info in contexts {
        let mut okm = vec![0u8; 32]; // Default to 256-bit keys
        hk.expand(info, &mut okm)
            .map_err(|e| CryptoError::key_derivation_failed(format!("HKDF failed: {}", e)))?;
        results.push(okm);
    }

    Ok(results)
}

/// Generate a random byte array
///
/// Uses the OS CSPRNG via getrandom.
pub fn random_bytes<const N: usize>() -> CryptoResult<[u8; N]> {
    let mut bytes = [0u8; N];
    getrandom::getrandom(&mut bytes)
        .map_err(|e| CryptoError::RandomGenerationFailed(e.to_string()))?;
    Ok(bytes)
}

/// Generate a random byte vector
///
/// Uses the OS CSPRNG via getrandom.
pub fn random_vec(len: usize) -> CryptoResult<Vec<u8>> {
    let mut bytes = vec![0u8; len];
    getrandom::getrandom(&mut bytes)
        .map_err(|e| CryptoError::RandomGenerationFailed(e.to_string()))?;
    Ok(bytes)
}

/// XOR two byte arrays together
///
/// The output is written to the first array.
/// Arrays must be the same length.
pub fn xor_in_place(a: &mut [u8], b: &[u8]) -> CryptoResult<()> {
    if a.len() != b.len() {
        return Err(CryptoError::invalid_key("Arrays must be same length for XOR"));
    }

    for (x, y) in a.iter_mut().zip(b.iter()) {
        *x ^= y;
    }

    Ok(())
}

/// Concatenate multiple byte slices
///
/// Efficiently concatenates multiple slices into a single vector.
pub fn concat_bytes(slices: &[&[u8]]) -> Vec<u8> {
    let total_len: usize = slices.iter().map(|s| s.len()).sum();
    let mut result = Vec::with_capacity(total_len);

    for slice in slices {
        result.extend_from_slice(slice);
    }

    result
}

/// Encode bytes to base64url (URL-safe base64)
pub fn base64url_encode(data: &[u8]) -> String {
    use base64::{engine::general_purpose::URL_SAFE_NO_PAD, Engine};
    URL_SAFE_NO_PAD.encode(data)
}

/// Decode base64url (URL-safe base64) to bytes
pub fn base64url_decode(data: &str) -> CryptoResult<Vec<u8>> {
    use base64::{engine::general_purpose::URL_SAFE_NO_PAD, Engine};
    URL_SAFE_NO_PAD
        .decode(data)
        .map_err(|e| CryptoError::serialization_error(format!("Base64 decode failed: {}", e)))
}

/// Encode bytes to base32 (for human-readable codes)
pub fn base32_encode(data: &[u8]) -> String {
    use base32::Alphabet;
    base32::encode(Alphabet::RFC4648 { padding: false }, data)
}

/// Decode base32 to bytes
pub fn base32_decode(data: &str) -> CryptoResult<Vec<u8>> {
    use base32::Alphabet;
    base32::decode(Alphabet::RFC4648 { padding: false }, data)
        .ok_or_else(|| CryptoError::serialization_error("Base32 decode failed"))
}

/// Truncate a byte array to a smaller size
///
/// Used for creating fingerprints and identifiers.
pub fn truncate_bytes<const N: usize>(data: &[u8]) -> [u8; N] {
    let mut result = [0u8; N];
    let len = data.len().min(N);
    result[..len].copy_from_slice(&data[..len]);
    result
}

/// Check if all bytes in a slice are zero
///
/// Used to validate that keys have been properly initialized.
pub fn is_all_zeros(data: &[u8]) -> bool {
    data.iter().all(|&b| b == 0)
}

/// Securely clear a vector
///
/// Overwrites the vector with zeros before clearing.
pub fn secure_clear_vec(vec: &mut Vec<u8>) {
    vec.zeroize();
    vec.clear();
}

/// Compute a 6-digit SAS (Short Authentication String)
///
/// This is used for verifying key fingerprints during pairing.
/// The SAS is derived from the shared secret using a KDF.
pub fn compute_sas(shared_secret: &[u8], device_a: &[u8], device_b: &[u8]) -> String {
    // Use HKDF to derive a 4-byte value
    let info = concat_bytes(&[b"truffle-sas-v1", device_a, device_b]);
    let derived = hkdf_sha256(&[], shared_secret, &info, 4).unwrap_or_default();

    // Convert to a 6-digit number
    let value = u32::from_be_bytes([derived[0], derived[1], derived[2], derived[3]]);
    format!("{:06}", value % 1_000_000)
}

/// Verify a 6-digit SAS
///
/// Returns true if the SAS matches the expected value.
pub fn verify_sas(sas: &str, shared_secret: &[u8], device_a: &[u8], device_b: &[u8]) -> bool {
    let expected = compute_sas(shared_secret, device_a, device_b);
    constant_time_eq(sas.as_bytes(), expected.as_bytes())
}

/// Validate that a string contains only digits
pub fn is_valid_digits(s: &str) -> bool {
    !s.is_empty() && s.chars().all(|c| c.is_ascii_digit())
}

/// Compute BLAKE2b hash (for non-cryptographic use cases)
///
/// Faster than SHA3-256 for large data.
///
/// # Errors
///
/// Returns `CryptoError::InternalError` if the hash computation fails.
pub fn blake2b_256(data: &[u8]) -> CryptoResult<[u8; 32]> {
    use blake2::digest::{Update, VariableOutput};
    use blake2::Blake2bVar;

    let mut hasher = Blake2bVar::new(32)
        .map_err(|e| CryptoError::InternalError(format!("BLAKE2b init failed: {:?}", e)))?;
    hasher.update(data);
    let mut result = [0u8; 32];
    hasher.finalize_variable(&mut result)
        .map_err(|e| CryptoError::InternalError(format!("BLAKE2b finalize failed: {:?}", e)))?;
    Ok(result)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_secure_compare() {
        let a = [1, 2, 3, 4];
        let b = [1, 2, 3, 4];
        let c = [1, 2, 3, 5];

        assert!(secure_compare(&a, &b));
        assert!(!secure_compare(&a, &c));
    }

    #[test]
    fn test_sha3_256() {
        let data = b"hello world";
        let hash1 = sha3_256(data);
        let hash2 = sha3_256(data);
        assert_eq!(hash1, hash2);

        let hash3 = sha3_256(b"different");
        assert_ne!(hash1, hash3);
    }

    #[test]
    fn test_hmac_sha256() {
        let key = b"secret key";
        let data = b"message to authenticate";
        let mac1 = hmac_sha256(key, data);
        let mac2 = hmac_sha256(key, data);
        assert_eq!(mac1, mac2);

        let mac3 = hmac_sha256(b"different key", data);
        assert_ne!(mac1, mac3);
    }

    #[test]
    fn test_verify_hmac_sha256() {
        let key = b"secret key";
        let data = b"message to authenticate";
        let mac = hmac_sha256(key, data);

        assert!(verify_hmac_sha256(key, data, &mac));
        assert!(!verify_hmac_sha256(b"wrong key", data, &mac));
    }

    #[test]
    fn test_hkdf_sha256() {
        let salt = b"salt";
        let ikm = b"input key material";
        let info = b"application info";

        let key1 = hkdf_sha256(salt, ikm, info, 32).unwrap();
        let key2 = hkdf_sha256(salt, ikm, info, 32).unwrap();
        assert_eq!(key1, key2);

        let key3 = hkdf_sha256(b"different salt", ikm, info, 32).unwrap();
        assert_ne!(key1, key3);
    }

    #[test]
    fn test_random_bytes() {
        let r1: [u8; 32] = random_bytes().unwrap();
        let r2: [u8; 32] = random_bytes().unwrap();
        assert_ne!(r1, r2);
    }

    #[test]
    fn test_xor_in_place() {
        let mut a = [0b1010, 0b1100];
        let b = [0b1000, 0b0100];
        xor_in_place(&mut a, &b).unwrap();
        assert_eq!(a, [0b0010, 0b1000]);
    }

    #[test]
    fn test_base64url_roundtrip() {
        let data = b"hello world!";
        let encoded = base64url_encode(data);
        let decoded = base64url_decode(&encoded).unwrap();
        assert_eq!(data.to_vec(), decoded);
    }

    #[test]
    fn test_compute_sas() {
        let secret = b"shared secret";
        let device_a = b"device_a";
        let device_b = b"device_b";

        let sas = compute_sas(secret, device_a, device_b);
        assert_eq!(sas.len(), 6);
        assert!(is_valid_digits(&sas));

        // Same inputs should produce same SAS
        let sas2 = compute_sas(secret, device_a, device_b);
        assert_eq!(sas, sas2);

        // Different inputs should produce different SAS
        let sas3 = compute_sas(b"different", device_a, device_b);
        assert_ne!(sas, sas3);
    }

    #[test]
    fn test_verify_sas() {
        let secret = b"shared secret";
        let device_a = b"device_a";
        let device_b = b"device_b";

        let sas = compute_sas(secret, device_a, device_b);
        assert!(verify_sas(&sas, secret, device_a, device_b));
        assert!(!verify_sas("000000", secret, device_a, device_b));
    }

    #[test]
    fn test_is_all_zeros() {
        assert!(is_all_zeros(&[0, 0, 0]));
        assert!(!is_all_zeros(&[0, 0, 1]));
        assert!(!is_all_zeros(&[1, 0, 0]));
    }
}
