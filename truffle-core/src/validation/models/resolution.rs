//! Resolution Types
//!
//! Defines how contradictions can be resolved in the knowledge graph.
//! Resolutions represent the outcome of a decision about conflicting facts.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// How a contradiction was resolved
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ContradictionResolution {
    /// Type of resolution applied
    pub resolution_type: ResolutionType,

    /// User or system that resolved the contradiction
    pub resolved_by: Uuid,

    /// When the resolution was made
    pub resolved_at: DateTime<Utc>,

    /// Optional notes explaining the resolution
    #[serde(skip_serializing_if = "Option::is_none")]
    pub notes: Option<String>,

    /// IDs of facts affected by this resolution
    #[serde(skip_serializing_if = "Vec::is_empty", default)]
    pub affected_fact_ids: Vec<Uuid>,

    /// The chosen fact value (if applicable)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub chosen_value: Option<super::FactValue>,

    /// Resolution metadata
    #[serde(skip_serializing_if = "Option::is_none")]
    pub metadata: Option<serde_json::Value>,
}

impl ContradictionResolution {
    /// Create a new resolution
    pub fn new(resolution_type: ResolutionType, resolved_by: Uuid) -> Self {
        Self {
            resolution_type,
            resolved_by,
            resolved_at: Utc::now(),
            notes: None,
            affected_fact_ids: Vec::new(),
            chosen_value: None,
            metadata: None,
        }
    }

    /// Set notes for the resolution
    pub fn with_notes(mut self, notes: impl Into<String>) -> Self {
        self.notes = Some(notes.into());
        self
    }

    /// Add an affected fact
    pub fn with_affected_fact(mut self, fact_id: Uuid) -> Self {
        self.affected_fact_ids.push(fact_id);
        self
    }

    /// Set affected facts
    pub fn with_affected_facts(mut self, fact_ids: Vec<Uuid>) -> Self {
        self.affected_fact_ids = fact_ids;
        self
    }

    /// Set the chosen value
    pub fn with_chosen_value(mut self, value: super::FactValue) -> Self {
        self.chosen_value = Some(value);
        self
    }

    /// Set metadata
    pub fn with_metadata(mut self, metadata: serde_json::Value) -> Self {
        self.metadata = Some(metadata);
        self
    }

    /// Create a merge resolution
    pub fn merge(resolved_by: Uuid, chosen_value: super::FactValue) -> Self {
        Self::new(ResolutionType::Merge, resolved_by)
            .with_chosen_value(chosen_value)
    }

    /// Create a split resolution
    pub fn split(resolved_by: Uuid, notes: impl Into<String>) -> Self {
        Self::new(ResolutionType::Split, resolved_by)
            .with_notes(notes)
    }

    /// Create an override resolution
    pub fn override_(resolved_by: Uuid, chosen_value: super::FactValue, notes: impl Into<String>) -> Self {
        Self::new(ResolutionType::Override, resolved_by)
            .with_chosen_value(chosen_value)
            .with_notes(notes)
    }

    /// Create a deprecate resolution
    pub fn deprecate(resolved_by: Uuid, deprecated_fact_ids: Vec<Uuid>, notes: impl Into<String>) -> Self {
        Self::new(ResolutionType::Deprecate, resolved_by)
            .with_affected_facts(deprecated_fact_ids)
            .with_notes(notes)
    }

    /// Get a summary of the resolution
    pub fn summary(&self) -> String {
        let base = format!("{} by {} at {}", 
            self.resolution_type.display_name(),
            self.resolved_by,
            self.resolved_at.format("%Y-%m-%d %H:%M:%S")
        );
        
        if let Some(notes) = &self.notes {
            format!("{} - {}", base, notes)
        } else {
            base
        }
    }
}

/// Types of resolution strategies
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Hash)]
#[serde(rename_all = "snake_case")]
pub enum ResolutionType {
    /// Merge facts into a single value
    /// (e.g., combine partial information from both sources)
    Merge,

    /// Split into separate facts for different contexts
    /// (e.g., different values apply to different time periods)
    Split,

    /// Override one fact with another
    /// (e.g., choose the more authoritative source)
    Override,

    /// Deprecate one or both facts
    /// (e.g., mark as outdated or incorrect)
    Deprecate,
}

