//! Comprehensive Cryptographic Test Suite for Project Truffle
//!
//! This test suite validates all cryptographic operations including:
//! - Symmetric encryption (AES-256-GCM, ChaCha20-Poly1305)
//! - Key generation and derivation (HKDF-SHA256)
//! - X3DH key exchange (Signal Protocol with post-quantum hybrid)
//! - Device pairing ceremony (QR codes, SAS verification)
//! - CRDT encryption for Yjs sync
//! - Export functionality
//!
//! ## Security Test Categories
//!
//! 1. **Correctness Tests**: Verify encryption/decryption round-trips
//! 2. **Authentication Tests**: Verify tampering is detected
//! 3. **Edge Case Tests**: Empty inputs, large inputs, boundary conditions
//! 4. **Constant-Time Tests**: Verify constant-time operations

use truffle_crypto::*;
use truffle_crypto::symmetric::*;
use truffle_crypto::keys::*;
use truffle_crypto::x3dh::*;
use truffle_crypto::pairing::*;
use truffle_crypto::crdt_crypto::*;
use truffle_crypto::export::*;

// ============================================================================
// Symmetric Encryption Tests
// ============================================================================

#[test]
fn test_aes_gcm_basic_roundtrip() {
    let key = [0x42u8; 32];
    let plaintext = b"Hello, World!";
    let aad = b"additional data";

    let encrypted = encrypt_aes_gcm(&key, plaintext, aad).unwrap();
    let decrypted = decrypt_aes_gcm(&key, &encrypted, aad).unwrap();

    assert_eq!(decrypted, plaintext.as_slice());
}

#[test]
fn test_aes_gcm_empty_plaintext() {
    let key = [0x42u8; 32];
    let plaintext = b"";
    let aad = b"";

    let encrypted = encrypt_aes_gcm(&key, plaintext, aad).unwrap();
    let decrypted = decrypt_aes_gcm(&key, &encrypted, aad).unwrap();

    assert!(decrypted.is_empty());
}

#[test]
fn test_aes_gcm_large_plaintext() {
    let key = [0x42u8; 32];
    let plaintext = vec![0xABu8; 1024 * 1024]; // 1MB
    let aad = b"large data test";

    let encrypted = encrypt_aes_gcm(&key, &plaintext, aad).unwrap();
    let decrypted = decrypt_aes_gcm(&key, &encrypted, aad).unwrap();

    assert_eq!(decrypted, plaintext);
}

#[test]
fn test_aes_gcm_wrong_key_fails() {
    let key1 = [0x42u8; 32];
    let key2 = [0x43u8; 32];
    let plaintext = b"secret message";
    let aad = b"";

    let encrypted = encrypt_aes_gcm(&key1, plaintext, aad).unwrap();
    let result = decrypt_aes_gcm(&key2, &encrypted, aad);

    assert!(result.is_err());
}

#[test]
fn test_aes_gcm_tampered_ciphertext_fails() {
    let key = [0x42u8; 32];
    let plaintext = b"secret message";
    let aad = b"";

    let mut encrypted = encrypt_aes_gcm(&key, plaintext, aad).unwrap();
    encrypted.ciphertext[0] ^= 0xFF; // Flip bits

    let result = decrypt_aes_gcm(&key, &encrypted, aad);
    assert!(result.is_err());
}

#[test]
fn test_aes_gcm_wrong_aad_fails() {
    let key = [0x42u8; 32];
    let plaintext = b"secret message";
    let aad1 = b"correct aad";
    let aad2 = b"wrong aad";

    let encrypted = encrypt_aes_gcm(&key, plaintext, aad1).unwrap();
    let result = decrypt_aes_gcm(&key, &encrypted, aad2);

    assert!(result.is_err());
}

#[test]
fn test_chacha20_basic_roundtrip() {
    let key = [0x42u8; 32];
    let plaintext = b"Hello, World!";
    let aad = b"additional data";

    let encrypted = encrypt_chacha20(&key, plaintext, aad).unwrap();
    let decrypted = decrypt_chacha20(&key, &encrypted, aad).unwrap();

    assert_eq!(decrypted, plaintext.as_slice());
}

#[test]
fn test_chacha20_tampered_tag_fails() {
    let key = [0x42u8; 32];
    let plaintext = b"secret message";
    let aad = b"";

    let mut encrypted = encrypt_chacha20(&key, plaintext, aad).unwrap();
    encrypted.tag[0] ^= 0xFF; // Corrupt tag

    let result = decrypt_chacha20(&key, &encrypted, aad);
    assert!(result.is_err());
}

