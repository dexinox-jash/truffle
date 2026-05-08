//! Cryptographic Module Unit Tests
//! 
//! Validates X3DH key exchange, AES-256-GCM encryption, and post-quantum hybrid.
//! Critical for STRIDE threat model compliance.

#[cfg(test)]
mod x3dh_tests {
    use crate::crypto::x3dh::{IdentityKey, DeviceKeys, X3DHKeyBundle, x3dh_handshake};
    use crate::crypto::errors::CryptoError;

    #[test]
    fn test_identity_key_generation() {
        let identity = IdentityKey::generate();
        
        // Verify Ed25519 format (32-byte public key)
        assert_eq!(identity.public_key().as_bytes().len(), 32);
        assert!(identity.public_key().is_valid());
        
        // Verify private key format (64 bytes for Ed25519 expanded)
        assert_eq!(identity.private_key().as_bytes().len(), 64);
    }

    #[test]
    fn test_unique_key_generation() {
        let key1 = IdentityKey::generate();
        let key2 = IdentityKey::generate();
        
        // Keys should be unique
        assert_ne!(key1.public_key().as_bytes(), key2.public_key().as_bytes());
    }

    #[test]
    fn test_x3dh_handshake() {
        let alice = DeviceKeys::generate();
        let bob = DeviceKeys::generate();
        
        let alice_bundle = X3DHKeyBundle {
            identity_key: alice.identity_key.public_key().clone(),
            ephemeral_key: alice.ephemeral_keys[0].public_key().clone(),
            pre_key: alice.pre_keys[0].public_key.clone(),
            pre_key_signature: alice.pre_keys[0].signature.clone(),
        };
        
        let bob_bundle = X3DHKeyBundle {
            identity_key: bob.identity_key.public_key().clone(),
            ephemeral_key: bob.ephemeral_keys[0].public_key().clone(),
            pre_key: bob.pre_keys[0].public_key.clone(),
            pre_key_signature: bob.pre_keys[0].signature.clone(),
        };
        
        // Both parties derive same shared secret
        let alice_shared = x3dh_handshake(&alice, &bob_bundle).unwrap();
        let bob_shared = x3dh_handshake(&bob, &alice_bundle).unwrap();
        
        assert_eq!(alice_shared.as_bytes(), bob_shared.as_bytes());
        assert_eq!(alice_shared.as_bytes().len(), 32); // 256-bit shared secret
    }

    #[test]
    fn test_invalid_prekey_signature() {
        let alice = DeviceKeys::generate();
        let bob = DeviceKeys::generate();
        
        // Create bundle with invalid signature
        let invalid_bundle = X3DHKeyBundle {
            identity_key: bob.identity_key.public_key().clone(),
            ephemeral_key: bob.ephemeral_keys[0].public_key().clone(),
            pre_key: bob.pre_keys[0].public_key.clone(),
            pre_key_signature: vec![0u8; 64], // Invalid signature
        };
        
        // Handshake should fail
        let result = x3dh_handshake(&alice, &invalid_bundle);
        assert!(matches!(result, Err(CryptoError::InvalidSignature)));
    }

    #[test]
    fn test_different_ephemeral_different_secrets() {
        let alice = DeviceKeys::generate();
        let bob = DeviceKeys::generate();
        
        // First handshake with ephemeral key 0
        let bundle1 = X3DHKeyBundle {
            identity_key: alice.identity_key.public_key().clone(),
            ephemeral_key: alice.ephemeral_keys[0].public_key().clone(),
            pre_key: alice.pre_keys[0].public_key.clone(),
            pre_key_signature: alice.pre_keys[0].signature.clone(),
        };
        
        // Second handshake with ephemeral key 1
        let bundle2 = X3DHKeyBundle {
            identity_key: alice.identity_key.public_key().clone(),
            ephemeral_key: alice.ephemeral_keys[1].public_key().clone(),
            pre_key: alice.pre_keys[0].public_key.clone(),
            pre_key_signature: alice.pre_keys[0].signature.clone(),
        };
        
        let shared1 = x3dh_handshake(&bob, &bundle1).unwrap();
        let shared2 = x3dh_handshake(&bob, &bundle2).unwrap();
        
        // Different ephemeral keys should produce different secrets
        assert_ne!(shared1.as_bytes(), shared2.as_bytes());
    }
}

#[cfg(test)]
mod aes_gcm_tests {
    use crate::crypto::aes_gcm::{encrypt_aes_gcm, decrypt_aes_gcm, AesGcmError};
    use rand::RngCore;

    #[test]
    fn test_encrypt_decrypt() {
        let key = generate_random_key();
        let plaintext = b"Hello, Truffle!";
        
        let encrypted = encrypt_aes_gcm(plaintext, &key).unwrap();
        let decrypted = decrypt_aes_gcm(&encrypted, &key).unwrap();
        
        assert_eq!(decrypted, plaintext);
    }

