//! Entity Confidence Scoring System
//!
//! Implements Bayesian confidence scoring for knowledge graph entities.
//! Provides:
//! - Confidence calculation with source reliability weighting
//! - Bayesian updating based on new evidence
//! - Conflict resolution for contradictory evidence
//! - Uncertainty quantification
//!
//! # Example Usage
//!
//! ```rust,no_run
//! use truffle_core::validation::confidence::{
//!     ConfidenceScorer, Evidence, EvidenceType, SourceReliability, SourceType
//! };
//! use uuid::Uuid;
//!
//! // Create scorer
//! let scorer = ConfidenceScorer::new();
//!
//! // Calculate entity confidence
//! let score = scorer.calculate_entity_confidence(&entity);
//! println!("Confidence: {:.2}", score.overall);
//!
//! // Update with new evidence
//! let evidence = Evidence::confirmatory(0.9, 0.7);
//! let result = scorer.update_with_evidence(0.7, &evidence);
//! println!("Updated: {:.2} -> {:.2}", result.prior, result.posterior);
//! ```

pub mod bayesian;
pub mod scorer;
pub mod source_reliability;

// Re-export main types
pub use bayesian::{
    BayesianUpdateResult, BayesianUpdater, ConflictResolutionResult,
    ConflictResolutionStrategy, Evidence, EvidenceType,
};
pub use scorer::{
    AttributeConfidence, ConfidenceFactor, ConfidenceRecommendation,
    ConfidenceScore, ConfidenceScorer, ConfidenceStatistics, ConfidenceTier,
    FactorCategory, ScorerConfig,
};
pub use source_reliability::{
    SourceRegistry, SourceReliability, SourceType,
};

use crate::Result;

/// Confidence validation error types
#[derive(Debug, thiserror::Error)]
pub enum ConfidenceError {
    /// Invalid confidence value (not in [0, 1] range)
    #[error("Invalid confidence value: {0} (must be in [0, 1])")]
    InvalidConfidence(f32),
    
    /// Invalid weight value
    #[error("Invalid weight value: {0} (must be in [0, 1])")]
    InvalidWeight(f32),
    
    /// Source not found
    #[error("Source not found: {0}")]
    SourceNotFound(uuid::Uuid),
    
    /// Calculation error
    #[error("Calculation error: {0}")]
    CalculationError(String),
}

/// Initialize the confidence module
pub fn init() {
    // Module initialization if needed
}

/// Validate a confidence value is in valid range [0, 1]
pub fn validate_confidence(value: f32) -> Result<f32, ConfidenceError> {
    if value.is_finite() && value >= 0.0 && value <= 1.0 {
        Ok(value)
    } else {
        Err(ConfidenceError::InvalidConfidence(value))
    }
}

/// Validate a weight value is in valid range [0, 1]
pub fn validate_weight(value: f32) -> Result<f32, ConfidenceError> {
    if value.is_finite() && value >= 0.0 && value <= 1.0 {
        Ok(value)
    } else {
        Err(ConfidenceError::InvalidWeight(value))
    }
}

/// Convenience function to calculate entity confidence
pub fn calculate_confidence(entity: &crate::models::Entity) -> ConfidenceScore {
    let scorer = ConfidenceScorer::new();
    scorer.calculate_entity_confidence(entity)
}

/// Confidence module version
pub const VERSION: &str = "1.0.0";

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validate_confidence() {
        assert!(validate_confidence(0.5).is_ok());
        assert!(validate_confidence(0.0).is_ok());
        assert!(validate_confidence(1.0).is_ok());
        assert!(validate_confidence(-0.1).is_err());
        assert!(validate_confidence(1.1).is_err());
        assert!(validate_confidence(f32::NAN).is_err());
    }

    #[test]
    fn test_validate_weight() {
        assert!(validate_weight(0.5).is_ok());
        assert!(validate_weight(0.0).is_ok());
        assert!(validate_weight(1.0).is_ok());
        assert!(validate_weight(1.5).is_err());
    }

    #[test]
    fn test_module_version() {
        assert!(!VERSION.is_empty());
    }
}