#[test]
fn test_auto_encryption_roundtrip() {
    let key = [0x42u8; 32];
    let plaintext = b"auto detection test";
    let aad = b"aad";

    // Test AES path
    let encrypted_aes = encrypt_auto(&key, plaintext, aad, false).unwrap();
    assert_eq!(encrypted_aes.algorithm, 1);
    let decrypted_aes = decrypt_auto(&key, &encrypted_aes, aad).unwrap();
    assert_eq!(decrypted_aes, plaintext.as_slice());

    // Test ChaCha path
    let encrypted_chacha = encrypt_auto(&key, plaintext, aad, true).unwrap();
    assert_eq!(encrypted_chacha.algorithm, 2);
    let decrypted_chacha = decrypt_auto(&key, &encrypted_chacha, aad).unwrap();
    assert_eq!(decrypted_chacha, plaintext.as_slice());
}

#[test]
fn test_encrypted_data_serialization() {
    let key = [0x42u8; 32];
    let plaintext = b"serialization test";
    let aad = b"";

    let encrypted = encrypt_aes_gcm(&key, plaintext, aad).unwrap();
    let bytes = encrypted.to_bytes();
    let deserialized = EncryptedData::from_bytes(&bytes).unwrap();

    assert_eq!(encrypted.algorithm, deserialized.algorithm);
    assert_eq!(encrypted.nonce, deserialized.nonce);
    assert_eq!(encrypted.ciphertext, deserialized.ciphertext);
    assert_eq!(encrypted.tag, deserialized.tag);

    let decrypted = decrypt_aes_gcm(&key, &deserialized, aad).unwrap();
    assert_eq!(decrypted, plaintext.as_slice());
}

// ============================================================================
// Key Management Tests
// ============================================================================

#[test]
fn test_ed25519_identity_generation() {
    let identity = Ed25519Identity::generate().unwrap();

    // Public key should be 32 bytes
    assert_eq!(identity.public_key().len(), 32);

    // Fingerprint should be 16 bytes (32 hex chars)
    assert_eq!(identity.fingerprint_hex().len(), 32);
}

#[test]
fn test_ed25519_sign_and_verify() {
    let identity = Ed25519Identity::generate().unwrap();
    let message = b"test message for signing";

    let signature = identity.sign(message);

    // Signature should be 64 bytes
    assert_eq!(signature.len(), 64);

    // Should verify correctly
    assert!(identity.verify(message, &signature));

    // Wrong message should fail
    assert!(!identity.verify(b"wrong message", &signature));

    // Corrupted signature should fail
    let mut corrupted = signature;
    corrupted[0] ^= 0xFF;
    assert!(!identity.verify(message, &corrupted));
}

#[test]
fn test_x25519_key_exchange() {
    let alice = X25519Keypair::generate().unwrap();
    let bob = X25519Keypair::generate().unwrap();

    // Both parties compute shared secret
    let alice_shared = alice.diffie_hellman(&bob.public_key).unwrap();
    let bob_shared = bob.diffie_hellman(&alice.public_key).unwrap();

    // Should be identical
    assert_eq!(alice_shared, bob_shared);
    assert_eq!(alice_shared.len(), 32);
}

#[test]
fn test_x25519_different_keys_produce_different_secrets() {
    let alice = X25519Keypair::generate().unwrap();
    let bob1 = X25519Keypair::generate().unwrap();
    let bob2 = X25519Keypair::generate().unwrap();

    let shared1 = alice.diffie_hellman(&bob1.public_key).unwrap();
    let shared2 = alice.diffie_hellman(&bob2.public_key).unwrap();

    assert_ne!(shared1, shared2);
}

#[test]
fn test_hkdf_key_derivation() {
    let ikm = b"input key material";
    let salt = b"salt value";
    let info = b"application-specific info";

    // Derive key
    let key1 = hkdf_derive(ikm, Some(salt), info, 32).unwrap();
    let key2 = hkdf_derive(ikm, Some(salt), info, 32).unwrap();

    // Same input should produce same output
    assert_eq!(key1, key2);
    assert_eq!(key1.len(), 32);

    // Different info should produce different output
    let key3 = hkdf_derive(ikm, Some(salt), b"different info", 32).unwrap();
    assert_ne!(key1, key3);

    // Different salt should produce different output
    let key4 = hkdf_derive(ikm, Some(b"different salt"), info, 32).unwrap();
    assert_ne!(key1, key4);
}

