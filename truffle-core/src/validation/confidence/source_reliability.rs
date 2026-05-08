//! Source Reliability Tracking
//!
//! Tracks the historical accuracy and quality of information sources
//! for use in confidence calculations.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;

/// Reliability information for a data source
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct SourceReliability {
    /// Unique source identifier
    pub source_id: Uuid,
    /// Source name or description
    pub name: String,
    /// Historical accuracy score (0.0 - 1.0)
    pub reliability_score: f32,
    /// Number of observations/samples used to calculate reliability
    pub sample_size: usize,
    /// When the source was first tracked
    pub first_seen_at: DateTime<Utc>,
    /// When reliability was last updated
    pub last_updated: DateTime<Utc>,
    /// Number of correct predictions/observations
    pub correct_count: usize,
    /// Number of incorrect predictions/observations
    pub incorrect_count: usize,
    /// Source type/category
    pub source_type: SourceType,
    /// Additional metadata
    #[serde(flatten)]
    pub metadata: HashMap<String, serde_json::Value>,
}

impl SourceReliability {
    /// Create a new source reliability tracker with default values
    pub fn new(source_id: Uuid, name: impl Into<String>, source_type: SourceType) -> Self {
        let now = Utc::now();
        Self {
            source_id,
            name: name.into(),
            reliability_score: 0.5, // Start with neutral reliability
            sample_size: 0,
            first_seen_at: now,
            last_updated: now,
            correct_count: 0,
            incorrect_count: 0,
            source_type,
            metadata: HashMap::new(),
        }
    }

    /// Record a correct observation from this source
    pub fn record_correct(&mut self) {
        self.correct_count += 1;
        self.sample_size += 1;
        self.update_reliability();
    }

    /// Record an incorrect observation from this source
    pub fn record_incorrect(&mut self) {
        self.incorrect_count += 1;
        self.sample_size += 1;
        self.update_reliability();
    }

    /// Update reliability score based on current counts using Laplace smoothing
    fn update_reliability(&mut self) {
        // Laplace smoothing: (correct + 1) / (total + 2)
        // This prevents 0 or 1 reliability with small sample sizes
        let smoothed_correct = self.correct_count as f32 + 1.0;
        let smoothed_total = self.sample_size as f32 + 2.0;
        
        self.reliability_score = (smoothed_correct / smoothed_total).clamp(0.0, 1.0);
        self.last_updated = Utc::now();
    }

    /// Get the accuracy rate without smoothing
    pub fn raw_accuracy(&self) -> f32 {
        if self.sample_size == 0 {
            0.5
        } else {
            (self.correct_count as f32) / (self.sample_size as f32)
        }
    }

    /// Get confidence interval for the reliability score
    /// Returns (lower_bound, upper_bound) at 95% confidence
    pub fn confidence_interval(&self) -> (f32, f32) {
        if self.sample_size < 30 {
            // Not enough samples for reliable interval
            return (0.0, 1.0);
        }

        let p = self.reliability_score;
        let n = self.sample_size as f32;
        let z = 1.96; // 95% confidence

        let margin = z * ((p * (1.0 - p)) / n).sqrt();
        
        ((p - margin).max(0.0), (p + margin).min(1.0))
    }

    /// Check if source has sufficient samples for reliable assessment
    pub fn is_reliable(&self, min_samples: usize) -> bool {
        self.sample_size >= min_samples && self.reliability_score >= 0.7
    }

    /// Calculate a recency-weighted reliability score
    /// More recent observations have higher weight
    pub fn recency_weighted_reliability(&self, half_life_days: f64) -> f32 {
        let age = (Utc::now() - self.last_updated).num_days() as f64;
        let decay = (-age / half_life_days).exp();
        
        // Blend current reliability with neutral (0.5) based on recency
        let recency_factor = decay as f32;
        self.reliability_score * recency_factor + 0.5 * (1.0 - recency_factor)
    }

