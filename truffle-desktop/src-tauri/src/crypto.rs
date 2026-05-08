// SPEC v2.0 SECTION 2.3: Cryptographic Operations
// Zero-knowledge encryption for sync

use anyhow::{Context, Result};
use ring::aead::{self, Aad, Nonce, UnboundKey, AES_256_GCM};
use ring::rand::{SecureRandom, SystemRandom};

/// Encrypt data using AES-256-GCM
pub fn encrypt_aes_gcm(plaintext: &[u8], key: &[u8; 32]) -> Result<Vec<u8>> {
    let rng = SystemRandom::new();
    
    // Generate random nonce
    let mut nonce_bytes = [0u8; 12];
    rng.fill(&mut nonce_bytes)
        .context("Failed to generate nonce")?;
    
    // Create sealing key
    let unbound_key = UnboundKey::new(&AES_256_GCM, key)
        .context("Invalid key")?;
    let sealing_key = aead::LessSafeKey::new(unbound_key);
    
    // Encrypt
    let mut ciphertext = plaintext.to_vec();
    sealing_key
        .seal_in_place_append_tag(
            Nonce::assume_unique_for_key(nonce_bytes),
            Aad::empty(),
            &mut ciphertext,
        )
        .context("Encryption failed")?;
    
    // Prepend nonce to ciphertext
    let mut result = nonce_bytes.to_vec();
    result.extend_from_slice(&ciphertext);
    
    Ok(result)
}

/// Decrypt data using AES-256-GCM
pub fn decrypt_aes_gcm(ciphertext: &[u8], key: &[u8; 32]) -> Result<Vec<u8>> {
    if ciphertext.len() < 12 {
        return Err(anyhow::anyhow!("Ciphertext too short"));
    }
    
    // Extract nonce
    let nonce_bytes = &ciphertext[..12];
    let encrypted_data = &ciphertext[12..];
    
    // Create opening key
    let unbound_key = UnboundKey::new(&AES_256_GCM, key)
        .context("Invalid key")?;
    let opening_key = aead::LessSafeKey::new(unbound_key);
    
    // Decrypt
    let mut plaintext = encrypted_data.to_vec();
    opening_key
        .open_in_place(
            Nonce::assume_unique_for_key(nonce_bytes.try_into()?),
            Aad::empty(),
            &mut plaintext,
        )
        .context("Decryption failed")?;
    
    // Remove tag (last 16 bytes)
    plaintext.truncate(plaintext.len() - 16);
    
    Ok(plaintext)
}

/// Generate a random 256-bit key
pub fn generate_key() -> Result<[u8; 32]> {
    let rng = SystemRandom::new();
    let mut key = [0u8; 32];
    rng.fill(&mut key)
        .context("Failed to generate key")?;
    Ok(key)
}

/// Derive key using HKDF
pub fn derive_key(input_key_material: &[u8], salt: &[u8], info: &[u8]) -> Result<[u8; 32]> {
    use hkdf::Hkdf;
    use sha2::Sha256;
    
    let hkdf = Hkdf::<Sha256>::new(Some(salt), input_key_material);
    let mut okm = [0u8; 32];
    hkdf.expand(info, &mut okm)
        .context("HKDF expansion failed")?;
    
    Ok(okm)
}

/// Hash data using SHA-256
pub fn sha256(data: &[u8]) -> [u8; 32] {
    use sha2::Digest;
    let mut hasher = sha2::Sha256::new();
    hasher.update(data);
    hasher.finalize().into()
}

/// Generate Ed25519 keypair for device identity
pub fn generate_identity_keypair() -> Result<(Vec<u8>, Vec<u8>)> {
    use ed25519_dalek::SigningKey;
    use rand::rngs::OsRng;
    
    let signing_key = SigningKey::generate(&mut OsRng);
    let verifying_key = signing_key.verifying_key();
    
    Ok((
        signing_key.to_bytes().to_vec(),
        verifying_key.to_bytes().to_vec(),
    ))
}

/// Sign data with Ed25519
pub fn sign_ed25519(data: &[u8], private_key: &[u8; 32]) -> Result<Vec<u8>> {
    use ed25519_dalek::{Signer, SigningKey};
    
    let signing_key = SigningKey::from_bytes(private_key);
    let signature = signing_key.sign(data);
    
    Ok(signature.to_bytes().to_vec())
}

/// Verify Ed25519 signature
pub fn verify_ed25519(data: &[u8], signature: &[u8; 64], public_key: &[u8; 32]) -> Result<bool> {
    use ed25519_dalek::{Verifier, VerifyingKey, Signature};
    
    let verifying_key = VerifyingKey::from_bytes(public_key)
        .context("Invalid public key")?;
    let sig = Signature::from_bytes(signature);
    
    match verifying_key.verify(data, &sig) {
        Ok(_) => Ok(true),
        Err(_) => Ok(false),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_encrypt_decrypt() {
        let key = generate_key().unwrap();
        let plaintext = b"Hello, World!";
        
        let ciphertext = encrypt_aes_gcm(plaintext, &key).unwrap();
        let decrypted = decrypt_aes_gcm(&ciphertext, &key).unwrap();
        
        assert_eq!(plaintext.to_vec(), decrypted);
    }

    #[test]
    fn test_hkdf() {
        let ikm = b"input key material";
        let salt = b"salt";
        let info = b"info";
        
        let key = derive_key(ikm, salt, info).unwrap();
        assert_eq!(key.len(), 32);
    }

    #[test]
    fn test_ed25519_sign_verify() {
        let (private_key, public_key) = generate_identity_keypair().unwrap();
        let data = b"test data";
        
        let signature = sign_ed25519(data, private_key.as_slice().try_into().unwrap()).unwrap();
        let valid = verify_ed25519(
            data,
            signature.as_slice().try_into().unwrap(),
            public_key.as_slice().try_into().unwrap(),
        ).unwrap();
        
        assert!(valid);
    }
}
