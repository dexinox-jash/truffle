//! Confidence Scorer
//!
//! Main scoring engine for calculating entity confidence scores.
//! Aggregates confidence across all attributes with source reliability
//! and recency weighting.

use chrono::{DateTime, Duration, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;

use super::bayesian::{BayesianUpdateResult, BayesianUpdater, Evidence, EvidenceType};
use super::source_reliability::{SourceReliability, SourceRegistry, SourceType};
use crate::models::Entity;

/// A confidence factor contributing to overall entity confidence
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ConfidenceFactor {
    /// Factor identifier
    pub name: String,
    /// Factor category
    pub category: FactorCategory,
    /// Confidence value (0.0 - 1.0)
    pub value: f32,
    /// Weight in aggregation (0.0 - 1.0)
    pub weight: f32,
    /// Source of this factor
    pub source_id: Option<Uuid>,
    /// Timestamp when factor was recorded
    pub recorded_at: DateTime<Utc>,
    /// Optional description
    pub description: Option<String>,
}

impl ConfidenceFactor {
    /// Create a new confidence factor
    pub fn new(
        name: impl Into<String>,
        category: FactorCategory,
        value: f32,
        weight: f32,
    ) -> Self {
        Self {
            name: name.into(),
            category,
            value: value.clamp(0.0, 1.0),
            weight: weight.clamp(0.0, 1.0),
            source_id: None,
            recorded_at: Utc::now(),
            description: None,
        }
    }

    /// Set source
    pub fn with_source(mut self, source_id: Uuid) -> Self {
        self.source_id = Some(source_id);
        self
    }

    /// Set description
    pub fn with_description(mut self, desc: impl Into<String>) -> Self {
        self.description = Some(desc.into());
        self
    }

    /// Calculate weighted contribution
    pub fn weighted_contribution(&self) -> f32 {
        self.value * self.weight
    }
}

/// Categories of confidence factors
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Hash)]
#[serde(rename_all = "snake_case")]
pub enum FactorCategory {
    /// Entity extraction confidence
    Extraction,
    /// Source reliability
    SourceReliability,
    /// Human verification
    Verification,
    /// Cross-reference with other entities
    CrossReference,
    /// Temporal freshness
    Recency,
    /// Graph connectivity
    Connectivity,
    /// Consistency with schema
    SchemaConsistency,
    /// Semantic similarity
    SemanticMatch,
}

impl FactorCategory {
    /// Get default weight for this category
    pub fn default_weight(&self) -> f32 {
        match self {
            Self::Extraction => 0.25,
            Self::SourceReliability => 0.20,
            Self::Verification => 0.30,
            Self::CrossReference => 0.10,
            Self::Recency => 0.05,
            Self::Connectivity => 0.05,
            Self::SchemaConsistency => 0.03,
            Self::SemanticMatch => 0.02,
        }
    }

    /// Get display name
    pub fn display_name(&self) -> &'static str {
        match self {
            Self::Extraction => "Extraction Confidence",
            Self::SourceReliability => "Source Reliability",
            Self::Verification => "Human Verification",
            Self::CrossReference => "Cross Reference",
            Self::Recency => "Data Recency",
            Self::Connectivity => "Graph Connectivity",
            Self::SchemaConsistency => "Schema Consistency",
            Self::SemanticMatch => "Semantic Match",
        }
    }
}

/// Structured confidence score result
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ConfidenceScore {
    /// Overall aggregated confidence (0.0 - 1.0)
    pub overall: f32,
    /// Confidence by attribute
    pub by_attribute: HashMap<String, f32>,
    /// Entropy-based uncertainty measure (0.0 = certain, 1.0 = uncertain)
    pub uncertainty: f32,
    /// Contributing confidence factors
    pub factors: Vec<ConfidenceFactor>,
    /// Timestamp of calculation
    pub calculated_at: DateTime<Utc>,
    /// Number of evidence items considered
    pub evidence_count: usize,
    /// Confidence tier
    pub tier: ConfidenceTier,
    /// Recommended action
    pub recommendation: ConfidenceRecommendation,
}

impl ConfidenceScore {
    /// Create a new confidence score
    pub fn new(overall: f32) -> Self {
        Self {
            overall: overall.clamp(0.0, 1.0),
            by_attribute: HashMap::new(),
            uncertainty: 0.0,
            factors: Vec::new(),
            calculated_at: Utc::now(),
            evidence_count: 0,
            tier: ConfidenceTier::from_score(overall),
            recommendation: ConfidenceRecommendation::from_score(overall),
        }
    }