    #[test]
    fn test_random_nonce() {
        let key = generate_random_key();
        let plaintext = b"test";
        
        let encrypted1 = encrypt_aes_gcm(plaintext, &key).unwrap();
        let encrypted2 = encrypt_aes_gcm(plaintext, &key).unwrap();
        
        // Nonces must be different
        assert_ne!(encrypted1.nonce, encrypted2.nonce);
        assert_eq!(encrypted1.nonce.len(), 12); // 96-bit nonce
    }

    #[test]
    fn test_empty_plaintext() {
        let key = generate_random_key();
        let plaintext = b"";
        
        let encrypted = encrypt_aes_gcm(plaintext, &key).unwrap();
        let decrypted = decrypt_aes_gcm(&encrypted, &key).unwrap();
        
        assert_eq!(decrypted, plaintext);
    }

    #[test]
    fn test_large_plaintext() {
        let key = generate_random_key();
        let plaintext = vec![0u8; 1024 * 1024]; // 1MB
        
        let encrypted = encrypt_aes_gcm(&plaintext, &key).unwrap();
        let decrypted = decrypt_aes_gcm(&encrypted, &key).unwrap();
        
        assert_eq!(decrypted, plaintext);
    }

    #[test]
    fn test_ciphertext_tampering() {
        let key = generate_random_key();
        let plaintext = b"Sensitive data";
        
        let mut encrypted = encrypt_aes_gcm(plaintext, &key).unwrap();
        encrypted.ciphertext[0] ^= 0xFF; // Flip bits
        
        let result = decrypt_aes_gcm(&encrypted, &key);
        assert!(matches!(result, Err(AesGcmError::AuthenticationFailed)));
    }

    #[test]
    fn test_nonce_tampering() {
        let key = generate_random_key();
        let plaintext = b"Sensitive data";
        
        let mut encrypted = encrypt_aes_gcm(plaintext, &key).unwrap();
        encrypted.nonce[0] ^= 0xFF; // Flip bits
        
        let result = decrypt_aes_gcm(&encrypted, &key);
        assert!(matches!(result, Err(AesGcmError::AuthenticationFailed)));
    }

    #[test]
    fn test_tag_tampering() {
        let key = generate_random_key();
        let plaintext = b"Sensitive data";
        
        let mut encrypted = encrypt_aes_gcm(plaintext, &key).unwrap();
        encrypted.tag[0] ^= 0xFF; // Flip bits
        
        let result = decrypt_aes_gcm(&encrypted, &key);
        assert!(matches!(result, Err(AesGcmError::AuthenticationFailed)));
    }

    #[test]
    fn test_wrong_key() {
        let key = generate_random_key();
        let wrong_key = generate_random_key();
        let plaintext = b"Sensitive data";
        
        let encrypted = encrypt_aes_gcm(plaintext, &key).unwrap();
        let result = decrypt_aes_gcm(&encrypted, &wrong_key);
        
        assert!(matches!(result, Err(AesGcmError::AuthenticationFailed)));
    }

    #[test]
    fn test_invalid_key_size() {
        let invalid_key = vec![0u8; 16]; // 128-bit instead of 256-bit
        let plaintext = b"test";
        
        let result = encrypt_aes_gcm(plaintext, &invalid_key);
        assert!(matches!(result, Err(AesGcmError::InvalidKeySize)));
    }

    fn generate_random_key() -> Vec<u8> {
        let mut key = vec![0u8; 32];
        rand::thread_rng().fill_bytes(&mut key);
        key
    }
}

#[cfg(test)]
mod kyber_tests {
    use crate::crypto::kyber::{generate_keypair, encapsulate, decapsulate};

    #[test]
    fn test_keypair_generation() {
        let keypair = generate_keypair().unwrap();
        
        // Kyber-768 key sizes
        assert_eq!(keypair.public_key.len(), 1184);
        assert_eq!(keypair.secret_key.len(), 2400);
    }

    #[test]
    fn test_encapsulate_decapsulate() {
        let keypair = generate_keypair().unwrap();
        
        let (ciphertext, shared_secret1) = encapsulate(&keypair.public_key).unwrap();
        let shared_secret2 = decapsulate(&ciphertext, &keypair.secret_key).unwrap();
        
        // Both parties derive same shared secret
        assert_eq!(shared_secret1, shared_secret2);
        assert_eq!(shared_secret1.len(), 32); // 256-bit shared secret
        
        // Ciphertext size for Kyber-768
        assert_eq!(ciphertext.len(), 1088);
    }

    #[test]
    fn test_different_encapsulations_different_secrets() {
        let keypair = generate_keypair().unwrap();
        
        let (_, secret1) = encapsulate(&keypair.public_key).unwrap();
        let (_, secret2) = encapsulate(&keypair.public_key).unwrap();
        
        // Each encapsulation produces different secret
        assert_ne!(secret1, secret2);
    }
}

