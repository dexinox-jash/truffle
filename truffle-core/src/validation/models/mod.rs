//! Validation Models
//!
//! Core data structures for the Fact Consistency Checker.
//! These models represent facts, contradictions, and their resolutions
//! in the knowledge graph validation system.

pub mod contradiction;
pub mod fact;
pub mod resolution;

// Re-export all types for convenient access
pub use contradiction::{
    Contradiction,
    ContradictionSeverity,
    ContradictionSummary,
    ContradictionType,
};

pub use fact::{
    Fact,
    FactSource,
    FactValue,
};

pub use resolution::{
    ContradictionResolution,
    ResolutionPolicy,
    ResolutionStats,
    ResolutionType,
};

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Unique identifier for a validation batch
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct ValidationBatchId(pub Uuid);

impl ValidationBatchId {
    /// Create a new validation batch ID
    pub fn new() -> Self {
        Self(Uuid::new_v4())
    }
}

impl Default for ValidationBatchId {
    fn default() -> Self {
        Self::new()
    }
}

impl From<Uuid> for ValidationBatchId {
    fn from(id: Uuid) -> Self {
        Self(id)
    }
}

impl From<ValidationBatchId> for Uuid {
    fn from(id: ValidationBatchId) -> Self {
        id.0
    }
}

impl std::fmt::Display for ValidationBatchId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// Status of a validation operation
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ValidationStatus {
    /// Validation is pending
    Pending,
    /// Validation is in progress
    InProgress,
    /// Validation completed successfully with no issues
    Success,
    /// Validation completed with warnings
    Warning,
    /// Validation completed with contradictions found
    ContradictionsFound,
    /// Validation failed
    Failed,
}

impl ValidationStatus {
    /// Check if validation is complete
    pub fn is_complete(&self) -> bool {
        matches!(
            self,
            Self::Success | Self::Warning | Self::ContradictionsFound | Self::Failed
        )
    }

    /// Check if validation was successful (no critical issues)
    pub fn is_successful(&self) -> bool {
        matches!(self, Self::Success | Self::Warning)
    }

    /// Check if contradictions were found
    pub fn has_contradictions(&self) -> bool {
        matches!(self, Self::ContradictionsFound)
    }
}

impl Default for ValidationStatus {
    fn default() -> Self {
        Self::Pending
    }
}

/// Request to validate facts for an entity
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationRequest {
    /// Unique ID for this validation request
    pub id: Uuid,
    /// Entity to validate
    pub entity_id: Uuid,
    /// Specific attributes to validate (empty = all)
    pub attributes: Vec<String>,
    /// Time range to consider
    #[serde(skip_serializing_if = "Option::is_none")]
    pub time_range: Option<(DateTime<Utc>, DateTime<Utc>)>,
    /// When the request was created
    pub created_at: DateTime<Utc>,
    /// User or system that initiated the request
    pub requested_by: Uuid,
}

impl ValidationRequest {
    /// Create a new validation request
    pub fn new(entity_id: Uuid, requested_by: Uuid) -> Self {
        Self {
            id: Uuid::new_v4(),
            entity_id,
            attributes: Vec::new(),
            time_range: None,
            created_at: Utc::now(),
            requested_by,
        }
    }

    /// Set specific attributes to validate
    pub fn with_attributes(mut self, attributes: Vec<String>) -> Self {
        self.attributes = attributes;
        self
    }

    /// Set time range for validation
    pub fn with_time_range(mut self, from: DateTime<Utc>, until: DateTime<Utc>) -> Self {
        self.time_range = Some((from, until));
        self
    }
}

/// Result of a validation operation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationResult {
    /// ID of the validation request
    pub request_id: Uuid,
    /// Status of the validation
    pub status: ValidationStatus,
    /// Contradictions found (if any)
    pub contradictions: Vec<Contradiction>,
    /// Summary statistics
    pub summary: ValidationSummary,
    /// When validation completed
    pub completed_at: DateTime<Utc>,
    /// Duration of validation in milliseconds
    pub duration_ms: u64,
}