    /// Add attribute confidence
    pub fn with_attribute(mut self, name: impl Into<String>, confidence: f32) -> Self {
        self.by_attribute.insert(name.into(), confidence.clamp(0.0, 1.0));
        self
    }

    /// Add confidence factor
    pub fn with_factor(mut self, factor: ConfidenceFactor) -> Self {
        self.factors.push(factor);
        self
    }

    /// Set uncertainty
    pub fn with_uncertainty(mut self, uncertainty: f32) -> Self {
        self.uncertainty = uncertainty.clamp(0.0, 1.0);
        self
    }

    /// Set evidence count
    pub fn with_evidence_count(mut self, count: usize) -> Self {
        self.evidence_count = count;
        self
    }

    /// Calculate entropy-based uncertainty from factors
    /// Higher entropy (more disagreement) = higher uncertainty
    pub fn calculate_uncertainty(factors: &[ConfidenceFactor]) -> f32 {
        if factors.is_empty() {
            return 1.0; // Maximum uncertainty with no factors
        }

        // Calculate weighted variance as uncertainty measure
        let weighted_values: Vec<f32> = factors
            .iter()
            .map(|f| f.value * f.weight)
            .collect();
        
        let total_weight: f32 = factors.iter().map(|f| f.weight).sum();
        
        if total_weight == 0.0 {
            return 1.0;
        }

        let mean: f32 = weighted_values.iter().sum::<f32>() / total_weight;
        
        let variance: f32 = factors
            .iter()
            .map(|f| f.weight * (f.value - mean).powi(2))
            .sum::<f32>() / total_weight;

        // Normalize to 0-1 range (max variance is 0.25 at p=0.5)
        (variance * 4.0).min(1.0)
    }

    /// Get the minimum attribute confidence
    pub fn min_attribute_confidence(&self) -> Option<f32> {
        self.by_attribute.values().copied().reduce(f32::min)
    }

    /// Get the maximum attribute confidence
    pub fn max_attribute_confidence(&self) -> Option<f32> {
        self.by_attribute.values().copied().reduce(f32::max)
    }

    /// Check if entity meets confidence threshold
    pub fn meets_threshold(&self, threshold: f32) -> bool {
        self.overall >= threshold
    }

    /// Check if score is considered high confidence
    pub fn is_high_confidence(&self) -> bool {
        self.tier == ConfidenceTier::High || self.tier == ConfidenceTier::VeryHigh
    }

    /// Check if manual review is recommended
    pub fn needs_review(&self) -> bool {
        matches!(
            self.recommendation,
            ConfidenceRecommendation::ManualReview | ConfidenceRecommendation::Reject
        )
    }
}

/// Confidence quality tiers
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum ConfidenceTier {
    /// Very high confidence (>= 0.9)
    VeryHigh,
    /// High confidence (0.8 - 0.9)
    High,
    /// Medium confidence (0.6 - 0.8)
    Medium,
    /// Low confidence (0.4 - 0.6)
    Low,
    /// Very low confidence (< 0.4)
    VeryLow,
}

impl ConfidenceTier {
    /// Determine tier from score
    pub fn from_score(score: f32) -> Self {
        match score {
            s if s >= 0.9 => Self::VeryHigh,
            s if s >= 0.8 => Self::High,
            s if s >= 0.6 => Self::Medium,
            s if s >= 0.4 => Self::Low,
            _ => Self::VeryLow,
        }
    }

    /// Get minimum score for this tier
    pub fn min_score(&self) -> f32 {
        match self {
            Self::VeryHigh => 0.9,
            Self::High => 0.8,
            Self::Medium => 0.6,
            Self::Low => 0.4,
            Self::VeryLow => 0.0,
        }
    }

    /// Get display name
    pub fn display_name(&self) -> &'static str {
        match self {
            Self::VeryHigh => "Very High",
            Self::High => "High",
            Self::Medium => "Medium",
            Self::Low => "Low",
            Self::VeryLow => "Very Low",
        }
    }
}

/// Recommendations based on confidence score
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum ConfidenceRecommendation {
    /// Accept without review
    Accept,
    /// Accept but monitor
    AcceptWithMonitoring,
    /// Manual review recommended
    ManualReview,
    /// Reject entity
    Reject,
}