#[test]
fn test_derive_sync_key() {
    let shared_secret = b"shared secret from x3dh handshake";

    let key1 = derive_sync_key(shared_secret).unwrap();
    let key2 = derive_sync_key(shared_secret).unwrap();

    // Same input should produce same key
    assert_eq!(key1, key2);
    assert_eq!(key1.len(), 32);
}

#[test]
fn test_key_hierarchy_derivation() {
    let master_seed = [0x42u8; 32];

    let (ed1, x1, s1, _k1) = derive_key_hierarchy(&master_seed).unwrap();
    let (ed2, x2, s2, _k2) = derive_key_hierarchy(&master_seed).unwrap();

    // Same seed should produce same keys (except Kyber)
    assert_eq!(ed1.public_key, ed2.public_key);
    assert_eq!(x1.public_key, x2.public_key);
    assert_eq!(s1.as_bytes(), s2.as_bytes());
}

#[test]
fn test_key_manager() {
    let manager = KeyManager::generate().unwrap();

    // Should have all key types
    assert!(!manager.ed25519_identity().public_key().is_empty());
    assert!(!manager.x25519_identity().public_key().is_empty());
    assert!(!manager.sync_key().as_bytes().is_empty());
    assert!(!manager.kyber_keypair().public_key().is_empty());

    // Should be able to sign and verify
    let message = b"test message";
    let signature = manager.sign(message);
    assert!(manager.verify(message, &signature));

    // Device fingerprint should be valid
    let fingerprint = manager.device_fingerprint();
    assert_eq!(fingerprint.len(), 32);
}

#[test]
fn test_symmetric_key_generation() {
    let key1 = SymmetricKey::generate().unwrap();
    let key2 = SymmetricKey::generate().unwrap();

    // Two random keys should be different
    assert_ne!(key1.as_bytes(), key2.as_bytes());

    // Key should be 32 bytes
    assert_eq!(key1.as_bytes().len(), 32);
}

#[test]
fn test_kyber_encapsulation() {
    let keypair = Kyber768Keypair::generate().unwrap();

    // Encapsulate shared secret
    let (ciphertext, shared1) = keypair.encapsulate().unwrap();

    // Decapsulate shared secret
    let shared2 = keypair.decapsulate(&ciphertext).unwrap();

    // Should be identical
    assert_eq!(shared1, shared2);
    assert_eq!(shared1.len(), 32);
}

#[test]
fn test_kyber_different_encapsulations_produce_different_secrets() {
    let keypair = Kyber768Keypair::generate().unwrap();

    let (_, shared1) = keypair.encapsulate().unwrap();
    let (_, shared2) = keypair.encapsulate().unwrap();

    // Different encapsulations should produce different secrets
    assert_ne!(shared1, shared2);
}

// ============================================================================
// X3DH Tests
// ============================================================================

#[test]
fn test_x3dh_bundle_generation() {
    let bundle = X3DHKeyBundle::generate().unwrap();

    // Should have all public keys
    assert!(!bundle.public.identity_key.is_empty());
    assert!(!bundle.public.signed_prekey.is_empty());
    assert!(!bundle.public.kyber_public_key.is_empty());

    // Should have one-time prekey
    assert!(bundle.public.one_time_prekey.is_some());

    // Signature should be valid
    assert!(bundle.public.verify_signature());
}

#[test]
fn test_x3dh_full_handshake() {
    // Generate Alice's identity
    let alice_identity = X25519Keypair::generate().unwrap();

    // Generate Bob's bundle
    let bob_bundle = X3DHKeyBundle::generate().unwrap();

    // Alice initiates
    let (alice_result, alice_ephemeral) = x3dh_initiate(&alice_identity, &bob_bundle.public).unwrap();

    // Bob responds
    let bob_result = x3dh_respond(
        &bob_bundle.private,
        bob_bundle.public.identity_key,
        alice_identity.public_key,
        alice_ephemeral,
        alice_result.kyber_ciphertext.as_deref(),
    ).unwrap();

    // Both should have the same shared secret
    assert_eq!(alice_result.shared_secret, bob_result.shared_secret);
}

