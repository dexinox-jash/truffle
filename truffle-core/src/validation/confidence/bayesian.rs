//! Bayesian Confidence Updater
//!
//! Implements Bayes' theorem for updating entity confidence based on new evidence.
//! Provides support for handling conflicting evidence and multiple update strategies.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use super::source_reliability::{SourceReliability, SourceType};

/// Evidence for Bayesian updating
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Evidence {
    /// Unique evidence ID
    pub id: Uuid,
    /// Evidence value (confidence this evidence provides: 0.0 - 1.0)
    pub value: f32,
    /// Source of the evidence
    pub source_id: Option<Uuid>,
    /// Type of evidence
    pub evidence_type: EvidenceType,
    /// Timestamp when evidence was observed
    pub observed_at: DateTime<Utc>,
    /// Prior probability before this evidence
    pub prior_probability: f32,
    /// Likelihood of evidence given hypothesis is true
    pub likelihood_true: f32,
    /// Likelihood of evidence given hypothesis is false
    pub likelihood_false: f32,
    /// Weight/importance of this evidence (0.0 - 1.0)
    pub weight: f32,
    /// Optional description
    pub description: Option<String>,
}

impl Evidence {
    /// Create new evidence with default likelihoods
    pub fn new(
        value: f32,
        evidence_type: EvidenceType,
        prior_probability: f32,
    ) -> Self {
        Self {
            id: Uuid::new_v4(),
            value: value.clamp(0.0, 1.0),
            source_id: None,
            evidence_type,
            observed_at: Utc::now(),
            prior_probability: prior_probability.clamp(0.0, 1.0),
            likelihood_true: 0.9,   // Default: evidence is likely if hypothesis true
            likelihood_false: 0.3,  // Default: evidence is less likely if hypothesis false
            weight: 1.0,
            description: None,
        }
    }

    /// Create confirmatory evidence (supports the hypothesis)
    pub fn confirmatory(value: f32, prior: f32) -> Self {
        Self::new(value, EvidenceType::Confirmatory, prior)
            .with_likelihoods(0.9, 0.2)
    }

    /// Create disconfirmatory evidence (opposes the hypothesis)
    pub fn disconfirmatory(value: f32, prior: f32) -> Self {
        Self::new(value, EvidenceType::Disconfirmatory, prior)
            .with_likelihoods(0.2, 0.8)
    }

    /// Create ambiguous evidence (uncertain diagnostic value)
    pub fn ambiguous(value: f32, prior: f32) -> Self {
        Self::new(value, EvidenceType::Ambiguous, prior)
            .with_likelihoods(0.6, 0.5)
    }

    /// Set likelihoods
    pub fn with_likelihoods(mut self, if_true: f32, if_false: f32) -> Self {
        self.likelihood_true = if_true.clamp(0.0, 1.0);
        self.likelihood_false = if_false.clamp(0.0, 1.0);
        self
    }

    /// Set source
    pub fn with_source(mut self, source_id: Uuid) -> Self {
        self.source_id = Some(source_id);
        self
    }

    /// Set weight
    pub fn with_weight(mut self, weight: f32) -> Self {
        self.weight = weight.clamp(0.0, 1.0);
        self
    }

    /// Set description
    pub fn with_description(mut self, desc: impl Into<String>) -> Self {
        self.description = Some(desc.into());
        self
    }

    /// Calculate evidence probability (denominator in Bayes' theorem)
    /// P(E) = P(E|H) * P(H) + P(E|¬H) * P(¬H)
    pub fn evidence_probability(&self) -> f32 {
        let p_e_given_h = self.likelihood_true;
        let p_e_given_not_h = self.likelihood_false;
        let p_h = self.prior_probability;
        let p_not_h = 1.0 - p_h;

        p_e_given_h * p_h + p_e_given_not_h * p_not_h
    }

    /// Calculate the diagnostic value of this evidence
    /// Higher values mean the evidence better discriminates true from false
    pub fn diagnostic_value(&self) -> f32 {
        (self.likelihood_true - self.likelihood_false).abs()
    }

    /// Check if evidence is diagnostic (meaningfully distinguishes hypothesis)
    pub fn is_diagnostic(&self, threshold: f32) -> bool {
        self.diagnostic_value() >= threshold
    }

    /// Get age of evidence in hours
    pub fn age_hours(&self) -> f64 {
        (Utc::now() - self.observed_at).num_hours() as f64
    }
}

/// Types of evidence
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum EvidenceType {
    /// Evidence that supports the hypothesis
    Confirmatory,
    /// Evidence that opposes the hypothesis
    Disconfirmatory,
    /// Evidence with unclear diagnostic value
    Ambiguous,
    /// Corroborating evidence from another source
    Corroborating,
    /// Conflicting evidence from another source
    Conflicting,
    /// Direct observation/measurement
    Observation,
    /// Inferred from related data
    Inference,
}