impl ResolutionType {
    /// Get a human-readable name
    pub fn display_name(&self) -> &'static str {
        match self {
            Self::Merge => "Merge",
            Self::Split => "Split",
            Self::Override => "Override",
            Self::Deprecate => "Deprecate",
        }
    }

    /// Get a description of this resolution type
    pub fn description(&self) -> &'static str {
        match self {
            Self::Merge => "Combine facts into a single value",
            Self::Split => "Separate facts for different contexts or time periods",
            Self::Override => "Choose one fact as authoritative over another",
            Self::Deprecate => "Mark fact(s) as outdated or incorrect",
        }
    }

    /// Check if this resolution requires a chosen value
    pub fn requires_chosen_value(&self) -> bool {
        matches!(self, Self::Merge | Self::Override)
    }

    /// Check if this resolution affects multiple facts
    pub fn affects_multiple_facts(&self) -> bool {
        matches!(self, Self::Split | Self::Deprecate)
    }

    /// Check if this resolution preserves all facts
    pub fn preserves_all_facts(&self) -> bool {
        matches!(self, Self::Merge | Self::Split)
    }

    /// Get default resolution strategy for a contradiction type
    pub fn default_for(contradiction_type: super::ContradictionType) -> Self {
        match contradiction_type {
            super::ContradictionType::Temporal => Self::Split,
            super::ContradictionType::Factual => Self::Override,
            super::ContradictionType::SourceConflict => Self::Merge,
        }
    }
}

impl std::fmt::Display for ResolutionType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.display_name())
    }
}

/// Resolution policy for automatic contradiction handling
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ResolutionPolicy {
    /// Policy name
    pub name: String,

    /// Priority ordering for source types (highest first)
    pub source_priority: Vec<String>,

    /// Priority ordering for users (highest first)
    pub user_priority: Vec<Uuid>,

    /// Confidence threshold for auto-resolution
    pub auto_resolve_threshold: f32,

    /// Whether to auto-resolve temporal conflicts
    pub auto_resolve_temporal: bool,

    /// Whether to auto-resolve source conflicts
    pub auto_resolve_source_conflicts: bool,

    /// Whether to require human review for critical contradictions
    pub require_review_for_critical: bool,
}

impl ResolutionPolicy {
    /// Create a new default policy
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            source_priority: vec![
                "manual_verified".to_string(),
                "manual".to_string(),
                "extraction".to_string(),
                "import".to_string(),
            ],
            user_priority: Vec::new(),
            auto_resolve_threshold: 0.9,
            auto_resolve_temporal: true,
            auto_resolve_source_conflicts: true,
            require_review_for_critical: true,
        }
    }

    /// Set source priority
    pub fn with_source_priority(mut self, priority: Vec<String>) -> Self {
        self.source_priority = priority;
        self
    }

    /// Set user priority
    pub fn with_user_priority(mut self, priority: Vec<Uuid>) -> Self {
        self.user_priority = priority;
        self
    }

    /// Set auto-resolve threshold
    pub fn with_threshold(mut self, threshold: f32) -> Self {
        self.auto_resolve_threshold = threshold.clamp(0.0, 1.0);
        self
    }

    /// Get the priority score for a source type
    pub fn source_priority_score(&self, source_type: &str) -> usize {
        self.source_priority
            .iter()
            .position(|s| s == source_type)
            .unwrap_or(self.source_priority.len())
    }

    /// Get the priority score for a user
    pub fn user_priority_score(&self, user_id: Uuid) -> usize {
        self.user_priority
            .iter()
            .position(|u| *u == user_id)
            .unwrap_or(self.user_priority.len())
    }

    /// Check if a contradiction should be auto-resolved
    pub fn should_auto_resolve(&self, severity: super::ContradictionSeverity, confidence: f32) -> bool {
        if severity == super::ContradictionSeverity::Critical && self.require_review_for_critical {
            return false;
        }
        
        confidence >= self.auto_resolve_threshold
    }
}

impl Default for ResolutionPolicy {
    fn default() -> Self {
        Self::new("default")
    }
}

/// Resolution statistics
#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq)]
pub struct ResolutionStats {
    /// Total number of resolutions
    pub total_resolutions: usize,
    /// Number by type
    pub by_type: std::collections::HashMap<ResolutionType, usize>,
    /// Number auto-resolved
    pub auto_resolved: usize,
    /// Number manually resolved
    pub manually_resolved: usize,
    /// Average time to resolution (in seconds)
    pub avg_resolution_time_secs: Option<f64>,
    /// Resolutions by user
    pub by_user: std::collections::HashMap<Uuid, usize>,
}

impl ResolutionStats {
    /// Create new empty stats
    pub fn new() -> Self {
        Self::default()
    }

    /// Add a resolution to the stats
    pub fn add_resolution(&mut self, resolution: &ContradictionResolution, auto: bool) {
        self.total_resolutions += 1;
        *self.by_type.entry(resolution.resolution_type).or_insert(0) += 1;
        *self.by_user.entry(resolution.resolved_by).or_insert(0) += 1;
        
        if auto {
            self.auto_resolved += 1;
        } else {
            self.manually_resolved += 1;
        }
    }

    /// Record resolution time
    pub fn record_resolution_time(&mut self, detection_time: DateTime<Utc>, resolution_time: DateTime<Utc>) {
        let duration = resolution_time.signed_duration_since(detection_time);
        let secs = duration.num_seconds() as f64;
        
        match self.avg_resolution_time_secs {
            Some(avg) => {
                self.avg_resolution_time_secs = Some(
                    (avg * (self.total_resolutions - 1) as f64 + secs) / self.total_resolutions as f64
                );
            }
            None => {
                self.avg_resolution_time_secs = Some(secs);
            }
        }
    }