#[test]
fn test_x3dh_without_one_time_prekey() {
    let alice_identity = X25519Keypair::generate().unwrap();
    let bob_bundle = X3DHKeyBundle::generate_without_opk().unwrap();

    assert!(bob_bundle.public.one_time_prekey.is_none());

    let (alice_result, alice_ephemeral) = x3dh_initiate(&alice_identity, &bob_bundle.public).unwrap();

    let bob_result = x3dh_respond(
        &bob_bundle.private,
        bob_bundle.public.identity_key,
        alice_identity.public_key,
        alice_ephemeral,
        alice_result.kyber_ciphertext.as_deref(),
    ).unwrap();

    assert_eq!(alice_result.shared_secret, bob_result.shared_secret);
}

#[test]
fn test_x3dh_invalid_signature_fails() {
    let alice_identity = X25519Keypair::generate().unwrap();
    let mut bob_bundle = X3DHKeyBundle::generate().unwrap();

    // Corrupt the signature
    bob_bundle.public.signed_prekey_signature[0] ^= 0xFF;

    // Should fail signature verification
    let result = x3dh_initiate(&alice_identity, &bob_bundle.public);
    assert!(result.is_err());
}

#[test]
fn test_x3dh_bundle_serialization() {
    let bundle = X3DHKeyBundle::generate().unwrap();
    let bytes = bundle.public.to_bytes();
    let deserialized = X3DHPublicBundle::from_bytes(&bytes).unwrap();

    assert_eq!(bundle.public.identity_key, deserialized.identity_key);
    assert_eq!(bundle.public.signed_prekey, deserialized.signed_prekey);
    assert_eq!(bundle.public.kyber_public_key, deserialized.kyber_public_key);
    assert_eq!(bundle.public.one_time_prekey, deserialized.one_time_prekey);
}

#[test]
fn test_x3dh_different_handshakes_produce_different_secrets() {
    let alice_identity = X25519Keypair::generate().unwrap();
    let bob_bundle1 = X3DHKeyBundle::generate().unwrap();
    let bob_bundle2 = X3DHKeyBundle::generate().unwrap();

    let (result1, _) = x3dh_initiate(&alice_identity, &bob_bundle1.public).unwrap();
    let (result2, _) = x3dh_initiate(&alice_identity, &bob_bundle2.public).unwrap();

    assert_ne!(result1.shared_secret, result2.shared_secret);
}

// ============================================================================
// Pairing Tests
// ============================================================================

#[test]
fn test_generate_qr_code() {
    let device_id = "test-device-123";
    let public_key = vec![0x42u8; 100];
    let relay = "wss://relay.truffle.io/pair";
    let token = "test-token";

    let qr_svg = generate_qr_code(device_id, &public_key, relay, token);

    // Should be valid SVG
    assert!(qr_svg.contains("<svg"));
    assert!(qr_svg.contains("</svg>"));
}

#[test]
fn test_sas_generation() {
    let shared_secret = [0x42u8; 32];
    let alice_identity = [0x01u8; 32];
    let bob_identity = [0x02u8; 32];

    let sas1 = SasCode::generate(&shared_secret, &alice_identity, &bob_identity);
    let sas2 = SasCode::generate(&shared_secret, &alice_identity, &bob_identity);

    // Same inputs should produce same SAS
    assert_eq!(sas1, sas2);

    // Should be 6 digits
    assert_eq!(sas1.as_str().len(), 6);

    // Should be numeric
    assert!(sas1.as_str().parse::<u32>().is_ok());
}

#[test]
fn test_sas_different_inputs_produce_different_codes() {
    let shared_secret = [0x42u8; 32];
    let alice1 = [0x01u8; 32];
    let alice2 = [0x02u8; 32];
    let bob = [0x03u8; 32];

    let sas1 = SasCode::generate(&shared_secret, &alice1, &bob);
    let sas2 = SasCode::generate(&shared_secret, &alice2, &bob);

    assert_ne!(sas1, sas2);
}

#[test]
fn test_sas_verification() {
    let sas1 = SasCode("123456".to_string());
    let sas2 = SasCode("123456".to_string());
    let sas3 = SasCode("654321".to_string());

    assert!(sas1.verify(&sas2));
    assert!(!sas1.verify(&sas3));

    assert!(verify_sas("123456", "123456"));
    assert!(!verify_sas("123456", "654321"));
}

#[test]
fn test_sas_formatted() {
    let sas = SasCode("123456".to_string());
    assert_eq!(sas.formatted(), "123-456");
}

