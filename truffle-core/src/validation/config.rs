//! Validation Configuration
//!
//! Configuration structures for the validation engine.

use serde::{Deserialize, Serialize};

/// Configuration for the validation engine
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationConfig {
    /// Contradiction detection configuration
    pub contradiction: ContradictionConfig,
    /// Confidence scoring configuration
    pub confidence: ConfidenceConfig,
    /// Redundancy detection configuration
    pub redundancy: RedundancyConfig,
}

impl ValidationConfig {
    /// Create default configuration
    pub fn new() -> Self {
        Self::default()
    }

    /// Enable all validation features with strict settings
    pub fn strict() -> Self {
        Self {
            contradiction: ContradictionConfig {
                enabled: true,
                check_temporal: true,
                check_factual: true,
                min_confidence_threshold: 0.5,
            },
            confidence: ConfidenceConfig {
                enabled: true,
                use_bayesian_updating: true,
                min_sources: 1,
                decay_days: 90,
            },
            redundancy: RedundancyConfig {
                enabled: true,
                similarity_threshold: 0.85,
                check_names: true,
                check_embeddings: true,
                check_relationships: true,
            },
        }
    }
}

impl Default for ValidationConfig {
    fn default() -> Self {
        Self {
            contradiction: ContradictionConfig::default(),
            confidence: ConfidenceConfig::default(),
            redundancy: RedundancyConfig::default(),
        }
    }
}

/// Configuration for contradiction detection
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContradictionConfig {
    /// Enable contradiction detection
    pub enabled: bool,
    /// Check for temporal contradictions
    pub check_temporal: bool,
    /// Check for factual contradictions
    pub check_factual: bool,
    /// Minimum confidence to consider a fact
    pub min_confidence_threshold: f32,
}

impl Default for ContradictionConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            check_temporal: true,
            check_factual: true,
            min_confidence_threshold: 0.3,
        }
    }
}

/// Configuration for confidence scoring
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConfidenceConfig {
    /// Enable confidence scoring
    pub enabled: bool,
    /// Use Bayesian updating
    pub use_bayesian_updating: bool,
    /// Minimum sources required for high confidence
    pub min_sources: usize,
    /// Days before confidence decays
    pub decay_days: u64,
}

impl Default for ConfidenceConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            use_bayesian_updating: true,
            min_sources: 1,
            decay_days: 180,
        }
    }
}

/// Configuration for redundancy detection
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RedundancyConfig {
    /// Enable redundancy detection
    pub enabled: bool,
    /// Similarity threshold for duplicates (0.0 - 1.0)
    pub similarity_threshold: f32,
    /// Include name similarity in calculation
    pub check_names: bool,
    /// Include embedding similarity in calculation
    pub check_embeddings: bool,
    /// Include relationship overlap in calculation
    pub check_relationships: bool,
}

impl Default for RedundancyConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            similarity_threshold: 0.75,
            check_names: true,
            check_embeddings: true,
            check_relationships: true,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let config = ValidationConfig::default();
        assert!(config.contradiction.enabled);
        assert!(config.confidence.enabled);
        assert!(config.redundancy.enabled);
    }

    #[test]
    fn test_strict_config() {
        let config = ValidationConfig::strict();
        assert!(config.contradiction.enabled);
        assert!(config.redundancy.check_embeddings);
        assert_eq!(config.redundancy.similarity_threshold, 0.85);
    }
}