impl ValidationResult {
    /// Create a new successful validation result
    pub fn success(request_id: Uuid, duration_ms: u64) -> Self {
        Self {
            request_id,
            status: ValidationStatus::Success,
            contradictions: Vec::new(),
            summary: ValidationSummary::default(),
            completed_at: Utc::now(),
            duration_ms,
        }
    }

    /// Create a result with contradictions
    pub fn with_contradictions(
        request_id: Uuid,
        contradictions: Vec<Contradiction>,
        duration_ms: u64,
    ) -> Self {
        let summary = ValidationSummary {
            total_facts_checked: 0, // Would be set by validator
            contradictions_found: contradictions.len(),
            facts_with_issues: contradictions.iter().map(|c| c.entity_id).collect::<std::collections::HashSet<_>>().len(),
        };

        Self {
            request_id,
            status: ValidationStatus::ContradictionsFound,
            contradictions,
            summary,
            completed_at: Utc::now(),
            duration_ms,
        }
    }

    /// Check if this result contains any critical contradictions
    pub fn has_critical_issues(&self) -> bool {
        self.contradictions.iter().any(|c| {
            c.severity == ContradictionSeverity::Critical && !c.is_resolved()
        })
    }

    /// Get unresolved contradictions
    pub fn unresolved_contradictions(&self) -> Vec<&Contradiction> {
        self.contradictions.iter().filter(|c| !c.is_resolved()).collect()
    }
}

/// Summary statistics for a validation run
#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq)]
pub struct ValidationSummary {
    /// Total number of facts checked
    pub total_facts_checked: usize,
    /// Number of contradictions found
    pub contradictions_found: usize,
    /// Number of entities with issues
    pub facts_with_issues: usize,
}

impl ValidationSummary {
    /// Create a new summary
    pub fn new(total_facts: usize) -> Self {
        Self {
            total_facts_checked: total_facts,
            contradictions_found: 0,
            facts_with_issues: 0,
        }
    }

    /// Add a contradiction to the summary
    pub fn add_contradiction(&mut self) {
        self.contradictions_found += 1;
    }

    /// Mark facts as having issues
    pub fn mark_facts_with_issues(&mut self, count: usize) {
        self.facts_with_issues += count;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validation_batch_id() {
        let id = ValidationBatchId::new();
        let uuid: Uuid = id.into();
        let back = ValidationBatchId::from(uuid);
        assert_eq!(id, back);
    }

    #[test]
    fn test_validation_status() {
        assert!(!ValidationStatus::Pending.is_complete());
        assert!(!ValidationStatus::InProgress.is_complete());
        assert!(ValidationStatus::Success.is_complete());
        assert!(ValidationStatus::Failed.is_complete());
        
        assert!(ValidationStatus::Success.is_successful());
        assert!(ValidationStatus::Warning.is_successful());
        assert!(!ValidationStatus::ContradictionsFound.is_successful());
        assert!(!ValidationStatus::Failed.is_successful());
    }

    #[test]
    fn test_validation_request() {
        let entity_id = Uuid::new_v4();
        let requester = Uuid::new_v4();
        
        let request = ValidationRequest::new(entity_id, requester)
            .with_attributes(vec!["employer".to_string(), "title".to_string()]);
        
        assert_eq!(request.entity_id, entity_id);
        assert_eq!(request.attributes.len(), 2);
        assert!(request.time_range.is_none());
    }

    #[test]
    fn test_validation_result() {
        let request_id = Uuid::new_v4();
        let result = ValidationResult::success(request_id, 150);
        
        assert_eq!(result.status, ValidationStatus::Success);
        assert!(result.contradictions.is_empty());
        assert!(!result.has_critical_issues());
    }

    #[test]
    fn test_validation_summary() {
        let mut summary = ValidationSummary::new(100);
        assert_eq!(summary.total_facts_checked, 100);
        
        summary.add_contradiction();
        summary.add_contradiction();
        summary.mark_facts_with_issues(5);
        
        assert_eq!(summary.contradictions_found, 2);
        assert_eq!(summary.facts_with_issues, 5);
    }
}