#[test]
fn test_pairing_qr_data() {
    let qr_data = PairingQrData::new(
        "device-123".to_string(),
        vec![0x42u8; 100],
        "wss://relay.truffle.io".to_string(),
        "token-abc".to_string(),
    );

    let json = qr_data.to_json();
    let parsed = PairingQrData::from_json(&json).unwrap();

    assert_eq!(qr_data.device_id, parsed.device_id);
    assert_eq!(qr_data.relay_endpoint, parsed.relay_endpoint);
    assert_eq!(qr_data.pairing_token, parsed.pairing_token);
}

#[test]
fn test_qr_expiration() {
    use std::time::{SystemTime, UNIX_EPOCH};

    let mut qr_data = PairingQrData::new(
        "device-123".to_string(),
        vec![0x42u8; 100],
        "wss://relay.truffle.io".to_string(),
        "token-abc".to_string(),
    );

    // Set timestamp to 15 minutes ago
    qr_data.timestamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs() - 900;

    assert!(qr_data.is_expired(600)); // 10 minute expiry
    assert!(!qr_data.is_expired(1200)); // 20 minute expiry
}

#[test]
fn test_pairing_token_generation() {
    let token1 = generate_pairing_token().unwrap();
    let token2 = generate_pairing_token().unwrap();

    // Should be different
    assert_ne!(token1, token2);

    // Should be 64 hex characters (32 bytes)
    assert_eq!(token1.len(), 64);
}

#[test]
fn test_device_fingerprint_formatting() {
    let public_key = [0x42u8; 32];
    let fingerprint = format_device_fingerprint(&public_key);

    // Should be formatted as XX-XX-XX-XX
    assert_eq!(fingerprint.len(), 11);
    assert_eq!(fingerprint.matches('-').count(), 3);
}

#[test]
fn test_pairing_ceremony_state_machine() {
    let mut ceremony = PairingCeremony::new();

    assert_eq!(*ceremony.state(), PairingState::Initial);

    // Start as primary
    let qr = ceremony.start_primary("wss://relay.truffle.io").unwrap();
    assert!(!qr.is_empty());
    assert_eq!(*ceremony.state(), PairingState::QrGenerated);

    // Get primary bundle
    let bundle = ceremony.get_primary_bundle().unwrap();
    assert!(!bundle.identity_key.is_empty());
}

// ============================================================================
// CRDT Encryption Tests
// ============================================================================

#[test]
fn test_crdt_basic_roundtrip() {
    let sync_key = [0x42u8; 32];
    let device_id = [0x01u8; 8];
    let crypto = CrdtCrypto::with_defaults(sync_key, device_id);

    let update = b"test yjs crdt update data";

    let encrypted = crypto.encrypt_update(update, None).unwrap();
    let decrypted = crypto.decrypt_update(&encrypted).unwrap();

    assert_eq!(decrypted.crdt_update, update.as_slice());
}

#[test]
fn test_crdt_with_tombstones() {
    let sync_key = [0x42u8; 32];
    let device_id = [0x01u8; 8];
    let crypto = CrdtCrypto::with_defaults(sync_key, device_id);

    let update = b"test update with deletions";
    let tombstones = vec![
        "550e8400-e29b-41d4-a716-446655440000".to_string(),
        "550e8400-e29b-41d4-a716-446655440001".to_string(),
    ];

    let encrypted = crypto.encrypt_update(update, Some(tombstones.clone())).unwrap();
    assert!(encrypted.has_tombstones());

    let decrypted = crypto.decrypt_update(&encrypted).unwrap();
    assert_eq!(decrypted.crdt_update, update.as_slice());
    assert_eq!(decrypted.tombstones, tombstones);
}

#[test]
fn test_crdt_hmac_verification() {
    let sync_key = [0x42u8; 32];
    let device_id = [0x01u8; 8];
    let crypto = CrdtCrypto::with_defaults(sync_key, device_id);

    let update = b"test update";
    let encrypted = crypto.encrypt_update(update, None).unwrap();

    // HMAC should be valid
    assert!(encrypted.verify_hmac(&sync_key));

    // Wrong key should fail
    let wrong_key = [0x43u8; 32];
    assert!(!encrypted.verify_hmac(&wrong_key));
}