    /// Get the effective weight for this source in confidence aggregation
    /// Weights decrease with age and increase with sample size
    pub fn effective_weight(&self) -> f32 {
        let sample_weight = (self.sample_size as f32).sqrt().min(10.0) / 10.0;
        let reliability_weight = self.reliability_score;
        let recency_boost = if self.sample_size > 0 {
            let hours_since_update = (Utc::now() - self.last_updated).num_hours() as f32;
            (-hours_since_update / 168.0).exp().min(1.0) // Decay over a week
        } else {
            0.5
        };
        
        sample_weight * reliability_weight * recency_boost
    }

    /// Add metadata
    pub fn with_metadata(mut self, key: impl Into<String>, value: impl Into<serde_json::Value>) -> Self {
        self.metadata.insert(key.into(), value.into());
        self
    }
}

impl Default for SourceReliability {
    fn default() -> Self {
        Self::new(Uuid::new_v4(), "Unknown", SourceType::Unknown)
    }
}

/// Types of information sources
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Hash)]
#[serde(rename_all = "snake_case")]
pub enum SourceType {
    /// Human user input/verification
    Human,
    /// Automated extraction from transcripts
    Extraction,
    /// External API or service
    ExternalApi,
    /// Machine learning model
    MlModel,
    /// Imported from another system
    Import,
    /// Derived from graph inference
    Inference,
    /// Unknown or unspecified source
    Unknown,
}

impl SourceType {
    /// Get base reliability for source type (prior belief)
    pub fn base_reliability(&self) -> f32 {
        match self {
            Self::Human => 0.9,        // Humans are generally reliable
            Self::Extraction => 0.75,  // Automated extraction is good but not perfect
            Self::ExternalApi => 0.8,  // External APIs are usually reliable
            Self::MlModel => 0.7,      // ML models vary in accuracy
            Self::Import => 0.6,       // Imports may have quality issues
            Self::Inference => 0.5,    // Inferred data is less certain
            Self::Unknown => 0.5,      // Unknown sources are neutral
        }
    }

    /// Get display name for source type
    pub fn display_name(&self) -> &'static str {
        match self {
            Self::Human => "Human Verification",
            Self::Extraction => "Automated Extraction",
            Self::ExternalApi => "External API",
            Self::MlModel => "ML Model",
            Self::Import => "Data Import",
            Self::Inference => "Graph Inference",
            Self::Unknown => "Unknown Source",
        }
    }
}

/// Registry for tracking multiple sources
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct SourceRegistry {
    sources: HashMap<Uuid, SourceReliability>,
}

impl SourceRegistry {
    /// Create a new empty registry
    pub fn new() -> Self {
        Self {
            sources: HashMap::new(),
        }
    }

    /// Register a new source
    pub fn register(&mut self, source: SourceReliability) {
        self.sources.insert(source.source_id, source);
    }

    /// Get source reliability by ID
    pub fn get(&self, source_id: &Uuid) -> Option<&SourceReliability> {
        self.sources.get(source_id)
    }

    /// Get mutable reference to source
    pub fn get_mut(&mut self, source_id: &Uuid) -> Option<&mut SourceReliability> {
        self.sources.get_mut(source_id)
    }

    /// Record a correct observation for a source
    pub fn record_correct(&mut self, source_id: &Uuid) {
        if let Some(source) = self.sources.get_mut(source_id) {
            source.record_correct();
        }
    }

    /// Record an incorrect observation for a source
    pub fn record_incorrect(&mut self, source_id: &Uuid) {
        if let Some(source) = self.sources.get_mut(source_id) {
            source.record_incorrect();
        }
    }

    /// Get all sources
    pub fn all_sources(&self) -> Vec<&SourceReliability> {
        self.sources.values().collect()
    }

    /// Get sources by type
    pub fn sources_by_type(&self, source_type: SourceType) -> Vec<&SourceReliability> {
        self.sources
            .values()
            .filter(|s| s.source_type == source_type)
            .collect()
    }

