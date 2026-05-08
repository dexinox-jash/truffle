//! Field-Level Encryption

use serde::{Deserialize, Serialize};

pub struct FieldEncryption;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct EncryptedField {
    pub field_path: String,
    pub ciphertext: Vec<u8>,
}

impl FieldEncryption {
    pub fn new() -> Self {
        Self
    }
}
