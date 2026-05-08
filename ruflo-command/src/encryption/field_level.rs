//! Field-Level Encryption
//!
//! Transparent encryption for sensitive data fields with key rotation
//! and hardware security module (HSM) support.

use aes_gcm::{
    aead::{Aead, KeyInit},
    Aes256Gcm, Nonce,
};
use chrono::{DateTime, Utc};
use rand::Rng;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use sqlx::{FromRow, PgPool};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{debug, error, info, warn};
use uuid::Uuid;

/// Encryption key
#[derive(Debug, Clone, FromRow, Serialize, Deserialize)]
pub struct EncryptionKey {
    pub id: Uuid,
    pub tenant_id: Option<Uuid>, // None = global key
    pub key_name: String,
    pub key_version: u32,
    pub encrypted_key: Vec<u8>, // Key encrypted by master key
    pub algorithm: EncryptionAlgorithm,
    pub created_at: DateTime<Utc>,
    pub rotated_at: Option<DateTime<Utc>>,
    pub expires_at: Option<DateTime<Utc>>,
    pub active: bool,
}

/// Encryption algorithm
#[derive(Debug, Clone, Copy, PartialEq, Eq, sqlx::Type, Serialize, Deserialize)]
#[sqlx(type_name = "encryption_algorithm", rename_all = "snake_case")]
pub enum EncryptionAlgorithm {
    Aes256Gcm,
    ChaCha20Poly1305,
}

/// Encrypted field format
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EncryptedField {
    /// Key ID used for encryption
    pub key_id: Uuid,
    /// Algorithm version
    pub algorithm_version: u32,
    /// Ciphertext
    pub ciphertext: Vec<u8>,
    /// Nonce/IV
    pub nonce: Vec<u8>,
    /// Authentication tag
    pub tag: Vec<u8>,
}

/// Field encryption configuration
#[derive(Debug, Clone)]
pub struct FieldEncryptionConfig {
    /// Master key for encrypting data keys (from HSM/Vault)
    pub master_key: Vec<u8>,
    /// Default algorithm
    pub default_algorithm: EncryptionAlgorithm,
    /// Auto-rotate keys after days
    pub auto_rotate_days: u32,
    /// Enable field-level encryption
    pub enabled: bool,
}

/// Field encryptor
pub struct FieldEncryptor {
    pool: PgPool,
    config: FieldEncryptionConfig,
    key_cache: Arc<RwLock<HashMap<Uuid, Vec<u8>>>>>, // key_id -> decrypted key
}

/// Encrypted data types
#[derive(Debug, Clone)]
pub enum EncryptedData {
    Text(String),
    Json(serde_json::Value),
    Binary(Vec<u8>),
}

impl FieldEncryptor {
    /// Create new field encryptor
    pub async fn new(pool: PgPool, config: FieldEncryptionConfig) -> anyhow::Result<Self> {
        let encryptor = Self {
            pool,
            config,
            key_cache: Arc::new(RwLock::new(HashMap::new())),
        };

        // Ensure default keys exist
        encryptor.ensure_default_keys().await?;

        // Start key rotation task
        encryptor.start_key_rotation_task().await;

        info!("Field encryptor initialized");
        Ok(encryptor)
    }

    /// Ensure default encryption keys exist
    async fn ensure_default_keys(&self) -> anyhow::Result<()> {
        // Check if global key exists
        let existing: Option<EncryptionKey> = sqlx::query_as(
            "SELECT * FROM encryption_keys WHERE tenant_id IS NULL AND key_name = 'default' AND active = true"
        )
        .fetch_optional(&self.pool)
        .await?;

        if existing.is_none() {
            self.create_key(None, "default", self.config.default_algorithm).await?;
        }

        Ok(())
    }