impl ConfidenceRecommendation {
    /// Determine recommendation from score
    pub fn from_score(score: f32) -> Self {
        match score {
            s if s >= 0.85 => Self::Accept,
            s if s >= 0.7 => Self::AcceptWithMonitoring,
            s if s >= 0.5 => Self::ManualReview,
            _ => Self::Reject,
        }
    }

    /// Get display message
    pub fn message(&self) -> &'static str {
        match self {
            Self::Accept => "Entity is reliable and can be used without review",
            Self::AcceptWithMonitoring => "Entity is reliable but should be monitored",
            Self::ManualReview => "Entity should be manually reviewed before use",
            Self::Reject => "Entity is not reliable enough for use",
        }
    }
}

/// Attribute value with confidence metadata
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct AttributeConfidence {
    /// Attribute name
    pub name: String,
    /// Attribute value
    pub value: serde_json::Value,
    /// Confidence score for this attribute
    pub confidence: f32,
    /// Source of the attribute
    pub source_id: Option<Uuid>,
    /// When attribute was extracted/updated
    pub timestamp: DateTime<Utc>,
}

/// Configuration for confidence scoring
#[derive(Debug, Clone)]
pub struct ScorerConfig {
    /// Default confidence for unknown sources
    pub default_source_confidence: f32,
    /// Recency decay half-life (in days)
    pub recency_half_life: f64,
    /// Minimum samples for reliable source assessment
    pub min_source_samples: usize,
    /// Verification boost factor
    pub verification_boost: f32,
    /// Uncertainty penalty factor
    pub uncertainty_penalty: f32,
    /// Enable Bayesian updating
    pub use_bayesian: bool,
    /// Minimum confidence threshold
    pub min_confidence_threshold: f32,
}

impl Default for ScorerConfig {
    fn default() -> Self {
        Self {
            default_source_confidence: 0.5,
            recency_half_life: 7.0, // 1 week
            min_source_samples: 10,
            verification_boost: 0.15,
            uncertainty_penalty: 0.1,
            use_bayesian: true,
            min_confidence_threshold: 0.6,
        }
    }
}

/// Confidence scoring engine
pub struct ConfidenceScorer {
    config: ScorerConfig,
    source_registry: SourceRegistry,
    bayesian_updater: BayesianUpdater,
}

impl ConfidenceScorer {
    /// Create a new confidence scorer with default configuration
    pub fn new() -> Self {
        Self {
            config: ScorerConfig::default(),
            source_registry: SourceRegistry::new(),
            bayesian_updater: BayesianUpdater::new(),
        }
    }

    /// Create with custom configuration
    pub fn with_config(config: ScorerConfig) -> Self {
        Self {
            config,
            source_registry: SourceRegistry::new(),
            bayesian_updater: BayesianUpdater::new(),
        }
    }

    /// Register a source in the registry
    pub fn register_source(&mut self, source: SourceReliability) {
        self.source_registry.register(source);
    }

    /// Get mutable reference to source registry
    pub fn source_registry_mut(&mut self) -> &mut SourceRegistry {
        &mut self.source_registry
    }

    /// Calculate confidence for an entity
    /// 
    /// This aggregates confidence across all entity attributes,
    /// weighting by source reliability and recency.
    pub fn calculate_entity_confidence(&self, entity: &Entity) -> ConfidenceScore {
        let mut factors = Vec::new();
        let mut by_attribute = HashMap::new();

        // 1. Base extraction confidence
        let extraction_factor = ConfidenceFactor::new(
            "extraction_confidence",
            FactorCategory::Extraction,
            entity.extraction_confidence,
            FactorCategory::Extraction.default_weight(),
        );
        factors.push(extraction_factor.clone());
        by_attribute.insert("entity".to_string(), entity.extraction_confidence);

        // 2. Verification status
        if entity.verified {
            let verification_factor = ConfidenceFactor::new(
                "human_verification",
                FactorCategory::Verification,
                1.0,
                FactorCategory::Verification.default_weight(),
            )
            .with_description("Entity has been manually verified");
            factors.push(verification_factor);
        }

        // 3. Recency factor
        let recency_factor = self.calculate_recency_factor(&entity.last_updated_at);
        factors.push(ConfidenceFactor::new(
            "data_recency",
            FactorCategory::Recency,
            recency_factor,
            FactorCategory::Recency.default_weight(),
        ));

        // 4. Source reliability (if source is known)
        if let Some(source_id) = entity.source_meeting_id {
            let source_confidence = self
                .source_registry
                .get(&source_id)
                .map(|s| s.reliability_score)
                .unwrap_or(self.config.default_source_confidence);
            
            factors.push(
                ConfidenceFactor::new(
                    "source_reliability",
                    FactorCategory::SourceReliability,
                    source_confidence,
                    FactorCategory::SourceReliability.default_weight(),
                )
                .with_source(source_id),
            );
        }

        // Calculate weighted overall confidence
        let overall = self.aggregate_confidence(&factors);

        // Calculate uncertainty
        let uncertainty = ConfidenceScore::calculate_uncertainty(&factors);

        // Apply uncertainty penalty
        let penalized_overall = overall * (1.0 - uncertainty * self.config.uncertainty_penalty);

        // Build final score
        ConfidenceScore::new(penalized_overall)
            .with_uncertainty(uncertainty)
            .with_evidence_count(factors.len())
    }