/// Result of Bayesian update
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct BayesianUpdateResult {
    /// The updated posterior probability
    pub posterior: f32,
    /// The prior probability before update
    pub prior: f32,
    /// Evidence used for update
    pub evidence: Evidence,
    /// Bayes factor (strength of evidence)
    pub bayes_factor: f32,
    /// Log odds change
    pub log_odds_change: f32,
    /// Whether this was a significant update
    pub significant: bool,
}

/// Bayesian updater for confidence values
#[derive(Debug, Clone, Default)]
pub struct BayesianUpdater {
    /// Minimum evidence weight to consider significant
    pub significance_threshold: f32,
    /// Maximum confidence cap
    pub max_confidence: f32,
    /// Minimum confidence floor
    pub min_confidence: f32,
    /// Whether to apply recency weighting
    pub recency_weighting: bool,
    /// Half-life for recency decay (in hours)
    pub recency_half_life: f64,
}

impl BayesianUpdater {
    /// Create a new Bayesian updater with default settings
    pub fn new() -> Self {
        Self {
            significance_threshold: 0.05,
            max_confidence: 0.99,
            min_confidence: 0.01,
            recency_weighting: true,
            recency_half_life: 168.0, // 1 week
        }
    }

    /// Update configuration
    pub fn with_config(
        mut self,
        min_confidence: f32,
        max_confidence: f32,
        significance_threshold: f32,
    ) -> Self {
        self.min_confidence = min_confidence.clamp(0.0, 1.0);
        self.max_confidence = max_confidence.clamp(0.0, 1.0);
        self.significance_threshold = significance_threshold.clamp(0.0, 1.0);
        self
    }

    /// Perform a single Bayesian update
    /// 
    /// Bayes' Theorem: P(H|E) = P(E|H) * P(H) / P(E)
    /// 
    /// Where:
    /// - P(H|E) = posterior (what we want to calculate)
    /// - P(E|H) = likelihood_true (probability of evidence given hypothesis is true)
    /// - P(H) = prior (prior probability)
    /// - P(E) = evidence_probability (total probability of evidence)
    pub fn update(&self, prior: f32, evidence: &Evidence) -> BayesianUpdateResult {
        let prior = prior.clamp(self.min_confidence, self.max_confidence);
        
        // Apply recency weighting if enabled
        let weight = if self.recency_weighting {
            let age = evidence.age_hours();
            let decay = (-age / self.recency_half_life).exp();
            evidence.weight * decay as f32
        } else {
            evidence.weight
        };

        // Get likelihoods (use evidence values)
        let likelihood_true = evidence.likelihood_true;
        let likelihood_false = evidence.likelihood_false;
        
        // Calculate evidence probability
        let p_h = prior;
        let p_not_h = 1.0 - p_h;
        let evidence_prob = likelihood_true * p_h + likelihood_false * p_not_h;

        // Avoid division by zero
        let evidence_prob = evidence_prob.max(1e-10);

        // Calculate posterior
        let mut posterior = (likelihood_true * p_h) / evidence_prob;
        
        // Apply weighting: blend between prior and full update based on weight
        posterior = prior + weight * (posterior - prior);
        
        // Clamp to valid range
        posterior = posterior.clamp(self.min_confidence, self.max_confidence);

        // Calculate Bayes factor
        let bayes_factor = if likelihood_false > 0.0 {
            likelihood_true / likelihood_false
        } else {
            100.0 // Very strong evidence
        };

        // Calculate log odds change
        let prior_odds = p_h / (1.0 - p_h);
        let posterior_odds = posterior / (1.0 - posterior);
        let log_odds_change = posterior_odds.ln() - prior_odds.ln();

        // Check if significant
        let significant = (posterior - prior).abs() >= self.significance_threshold;

        BayesianUpdateResult {
            posterior,
            prior,
            evidence: evidence.clone(),
            bayes_factor,
            log_odds_change,
            significant,
        }
    }

    /// Update with multiple evidence items (sequential updating)
    /// Updates are applied in order, with each posterior becoming the next prior
    pub fn update_multiple(
        &self,
        initial_prior: f32,
        evidence_items: &[Evidence],
    ) -> (f32, Vec<BayesianUpdateResult>) {
        let mut current_prior = initial_prior;
        let mut results = Vec::with_capacity(evidence_items.len());

        for evidence in evidence_items {
            let result = self.update(current_prior, evidence);
            current_prior = result.posterior;
            results.push(result);
        }

        (current_prior, results)
    }