#[test]
fn test_crdt_tampered_ciphertext_fails() {
    let sync_key = [0x42u8; 32];
    let device_id = [0x01u8; 8];
    let crypto = CrdtCrypto::with_defaults(sync_key, device_id);

    let update = b"test update";
    let mut encrypted = crypto.encrypt_update(update, None).unwrap();

    // Tamper with ciphertext
    encrypted.ciphertext[0] ^= 0xFF;

    // Decryption should fail
    let result = crypto.decrypt_update(&encrypted);
    assert!(result.is_err());
}

#[test]
fn test_crdt_wrong_key_fails() {
    let sync_key1 = [0x42u8; 32];
    let sync_key2 = [0x43u8; 32];
    let device_id = [0x01u8; 8];
    let crypto1 = CrdtCrypto::with_defaults(sync_key1, device_id);
    let crypto2 = CrdtCrypto::with_defaults(sync_key2, device_id);

    let update = b"test update";
    let encrypted = crypto1.encrypt_update(update, None).unwrap();

    // Decryption with wrong key should fail
    let result = crypto2.decrypt_update(&encrypted);
    assert!(result.is_err());
}

#[test]
fn test_crdt_convenience_functions() {
    let key = [0x42u8; 32];
    let update = b"convenience test";

    let encrypted = encrypt_crdt_update(&key, update).unwrap();
    let decrypted = decrypt_crdt_update(&key, &encrypted).unwrap();

    assert_eq!(decrypted, update.as_slice());
}

#[test]
fn test_encrypted_crdt_serialization() {
    let sync_key = [0x42u8; 32];
    let device_id = [0x01u8; 8];
    let crypto = CrdtCrypto::with_defaults(sync_key, device_id);

    let update = b"serialization test";
    let encrypted = crypto.encrypt_update(update, None).unwrap();

    let bytes = encrypted.to_bytes();
    let deserialized = EncryptedCrdtUpdate::from_bytes(&bytes).unwrap();

    assert_eq!(deserialized.protocol_version, encrypted.protocol_version);
    assert_eq!(deserialized.algorithm, encrypted.algorithm);
    assert_eq!(deserialized.device_id, encrypted.device_id);
    assert_eq!(deserialized.timestamp, encrypted.timestamp);
    assert_eq!(deserialized.ciphertext, encrypted.ciphertext);

    // Should still decrypt correctly
    let decrypted = crypto.decrypt_update(&deserialized).unwrap();
    assert_eq!(decrypted.crdt_update, update.as_slice());
}

#[test]
fn test_chacha20_crdt_encryption() {
    let sync_key = [0x42u8; 32];
    let device_id = [0x01u8; 8];
    let config = CrdtConfig {
        prefer_chacha: true,
        compress: false,
        schema_version: "1.0".to_string(),
    };
    let crypto = CrdtCrypto::new(sync_key, device_id, config);

    let update = b"chacha20 test";
    let encrypted = crypto.encrypt_update(update, None).unwrap();

    assert_eq!(encrypted.algorithm, ALGORITHM_CHACHA20);

    let decrypted = crypto.decrypt_update(&encrypted).unwrap();
    assert_eq!(decrypted.crdt_update, update.as_slice());
}

// ============================================================================
// Export Tests
// ============================================================================

#[test]
fn test_export_to_markdown() {
    let node = WikiNode {
        id: "test-123".to_string(),
        node_type: NodeType::Entity,
        title: "Test Entity".to_string(),
        content: "# Test\n\nThis is a test.".to_string(),
        backlinks: vec![],
        forward_links: vec![WikiLink {
            target_id: "concept-1".to_string(),
            target_title: "expenses".to_string(),
            context: None,
        }],
        provenance: Provenance {
            source_artifacts: vec!["artifact-1".to_string()],
            compiled_at: "2024-01-15T10:00:00Z".to_string(),
            model_version: "gemma-4-v1".to_string(),
            confidence_score: 0.95,
        },
        temporal_vectors: TemporalVectors {
            mentioned_dates: vec!["2024-01-15".to_string()],
            fiscal_quarter: Some("Q1-2024".to_string()),
        },
        privacy_classification: PrivacyClassification::Personal,
        tags: vec!["test".to_string()],
    };

    let markdown = export_to_markdown(&node);

    // Should have YAML frontmatter
    assert!(markdown.starts_with("---"));
    assert!(markdown.contains("id: test-123"));
    assert!(markdown.contains("node_type: entity"));
    assert!(markdown.contains("title: \"Test Entity\""));

    // Should have content
    assert!(markdown.contains("# Test"));

    // Should have related links
    assert!(markdown.contains("## Related"));
    assert!(markdown.contains("[[expenses]]"));
}

