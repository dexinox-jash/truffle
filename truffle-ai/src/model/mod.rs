//! Model Management Module
//! 
//! Handles model lifecycle: download, verification, caching, and inference.
//! 
//! # Model Specifications (SPEC v2.0 Section 4.1)
//! 
//! - **Primary Model**: Gemma 4 E2B (2B parameters, Q4_K_M quantized)
//! - **Backend**: llama.cpp (commit b2699)
//! - **Hardware**: Metal (macOS) / CUDA (Windows) / Vulkan (Linux)
//! - **Size**: ~1.3GB
//! - **RAM Usage**: ~2.5GB at runtime
//! - **Verification**: SHA256 checksum on startup

use std::path::PathBuf;
use serde::{Deserialize, Serialize};

pub mod manager;
pub mod gemma4;
pub mod verifier;

pub use manager::ModelManager;
pub use gemma4::Gemma4Engine;
pub use verifier::ModelVerifier;

/// Model variant for A/B testing (SPEC v2.0 Section 4.1.2)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ModelVariant {
    /// E2B (2B parameters) - Default for 90% of users
    E2B,
    /// E4B (4B parameters) - Experimental for 10% of users
    E4B,
    /// E9B (9B parameters) - Premium tier only
    E9B,
}

impl Default for ModelVariant {
    fn default() -> Self {
        ModelVariant::E2B
    }
}

impl ModelVariant {
    /// Get the model filename for this variant
    pub fn filename(&self) -> &'static str {
        match self {
            ModelVariant::E2B => "gemma-4-2b-it-Q4_K_M.gguf",
            ModelVariant::E4B => "gemma-4-4b-it-Q4_K_M.gguf",
            ModelVariant::E9B => "gemma-4-9b-it-Q4_K_M.gguf",
        }
    }
    
    /// Get expected file size in bytes
    pub fn expected_size(&self) -> u64 {
        match self {
            ModelVariant::E2B => 1_400_000_000,  // ~1.3GB
            ModelVariant::E4B => 2_500_000_000,  // ~2.3GB
            ModelVariant::E9B => 5_500_000_000,  // ~5.1GB
        }
    }
    
    /// Get expected RAM usage in bytes
    pub fn ram_usage(&self) -> usize {
        match self {
            ModelVariant::E2B => 2_500_000_000,  // ~2.5GB
            ModelVariant::E4B => 4_500_000_000,  // ~4.5GB
            ModelVariant::E9B => 9_000_000_000,  // ~9GB
        }
    }
    
    /// Check if this variant can run on the current system
    pub fn is_compatible(&self) -> bool {
        use sysinfo::{System, SystemExt};
        
        let mut sys = System::new_all();
        sys.refresh_all();
        let total_memory = (sys.total_memory() * 1024) as usize;
        
        total_memory >= self.ram_usage()
    }
}

/// Configuration for model management
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelConfig {
    /// Model variant to use
    pub variant: ModelVariant,
    /// Directory to store models
    pub model_dir: PathBuf,
    /// CDN URL for model downloads
    pub download_url: String,
    /// Number of GPU layers to offload
    pub gpu_layers: i32,
    /// Context size (number of tokens)
    pub context_size: usize,
    /// Number of threads for inference
    pub threads: usize,
    /// Batch size for processing
    pub batch_size: usize,
    /// Enable memory mapping
    pub use_mmap: bool,
    /// Enable memory locking
    pub use_mlock: bool,
    /// Seed for reproducibility
    pub seed: u32,
    /// Temperature for sampling
    pub temperature: f32,
    /// Maximum tokens to generate
    pub max_tokens: usize,
}

impl Default for ModelConfig {
    fn default() -> Self {
        Self {
            variant: ModelVariant::default(),
            model_dir: PathBuf::from("models"),
            download_url: crate::DEFAULT_MODEL_URL.to_string(),
            gpu_layers: -1, // Auto-detect
            context_size: 8192,
            threads: num_cpus::get(),
            batch_size: 512,
            use_mmap: true,
            use_mlock: false,
            seed: 42,
            temperature: 0.1, // Low temp for deterministic JSON
            max_tokens: 2048,
        }
    }
}

impl ModelConfig {
    /// Get the full path to the model file
    pub fn model_path(&self) -> PathBuf {
        self.model_dir.join(self.variant.filename())
    }
    
    /// Get the download URL for the current variant
    pub fn variant_download_url(&self) -> String {
        format!("{}/{}", self.download_url, self.variant.filename())
    }
}

/// Model metadata stored alongside the model file
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelMetadata {
    /// Model variant
    pub variant: ModelVariant,
    /// Version string
    pub version: String,
    /// SHA256 hash of the model file
    pub sha256: String,
    /// Download timestamp
    pub downloaded_at: chrono::DateTime<chrono::Utc>,
    /// File size in bytes
    pub file_size: u64,
    /// llama.cpp commit hash used for testing
    pub llama_commit: String,
}

impl ModelMetadata {
    /// Create metadata for the default E2B model
    pub fn default_e2b() -> Self {
        Self {
            variant: ModelVariant::E2B,
            version: crate::PIPELINE_VERSION.to_string(),
            sha256: crate::DEFAULT_MODEL_SHA256.to_string(),
            downloaded_at: chrono::Utc::now(),
            file_size: ModelVariant::E2B.expected_size(),
            llama_commit: "b2699".to_string(),
        }
    }
}

/// Download progress callback
pub type DownloadProgressCallback = Box<dyn Fn(u64, u64) + Send + Sync>;

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_model_variant_filename() {
        assert_eq!(ModelVariant::E2B.filename(), "gemma-4-2b-it-Q4_K_M.gguf");
        assert_eq!(ModelVariant::E4B.filename(), "gemma-4-4b-it-Q4_K_M.gguf");
    }
    
    #[test]
    fn test_model_config_default() {
        let config = ModelConfig::default();
        assert_eq!(config.variant, ModelVariant::E2B);
        assert_eq!(config.temperature, 0.1);
        assert_eq!(config.max_tokens, 2048);
    }
}
