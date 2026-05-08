//! Sync Protocol Unit Tests
//! 
//! Validates CRDT encryption, message authentication, and tombstone propagation.

#[cfg(test)]
mod crdt_encryption_tests {
    use crate::sync::crypto::{encrypt_crdt_update, decrypt_crdt_update};
    use crate::crypto::aes_gcm::AesGcmCiphertext;
    use rand::RngCore;

    #[test]
    fn test_crdt_update_encryption() {
        let update = vec![1u8, 2, 3, 4, 5];
        let key = generate_random_key();
        
        let encrypted = encrypt_crdt_update(&update, &key).unwrap();
        
        // Encrypted should be different from plaintext
        assert_ne!(encrypted.ciphertext, update);
        assert_eq!(encrypted.nonce.len(), 12);
        assert_eq!(encrypted.tag.len(), 16);
    }

    #[test]
    fn test_crdt_update_decryption() {
        let update = vec![1u8, 2, 3, 4, 5];
        let key = generate_random_key();
        
        let encrypted = encrypt_crdt_update(&update, &key).unwrap();
        let decrypted = decrypt_crdt_update(&encrypted, &key).unwrap();
        
        assert_eq!(decrypted, update);
    }

    #[test]
    fn test_wrong_key_fails() {
        let update = vec![1u8, 2, 3, 4, 5];
        let key = generate_random_key();
        let wrong_key = generate_random_key();
        
        let encrypted = encrypt_crdt_update(&update, &key).unwrap();
        let result = decrypt_crdt_update(&encrypted, &wrong_key);
        
        assert!(result.is_err());
    }

    fn generate_random_key() -> Vec<u8> {
        let mut key = vec![0u8; 32];
        rand::thread_rng().fill_bytes(&mut key);
        key
    }
}

#[cfg(test)]
mod message_authentication_tests {
    use crate::sync::protocol::{SyncMessage, sign_message, verify_message};
    use crate::crypto::hkdf::HkdfKeys;
    use rand::RngCore;

    #[test]
    fn test_message_signing() {
        let payload = vec![1u8, 2, 3, 4, 5];
        let keys = generate_keys();
        
        let message = SyncMessage::new(payload.clone());
        let signed = sign_message(&message, &keys.auth_key).unwrap();
        
        assert_eq!(signed.payload, payload);
        assert!(signed.mac.is_some());
    }

    #[test]
    fn test_message_verification() {
        let payload = vec![1u8, 2, 3, 4, 5];
        let keys = generate_keys();
        
        let message = SyncMessage::new(payload);
        let signed = sign_message(&message, &keys.auth_key).unwrap();
        
        let result = verify_message(&signed, &keys.auth_key);
        assert!(result.is_ok());
    }

    #[test]
    fn test_tampered_message_fails() {
        let payload = vec![1u8, 2, 3, 4, 5];
        let keys = generate_keys();
        
        let message = SyncMessage::new(payload);
        let mut signed = sign_message(&message, &keys.auth_key).unwrap();
        
        // Tamper with payload
        signed.payload[0] ^= 0xFF;
        
        let result = verify_message(&signed, &keys.auth_key);
        assert!(result.is_err());
    }

    #[test]
    fn test_wrong_auth_key_fails() {
        let payload = vec![1u8, 2, 3, 4, 5];
        let keys = generate_keys();
        let wrong_keys = generate_keys();
        
        let message = SyncMessage::new(payload);
        let signed = sign_message(&message, &keys.auth_key).unwrap();
        
        let result = verify_message(&signed, &wrong_keys.auth_key);
        assert!(result.is_err());
    }

    fn generate_keys() -> HkdfKeys {
        let mut sync_key = vec![0u8; 32];
        let mut auth_key = vec![0u8; 32];
        rand::thread_rng().fill_bytes(&mut sync_key);
        rand::thread_rng().fill_bytes(&mut auth_key);
        
        HkdfKeys { sync_key, auth_key }
    }
}

#[cfg(test)]
mod tombstone_tests {
    use crate::sync::tombstone::{Tombstone, create_tombstone, is_tombstone};
    use uuid::Uuid;

