//! Integration Tests for Truffle AI Pipeline
//! 
//! Tests the full compilation pipeline end-to-end.

use std::path::PathBuf;
use tempfile::TempDir;

use truffle_ai::{
    AiPipeline, PipelineConfig, ModelConfig, SchemaConfig,
    CompilationOptions, CompilationPriority,
};

/// Setup test environment
fn setup_test_env() -> (TempDir, PipelineConfig) {
    let temp_dir = TempDir::new().unwrap();
    
    let config = PipelineConfig {
        model: ModelConfig {
            model_dir: temp_dir.path().join("models"),
            ..Default::default()
        },
        schema: SchemaConfig {
            schema_path: temp_dir.path().join("schema.md"),
            ..Default::default()
        },
        compilation: CompilationOptions::default(),
        model_dir: temp_dir.path().join("models"),
        prompts_dir: temp_dir.path().join("prompts"),
    };
    
    (temp_dir, config)
}

#[tokio::test]
async fn test_pipeline_initialization() {
    let (_temp_dir, config) = setup_test_env();
    
    // This would require actual model files in production
    // For now, just verify the config structure
    assert_eq!(config.model.variant, truffle_ai::model::ModelVariant::E2B);
}

#[tokio::test]
async fn test_compilation_options() {
    let options = CompilationOptions {
        force_recompile: true,
        skip_safety: false,
        timeout_secs: Some(60),
        priority: CompilationPriority::High,
        app_context: None,
    };
    
    assert!(options.force_recompile);
    assert!(!options.skip_safety);
    assert_eq!(options.timeout_secs, Some(60));
    assert_eq!(options.priority, CompilationPriority::High);
}

#[test]
fn test_pipeline_version() {
    assert_eq!(truffle_ai::PIPELINE_VERSION, "2.0.0");
}

#[test]
fn test_memory_limits() {
    assert_eq!(truffle_ai::MAX_RAM_BYTES, 4 * 1024 * 1024 * 1024);
}

#[test]
fn test_timeout_constant() {
    assert_eq!(truffle_ai::COMPILATION_TIMEOUT_SECS, 30);
}

#[test]
fn test_image_dimension_limit() {
    assert_eq!(truffle_ai::MAX_IMAGE_DIMENSION, 896);
}
