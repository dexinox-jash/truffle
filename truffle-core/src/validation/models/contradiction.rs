//! Contradiction Model
//!
//! Represents a detected contradiction between facts in the knowledge graph.
//! Contradictions occur when two or more facts about the same entity
//! cannot both be true simultaneously.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use super::ContradictionResolution;

/// A detected contradiction between facts
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Contradiction {
    /// Unique identifier
    pub id: Uuid,

    /// Type of contradiction
    pub contradiction_type: ContradictionType,

    /// Entity involved in the contradiction
    pub entity_id: Uuid,

    /// First conflicting fact
    pub fact_a_id: Uuid,

    /// Second conflicting fact
    pub fact_b_id: Uuid,

    /// Severity level
    pub severity: ContradictionSeverity,

    /// When the contradiction was detected
    pub detected_at: DateTime<Utc>,

    /// Resolution details (if resolved)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub resolution: Option<ContradictionResolution>,

    /// Human-readable description of the contradiction
    pub description: String,

    /// Optional metadata about the detection
    #[serde(skip_serializing_if = "Option::is_none")]
    pub detection_metadata: Option<serde_json::Value>,
}

impl Contradiction {
    /// Create a new contradiction
    pub fn new(
        entity_id: Uuid,
        fact_a_id: Uuid,
        fact_b_id: Uuid,
        contradiction_type: ContradictionType,
    ) -> Self {
        Self {
            id: Uuid::new_v4(),
            contradiction_type,
            entity_id,
            fact_a_id,
            fact_b_id,
            severity: ContradictionSeverity::Medium,
            detected_at: Utc::now(),
            resolution: None,
            description: String::new(),
            detection_metadata: None,
        }
    }

    /// Set the severity
    pub fn with_severity(mut self, severity: ContradictionSeverity) -> Self {
        self.severity = severity;
        self
    }

    /// Set the description
    pub fn with_description(mut self, description: impl Into<String>) -> Self {
        self.description = description.into();
        self
    }

    /// Set detection metadata
    pub fn with_metadata(mut self, metadata: serde_json::Value) -> Self {
        self.detection_metadata = Some(metadata);
        self
    }

    /// Mark as resolved
    pub fn resolve(&mut self, resolution: ContradictionResolution) {
        self.resolution = Some(resolution);
    }

    /// Check if this contradiction has been resolved
    pub fn is_resolved(&self) -> bool {
        self.resolution.is_some()
    }

    /// Get the resolution type (if resolved)
    pub fn resolution_type(&self) -> Option<super::ResolutionType> {
        self.resolution.as_ref().map(|r| r.resolution_type)
    }

    /// Calculate severity based on fact confidence and type
    pub fn calculate_severity(confidence_a: f32, confidence_b: f32, conflict_type: &ContradictionType) -> ContradictionSeverity {
        let avg_confidence = (confidence_a + confidence_b) / 2.0;
        
        match conflict_type {
            ContradictionType::Temporal => {
                // Temporal contradictions are often resolvable
                if avg_confidence > 0.9 {
                    ContradictionSeverity::Medium
                } else if avg_confidence > 0.7 {
                    ContradictionSeverity::Low
                } else {
                    ContradictionSeverity::Low
                }
            }
            ContradictionType::Factual => {
                // Factual contradictions are more serious
                if avg_confidence > 0.9 {
                    ContradictionSeverity::High
                } else if avg_confidence > 0.7 {
                    ContradictionSeverity::Medium
                } else {
                    ContradictionSeverity::Low
                }
            }
            ContradictionType::SourceConflict => {
                // Source conflicts depend on confidence
                if avg_confidence > 0.9 {
                    ContradictionSeverity::Critical
                } else if avg_confidence > 0.8 {
                    ContradictionSeverity::High
                } else if avg_confidence > 0.6 {
                    ContradictionSeverity::Medium
                } else {
                    ContradictionSeverity::Low
                }
            }
        }
    }
}