    /// Calculate entity confidence with attribute-level detail
    pub fn calculate_entity_confidence_detailed(
        &self,
        entity: &Entity,
        attributes: &[AttributeConfidence],
    ) -> ConfidenceScore {
        let mut score = self.calculate_entity_confidence(entity);

        // Add attribute-level confidences
        for attr in attributes {
            score.by_attribute.insert(attr.name.clone(), attr.confidence);
        }

        // Recalculate with attributes included
        for (name, conf) in &score.by_attribute {
            if name != "entity" {
                let factor = ConfidenceFactor::new(
                    format!("attr_{}", name),
                    FactorCategory::SchemaConsistency,
                    *conf,
                    0.05, // Lower weight for individual attributes
                );
                score.factors.push(factor);
            }
        }

        // Re-aggregate with all factors
        score.overall = self.aggregate_confidence(&score.factors);
        score.uncertainty = ConfidenceScore::calculate_uncertainty(&score.factors);
        score.evidence_count = score.factors.len();
        score.tier = ConfidenceTier::from_score(score.overall);
        score.recommendation = ConfidenceRecommendation::from_score(score.overall);

        score
    }

    /// Calculate confidence using Bayesian updating
    pub fn calculate_bayesian_confidence(
        &self,
        entity: &Entity,
        evidence_items: &[Evidence],
    ) -> (ConfidenceScore, Vec<BayesianUpdateResult>) {
        let prior = if let Some(source_id) = entity.source_meeting_id {
            self.source_registry
                .get(&source_id)
                .map(|s| s.reliability_score)
                .unwrap_or(0.5)
        } else {
            0.5
        };

        let (posterior, results) = self.bayesian_updater.update_multiple(prior, evidence_items);

        let mut score = ConfidenceScore::new(posterior)
            .with_evidence_count(evidence_items.len());

        // Add factors from evidence
        for result in &results {
            let factor = ConfidenceFactor::new(
                format!("evidence_{}", result.evidence.id),
                match result.evidence.evidence_type {
                    EvidenceType::Confirmatory => FactorCategory::CrossReference,
                    EvidenceType::Verification => FactorCategory::Verification,
                    _ => FactorCategory::CrossReference,
                },
                result.posterior,
                result.evidence.weight * result.evidence.diagnostic_value(),
            );
            score.factors.push(factor);
        }

        score.uncertainty = ConfidenceScore::calculate_uncertainty(&score.factors);

        (score, results)
    }

    /// Aggregate confidence using weighted average
    /// 
    /// Formula: overall = Σ(confidence_i * weight_i) / Σ(weight_i)
    pub fn aggregate_confidence(&self, factors: &[ConfidenceFactor]) -> f32 {
        if factors.is_empty() {
            return 0.5; // Neutral when no factors
        }

        let weighted_sum: f32 = factors
            .iter()
            .map(|f| f.value * f.weight)
            .sum();
        
        let total_weight: f32 = factors.iter().map(|f| f.weight).sum();

        if total_weight == 0.0 {
            0.5
        } else {
            (weighted_sum / total_weight).clamp(0.0, 1.0)
        }
    }

    /// Calculate recency factor (decays with age)
    fn calculate_recency_factor(&self, last_updated: &DateTime<Utc>) -> f32 {
        let age = (Utc::now() - *last_updated).num_days() as f64;
        let decay = (-age / self.config.recency_half_life).exp();
        decay as f32
    }

