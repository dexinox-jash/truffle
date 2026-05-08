//! Truffle AI - On-Device Multimodal Processing Pipeline
//! 
//! This crate provides the AI processing capabilities for Project Truffle,
//! implementing on-device multimodal AI using Gemma 4 E2B via llama.cpp.
//! 
//! # Architecture
//! 
//! ```text
//! ┌─────────────────────────────────────────────────────────────────┐
//! │                    TRUFFLE AI PIPELINE                          │
//! ├─────────────────────────────────────────────────────────────────┤
//! │  ┌─────────────┐    ┌─────────────┐    ┌─────────────────────┐ │
//! │  │   Schema    │───▶│  Compiler   │───▶│   WikiNode Output   │ │
//! │  │   Loader    │    │   Engine    │    │   (JSON/MD)         │ │
//! │  └─────────────┘    └──────┬──────┘    └─────────────────────┘ │
//! │                            │                                    │
//! │         ┌──────────────────┼──────────────────┐                │
//! │         ▼                  ▼                  ▼                │
//! │  ┌─────────────┐    ┌─────────────┐    ┌─────────────┐        │
//! │  │   Prompt    │    │   Gemma 4   │    │   Safety    │        │
//! │  │  Templates  │    │    E2B      │    │  Classifier │        │
//! │  └─────────────┘    └─────────────┘    └─────────────┘        │
//! └─────────────────────────────────────────────────────────────────┘
//! ```
//! 
//! # Key Constraints (SPEC v2.0 Section 1.2)
//! 
//! - **Sovereignty**: All processing happens on-device
//! - **Zero-Knowledge**: No data leaves device unencrypted
//! - **Survival Mode**: Works 100% offline
//! - **Performance**: 30s timeout, 4GB RAM cap
//! 
//! # Example Usage
//! 
//! ```rust,no_run
//! use truffle_ai::{AiPipeline, ModelConfig, SchemaConfig};
//! 
//! #[tokio::main]
//! async fn main() -> anyhow::Result<()> {
//!     // Initialize the AI pipeline
//!     let config = ModelConfig::default();
//!     let pipeline = AiPipeline::new(config).await?;
//!     
//!     // Compile a screenshot
//!     let result = pipeline.compile_image("screenshot.png").await?;
//!     println!("Compiled: {:?}", result);
//!     
//!     Ok(())
//! }
//! ```

#![warn(missing_docs)]
#![warn(clippy::all)]
#![warn(clippy::pedantic)]

use std::path::PathBuf;
use thiserror::Error;

// Module declarations
pub mod model;
pub mod pipeline;
pub mod schema;

// Re-export main types
pub use model::{ModelConfig, ModelManager, ModelVerifier};
pub use pipeline::{AiPipeline, CompilationOptions, CompilationResult};
pub use schema::{SchemaConfig, SchemaLoader};

/// Version of the AI pipeline (follows SPEC v2.0)
pub const PIPELINE_VERSION: &str = "2.0.0";

/// Default model configuration
pub const DEFAULT_MODEL: &str = "gemma-4-2b-it-Q4_K_M.gguf";
pub const DEFAULT_MODEL_SHA256: &str = "a1b2c3d4e5f6789012345678901234567890abcdef1234567890abcdef123456";
pub const DEFAULT_MODEL_URL: &str = "https://models.truffle.io/gemma-4-2b-it-Q4_K_M.gguf";

/// Maximum RAM usage in bytes (4GB cap per SPEC v2.0)
pub const MAX_RAM_BYTES: usize = 4 * 1024 * 1024 * 1024;

/// Timeout for compilation in seconds (30s per SPEC v2.0)
pub const COMPILATION_TIMEOUT_SECS: u64 = 30;

/// Maximum image dimension (896px per SPEC v2.0)
pub const MAX_IMAGE_DIMENSION: u32 = 896;

/// Errors that can occur in the AI pipeline
#[derive(Error, Debug)]
pub enum AiError {
    /// Model not found or invalid
    #[error("Model error: {0}")]
    Model(String),
    
    /// Compilation failed
    #[error("Compilation failed: {0}")]
    Compilation(String),
    
    /// Safety filter triggered
    #[error("Content safety violation: {category}")]
    SafetyViolation { category: String },
    
    /// Schema validation error
    #[error("Schema error: {0}")]
    Schema(String),
    
    /// I/O error
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),
    
    /// JSON parsing error
    #[error("JSON error: {0}")]
    Json(#[from] serde_json::Error),
    
    /// Timeout error
    #[error("Operation timed out after {0}s")]
    Timeout(u64),
    
    /// Memory limit exceeded
    #[error("Memory limit exceeded: {used}MB > {limit}MB")]
    MemoryLimit { used: usize, limit: usize },
    
    /// Generic error
    #[error("{0}")]
    Other(String),
}

/// Result type for AI operations
pub type AiResult<T> = Result<T, AiError>;

/// Configuration for the AI pipeline
#[derive(Debug, Clone)]
pub struct PipelineConfig {
    /// Model configuration
    pub model: ModelConfig,
    /// Schema configuration
    pub schema: SchemaConfig,
    /// Compilation options
    pub compilation: CompilationOptions,
    /// Model directory path
    pub model_dir: PathBuf,
    /// Prompts directory path
    pub prompts_dir: PathBuf,
}

impl Default for PipelineConfig {
    fn default() -> Self {
        Self {
            model: ModelConfig::default(),
            schema: SchemaConfig::default(),
            compilation: CompilationOptions::default(),
            model_dir: PathBuf::from("models"),
            prompts_dir: PathBuf::from("prompts"),
        }
    }
}

/// Initialize the AI pipeline with logging and metrics
pub fn init() {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::from_default_env()
                .add_directive("truffle_ai=info".parse().unwrap())
        )
        .init();
    
    tracing::info!("Truffle AI Pipeline v{} initialized", PIPELINE_VERSION);
}

/// Check if the system meets minimum requirements
pub fn check_system_requirements() -> AiResult<()> {
    use sysinfo::{System, SystemExt};
    
    let mut sys = System::new_all();
    sys.refresh_all();
    
    let total_memory = sys.total_memory() * 1024; // Convert KB to bytes
    
    if total_memory < MAX_RAM_BYTES {
        tracing::warn!(
            "System has {}MB RAM, recommended is {}MB",
            total_memory / 1024 / 1024,
            MAX_RAM_BYTES / 1024 / 1024
        );
    }
    
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_pipeline_version() {
        assert_eq!(PIPELINE_VERSION, "2.0.0");
    }
    
    #[test]
    fn test_memory_limits() {
        assert_eq!(MAX_RAM_BYTES, 4 * 1024 * 1024 * 1024);
    }
    
    #[test]
    fn test_timeout() {
        assert_eq!(COMPILATION_TIMEOUT_SECS, 30);
    }
}
