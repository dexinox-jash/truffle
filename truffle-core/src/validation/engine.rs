//! Validation Engine
//!
//! Main orchestrator for the validation system. Coordinates contradiction checking,
//! confidence scoring, and redundancy detection.

use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{info, instrument};
use uuid::Uuid;

use crate::graph::GraphRepository;
use crate::validation::confidence::{ConfidenceScore, ConfidenceScorer};
use crate::validation::contradiction_checker::{ContradictionChecker, ValidationReport};
use crate::validation::models::{ContradictionResolution, DuplicateCandidate, Fact};
use crate::validation::redundancy::{DuplicateDetector, MergeEngine};
use crate::Result;

use super::config::ValidationConfig;

/// Main validation engine that orchestrates all validators
pub struct ValidationEngine {
    config: ValidationConfig,
    repository: Arc<RwLock<GraphRepository>>,
    contradiction_checker: ContradictionChecker,
    confidence_scorer: ConfidenceScorer,
}

impl ValidationEngine {
    /// Create a new validation engine
    pub fn new(config: ValidationConfig, repository: Arc<RwLock<GraphRepository>>) -> Self {
        Self {
            config: config.clone(),
            repository,
            contradiction_checker: ContradictionChecker::new(),
            confidence_scorer: ConfidenceScorer::new(config.confidence),
        }
    }

    /// Run full validation suite on all entities
    #[instrument(skip(self), level = "info")]
    pub async fn validate_all(&self) -> Result<EngineValidationSummary> {
        info!("Starting full validation suite");

        let mut summary = EngineValidationSummary::new();

        // Run contradiction detection
        if self.config.contradiction.enabled {
            let report = self.check_contradictions().await?;
            summary.contradictions = report.total_contradictions();
            summary.critical_contradictions = report.critical_count();
        }

        // Run confidence scoring
        if self.config.confidence.enabled {
            let scores = self.score_confidence().await?;
            summary.entities_scored = scores.len();
            summary.avg_confidence = self.calculate_avg_confidence(&scores);
        }

        // Run redundancy detection
        if self.config.redundancy.enabled {
            let duplicates = self.find_duplicates().await?;
            summary.duplicate_candidates = duplicates.len();
        }

        info!("Validation complete: {:?}", summary);
        Ok(summary)
    }

    /// Validate a specific entity
    #[instrument(skip(self, facts), level = "debug")]
    pub async fn validate_entity(
        &self,
        entity_id: Uuid,
        facts: &[Fact],
    ) -> Result<EntityValidationResult> {
        let mut result = EntityValidationResult::new(entity_id);

        // Check for contradictions
        if self.config.contradiction.enabled {
            result.contradictions = self.contradiction_checker.check_entity_facts(entity_id, facts);
        }

        // Calculate confidence
        if self.config.confidence.enabled {
            result.confidence = Some(self.confidence_scorer.calculate(facts));
        }

        Ok(result)
    }

    /// Check for contradictions across all entities
    pub async fn check_contradictions(&self) -> Result<ValidationReport> {
        // This would iterate through all entities and their facts
        // For now, return an empty report
        Ok(ValidationReport::new(0, 0))
    }

    /// Score confidence for all entities
    pub async fn score_confidence(&self) -> Result<Vec<ConfidenceScore>> {
        // This would iterate through all entities
        // For now, return empty
        Ok(Vec::new())
    }

    /// Find duplicate entities
    pub async fn find_duplicates(&self) -> Result<Vec<DuplicateCandidate>> {
        // This would use the duplicate detector
        // For now, return empty
        Ok(Vec::new())
    }

    /// Resolve a contradiction
    pub async fn resolve_contradiction(
        &self,
        resolution: &ContradictionResolution,
    ) -> Result<()> {
        info!("Resolving contradiction with {:?}", resolution.resolution_type);
        // Implementation would update facts based on resolution
        Ok(())
    }