    #[test]
    fn test_tombstone_creation() {
        let artifact_id = Uuid::new_v4();
        let device_id = "device-1";
        
        let tombstone = create_tombstone(artifact_id, device_id);
        
        assert_eq!(tombstone.artifact_id, artifact_id);
        assert_eq!(tombstone.device_id, device_id);
        assert!(tombstone.timestamp > 0);
        assert!(tombstone.is_deleted);
    }

    #[test]
    fn test_tombstone_detection() {
        let artifact_id = Uuid::new_v4();
        let tombstone = create_tombstone(artifact_id, "device-1");
        
        assert!(is_tombstone(&tombstone));
    }

    #[test]
    fn test_tombstone_serialization() {
        let artifact_id = Uuid::new_v4();
        let tombstone = create_tombstone(artifact_id, "device-1");
        
        let serialized = serde_json::to_string(&tombstone).unwrap();
        let deserialized: Tombstone = serde_json::from_str(&serialized).unwrap();
        
        assert_eq!(deserialized.artifact_id, artifact_id);
        assert!(deserialized.is_deleted);
    }
}

#[cfg(test)]
mod sync_header_tests {
    use crate::sync::protocol::{SyncHeader, create_header};
    use rand::RngCore;

    #[test]
    fn test_header_creation() {
        let device_id = "device-test-123";
        let header = create_header(device_id);
        
        assert_eq!(header.protocol_version, 1);
        assert_eq!(header.device_id, device_id);
        assert!(header.timestamp > 0);
        assert_eq!(header.nonce.len(), 12);
    }

    #[test]
    fn test_unique_nonces() {
        let device_id = "device-test";
        let header1 = create_header(device_id);
        let header2 = create_header(device_id);
        
        assert_ne!(header1.nonce, header2.nonce);
    }

    #[test]
    fn test_timestamp_ordering() {
        let device_id = "device-test";
        let header1 = create_header(device_id);
        std::thread::sleep(std::time::Duration::from_millis(10));
        let header2 = create_header(device_id);
        
        assert!(header2.timestamp > header1.timestamp);
    }
}

#[cfg(test)]
mod conflict_resolution_tests {
    use crate::sync::conflict::{resolve_conflict, ConflictResolution, Conflict};
    use crate::crdt::VectorClock;

    #[test]
    fn test_last_write_wins() {
        let conflict = Conflict {
            field: "title".to_string(),
            values: vec![
                ("Title A".to_string(), VectorClock::new("a", 1)),
                ("Title B".to_string(), VectorClock::new("b", 2)),
            ],
        };
        
        let resolved = resolve_conflict(&conflict, ConflictResolution::LastWriteWins);
        
        // Higher clock wins
        assert_eq!(resolved, "Title B");
    }

    #[test]
    fn test_multi_value_register() {
        let conflict = Conflict {
            field: "tags".to_string(),
            values: vec![
                ("tag1".to_string(), VectorClock::new("a", 1)),
                ("tag2".to_string(), VectorClock::new("b", 1)),
            ],
        };
        
        let resolved = resolve_conflict(&conflict, ConflictResolution::MultiValue);
        
        // Both values preserved
        assert!(resolved.contains("tag1"));
        assert!(resolved.contains("tag2"));
    }

    #[test]
    fn test_concurrent_conflict() {
        let conflict = Conflict {
            field: "content".to_string(),
            values: vec![
                ("Content A".to_string(), VectorClock::new("a", 1)),
                ("Content B".to_string(), VectorClock::new("b", 1)),
            ],
        };
        
        // Concurrent - neither happens before the other
        let resolved = resolve_conflict(&conflict, ConflictResolution::UserResolution);
        
        // Should indicate need for user resolution
        assert!(resolved.contains("Content A"));
        assert!(resolved.contains("Content B"));
    }
}

#[cfg(test)]
mod protocol_version_tests {
    use crate::sync::protocol::{check_protocol_version, ProtocolCompatibility};

    #[test]
    fn test_compatible_version() {
        let result = check_protocol_version(1, 1);
        assert!(matches!(result, ProtocolCompatibility::Compatible));
    }

    #[test]
    fn test_backward_compatible() {
        let result = check_protocol_version(1, 0);
        assert!(matches!(result, ProtocolCompatibility::BackwardCompatible));
    }

    #[test]
    fn test_incompatible_version() {
        let result = check_protocol_version(1, 2);
        assert!(matches!(result, ProtocolCompatibility::Incompatible));
    }
}