/// Types of contradictions that can occur
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Hash)]
#[serde(rename_all = "snake_case")]
pub enum ContradictionType {
    /// Facts conflict due to temporal overlap
    /// (e.g., worked at two companies simultaneously)
    Temporal,

    /// Facts have directly conflicting values
    /// (e.g., different locations at the same time)
    Factual,

    /// Same fact from different sources with different values
    /// (e.g., CRM says X, meeting notes say Y)
    SourceConflict,
}

impl ContradictionType {
    /// Get a human-readable name
    pub fn display_name(&self) -> &'static str {
        match self {
            Self::Temporal => "Temporal Conflict",
            Self::Factual => "Factual Conflict",
            Self::SourceConflict => "Source Conflict",
        }
    }

    /// Get a description of this contradiction type
    pub fn description(&self) -> &'static str {
        match self {
            Self::Temporal => "Facts overlap in time but should be mutually exclusive",
            Self::Factual => "Facts have directly conflicting values",
            Self::SourceConflict => "Same information from different sources differs",
        }
    }

    /// Check if this type of contradiction can be auto-resolved
    pub fn can_auto_resolve(&self) -> bool {
        match self {
            Self::Temporal => true,  // Often can be resolved with better time bounds
            Self::Factual => false,  // Usually requires human judgment
            Self::SourceConflict => true,  // Can sometimes be resolved by source priority
        }
    }
}

impl std::fmt::Display for ContradictionType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.display_name())
    }
}

/// Severity levels for contradictions
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Hash, PartialOrd, Ord)]
#[serde(rename_all = "snake_case")]
pub enum ContradictionSeverity {
    /// Minor issue, may not require immediate attention
    Low,
    /// Moderate issue, should be reviewed
    Medium,
    /// Serious issue, requires prompt resolution
    High,
    /// Critical issue, blocks operations
    Critical,
}

impl ContradictionSeverity {
    /// Get a human-readable name
    pub fn display_name(&self) -> &'static str {
        match self {
            Self::Low => "Low",
            Self::Medium => "Medium",
            Self::High => "High",
            Self::Critical => "Critical",
        }
    }

    /// Get a color code for UI display
    pub fn color(&self) -> &'static str {
        match self {
            Self::Low => "#6b7280",      // Gray
            Self::Medium => "#f59e0b",   // Amber
            Self::High => "#ef4444",     // Red
            Self::Critical => "#dc2626", // Dark red
        }
    }

    /// Check if this severity requires immediate attention
    pub fn requires_immediate_attention(&self) -> bool {
        matches!(self, Self::High | Self::Critical)
    }

    /// Check if this severity blocks data operations
    pub fn blocks_operations(&self) -> bool {
        matches!(self, Self::Critical)
    }
}

impl Default for ContradictionSeverity {
    fn default() -> Self {
        Self::Medium
    }
}

impl std::fmt::Display for ContradictionSeverity {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.display_name())
    }
}

/// Summary statistics for contradictions
#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq)]
pub struct ContradictionSummary {
    /// Total number of contradictions
    pub total: usize,
    /// Number by type
    pub by_type: std::collections::HashMap<ContradictionType, usize>,
    /// Number by severity
    pub by_severity: std::collections::HashMap<ContradictionSeverity, usize>,
    /// Number unresolved
    pub unresolved: usize,
    /// Number resolved
    pub resolved: usize,
    /// Number requiring immediate attention
    pub requiring_attention: usize,
}

impl ContradictionSummary {
    /// Create a new empty summary
    pub fn new() -> Self {
        Self::default()
    }

    /// Add a contradiction to the summary
    pub fn add(&mut self, contradiction: &Contradiction) {
        self.total += 1;
        
        *self.by_type.entry(contradiction.contradiction_type).or_insert(0) += 1;
        *self.by_severity.entry(contradiction.severity).or_insert(0) += 1;
        
        if contradiction.is_resolved() {
            self.resolved += 1;
        } else {
            self.unresolved += 1;
            if contradiction.severity.requires_immediate_attention() {
                self.requiring_attention += 1;
            }
        }
    }