#[test]
fn test_sanitize_filename() {
    assert_eq!(truffle_crypto::export::sanitize_filename("Hello World"), "hello-world");
    assert_eq!(truffle_crypto::export::sanitize_filename("Test & Company"), "test-company");
    assert_eq!(truffle_crypto::export::sanitize_filename("File--Name"), "file-name");
}

#[test]
fn test_compute_file_hash() {
    let content1 = b"test content";
    let content2 = b"test content";
    let content3 = b"different content";

    let hash1 = compute_file_hash(content1);
    let hash2 = compute_file_hash(content2);
    let hash3 = compute_file_hash(content3);

    assert_eq!(hash1, hash2);
    assert_ne!(hash1, hash3);
    assert_eq!(hash1.len(), 64); // SHA-256 hex = 64 chars
}

#[test]
fn test_create_integrity_file() {
    let mut files = std::collections::HashMap::new();
    files.insert("test/file".to_string(), "test content".to_string());

    let integrity = create_integrity_file(&files);

    assert!(integrity.contains("./test/file.md"));
    assert!(integrity.len() > 64); // Should have hash + path
}

#[test]
fn test_generate_index() {
    let nodes = vec![WikiNode {
        id: "test-1".to_string(),
        node_type: NodeType::Entity,
        title: "Test Entity".to_string(),
        content: "Test".to_string(),
        backlinks: vec![],
        forward_links: vec![],
        provenance: Provenance {
            source_artifacts: vec![],
            compiled_at: "2024-01-15T10:00:00Z".to_string(),
            model_version: "gemma-4-v1".to_string(),
            confidence_score: 0.95,
        },
        temporal_vectors: Default::default(),
        privacy_classification: PrivacyClassification::Personal,
        tags: vec![],
    }];

    let index = generate_index(&nodes);

    assert!(index.contains("# Truffle Knowledge Base"));
    assert!(index.contains("total_nodes: 1"));
    assert!(index.contains("## Entities"));
}

#[test]
fn test_estimate_export_time() {
    // 100 nodes should take ~1 second
    assert_eq!(estimate_export_time(100), 1);

    // 500 nodes should take ~5 seconds
    assert_eq!(estimate_export_time(500), 5);

    // Empty should still return 1 second minimum
    assert_eq!(estimate_export_time(0), 1);
}

#[test]
fn test_can_export_within_time_limit() {
    // 100 nodes in 5 minutes (300 seconds) - should succeed
    assert!(can_export_within_time_limit(100, 300));

    // 1 million nodes in 5 minutes - should fail
    assert!(!can_export_within_time_limit(1_000_000, 300));
}

#[test]
fn test_perform_export() {
    let nodes = vec![WikiNode {
        id: "test-1".to_string(),
        node_type: NodeType::Entity,
        title: "Test Entity".to_string(),
        content: "Test content".to_string(),
        backlinks: vec![],
        forward_links: vec![],
        provenance: Provenance {
            source_artifacts: vec![],
            compiled_at: "2024-01-15T10:00:00Z".to_string(),
            model_version: "gemma-4-v1".to_string(),
            confidence_score: 0.95,
        },
        temporal_vectors: Default::default(),
        privacy_classification: PrivacyClassification::Personal,
        tags: vec![],
    }];

    let config = ExportConfig::default();
    let files = perform_export(&nodes, &config).unwrap();

    // Should have entity file
    assert!(files.contains_key("entities/test-entity"));

    // Should have index
    assert!(files.contains_key("index.md"));

    // Should have metadata
    assert!(files.contains_key(".truffle/manifest.json"));
    assert!(files.contains_key(".truffle/integrity.sha256"));
}

// ============================================================================
// Utility Function Tests
// ============================================================================

#[test]
fn test_constant_time_eq() {
    let a = [1u8, 2, 3, 4];
    let b = [1u8, 2, 3, 4];
    let c = [1u8, 2, 3, 5];

    assert!(constant_time_eq(&a, &b));
    assert!(!constant_time_eq(&a, &c));
}

#[test]
fn test_secure_random_bytes() {
    let mut buf1 = [0u8; 32];
    let mut buf2 = [0u8; 32];

    secure_random_bytes(&mut buf1).unwrap();
    secure_random_bytes(&mut buf2).unwrap();

    // Probability of collision is negligible
    assert_ne!(buf1, buf2);
}

