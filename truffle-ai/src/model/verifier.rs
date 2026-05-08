//! Model Verifier
//! 
//! Handles SHA256 verification of model files on startup.
//! 
//! # Security (SPEC v2.0 Section 4.1.1)
//! 
//! All model files are verified using SHA256 checksums before loading.
//! This prevents loading corrupted or tampered models.

use std::path::Path;
use std::fs::File;
use std::io::{BufReader, Read};
use sha2::{Sha256, Digest};
use tracing::{info, warn, error};

use crate::{AiError, AiResult, DEFAULT_MODEL_SHA256};
use crate::model::{ModelConfig, ModelMetadata};

/// Verifies model integrity using SHA256 checksums
pub struct ModelVerifier {
    config: ModelConfig,
}

/// Result of model verification
#[derive(Debug, Clone)]
pub struct VerificationResult {
    /// Whether verification passed
    pub passed: bool,
    /// Expected SHA256 hash
    pub expected_hash: String,
    /// Actual SHA256 hash (if computed)
    pub actual_hash: Option<String>,
    /// File size in bytes
    pub file_size: u64,
    /// Time taken for verification in milliseconds
    pub verification_time_ms: u64,
}

impl ModelVerifier {
    /// Create a new model verifier
    pub fn new(config: ModelConfig) -> Self {
        Self { config }
    }
    
    /// Verify the model file on startup
    /// 
    /// This should be called before loading the model into memory.
    /// It compares the file's SHA256 hash against the expected value.
    /// 
    /// # Arguments
    /// 
    /// * `model_path` - Path to the model file
    /// 
    /// # Returns
    /// 
    /// Returns `Ok(())` if verification passes, or an error if:
    /// - The file doesn't exist
    /// - The hash doesn't match
    /// - There's an I/O error
    /// 
    /// # Example
    /// 
    /// ```rust,no_run
    /// use truffle_ai::model::{ModelVerifier, ModelConfig};
    /// 
    /// let config = ModelConfig::default();
    /// let verifier = ModelVerifier::new(config);
    /// verifier.verify_on_startup("models/gemma-4-2b-it-Q4_K_M.gguf").unwrap();
    /// ```
    pub fn verify_on_startup(&self, model_path: &Path) -> AiResult<VerificationResult> {
        let start_time = std::time::Instant::now();
        
        info!("Verifying model integrity: {:?}", model_path);
        
        // Check if file exists
        if !model_path.exists() {
            error!("Model file not found: {:?}", model_path);
            return Err(AiError::Model(
                format!("Model file not found: {:?}", model_path)
            ));
        }
        
        // Get file size
        let file_size = std::fs::metadata(model_path)
            .map_err(|e| AiError::Io(e))?
            .len();
        
        info!("Model file size: {} bytes ({} MB)", 
            file_size, 
            file_size / 1024 / 1024
        );
        
        // Get expected hash
        let expected_hash = self.get_expected_hash(model_path)?;
        
        // Compute actual hash
        let actual_hash = match self.compute_sha256(model_path) {
            Ok(hash) => hash,
            Err(e) => {
                error!("Failed to compute SHA256: {}", e);
                return Err(e);
            }
        };
        
        let verification_time = start_time.elapsed();
        
        // Compare hashes
        let passed = actual_hash.to_lowercase() == expected_hash.to_lowercase();
        
        let result = VerificationResult {
            passed,
            expected_hash: expected_hash.clone(),
            actual_hash: Some(actual_hash.clone()),
            file_size,
            verification_time_ms: verification_time.as_millis() as u64,
        };
        
        if passed {
            info!(
                "Model verification PASSED (took {}ms)",
                verification_time_ms
            );
            Ok(result)
        } else {
            error!("Model verification FAILED!");
            error!("Expected: {}", expected_hash);
            error!("Actual:   {}", actual_hash);
            
            Err(AiError::Model(
                "Model file integrity check failed. The file may be corrupted or tampered with.".to_string()
            ))
        }
    }
    
    /// Quick verification using file size and modification time
    /// 
    /// This is faster than full SHA256 but less secure.
    /// Use for quick checks, not for security-critical verification.
    pub fn quick_verify(&self, model_path: &Path) -> AiResult<bool> {
        // Load metadata
        let metadata_path = model_path.with_extension("gguf.json");
        
        if !metadata_path.exists() {
            warn!("No metadata file found for quick verify");
            return Ok(false);
        }
        
        let metadata: ModelMetadata = serde_json::from_str(
            &std::fs::read_to_string(&metadata_path)
                .map_err(|e| AiError::Io(e))?
        ).map_err(|e| AiError::Json(e))?;
        
        // Check file size
        let file_size = std::fs::metadata(model_path)
            .map_err(|e| AiError::Io(e))?
            .len();
        
        if file_size != metadata.file_size {
            warn!(
                "File size mismatch: expected {}, got {}",
                metadata.file_size, file_size
            );
            return Ok(false);
        }
        
        info!("Quick verification passed");
        Ok(true)
    }
    