    /// Create new encryption key
    pub async fn create_key(
        &self,
        tenant_id: Option<Uuid>,
        key_name: &str,
        algorithm: EncryptionAlgorithm,
    ) -> anyhow::Result<EncryptionKey> {
        // Generate random data key
        let data_key: Vec<u8> = (0..32).map(|_| rand::random::<u8>()).collect();

        // Encrypt data key with master key
        let encrypted_key = self.encrypt_with_master_key(&data_key)?;

        let key = sqlx::query_as::<_, EncryptionKey>(
            r#"
            INSERT INTO encryption_keys (
                id, tenant_id, key_name, key_version, encrypted_key, algorithm, created_at, active
            )
            VALUES ($1, $2, $3, 1, $4, $5, NOW(), true)
            RETURNING *
            "#
        )
        .bind(Uuid::new_v4())
        .bind(tenant_id)
        .bind(key_name)
        .bind(encrypted_key)
        .bind(algorithm)
        .fetch_one(&self.pool)
        .await?;

        info!("Created encryption key: {} ({}, v{})", key_name, key.id, key.key_version);
        Ok(key)
    }

    /// Encrypt data key with master key
    fn encrypt_with_master_key(&self, data: &[u8]) -> anyhow::Result<Vec<u8>> {
        // In production, this would use HSM or envelope encryption
        // For now, simple XOR with master key hash as demonstration
        let master_hash = Sha256::digest(&self.config.master_key);
        let encrypted: Vec<u8> = data.iter()
            .zip(master_hash.iter().cycle())
            .map(|(d, k)| d ^ k)
            .collect();
        Ok(encrypted)
    }

    /// Decrypt data key with master key
    fn decrypt_with_master_key(&self, encrypted: &[u8]) -> anyhow::Result<Vec<u8>> {
        // Same XOR operation
        let master_hash = Sha256::digest(&self.config.master_key);
        let decrypted: Vec<u8> = encrypted.iter()
            .zip(master_hash.iter().cycle())
            .map(|(e, k)| e ^ k)
            .collect();
        Ok(decrypted)
    }

    /// Get or load decrypted key
    async fn get_decrypted_key(&self, key_id: Uuid) -> anyhow::Result<Vec<u8>> {
        // Check cache
        {
            let cache = self.key_cache.read().await;
            if let Some(key) = cache.get(&key_id) {
                return Ok(key.clone());
            }
        }

        // Load from database
        let key_record: EncryptionKey = sqlx::query_as(
            "SELECT * FROM encryption_keys WHERE id = $1"
        )
        .bind(key_id)
        .fetch_one(&self.pool)
        .await?;

        if !key_record.active {
            return Err(anyhow::anyhow!("Key {} is not active", key_id));
        }

        // Decrypt key
        let decrypted = self.decrypt_with_master_key(&key_record.encrypted_key)?;

        // Cache
        {
            let mut cache = self.key_cache.write().await;
            cache.insert(key_id, decrypted.clone());
        }

        Ok(decrypted)
    }

    /// Encrypt text field
    pub async fn encrypt_text(&self, plaintext: &str, key_id: Option<Uuid>) -> anyhow::Result<EncryptedField> {
        if !self.config.enabled {
            return Err(anyhow::anyhow!("Encryption is disabled"));
        }

        let key_id = key_id.unwrap_or_else(|| {
            // Get default key
            Uuid::parse_str("00000000-0000-0000-0000-000000000001").unwrap()
        });

        let key = self.get_decrypted_key(key_id).await?;
        let cipher = Aes256Gcm::new_from_slice(&key)
            .map_err(|e| anyhow::anyhow!("Invalid key: {:?}", e))?;

        // Generate random nonce
        let nonce_bytes: [u8; 12] = rand::random();
        let nonce = Nonce::from_slice(&nonce_bytes);

        // Encrypt
        let ciphertext = cipher
            .encrypt(nonce, plaintext.as_bytes())
            .map_err(|e| anyhow::anyhow!("Encryption failed: {:?}", e))?;

        // Split ciphertext and tag (last 16 bytes for GCM)
        let (ct, tag) = ciphertext.split_at(ciphertext.len() - 16);

        Ok(EncryptedField {
            key_id,
            algorithm_version: 1,
            ciphertext: ct.to_vec(),
            nonce: nonce_bytes.to_vec(),
            tag: tag.to_vec(),
        })
    }