    /// Check if there are any critical contradictions
    pub fn has_critical(&self) -> bool {
        self.by_severity.get(&ContradictionSeverity::Critical).copied().unwrap_or(0) > 0
    }

    /// Get the most common contradiction type
    pub fn most_common_type(&self) -> Option<ContradictionType> {
        self.by_type.iter()
            .max_by_key(|(_, count)| *count)
            .map(|(t, _)| *t)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_contradiction_creation() {
        let entity_id = Uuid::new_v4();
        let fact_a = Uuid::new_v4();
        let fact_b = Uuid::new_v4();
        
        let contradiction = Contradiction::new(
            entity_id,
            fact_a,
            fact_b,
            ContradictionType::Factual,
        );
        
        assert_eq!(contradiction.entity_id, entity_id);
        assert_eq!(contradiction.fact_a_id, fact_a);
        assert_eq!(contradiction.fact_b_id, fact_b);
        assert_eq!(contradiction.contradiction_type, ContradictionType::Factual);
        assert!(!contradiction.is_resolved());
    }

    #[test]
    fn test_contradiction_severity() {
        let entity_id = Uuid::new_v4();
        
        let contradiction = Contradiction::new(
            entity_id,
            Uuid::new_v4(),
            Uuid::new_v4(),
            ContradictionType::Factual,
        ).with_severity(ContradictionSeverity::High);
        
        assert_eq!(contradiction.severity, ContradictionSeverity::High);
        assert!(contradiction.severity.requires_immediate_attention());
    }

    #[test]
    fn test_severity_calculation() {
        // High confidence factual = High severity
        let severity = Contradiction::calculate_severity(
            0.95,
            0.95,
            &ContradictionType::Factual,
        );
        assert_eq!(severity, ContradictionSeverity::High);
        
        // Low confidence factual = Low severity
        let severity = Contradiction::calculate_severity(
            0.5,
            0.6,
            &ContradictionType::Factual,
        );
        assert_eq!(severity, ContradictionSeverity::Low);
        
        // High confidence source conflict = Critical
        let severity = Contradiction::calculate_severity(
            0.95,
            0.95,
            &ContradictionType::SourceConflict,
        );
        assert_eq!(severity, ContradictionSeverity::Critical);
    }

    #[test]
    fn test_contradiction_type_properties() {
        assert!(ContradictionType::Temporal.can_auto_resolve());
        assert!(!ContradictionType::Factual.can_auto_resolve());
        assert!(ContradictionType::SourceConflict.can_auto_resolve());
        
        assert_eq!(ContradictionType::Factual.display_name(), "Factual Conflict");
    }

    #[test]
    fn test_severity_ordering() {
        assert!(ContradictionSeverity::Low < ContradictionSeverity::Medium);
        assert!(ContradictionSeverity::Medium < ContradictionSeverity::High);
        assert!(ContradictionSeverity::High < ContradictionSeverity::Critical);
    }

    #[test]
    fn test_contradiction_summary() {
        let mut summary = ContradictionSummary::new();
        
        let c1 = Contradiction::new(
            Uuid::new_v4(),
            Uuid::new_v4(),
            Uuid::new_v4(),
            ContradictionType::Factual,
        ).with_severity(ContradictionSeverity::High);
        
        let c2 = Contradiction::new(
            Uuid::new_v4(),
            Uuid::new_v4(),
            Uuid::new_v4(),
            ContradictionType::Temporal,
        ).with_severity(ContradictionSeverity::Low);
        
        summary.add(&c1);
        summary.add(&c2);
        
        assert_eq!(summary.total, 2);
        assert_eq!(summary.unresolved, 2);
        assert_eq!(summary.requiring_attention, 1);
        assert!(!summary.has_critical());
    }
}
