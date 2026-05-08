//! Advanced AI Integration Module

pub mod custom_model;
pub mod suggestions;

pub use custom_model::{AiModelManager, ModelConfig};
pub use suggestions::{SuggestionEngine, AiSuggestion};
