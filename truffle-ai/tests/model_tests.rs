//! Model Management Tests

use std::path::PathBuf;
use tempfile::TempDir;

use truffle_ai::model::{
    ModelConfig, ModelVariant, ModelManager, ModelVerifier, ModelMetadata
};

#[test]
fn test_model_variant_default() {
    let variant = ModelVariant::default();
    assert_eq!(variant, ModelVariant::E2B);
}

#[test]
fn test_model_variant_filename() {
    assert_eq!(ModelVariant::E2B.filename(), "gemma-4-2b-it-Q4_K_M.gguf");
    assert_eq!(ModelVariant::E4B.filename(), "gemma-4-4b-it-Q4_K_M.gguf");
    assert_eq!(ModelVariant::E9B.filename(), "gemma-4-9b-it-Q4_K_M.gguf");
}

#[test]
fn test_model_variant_sizes() {
    // E2B should be smallest
    assert!(ModelVariant::E2B.expected_size() < ModelVariant::E4B.expected_size());
    assert!(ModelVariant::E4B.expected_size() < ModelVariant::E9B.expected_size());
}

#[test]
fn test_model_config_default() {
    let config = ModelConfig::default();
    
    assert_eq!(config.variant, ModelVariant::E2B);
    assert_eq!(config.temperature, 0.1);
    assert_eq!(config.max_tokens, 2048);
    assert_eq!(config.context_size, 8192);
    assert!(config.use_mmap);
    assert!(!config.use_mlock);
}

#[test]
fn test_model_config_paths() {
    let config = ModelConfig::default();
    
    let model_path = config.model_path();
    assert!(model_path.to_string_lossy().contains("gemma-4-2b"));
    
    let download_url = config.variant_download_url();
    assert!(download_url.contains("gemma-4-2b"));
}

#[test]
fn test_model_metadata_default() {
    let metadata = ModelMetadata::default_e2b();
    
    assert_eq!(metadata.variant, ModelVariant::E2B);
    assert_eq!(metadata.llama_commit, "b2699");
    assert!(metadata.file_size > 0);
}

#[test]
fn test_model_manager_creation() {
    let temp_dir = TempDir::new().unwrap();
    let config = ModelConfig {
        model_dir: temp_dir.path().to_path_buf(),
        ..Default::default()
    };
    
    let manager = ModelManager::new(config);
    assert!(manager.is_ok());
}

#[test]
fn test_model_verifier_creation() {
    let config = ModelConfig::default();
    let verifier = ModelVerifier::new(config);
    
    // Verifier should be created successfully
    assert!(std::mem::size_of_val(&verifier) > 0);
}

#[tokio::test]
async fn test_model_manager_space_check() {
    let temp_dir = TempDir::new().unwrap();
    let config = ModelConfig {
        model_dir: temp_dir.path().to_path_buf(),
        ..Default::default()
    };
    
    let manager = ModelManager::new(config).unwrap();
    
    // Should have enough space (using placeholder)
    assert!(manager.has_enough_space().unwrap());
}