    /// Update with source reliability information
    /// Adjusts likelihoods based on source quality
    pub fn update_with_source(
        &self,
        prior: f32,
        evidence: &Evidence,
        source: &SourceReliability,
    ) -> BayesianUpdateResult {
        // Adjust likelihoods based on source reliability
        let reliability = source.reliability_score;
        
        // More reliable sources have likelihoods closer to extremes
        // Less reliable sources have likelihoods closer to 0.5 (neutral)
        let adjusted_likelihood_true = evidence.likelihood_true * reliability 
            + 0.5 * (1.0 - reliability);
        let adjusted_likelihood_false = evidence.likelihood_false * reliability 
            + 0.5 * (1.0 - reliability);

        let mut adjusted_evidence = evidence.clone();
        adjusted_evidence.likelihood_true = adjusted_likelihood_true;
        adjusted_evidence.likelihood_false = adjusted_likelihood_false;
        adjusted_evidence.source_id = Some(source.source_id);

        self.update(prior, &adjusted_evidence)
    }

    /// Handle conflicting evidence using a weighted pooling approach
    /// Instead of sequential updating which can amplify conflicts,
    /// this pools evidence and calculates a consensus posterior
    pub fn update_with_conflicts(
        &self,
        prior: f32,
        evidence_items: &[Evidence],
    ) -> ConflictResolutionResult {
        if evidence_items.is_empty() {
            return ConflictResolutionResult {
                posterior: prior,
                prior,
                consensus_weight: 0.0,
                conflict_level: 0.0,
                resolution_strategy: ConflictResolutionStrategy::NoEvidence,
            };
        }

        if evidence_items.len() == 1 {
            let result = self.update(prior, &evidence_items[0]);
            return ConflictResolutionResult {
                posterior: result.posterior,
                prior,
                consensus_weight: result.evidence.weight,
                conflict_level: 0.0,
                resolution_strategy: ConflictResolutionStrategy::SingleEvidence,
            };
        }

        // Calculate individual posteriors and weights
        let mut posteriors = Vec::with_capacity(evidence_items.len());
        let mut weights = Vec::with_capacity(evidence_items.len());

        for evidence in evidence_items {
            let result = self.update(prior, evidence);
            posteriors.push(result.posterior);
            weights.push(evidence.weight * evidence.diagnostic_value());
        }

        // Calculate conflict level (variance in posteriors)
        let mean_posterior: f32 = posteriors.iter().sum::<f32>() / posteriors.len() as f32;
        let variance: f32 = posteriors
            .iter()
            .map(|p| (p - mean_posterior).powi(2))
            .sum::<f32>() / posteriors.len() as f32;
        let conflict_level = variance.sqrt(); // Standard deviation as conflict measure

        // Weighted average of posteriors (linear opinion pooling)
        let total_weight: f32 = weights.iter().sum();
        let weighted_posterior: f32 = if total_weight > 0.0 {
            posteriors
                .iter()
                .zip(weights.iter())
                .map(|(p, w)| p * w)
                .sum::<f32>() / total_weight
        } else {
            prior
        };

        // Blend with prior based on conflict level
        // Higher conflict = trust the consensus less, stick closer to prior
        let conflict_discount = 1.0 - conflict_level;
        let final_posterior = prior + conflict_discount * (weighted_posterior - prior);

        // Determine resolution strategy
        let strategy = if conflict_level > 0.5 {
            ConflictResolutionStrategy::HighConflictAveraging
        } else if conflict_level > 0.2 {
            ConflictResolutionStrategy::ModerateConflictPooling
        } else {
            ConflictResolutionStrategy::Consensus
        };

        ConflictResolutionResult {
            posterior: final_posterior.clamp(self.min_confidence, self.max_confidence),
            prior,
            consensus_weight: total_weight,
            conflict_level,
            resolution_strategy: strategy,
        }
    }

    /// Calculate the prior for a new entity based on source type
    pub fn initial_prior(source_type: Option<SourceType>) -> f32 {
        match source_type {
            Some(st) => st.base_reliability(),
            None => 0.5, // Neutral prior for unknown sources
        }
    }
}

/// Result of conflict resolution
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ConflictResolutionResult {
    /// Final posterior probability
    pub posterior: f32,
    /// Original prior
    pub prior: f32,
    /// Total weight of consensus
    pub consensus_weight: f32,
    /// Level of conflict detected (0.0 = no conflict, 1.0 = maximum conflict)
    pub conflict_level: f32,
    /// Strategy used to resolve conflict
    pub resolution_strategy: ConflictResolutionStrategy,
}