    /// Get the most common resolution type
    pub fn most_common_type(&self) -> Option<ResolutionType> {
        self.by_type.iter()
            .max_by_key(|(_, count)| *count)
            .map(|(t, _)| *t)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use super::super::{ContradictionType, ContradictionSeverity, FactValue};

    #[test]
    fn test_resolution_creation() {
        let resolver = Uuid::new_v4();
        let resolution = ContradictionResolution::new(ResolutionType::Merge, resolver);
        
        assert_eq!(resolution.resolution_type, ResolutionType::Merge);
        assert_eq!(resolution.resolved_by, resolver);
        assert!(resolution.notes.is_none());
    }

    #[test]
    fn test_merge_resolution() {
        let resolver = Uuid::new_v4();
        let value = FactValue::text("merged value");
        
        let resolution = ContradictionResolution::merge(resolver, value.clone());
        
        assert_eq!(resolution.resolution_type, ResolutionType::Merge);
        assert_eq!(resolution.chosen_value, Some(value));
    }

    #[test]
    fn test_deprecate_resolution() {
        let resolver = Uuid::new_v4();
        let fact_ids = vec![Uuid::new_v4(), Uuid::new_v4()];
        
        let resolution = ContradictionResolution::deprecate(
            resolver,
            fact_ids.clone(),
            "Outdated information"
        );
        
        assert_eq!(resolution.resolution_type, ResolutionType::Deprecate);
        assert_eq!(resolution.affected_fact_ids, fact_ids);
        assert_eq!(resolution.notes, Some("Outdated information".to_string()));
    }

    #[test]
    fn test_resolution_type_properties() {
        assert!(ResolutionType::Merge.requires_chosen_value());
        assert!(ResolutionType::Override.requires_chosen_value());
        assert!(!ResolutionType::Split.requires_chosen_value());
        assert!(!ResolutionType::Deprecate.requires_chosen_value());
        
        assert!(ResolutionType::Merge.preserves_all_facts());
        assert!(ResolutionType::Split.preserves_all_facts());
        assert!(!ResolutionType::Override.preserves_all_facts());
        assert!(!ResolutionType::Deprecate.preserves_all_facts());
    }

    #[test]
    fn test_default_resolutions() {
        assert_eq!(
            ResolutionType::default_for(ContradictionType::Temporal),
            ResolutionType::Split
        );
        assert_eq!(
            ResolutionType::default_for(ContradictionType::Factual),
            ResolutionType::Override
        );
        assert_eq!(
            ResolutionType::default_for(ContradictionType::SourceConflict),
            ResolutionType::Merge
        );
    }

    #[test]
    fn test_resolution_policy() {
        let policy = ResolutionPolicy::new("test_policy")
            .with_threshold(0.85)
            .with_source_priority(vec!["manual".to_string(), "auto".to_string()]);
        
        assert_eq!(policy.auto_resolve_threshold, 0.85);
        assert_eq!(policy.source_priority_score("manual"), 0);
        assert_eq!(policy.source_priority_score("auto"), 1);
        assert_eq!(policy.source_priority_score("unknown"), 2);
    }

    #[test]
    fn test_auto_resolve_decision() {
        let policy = ResolutionPolicy::new("test")
            .with_threshold(0.8);
        
        assert!(policy.should_auto_resolve(ContradictionSeverity::Low, 0.9));
        assert!(policy.should_auto_resolve(ContradictionSeverity::Medium, 0.85));
        assert!(!policy.should_auto_resolve(ContradictionSeverity::Low, 0.7));
        assert!(!policy.should_auto_resolve(ContradictionSeverity::Critical, 0.95));
    }

    #[test]
    fn test_resolution_stats() {
        let mut stats = ResolutionStats::new();
        
        let resolution = ContradictionResolution::new(
            ResolutionType::Override,
            Uuid::new_v4(),
        );
        
        stats.add_resolution(&resolution, false);
        stats.add_resolution(&resolution, true);
        
        assert_eq!(stats.total_resolutions, 2);
        assert_eq!(stats.manually_resolved, 1);
        assert_eq!(stats.auto_resolved, 1);
        assert_eq!(stats.most_common_type(), Some(ResolutionType::Override));
    }

    #[test]
    fn test_resolution_summary() {
        let resolver = Uuid::new_v4();
        let resolution = ContradictionResolution::new(
            ResolutionType::Merge,
            resolver,
        ).with_notes("Combined values from both sources");
        
        let summary = resolution.summary();
        assert!(summary.contains("Merge"));
        assert!(summary.contains(&resolver.to_string()));
        assert!(summary.contains("Combined values"));
    }
}