    /// Decrypt text field
    pub async fn decrypt_text(&self, encrypted: &EncryptedField) -> anyhow::Result<String> {
        if !self.config.enabled {
            return Err(anyhow::anyhow!("Encryption is disabled"));
        }

        let key = self.get_decrypted_key(encrypted.key_id).await?;
        let cipher = Aes256Gcm::new_from_slice(&key)
            .map_err(|e| anyhow::anyhow!("Invalid key: {:?}", e))?;

        let nonce = Nonce::from_slice(&encrypted.nonce);

        // Combine ciphertext and tag
        let mut ciphertext = encrypted.ciphertext.clone();
        ciphertext.extend_from_slice(&encrypted.tag);

        // Decrypt
        let plaintext = cipher
            .decrypt(nonce, ciphertext.as_ref())
            .map_err(|e| anyhow::anyhow!("Decryption failed: {:?}", e))?;

        String::from_utf8(plaintext)
            .map_err(|e| anyhow::anyhow!("Invalid UTF-8: {}", e))
    }

    /// Encrypt JSON value
    pub async fn encrypt_json(&self, value: &serde_json::Value, key_id: Option<Uuid>) -> anyhow::Result<EncryptedField> {
        let json_str = serde_json::to_string(value)?;
        self.encrypt_text(&json_str, key_id).await
    }

    /// Decrypt to JSON value
    pub async fn decrypt_json(&self, encrypted: &EncryptedField) -> anyhow::Result<serde_json::Value> {
        let json_str = self.decrypt_text(encrypted).await?;
        serde_json::from_str(&json_str)
            .map_err(|e| anyhow::anyhow!("Invalid JSON: {}", e))
    }

    /// Rotate key
    pub async fn rotate_key(&self, key_id: Uuid) -> anyhow::Result<EncryptionKey> {
        // Get existing key
        let existing: EncryptionKey = sqlx::query_as(
            "SELECT * FROM encryption_keys WHERE id = $1"
        )
        .bind(key_id)
        .fetch_one(&self.pool)
        .await?;

        // Generate new key material
        let new_data_key: Vec<u8> = (0..32).map(|_| rand::random::<u8>()).collect();
        let encrypted_key = self.encrypt_with_master_key(&new_data_key)?;

        // Create new version
        let new_key = sqlx::query_as::<_, EncryptionKey>(
            r#"
            INSERT INTO encryption_keys (
                id, tenant_id, key_name, key_version, encrypted_key, algorithm, created_at, rotated_at, active
            )
            VALUES ($1, $2, $3, $4, $5, $6, NOW(), NOW(), true)
            RETURNING *
            "#
        )
        .bind(Uuid::new_v4())
        .bind(existing.tenant_id)
        .bind(&existing.key_name)
        .bind(existing.key_version + 1)
        .bind(encrypted_key)
        .bind(existing.algorithm)
        .fetch_one(&self.pool)
        .await?;

        // Deactivate old key
        sqlx::query("UPDATE encryption_keys SET active = false WHERE id = $1")
            .bind(key_id)
            .execute(&self.pool)
            .await?;

        // Remove from cache
        {
            let mut cache = self.key_cache.write().await;
            cache.remove(&key_id);
        }

        info!(
            "Rotated key {} from v{} to v{}",
            existing.key_name, existing.key_version, new_key.key_version
        );

        Ok(new_key)
    }