/// Strategies for resolving conflicting evidence
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum ConflictResolutionStrategy {
    /// No evidence provided
    NoEvidence,
    /// Only single evidence item
    SingleEvidence,
    /// Evidence is in consensus
    Consensus,
    /// Moderate conflict, used weighted pooling
    ModerateConflictPooling,
    /// High conflict, used conservative averaging
    HighConflictAveraging,
    /// Manual resolution required
    ManualReview,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_bayesian_update_basic() {
        let updater = BayesianUpdater::new();
        let prior = 0.5;
        
        // Confirmatory evidence should increase confidence
        let evidence = Evidence::confirmatory(0.9, prior);
        let result = updater.update(prior, &evidence);
        
        assert!(result.posterior > prior);
        assert!(result.significant);
        assert!(result.bayes_factor > 1.0);
    }

    #[test]
    fn test_disconfirmatory_evidence() {
        let updater = BayesianUpdater::new();
        let prior = 0.8;
        
        // Disconfirmatory evidence should decrease confidence
        let evidence = Evidence::disconfirmatory(0.2, prior);
        let result = updater.update(prior, &evidence);
        
        assert!(result.posterior < prior);
    }

    #[test]
    fn test_evidence_probability() {
        let evidence = Evidence::confirmatory(0.8, 0.5)
            .with_likelihoods(0.9, 0.3);
        
        // P(E) = P(E|H) * P(H) + P(E|¬H) * P(¬H)
        // P(E) = 0.9 * 0.5 + 0.3 * 0.5 = 0.45 + 0.15 = 0.6
        let expected = 0.6;
        assert!((evidence.evidence_probability() - expected).abs() < 0.01);
    }

    #[test]
    fn test_multiple_updates() {
        let updater = BayesianUpdater::new();
        let prior = 0.5;
        
        let evidence_items = vec![
            Evidence::confirmatory(0.9, prior),
            Evidence::confirmatory(0.8, prior),
            Evidence::confirmatory(0.85, prior),
        ];
        
        let (final_posterior, results) = updater.update_multiple(prior, &evidence_items);
        
        assert!(final_posterior > prior);
        assert_eq!(results.len(), 3);
        
        // Each subsequent posterior should be higher than the previous
        assert!(results[0].posterior > prior);
        assert!(results[1].posterior > results[0].posterior);
        assert!(results[2].posterior > results[1].posterior);
    }

    #[test]
    fn test_conflict_resolution() {
        let updater = BayesianUpdater::new();
        let prior = 0.5;
        
        // Conflicting evidence
        let evidence_items = vec![
            Evidence::confirmatory(0.9, prior)
                .with_weight(1.0),
            Evidence::disconfirmatory(0.1, prior)
                .with_weight(1.0),
        ];
        
        let result = updater.update_with_conflicts(prior, &evidence_items);
        
        // Should detect high conflict
        assert!(result.conflict_level > 0.0);
        assert_eq!(result.resolution_strategy, ConflictResolutionStrategy::HighConflictAveraging);
        
        // Posterior should be closer to prior due to conflict
        assert!((result.posterior - prior).abs() < 0.3);
    }

    #[test]
    fn test_diagnostic_value() {
        let evidence_high = Evidence::new(0.8, EvidenceType::Confirmatory, 0.5)
            .with_likelihoods(0.95, 0.1);
        
        let evidence_low = Evidence::new(0.8, EvidenceType::Ambiguous, 0.5)
            .with_likelihoods(0.6, 0.5);
        
        assert!(evidence_high.diagnostic_value() > evidence_low.diagnostic_value());
        assert!(evidence_high.is_diagnostic(0.5));
        assert!(!evidence_low.is_diagnostic(0.5));
    }

    #[test]
    fn test_update_with_source() {
        let updater = BayesianUpdater::new();
        let prior = 0.5;
        
        let mut reliable_source = SourceReliability::new(
            Uuid::new_v4(),
            "Reliable",
            SourceType::Human
        );
        // Make it reliable
        for _ in 0..50 {
            reliable_source.record_correct();
        }
        
        let evidence = Evidence::confirmatory(0.9, prior);
        
        let result = updater.update_with_source(prior, &evidence, &reliable_source);
        
        // Should produce different result than without source
        let result_no_source = updater.update(prior, &evidence);
        assert!(result.posterior != result_no_source.posterior);
    }

    #[test]
    fn test_initial_prior() {
        assert_eq!(
            BayesianUpdater::initial_prior(Some(SourceType::Human)),
            SourceType::Human.base_reliability()
        );
        assert_eq!(
            BayesianUpdater::initial_prior(None),
            0.5
        );
    }

    #[test]
    fn test_confidence_bounds() {
        let updater = BayesianUpdater::new()
            .with_config(0.1, 0.9, 0.05);
        
        let prior = 0.5;
        let strong_evidence = Evidence::confirmatory(1.0, prior)
            .with_likelihoods(0.99, 0.01);
        
        let result = updater.update(prior, &strong_evidence);
        
        // Should be capped at max_confidence
        assert!(result.posterior <= 0.9);
        assert!(result.posterior >= 0.1);
    }
}