#[cfg(test)]
mod hkdf_tests {
    use crate::crypto::hkdf::{derive_keys, HkdfKeys};
    use rand::RngCore;

    #[test]
    fn test_key_derivation() {
        let shared_secret = generate_random_bytes(32);
        let salt = generate_random_bytes(32);
        
        let keys = derive_keys(&shared_secret, &salt).unwrap();
        
        assert_eq!(keys.sync_key.len(), 32);
        assert_eq!(keys.auth_key.len(), 32);
        
        // Keys should be different
        assert_ne!(keys.sync_key, keys.auth_key);
    }

    #[test]
    fn test_different_salts_different_keys() {
        let shared_secret = generate_random_bytes(32);
        let salt1 = generate_random_bytes(32);
        let salt2 = generate_random_bytes(32);
        
        let keys1 = derive_keys(&shared_secret, &salt1).unwrap();
        let keys2 = derive_keys(&shared_secret, &salt2).unwrap();
        
        assert_ne!(keys1.sync_key, keys2.sync_key);
        assert_ne!(keys1.auth_key, keys2.auth_key);
    }

    #[test]
    fn test_deterministic_derivation() {
        let shared_secret = generate_random_bytes(32);
        let salt = generate_random_bytes(32);
        
        let keys1 = derive_keys(&shared_secret, &salt).unwrap();
        let keys2 = derive_keys(&shared_secret, &salt).unwrap();
        
        // Same inputs produce same outputs
        assert_eq!(keys1.sync_key, keys2.sync_key);
        assert_eq!(keys1.auth_key, keys2.auth_key);
    }

    fn generate_random_bytes(len: usize) -> Vec<u8> {
        let mut bytes = vec![0u8; len];
        rand::thread_rng().fill_bytes(&mut bytes);
        bytes
    }
}

#[cfg(test)]
mod zero_knowledge_tests {
    use crate::crypto::aes_gcm::{encrypt_aes_gcm, decrypt_aes_gcm};
    use crate::crypto::verify::verify_zero_knowledge;
    use rand::RngCore;

    #[test]
    fn test_no_plaintext_paths() {
        let key = generate_random_key();
        let plaintext = b"{\"secret\": \"user data\"}";
        
        let encrypted = encrypt_aes_gcm(plaintext, &key).unwrap();
        
        // Ciphertext should not be valid UTF-8 (encrypted)
        let is_utf8 = String::from_utf8(encrypted.ciphertext.clone()).is_ok();
        assert!(!is_utf8 || encrypted.ciphertext != plaintext);
    }

    #[test]
    fn test_infrastructure_cannot_decrypt() {
        let user_key = generate_random_key();
        let infrastructure_key = generate_random_key();
        let plaintext = b"secret data";
        
        let encrypted = encrypt_aes_gcm(plaintext, &user_key).unwrap();
        
        // Infrastructure cannot decrypt
        let result = decrypt_aes_gcm(&encrypted, &infrastructure_key);
        assert!(result.is_err());
        
        // Verification confirms zero-knowledge
        let verification = verify_zero_knowledge(&encrypted, &user_key);
        assert!(!verification.infrastructure_can_decrypt);
    }

    #[test]
    fn test_all_sync_data_encrypted() {
        let key = generate_random_key();
        let sync_data = br#"{"nodes": [{"title": "Test", "content": "Secret"}]}"#;
        
        let encrypted = encrypt_aes_gcm(sync_data, &key).unwrap();
        let combined = [&encrypted.nonce[..], &encrypted.ciphertext[..], &encrypted.tag[..]].concat();
        
        // Combined blob should not contain plaintext
        let combined_str = String::from_utf8_lossy(&combined);
        assert!(!combined_str.contains("Test"));
        assert!(!combined_str.contains("Secret"));
    }

    fn generate_random_key() -> Vec<u8> {
        let mut key = vec![0u8; 32];
        rand::thread_rng().fill_bytes(&mut key);
        key
    }
}

#[cfg(test)]
mod nonce_tests {
    use crate::crypto::nonce::generate_nonce;
    use std::collections::HashSet;

    #[test]
    fn test_nonce_uniqueness() {
        let mut nonces = HashSet::new();
        
        for _ in 0..10000 {
            let nonce = generate_nonce(12);
            let nonce_hex = hex::encode(&nonce);
            
            assert!(!nonces.contains(&nonce_hex), "Duplicate nonce generated");
            nonces.insert(nonce_hex);
        }
    }

    #[test]
    fn test_nonce_size() {
        let nonce_96 = generate_nonce(12); // 96 bits
        assert_eq!(nonce_96.len(), 12);
        
        let nonce_128 = generate_nonce(16); // 128 bits
        assert_eq!(nonce_128.len(), 16);
    }

    #[test]
    fn test_randomness() {
        let nonce1 = generate_nonce(12);
        let nonce2 = generate_nonce(12);
        
        assert_ne!(nonce1, nonce2);
    }
}