    /// Get most reliable sources
    pub fn most_reliable(&self, limit: usize) -> Vec<&SourceReliability> {
        let mut sources: Vec<_> = self.sources.values().collect();
        sources.sort_by(|a, b| {
            b.reliability_score
                .partial_cmp(&a.reliability_score)
                .unwrap_or(std::cmp::Ordering::Equal)
        });
        sources.into_iter().take(limit).collect()
    }

    /// Get average reliability across all sources
    pub fn average_reliability(&self) -> f32 {
        if self.sources.is_empty() {
            return 0.5;
        }
        
        let sum: f32 = self.sources.values().map(|s| s.reliability_score).sum();
        sum / self.sources.len() as f32
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_source_reliability_creation() {
        let source_id = Uuid::new_v4();
        let source = SourceReliability::new(source_id, "Test Source", SourceType::MlModel);

        assert_eq!(source.source_id, source_id);
        assert_eq!(source.name, "Test Source");
        assert_eq!(source.source_type, SourceType::MlModel);
        assert_eq!(source.reliability_score, 0.5); // Default neutral
        assert_eq!(source.sample_size, 0);
    }

    #[test]
    fn test_record_observations() {
        let mut source = SourceReliability::new(Uuid::new_v4(), "Test", SourceType::Human);
        
        source.record_correct();
        source.record_correct();
        source.record_incorrect();
        
        assert_eq!(source.correct_count, 2);
        assert_eq!(source.incorrect_count, 1);
        assert_eq!(source.sample_size, 3);
        assert!(source.reliability_score > 0.5 && source.reliability_score < 0.9);
    }

    #[test]
    fn test_laplace_smoothing() {
        let mut source = SourceReliability::new(Uuid::new_v4(), "Test", SourceType::Human);
        
        // With no data, reliability should be near 0.5 due to smoothing
        assert!(source.reliability_score >= 0.49 && source.reliability_score <= 0.51);
        
        // After many correct observations, should approach 1.0
        for _ in 0..100 {
            source.record_correct();
        }
        assert!(source.reliability_score > 0.95);
    }

    #[test]
    fn test_raw_accuracy() {
        let mut source = SourceReliability::new(Uuid::new_v4(), "Test", SourceType::Human);
        
        assert_eq!(source.raw_accuracy(), 0.5); // Default when no data
        
        source.record_correct();
        source.record_correct();
        source.record_incorrect();
        
        assert_eq!(source.raw_accuracy(), 2.0 / 3.0);
    }

    #[test]
    fn test_confidence_interval() {
        let mut source = SourceReliability::new(Uuid::new_v4(), "Test", SourceType::Human);
        
        // With few samples, interval should be wide
        let (low, high) = source.confidence_interval();
        assert_eq!(low, 0.0);
        assert_eq!(high, 1.0);
        
        // Add many samples
        for _ in 0..50 {
            source.record_correct();
        }
        
        let (low, high) = source.confidence_interval();
        assert!(low > 0.8);
        assert!(high <= 1.0);
        assert!(high > low);
    }

    #[test]
    fn test_is_reliable() {
        let mut source = SourceReliability::new(Uuid::new_v4(), "Test", SourceType::Human);
        
        assert!(!source.is_reliable(10)); // Not enough samples
        
        for _ in 0..10 {
            source.record_correct();
        }
        
        assert!(source.is_reliable(10));
    }

    #[test]
    fn test_source_type_base_reliability() {
        assert!(SourceType::Human.base_reliability() > SourceType::MlModel.base_reliability());
        assert!(SourceType::Extraction.base_reliability() > SourceType::Inference.base_reliability());
    }

    #[test]
    fn test_source_registry() {
        let mut registry = SourceRegistry::new();
        
        let source1 = SourceReliability::new(Uuid::new_v4(), "Source 1", SourceType::Human);
        let source2 = SourceReliability::new(Uuid::new_v4(), "Source 2", SourceType::MlModel);
        
        registry.register(source1.clone());
        registry.register(source2.clone());
        
        assert!(registry.get(&source1.source_id).is_some());
        assert!(registry.get(&source2.source_id).is_some());
        
        let most_reliable = registry.most_reliable(1);
        assert_eq!(most_reliable.len(), 1);
    }
}
