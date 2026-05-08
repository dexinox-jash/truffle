//! Error types

use thiserror::Error;

#[derive(Error, Debug)]
pub enum IntelligenceError {
    #[error("Model error: {0}")]
    ModelError(String),
    
    #[error("API error: {0}")]
    ApiError(#[from] reqwest::Error),
    
    #[error("Serialization error: {0}")]
    SerializationError(#[from] serde_json::Error),
    
    #[error("Embedding error: {0}")]
    EmbeddingError(String),
    
    #[error("Transcription error: {0}")]
    TranscriptionError(String),
}