    /// Update entity confidence with new evidence
    pub fn update_with_evidence(
        &self,
        current_confidence: f32,
        evidence: &Evidence,
    ) -> BayesianUpdateResult {
        self.bayesian_updater.update(current_confidence, evidence)
    }

    /// Batch calculate confidence for multiple entities
    pub fn batch_calculate(&self, entities: &[Entity]) -> HashMap<Uuid, ConfidenceScore> {
        entities
            .iter()
            .map(|e| (e.id, self.calculate_entity_confidence(e)))
            .collect()
    }

    /// Filter entities by confidence threshold
    pub fn filter_by_confidence<'a>(
        &self,
        entities: &'a [Entity],
        threshold: f32,
    ) -> Vec<&'a Entity> {
        entities
            .iter()
            .filter(|e| {
                let score = self.calculate_entity_confidence(e);
                score.meets_threshold(threshold)
            })
            .collect()
    }

    /// Get confidence statistics for a set of entities
    pub fn confidence_statistics(&self, entities: &[Entity]) -> ConfidenceStatistics {
        if entities.is_empty() {
            return ConfidenceStatistics::default();
        }

        let scores: Vec<f32> = entities
            .iter()
            .map(|e| self.calculate_entity_confidence(e).overall)
            .collect();

        let sum: f32 = scores.iter().sum();
        let mean = sum / scores.len() as f32;

        let sorted_scores = {
            let mut s = scores.clone();
            s.sort_by(|a, b| a.partial_cmp(b).unwrap());
            s
        };

        let median = if scores.len() % 2 == 0 {
            let mid = scores.len() / 2;
            (sorted_scores[mid - 1] + sorted_scores[mid]) / 2.0
        } else {
            sorted_scores[scores.len() / 2]
        };

        let variance: f32 = scores
            .iter()
            .map(|s| (s - mean).powi(2))
            .sum::<f32>() / scores.len() as f32;

        ConfidenceStatistics {
            count: entities.len(),
            mean,
            median,
            std_dev: variance.sqrt(),
            min: *sorted_scores.first().unwrap(),
            max: *sorted_scores.last().unwrap(),
            high_confidence_count: scores.iter().filter(|&&s| s >= 0.8).count(),
            low_confidence_count: scores.iter().filter(|&&s| s < 0.5).count(),
        }
    }
}

impl Default for ConfidenceScorer {
    fn default() -> Self {
        Self::new()
    }
}

/// Statistics for a set of confidence scores
#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq)]
pub struct ConfidenceStatistics {
    /// Number of entities
    pub count: usize,
    /// Mean confidence
    pub mean: f32,
    /// Median confidence
    pub median: f32,
    /// Standard deviation
    pub std_dev: f32,
    /// Minimum confidence
    pub min: f32,
    /// Maximum confidence
    pub max: f32,
    /// Count of high confidence entities (>= 0.8)
    pub high_confidence_count: usize,
    /// Count of low confidence entities (< 0.5)
    pub low_confidence_count: usize,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_confidence_factor() {
        let factor = ConfidenceFactor::new("test", FactorCategory::Extraction, 0.8, 0.5);
        
        assert_eq!(factor.name, "test");
        assert_eq!(factor.category, FactorCategory::Extraction);
        assert_eq!(factor.value, 0.8);
        assert_eq!(factor.weight, 0.5);
        assert_eq!(factor.weighted_contribution(), 0.4);
    }

    #[test]
    fn test_confidence_score_creation() {
        let score = ConfidenceScore::new(0.85)
            .with_attribute("name", 0.9)
            .with_attribute("email", 0.8);
        
        assert_eq!(score.overall, 0.85);
        assert_eq!(score.by_attribute.len(), 2);
        assert_eq!(score.by_attribute.get("name"), Some(&0.9));
        assert!(score.is_high_confidence());
    }

    #[test]
    fn test_confidence_tier() {
        assert_eq!(ConfidenceTier::from_score(0.95), ConfidenceTier::VeryHigh);
        assert_eq!(ConfidenceTier::from_score(0.85), ConfidenceTier::High);
        assert_eq!(ConfidenceTier::from_score(0.7), ConfidenceTier::Medium);
        assert_eq!(ConfidenceTier::from_score(0.5), ConfidenceTier::Low);
        assert_eq!(ConfidenceTier::from_score(0.3), ConfidenceTier::VeryLow);
    }

