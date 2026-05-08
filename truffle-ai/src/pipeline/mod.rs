//! AI Processing Pipeline
//! 
//! The main compilation pipeline that transforms screenshots into structured knowledge.
//! 
//! # Pipeline Stages (SPEC v2.0 Section 2.2)
//! 
//! ```text
//! Screenshot → Safety Check → Preprocess → Prompt → Gemma 4 → Parse → Validate → WikiNode
//! ```
//! 
//! # Key Features
//! 
//! - **Safety First**: Content filtering before model inference
//! - **JSON Mode Only**: Enforced structured output
//! - **Timeout Handling**: 30 second limit per image
//! - **Memory Monitoring**: 4GB RAM cap
//! - **Error Recovery**: Quarantine failed compilations

use std::path::PathBuf;
use serde::{Deserialize, Serialize};

pub mod compiler;
pub mod prompt;
pub mod parser;
pub mod safety;

pub use compiler::{AiPipeline, CompilationOptions, CompilationResult};
pub use prompt::{PromptTemplate, PromptManager, PromptVersion};
pub use parser::{JsonParser, ValidationResult};
pub use safety::{SafetyClassifier, ContentCategory, SafetyResult};

/// Status of a compilation job
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum CompilationStatus {
    /// Waiting to be processed
    Pending,
    /// Currently being processed
    Processing,
    /// Successfully completed
    Completed,
    /// Failed with error
    Failed,
    /// Quarantined due to safety concern
    Quarantined,
    /// Timed out
    Timeout,
}

/// Priority levels for compilation queue (SPEC v2.0 Section 2.2.1)
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum CompilationPriority {
    /// P0: User-initiated "Compile Now"
    Immediate = 0,
    /// P1: Screenshots with messaging context
    High = 1,
    /// P2: Background batch (charging + WiFi only)
    Normal = 2,
    /// P3: Low priority batch
    Low = 3,
}

impl Default for CompilationPriority {
    fn default() -> Self {
        CompilationPriority::Normal
    }
}

/// Compilation job for the queue
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompilationJob {
    /// Unique job ID
    pub id: uuid::Uuid,
    /// Path to the screenshot image
    pub image_path: PathBuf,
    /// Compilation priority
    pub priority: CompilationPriority,
    /// Current status
    pub status: CompilationStatus,
    /// Number of retry attempts
    pub retry_count: u32,
    /// Creation timestamp
    pub created_at: chrono::DateTime<chrono::Utc>,
    /// App context (if available)
    pub app_context: Option<AppContext>,
}

/// Application context from screenshot capture
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppContext {
    /// Bundle ID of the source app
    pub bundle_id: String,
    /// App name
    pub app_name: String,
    /// Window title (if available)
    pub window_title: Option<String>,
    /// Screenshot timestamp
    pub captured_at: chrono::DateTime<chrono::Utc>,
}

/// Statistics for the compilation pipeline
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct PipelineStats {
    /// Total compilations attempted
    pub total_attempted: u64,
    /// Successful compilations
    pub successful: u64,
    /// Failed compilations
    pub failed: u64,
    /// Quarantined compilations
    pub quarantined: u64,
    /// Timed out compilations
    pub timed_out: u64,
    /// Average compilation time in milliseconds
    pub avg_time_ms: u64,
    /// Peak memory usage in bytes
    pub peak_memory_bytes: usize,
}

impl PipelineStats {
    /// Calculate success rate as percentage
    pub fn success_rate(&self) -> f64 {
        if self.total_attempted == 0 {
            return 100.0;
        }
        
        (self.successful as f64 / self.total_attempted as f64) * 100.0
    }
    
    /// Update average time with a new sample
    pub fn update_avg_time(&mut self, new_time_ms: u64) {
        let total_time = self.avg_time_ms * self.total_attempted + new_time_ms;
        self.total_attempted += 1;
        self.avg_time_ms = total_time / self.total_attempted;
    }
}

/// Configuration for compilation queue
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QueueConfig {
    /// Maximum queue size
    pub max_size: usize,
    /// Maximum retry attempts
    pub max_retries: u32,
    /// Retry delay in seconds
    pub retry_delay_secs: u64,
    /// Batch size for background processing
    pub batch_size: usize,
    /// Enable automatic retry
    pub auto_retry: bool,
}

impl Default for QueueConfig {
    fn default() -> Self {
        Self {
            max_size: 1000,
            max_retries: 3,
            retry_delay_secs: 60,
            batch_size: 10,
            auto_retry: true,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_compilation_priority_ordering() {
        assert!(CompilationPriority::Immediate < CompilationPriority::High);
        assert!(CompilationPriority::High < CompilationPriority::Normal);
        assert!(CompilationPriority::Normal < CompilationPriority::Low);
    }
    
    #[test]
    fn test_pipeline_stats() {
        let mut stats = PipelineStats::default();
        
        stats.total_attempted = 100;
        stats.successful = 95;
        stats.failed = 5;
        
        assert_eq!(stats.success_rate(), 95.0);
        
        stats.update_avg_time(2000);
        assert_eq!(stats.avg_time_ms, 2000);
        
        stats.update_avg_time(3000);
        assert_eq!(stats.avg_time_ms, 2500);
    }
}