#[test]
fn test_sha3_256_hash() {
    let data = b"hello world";
    let hash1 = sha3_256_hash(data);
    let hash2 = sha3_256_hash(data);

    assert_eq!(hash1, hash2);
    assert_eq!(hash1.len(), 32);
}

#[test]
fn test_sha2_256_hash() {
    let data = b"hello world";
    let hash1 = sha2_256_hash(data);
    let hash2 = sha2_256_hash(data);

    assert_eq!(hash1, hash2);
    assert_eq!(hash1.len(), 32);
}

#[test]
fn test_hmac_sha256() {
    let key = b"secret key";
    let data = b"message to authenticate";

    let tag1 = hmac_sha256(key, data);
    let tag2 = hmac_sha256(key, data);

    assert_eq!(tag1, tag2);
    assert!(verify_hmac_sha256(key, data, &tag1));

    // Different data should produce different tags
    let tag3 = hmac_sha256(key, b"different message");
    assert_ne!(tag1, tag3);
}

#[test]
fn test_generate_random_key() {
    let key1 = generate_random_key().unwrap();
    let key2 = generate_random_key().unwrap();

    assert_eq!(key1.len(), 32);
    assert_eq!(key2.len(), 32);
    assert_ne!(key1, key2);
}

#[test]
fn test_generate_random_nonce() {
    let nonce1 = generate_random_nonce::<12>().unwrap();
    let nonce2 = generate_random_nonce::<12>().unwrap();

    assert_eq!(nonce1.len(), 12);
    assert_eq!(nonce2.len(), 12);
    assert_ne!(nonce1, nonce2);
}

// ============================================================================
// Integration Tests
// ============================================================================

#[test]
fn test_full_encryption_pipeline() {
    // 1. Generate keys
    let key_manager = KeyManager::generate().unwrap();

    // 2. Generate X3DH bundle
    let x3dh_bundle = X3DHKeyBundle::generate().unwrap();

    // 3. Derive sync key
    let sync_key = derive_sync_key(&[0x42u8; 32]).unwrap();

    // 4. Encrypt CRDT update
    let device_id = [0x01u8; 8];
    let crypto = CrdtCrypto::with_defaults(sync_key, device_id);
    let update = b"yjs crdt update";
    let encrypted = crypto.encrypt_update(update, None).unwrap();

    // 5. Decrypt and verify
    let decrypted = crypto.decrypt_update(&encrypted).unwrap();
    assert_eq!(decrypted.crdt_update, update.as_slice());
}

#[test]
fn test_end_to_end_pairing_simulation() {
    // Simulate full pairing between two devices

    // Device A (Primary) generates keys
    let device_a_keys = KeyManager::generate().unwrap();
    let device_a_x3dh = X3DHKeyBundle::generate().unwrap();

    // Device B (Secondary) generates keys
    let device_b_keys = KeyManager::generate().unwrap();
    let device_b_x3dh = X3DHKeyBundle::generate().unwrap();

    // Device A initiates X3DH with Device B
    let (a_result, a_ephemeral) = x3dh_initiate(
        device_a_keys.x25519_identity(),
        &device_b_x3dh.public
    ).unwrap();

    // Device B responds
    let b_result = x3dh_respond(
        &device_b_x3dh.private,
        device_b_x3dh.public.identity_key,
        device_a_keys.x25519_identity().public_key,
        a_ephemeral,
        a_result.kyber_ciphertext.as_deref(),
    ).unwrap();

    // Both derive same shared secret
    assert_eq!(a_result.shared_secret, b_result.shared_secret);

    // Both derive sync key
    let a_sync_key = derive_sync_key(&a_result.shared_secret).unwrap();
    let b_sync_key = derive_sync_key(&b_result.shared_secret).unwrap();
    assert_eq!(a_sync_key, b_sync_key);

    // Device A encrypts a message
    let a_crypto = CrdtCrypto::with_defaults(a_sync_key, [0x01u8; 8]);
    let message = b"Hello from Device A!";
    let encrypted = a_crypto.encrypt_update(message, None).unwrap();

    // Device B decrypts the message
    let b_crypto = CrdtCrypto::with_defaults(b_sync_key, [0x02u8; 8]);
    let decrypted = b_crypto.decrypt_update(&encrypted).unwrap();

    assert_eq!(decrypted.crdt_update, message.as_slice());
}