    #[test]
    fn test_confidence_recommendation() {
        assert_eq!(
            ConfidenceRecommendation::from_score(0.9),
            ConfidenceRecommendation::Accept
        );
        assert_eq!(
            ConfidenceRecommendation::from_score(0.75),
            ConfidenceRecommendation::AcceptWithMonitoring
        );
        assert_eq!(
            ConfidenceRecommendation::from_score(0.6),
            ConfidenceRecommendation::ManualReview
        );
        assert_eq!(
            ConfidenceRecommendation::from_score(0.4),
            ConfidenceRecommendation::Reject
        );
    }

    #[test]
    fn test_calculate_uncertainty() {
        let factors = vec![
            ConfidenceFactor::new("a", FactorCategory::Extraction, 0.9, 0.5),
            ConfidenceFactor::new("b", FactorCategory::Extraction, 0.1, 0.5),
        ];
        
        // High disagreement should produce high uncertainty
        let uncertainty = ConfidenceScore::calculate_uncertainty(&factors);
        assert!(uncertainty > 0.3);

        // Low disagreement should produce low uncertainty
        let factors_agree = vec![
            ConfidenceFactor::new("a", FactorCategory::Extraction, 0.8, 0.5),
            ConfidenceFactor::new("b", FactorCategory::Extraction, 0.85, 0.5),
        ];
        let uncertainty_agree = ConfidenceScore::calculate_uncertainty(&factors_agree);
        assert!(uncertainty_agree < uncertainty);
    }

    #[test]
    fn test_aggregate_confidence() {
        let scorer = ConfidenceScorer::new();
        
        let factors = vec![
            ConfidenceFactor::new("a", FactorCategory::Extraction, 0.9, 0.5),
            ConfidenceFactor::new("b", FactorCategory::SourceReliability, 0.7, 0.5),
        ];
        
        // (0.9 * 0.5 + 0.7 * 0.5) / (0.5 + 0.5) = 0.8
        let overall = scorer.aggregate_confidence(&factors);
        assert!((overall - 0.8).abs() < 0.01);
    }

    #[test]
    fn test_entity_confidence() {
        let mut scorer = ConfidenceScorer::new();
        
        // Register a reliable source
        let source_id = Uuid::new_v4();
        let mut source = SourceReliability::new(source_id, "Test Source", SourceType::Human);
        for _ in 0..20 {
            source.record_correct();
        }
        scorer.register_source(source);
        
        // Create entity
        let mut entity = Entity::new("Test Entity", crate::models::EntityType::Person, Some(source_id));
        entity.extraction_confidence = 0.85;
        
        let score = scorer.calculate_entity_confidence(&entity);
        
        assert!(score.overall > 0.0);
        assert!(!score.factors.is_empty());
    }

    #[test]
    fn test_batch_calculate() {
        let scorer = ConfidenceScorer::new();
        
        let entities = vec![
            Entity::new("Entity 1", crate::models::EntityType::Person, None)
                .with_confidence(0.9),
            Entity::new("Entity 2", crate::models::EntityType::Organization, None)
                .with_confidence(0.6),
        ];
        
        let scores = scorer.batch_calculate(&entities);
        
        assert_eq!(scores.len(), 2);
        assert!(scores.contains_key(&entities[0].id));
        assert!(scores.contains_key(&entities[1].id));
    }

    #[test]
    fn test_confidence_statistics() {
        let scorer = ConfidenceScorer::new();
        
        let entities = vec![
            Entity::new("E1", crate::models::EntityType::Person, None).with_confidence(0.9),
            Entity::new("E2", crate::models::EntityType::Person, None).with_confidence(0.7),
            Entity::new("E3", crate::models::EntityType::Person, None).with_confidence(0.5),
        ];
        
        let stats = scorer.confidence_statistics(&entities);
        
        assert_eq!(stats.count, 3);
        assert!(stats.mean > 0.0);
        assert!(stats.high_confidence_count >= 1);
        assert!(stats.low_confidence_count >= 1);
    }

    #[test]
    fn test_filter_by_confidence() {
        let scorer = ConfidenceScorer::new();
        
        let entities = vec![
            Entity::new("E1", crate::models::EntityType::Person, None).with_confidence(0.9),
            Entity::new("E2", crate::models::EntityType::Person, None).with_confidence(0.5),
        ];
        
        let high_confidence = scorer.filter_by_confidence(&entities, 0.8);
        
        assert_eq!(high_confidence.len(), 1);
        assert_eq!(high_confidence[0].name, "E1");
    }
}