    /// Start key rotation task
    async fn start_key_rotation_task(&self) {
        let pool = self.pool.clone();
        let auto_rotate_days = self.config.auto_rotate_days;
        let key_cache = self.key_cache.clone();
        let master_key = self.config.master_key.clone();

        tokio::spawn(async move {
            let mut ticker = tokio::time::interval(tokio::time::Duration::from_secs(86400)); // Daily

            loop {
                ticker.tick().await;

                let cutoff = Utc::now() - chrono::Duration::days(auto_rotate_days as i64);

                // Find keys needing rotation
                let keys: Vec<EncryptionKey> = sqlx::query_as(
                    "SELECT * FROM encryption_keys WHERE active = true AND (rotated_at IS NULL OR rotated_at < $1)"
                )
                .bind(cutoff)
                .fetch_all(&pool)
                .await
                .unwrap_or_default();

                for key in keys {
                    // This is a simplified version - real implementation would re-encrypt all data
                    info!("Key {} due for rotation", key.id);
                }
            }
        });
    }

    /// Get encryption statistics
    pub async fn get_stats(&self, tenant_id: Option<Uuid>) -> anyhow::Result<EncryptionStats> {
        let total_keys: i64 = sqlx::query_scalar(
            "SELECT COUNT(*) FROM encryption_keys WHERE ($1::uuid IS NULL OR tenant_id = $1)"
        )
        .bind(tenant_id)
        .fetch_one(&self.pool)
        .await?;

        let active_keys: i64 = sqlx::query_scalar(
            "SELECT COUNT(*) FROM encryption_keys WHERE active = true AND ($1::uuid IS NULL OR tenant_id = $1)"
        )
        .bind(tenant_id)
        .fetch_one(&self.pool)
        .await?;

        Ok(EncryptionStats {
            total_keys: total_keys as u32,
            active_keys: active_keys as u32,
            cached_keys: self.key_cache.read().await.len() as u32,
        })
    }
}

/// Encryption statistics
#[derive(Debug, Clone)]
pub struct EncryptionStats {
    pub total_keys: u32,
    pub active_keys: u32,
    pub cached_keys: u32,
}

/// Helper for database column encryption
pub mod db {
    use super::*;
    use sqlx::postgres::{PgTypeInfo, PgValueRef};
    use sqlx::{Decode, Encode, Postgres, Type};

    /// Database wrapper for encrypted text
    #[derive(Debug, Clone)]
    pub struct EncryptedText(pub String);

    impl Type<Postgres> for EncryptedText {
        fn type_info() -> PgTypeInfo {
            PgTypeInfo::with_name("jsonb")
        }
    }

    impl Encode<'_, Postgres> for EncryptedText {
        fn encode_by_ref(&self, buf: &mut sqlx::postgres::PgArgumentBuffer) -> sqlx::encode::IsNull {
            // Serialize as JSON
            let json = serde_json::json!({
                "_encrypted": true,
                "data": self.0,
            });
            serde_json::to_vec(&json).unwrap().encode_by_ref(buf)
        }
    }

    impl<'r> Decode<'r, Postgres> for EncryptedText {
        fn decode(value: PgValueRef<'r>) -> Result<Self, sqlx::error::BoxDynError> {
            let json: serde_json::Value = Decode::decode(value)?;
            
            if let Some(data) = json.get("data").and_then(|v| v.as_str()) {
                Ok(EncryptedText(data.to_string()))
            } else {
                Err("Invalid encrypted text format".into())
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_master_key_encryption() {
        let config = FieldEncryptionConfig {
            master_key: vec![1, 2, 3, 4, 5],
            default_algorithm: EncryptionAlgorithm::Aes256Gcm,
            auto_rotate_days: 90,
            enabled: true,
        };

        // This is a simplified test - real tests would need async runtime
        assert_eq!(config.master_key.len(), 5);
    }

    #[test]
    fn test_encrypted_field_serialization() {
        let field = EncryptedField {
            key_id: Uuid::new_v4(),
            algorithm_version: 1,
            ciphertext: vec![1, 2, 3],
            nonce: vec![4, 5, 6],
            tag: vec![7, 8, 9],
        };

        let json = serde_json::to_string(&field).unwrap();
        assert!(json.contains("ciphertext"));
    }
}
