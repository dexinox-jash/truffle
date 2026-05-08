//! Decision Model
//!
//! Tracks decisions made across meetings with versioning.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// A decision tracked across meetings
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Decision {
    /// Unique identifier
    pub id: Uuid,
    
    /// Decision text
    pub decision_text: String,
    
    /// Summary
    pub summary: Option<String>,
    
    /// Version number
    pub version: i32,
    
    /// Previous version ID (for tracking evolution)
    pub previous_version_id: Option<Uuid>,
    
    /// Decision chain ID (groups versions of same decision)
    pub decision_chain_id: Uuid,
    
    /// Status
    pub status: String, // proposed, approved, rejected, superseded, implemented, abandoned
    
    /// Person who proposed
    pub proposed_by_id: Option<Uuid>,
    
    /// Persons who approved (JSON array)
    pub decided_by_ids: Option<String>,
    
    /// Persons who dissented (JSON array)
    pub dissenting_ids: Option<String>,
    
    /// Meeting where decided
    pub decided_in_meeting_id: Option<Uuid>,
    
    /// When proposed
    pub proposed_at: Option<DateTime<Utc>>,
    
    /// When decided
    pub decided_at: Option<DateTime<Utc>>,
    
    /// When implemented
    pub implemented_at: Option<DateTime<Utc>>,
    
    /// Impact score (0.0 - 1.0)
    pub impact_score: Option<f32>,
    
    /// Dependent decision IDs (JSON array)
    pub dependent_decision_ids: Option<String>,
    
    /// Created at
    pub created_at: DateTime<Utc>,
    
    /// Updated at
    pub updated_at: DateTime<Utc>,
}

impl Decision {
    /// Create a new decision
    pub fn new(decision_text: impl Into<String>, meeting_id: Uuid) -> Self {
        let chain_id = Uuid::new_v4();
        Self {
            id: Uuid::new_v4(),
            decision_text: decision_text.into(),
            summary: None,
            version: 1,
            previous_version_id: None,
            decision_chain_id: chain_id,
            status: "proposed".to_string(),
            proposed_by_id: None,
            decided_by_ids: None,
            dissenting_ids: None,
            decided_in_meeting_id: Some(meeting_id),
            proposed_at: Some(Utc::now()),
            decided_at: None,
            implemented_at: None,
            impact_score: None,
            dependent_decision_ids: None,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        }
    }
    
    /// Create a new version of an existing decision
    pub fn new_version(&self, new_text: impl Into<String>) -> Self {
        Self {
            id: Uuid::new_v4(),
            decision_text: new_text.into(),
            summary: self.summary.clone(),
            version: self.version + 1,
            previous_version_id: Some(self.id),
            decision_chain_id: self.decision_chain_id,
            status: "proposed".to_string(),
            proposed_by_id: self.proposed_by_id,
            decided_by_ids: None,
            dissenting_ids: None,
            decided_in_meeting_id: self.decided_in_meeting_id,
            proposed_at: Some(Utc::now()),
            decided_at: None,
            implemented_at: None,
            impact_score: self.impact_score,
            dependent_decision_ids: self.dependent_decision_ids.clone(),
            created_at: Utc::now(),
            updated_at: Utc::now(),
        }
    }
    
    /// Mark as approved
    pub fn approve(&mut self, approver_ids: Vec<Uuid>) {
        self.status = "approved".to_string();
        self.decided_by_ids = Some(
            serde_json::to_string(&approver_ids).unwrap_or_default()
        );
        self.decided_at = Some(Utc::now());
        self.updated_at = Utc::now();
    }
    
    /// Mark as implemented
    pub fn implement(&mut self) {
        self.status = "implemented".to_string();
        self.implemented_at = Some(Utc::now());
        self.updated_at = Utc::now();
    }
    
    /// Mark as superseded
    pub fn supersede(&mut self) {
        self.status = "superseded".to_string();
        self.updated_at = Utc::now();
    }
}

/// Decision status enum
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum DecisionStatus {
    Proposed,
    Approved,
    Rejected,
    Superseded,
    Implemented,
    Abandoned,
}

impl DecisionStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Proposed => "proposed",
            Self::Approved => "approved",
            Self::Rejected => "rejected",
            Self::Superseded => "superseded",
            Self::Implemented => "implemented",
            Self::Abandoned => "abandoned",
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_decision_creation() {
        let meeting_id = Uuid::new_v4();
        let decision = Decision::new("Use PostgreSQL", meeting_id);
        
        assert_eq!(decision.decision_text, "Use PostgreSQL");
        assert_eq!(decision.version, 1);
        assert_eq!(decision.status, "proposed");
    }
    
    #[test]
    fn test_decision_versioning() {
        let meeting_id = Uuid::new_v4();
        let v1 = Decision::new("Use PostgreSQL", meeting_id);
        let v2 = v1.new_version("Use PostgreSQL 15");
        
        assert_eq!(v2.version, 2);
        assert_eq!(v2.previous_version_id, Some(v1.id));
        assert_eq!(v2.decision_chain_id, v1.decision_chain_id);
    }
    
    #[test]
    fn test_decision_approval() {
        let meeting_id = Uuid::new_v4();
        let mut decision = Decision::new("Use PostgreSQL", meeting_id);
        
        decision.approve(vec![Uuid::new_v4()]);
        
        assert_eq!(decision.status, "approved");
        assert!(decision.decided_at.is_some());
    }
}