    /// Calculate average confidence
    fn calculate_avg_confidence(&self, scores: &[ConfidenceScore]) -> f32 {
        if scores.is_empty() {
            return 0.0;
        }
        let sum: f32 = scores.iter().map(|s| s.overall).sum();
        sum / scores.len() as f32
    }
}

/// Summary of a validation run from the engine
#[derive(Debug, Clone)]
pub struct EngineValidationSummary {
    /// Number of entities checked
    pub entities_checked: usize,
    /// Total contradictions found
    pub contradictions: usize,
    /// Critical contradictions requiring immediate attention
    pub critical_contradictions: usize,
    /// Number of entities with confidence scores
    pub entities_scored: usize,
    /// Average confidence across all entities
    pub avg_confidence: f32,
    /// Number of duplicate candidates found
    pub duplicate_candidates: usize,
    /// Validation completed successfully
    pub success: bool,
    /// Error message if validation failed
    pub error: Option<String>,
}

impl EngineValidationSummary {
    /// Create a new empty summary
    pub fn new() -> Self {
        Self {
            entities_checked: 0,
            contradictions: 0,
            critical_contradictions: 0,
            entities_scored: 0,
            avg_confidence: 0.0,
            duplicate_candidates: 0,
            success: true,
            error: None,
        }
    }
    
    /// Create with error
    pub fn with_error(error: impl Into<String>) -> Self {
        Self {
            entities_checked: 0,
            contradictions: 0,
            critical_contradictions: 0,
            entities_scored: 0,
            avg_confidence: 0.0,
            duplicate_candidates: 0,
            success: false,
            error: Some(error.into()),
        }
    }

    /// Check if validation found any issues
    pub fn has_issues(&self) -> bool {
        self.contradictions > 0 || self.duplicate_candidates > 0 || !self.success
    }

    /// Get a human-readable status
    pub fn status_text(&self) -> &'static str {
        if !self.success {
            "FAILED"
        } else if self.critical_contradictions > 0 {
            "CRITICAL"
        } else if self.has_issues() {
            "WARNINGS"
        } else {
            "OK"
        }
    }
}

impl Default for EngineValidationSummary {
    fn default() -> Self {
        Self::new()
    }
}

/// Validation result for a single entity
#[derive(Debug, Clone)]
pub struct EntityValidationResult {
    /// Entity ID
    pub entity_id: Uuid,
    /// Contradictions found for this entity
    pub contradictions: Vec<crate::validation::models::Contradiction>,
    /// Confidence score
    pub confidence: Option<ConfidenceScore>,
    /// Whether entity has any issues
    pub has_issues: bool,
}

impl EntityValidationResult {
    /// Create a new entity validation result
    pub fn new(entity_id: Uuid) -> Self {
        Self {
            entity_id,
            contradictions: Vec::new(),
            confidence: None,
            has_issues: false,
        }
    }

    /// Update has_issues flag based on findings
    pub fn update_status(&mut self) {
        self.has_issues = !self.contradictions.is_empty();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_engine_validation_summary() {
        let mut summary = EngineValidationSummary::new();
        assert!(!summary.has_issues());
        assert_eq!(summary.status_text(), "OK");

        summary.contradictions = 5;
        assert!(summary.has_issues());
        assert_eq!(summary.status_text(), "WARNINGS");

        summary.critical_contradictions = 1;
        assert_eq!(summary.status_text(), "CRITICAL");
    }

    #[test]
    fn test_entity_validation_result() {
        let entity_id = Uuid::new_v4();
        let mut result = EntityValidationResult::new(entity_id);

        assert!(!result.has_issues);
        assert_eq!(result.entity_id, entity_id);

        // Add a contradiction
        result.contradictions.push(crate::validation::models::Contradiction::new(
            entity_id,
            Uuid::new_v4(),
            Uuid::new_v4(),
            crate::validation::models::ContradictionType::Factual,
            crate::validation::models::ContradictionSeverity::High,
        ));
        result.update_status();

        assert!(result.has_issues);
    }
}