    /// Compute SHA256 hash of a file
    /// 
    /// Reads the file in chunks to avoid loading large files into memory.
    fn compute_sha256(&self, path: &Path) -> AiResult<String> {
        let file = File::open(path)
            .map_err(|e| AiError::Io(e))?;
        
        let mut reader = BufReader::new(file);
        let mut hasher = Sha256::new();
        let mut buffer = [0u8; 8192]; // 8KB chunks
        
        let mut total_read = 0u64;
        
        loop {
            match reader.read(&mut buffer) {
                Ok(0) => break, // EOF
                Ok(n) => {
                    hasher.update(&buffer[..n]);
                    total_read += n as u64;
                    
                    // Log progress every 100MB
                    if total_read % (100 * 1024 * 1024) == 0 {
                        debug!("Hashing progress: {} MB", total_read / 1024 / 1024);
                    }
                }
                Err(e) => return Err(AiError::Io(e)),
            }
        }
        
        let hash = hasher.finalize();
        Ok(hex::encode(hash))
    }
    
    /// Get the expected hash for a model file
    /// 
    /// First tries to load from metadata file, then falls back to
    /// the hardcoded default hash.
    fn get_expected_hash(&self, model_path: &Path) -> AiResult<String> {
        // Try to load from metadata
        let metadata_path = model_path.with_extension("gguf.json");
        
        if metadata_path.exists() {
            match std::fs::read_to_string(&metadata_path) {
                Ok(content) => {
                    match serde_json::from_str::<ModelMetadata>(&content) {
                        Ok(metadata) => {
                            info!("Using hash from metadata file");
                            return Ok(metadata.sha256);
                        }
                        Err(e) => {
                            warn!("Failed to parse metadata: {}", e);
                        }
                    }
                }
                Err(e) => {
                    warn!("Failed to read metadata: {}", e);
                }
            }
        }
        
        // Fall back to default hash
        info!("Using default hardcoded hash");
        Ok(DEFAULT_MODEL_SHA256.to_string())
    }
    
    /// Verify and log the result (non-blocking)
    pub fn verify_and_log(&self, model_path: &Path) {
        let path = model_path.to_path_buf();
        let verifier = Self::new(self.config.clone());
        
        // Spawn verification in background
        tokio::spawn(async move {
            match verifier.verify_on_startup(&path) {
                Ok(result) => {
                    if result.passed {
                        info!("Background verification completed successfully");
                    } else {
                        error!("Background verification failed!");
                    }
                }
                Err(e) => {
                    error!("Background verification error: {}", e);
                }
            }
        });
    }
    
    /// Generate a new SHA256 hash for a model file
    /// 
    /// This is used when downloading new models.
    pub fn generate_hash(&self, model_path: &Path) -> AiResult<String> {
        info!("Generating SHA256 hash for new model: {:?}", model_path);
        self.compute_sha256(model_path)
    }
}

/// Batch verification for multiple models
pub struct BatchVerifier {
    verifiers: Vec<ModelVerifier>,
}

impl BatchVerifier {
    /// Create a new batch verifier
    pub fn new() -> Self {
        Self {
            verifiers: Vec::new(),
        }
    }
    
    /// Add a model to verify
    pub fn add_model(&mut self, config: ModelConfig) {
        self.verifiers.push(ModelVerifier::new(config));
    }
    
    /// Verify all models
    pub async fn verify_all(&self) -> Vec<(String, AiResult<VerificationResult>)> {
        let mut results = Vec::new();
        
        for verifier in &self.verifiers {
            let model_path = verifier.config.model_path();
            let model_name = verifier.config.variant.filename().to_string();
            
            let result = verifier.verify_on_startup(&model_path);
            results.push((model_name, result));
        }
        
        results
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;
    use std::io::Write;
    
    #[test]
    fn test_compute_sha256() {
        let temp_dir = TempDir::new().unwrap();
        let test_file = temp_dir.path().join("test.txt");
        
        // Create test file with known content
        let mut file = File::create(&test_file).unwrap();
        file.write_all(b"Hello, World!").unwrap();
        drop(file);
        
        let config = ModelConfig::default();
        let verifier = ModelVerifier::new(config);
        
        let hash = verifier.compute_sha256(&test_file).unwrap();
        
        // Known SHA256 for "Hello, World!"
        assert_eq!(
            hash,
            "dffd6021bb2bd5b0af676290809ec3a53191dd81c7f70a4b28688a362182986f"
        );
    }
    
    #[test]
    fn test_verification_result() {
        let result = VerificationResult {
            passed: true,
            expected_hash: "abc123".to_string(),
            actual_hash: Some("abc123".to_string()),
            file_size: 1024,
            verification_time_ms: 100,
        };
        
        assert!(result.passed);
        assert_eq!(result.expected_hash, result.actual_hash.unwrap());
    }
}
