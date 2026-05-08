//! Relationship Model
//!
//! Knowledge graph relationships (edges) connecting entities.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// A relationship between two entities in the knowledge graph
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Relationship {
    /// Unique identifier
    pub id: Uuid,
    
    /// Source entity ID
    pub source_id: Uuid,
    
    /// Target entity ID
    pub target_id: Uuid,
    
    /// Relationship type (e.g., "works_at", "located_in", "knows")
    pub relation_type: String,
    
    /// Additional properties for the relationship
    #[serde(skip_serializing_if = "Option::is_none")]
    pub properties: Option<serde_json::Value>,
    
    /// When the relationship was first observed
    pub first_seen_at: DateTime<Utc>,
    
    /// When the relationship was last updated
    pub last_updated_at: DateTime<Utc>,
    
    /// Source meeting ID where relationship was extracted
    #[serde(skip_serializing_if = "Option::is_none")]
    pub source_meeting_id: Option<Uuid>,
    
    /// Extraction confidence (0.0 - 1.0)
    pub extraction_confidence: f32,
    
    /// Whether relationship has been manually verified
    pub verified: bool,
    
    /// Validity period start (for temporal relationships)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub valid_from: Option<DateTime<Utc>>,
    
    /// Validity period end (for temporal relationships)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub valid_until: Option<DateTime<Utc>>,
    
    /// Whether this relationship is deprecated/archived
    pub archived: bool,
    
    /// Reason for archiving (if archived)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub archive_reason: Option<String>,
}

impl Relationship {
    /// Create a new relationship
    pub fn new(
        source_id: Uuid,
        target_id: Uuid,
        relation_type: impl Into<String>,
        source_meeting_id: Option<Uuid>,
    ) -> Self {
        Self {
            id: Uuid::new_v4(),
            source_id,
            target_id,
            relation_type: relation_type.into(),
            properties: None,
            first_seen_at: Utc::now(),
            last_updated_at: Utc::now(),
            source_meeting_id,
            extraction_confidence: 0.8,
            verified: false,
            valid_from: None,
            valid_until: None,
            archived: false,
            archive_reason: None,
        }
    }
    
    /// Create a relationship with a specific ID (for deterministic testing)
    pub fn with_id(
        id: Uuid,
        source_id: Uuid,
        target_id: Uuid,
        relation_type: impl Into<String>,
    ) -> Self {
        Self {
            id,
            source_id,
            target_id,
            relation_type: relation_type.into(),
            properties: None,
            first_seen_at: Utc::now(),
            last_updated_at: Utc::now(),
            source_meeting_id: None,
            extraction_confidence: 0.8,
            verified: false,
            valid_from: None,
            valid_until: None,
            archived: false,
            archive_reason: None,
        }
    }
    
    /// Set properties
    pub fn with_properties(mut self, properties: serde_json::Value) -> Self {
        self.properties = Some(properties);
        self
    }
    
    /// Set confidence
    pub fn with_confidence(mut self, confidence: f32) -> Self {
        self.extraction_confidence = confidence.clamp(0.0, 1.0);
        self
    }
    
    /// Set validity period
    pub fn with_validity(
        mut self,
        from: DateTime<Utc>,
        until: Option<DateTime<Utc>>,
    ) -> Self {
        self.valid_from = Some(from);
        self.valid_until = until;
        self
    }
    
    /// Mark as verified
    pub fn verify(&mut self) {
        self.verified = true;
        self.last_updated_at = Utc::now();
    }
    
    /// Archive this relationship
    pub fn archive(&mut self, reason: impl Into<String>) {
        self.archived = true;
        self.archive_reason = Some(reason.into());
        self.last_updated_at = Utc::now();
    }
    
    /// Check if relationship is currently valid
    pub fn is_valid_at(&self, time: DateTime<Utc>) -> bool {
        if self.archived {
            return false;
        }
        
        let after_start = self.valid_from.map(|t| time >= t).unwrap_or(true);
        let before_end = self.valid_until.map(|t| time < t).unwrap_or(true);
        
        after_start && before_end
    }
    
    /// Check if this relationship is effectively the same as another
    /// (same source, target, and type)
    pub fn is_equivalent_to(&self, other: &Relationship) -> bool {
        self.source_id == other.source_id
            && self.target_id == other.target_id
            && self.relation_type == other.relation_type
    }
    
    /// Create inverse relationship
    pub fn inverse(&self, inverse_type: impl Into<String>) -> Self {
        let mut inverse = Self::new(
            self.target_id,
            self.source_id,
            inverse_type,
            self.source_meeting_id,
        );
        inverse.properties = self.properties.clone();
        inverse.extraction_confidence = self.extraction_confidence;
        inverse
    }
}

/// Relationship direction for queries
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum RelationshipDirection {
    /// Outgoing from source
    Outgoing,
    /// Incoming to target
    Incoming,
    /// Both directions
    Both,
}

impl Default for RelationshipDirection {
    fn default() -> Self {
        Self::Both
    }
}

/// Relationship type metadata
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct RelationshipType {
    /// Type name
    pub name: String,
    /// Display label
    pub label: String,
    /// Inverse relationship type (if applicable)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub inverse: Option<String>,
    /// Whether this relationship is symmetric
    pub symmetric: bool,
    /// Description
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
}

impl RelationshipType {
    /// Create a new relationship type
    pub fn new(name: impl Into<String>, label: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            label: label.into(),
            inverse: None,
            symmetric: false,
            description: None,
        }
    }
    
    /// Set inverse type
    pub fn with_inverse(mut self, inverse: impl Into<String>) -> Self {
        self.inverse = Some(inverse.into());
        self.symmetric = false;
        self
    }
    
    /// Mark as symmetric
    pub fn symmetric(mut self) -> Self {
        self.symmetric = true;
        self.inverse = None;
        self
    }
    
    /// Set description
    pub fn with_description(mut self, description: impl Into<String>) -> Self {
        self.description = Some(description.into());
        self
    }
}

/// Common relationship types
pub mod relation_types {
    use super::RelationshipType;
    
    /// Person works at Organization
    pub fn works_at() -> RelationshipType {
        RelationshipType::new("works_at", "works at")
            .with_inverse("employs")
    }
    
    /// Person knows Person
    pub fn knows() -> RelationshipType {
        RelationshipType::new("knows", "knows").symmetric()
    }
    
    /// Organization located in Location
    pub fn located_in() -> RelationshipType {
        RelationshipType::new("located_in", "located in")
    }
    
    /// Event involves Entity
    pub fn involves() -> RelationshipType {
        RelationshipType::new("involves", "involves")
    }
    
    /// Entity mentioned in Meeting
    pub fn mentioned_in() -> RelationshipType {
        RelationshipType::new("mentioned_in", "mentioned in")
    }
    
    /// Decision relates to Entity
    pub fn relates_to() -> RelationshipType {
        RelationshipType::new("relates_to", "relates to")
            .symmetric()
    }
    
    /// ActionItem assigned to Person
    pub fn assigned_to() -> RelationshipType {
        RelationshipType::new("assigned_to", "assigned to")
    }
    
    /// Person created/owns Entity
    pub fn created() -> RelationshipType {
        RelationshipType::new("created", "created")
            .with_inverse("created_by")
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_relationship_creation() {
        let source_id = Uuid::new_v4();
        let target_id = Uuid::new_v4();
        
        let rel = Relationship::new(
            source_id,
            target_id,
            "works_at",
            None,
        );
        
        assert_eq!(rel.source_id, source_id);
        assert_eq!(rel.target_id, target_id);
        assert_eq!(rel.relation_type, "works_at");
        assert!(!rel.verified);
        assert!(!rel.archived);
    }
    
    #[test]
    fn test_relationship_equivalence() {
        let source_id = Uuid::new_v4();
        let target_id = Uuid::new_v4();
        
        let rel1 = Relationship::new(
            source_id,
            target_id,
            "works_at",
            None,
        );
        
        let rel2 = Relationship::new(
            source_id,
            target_id,
            "works_at",
            None,
        );
        
        let rel3 = Relationship::new(
            target_id,
            source_id,
            "works_at",
            None,
        );
        
        assert!(rel1.is_equivalent_to(&rel2));
        assert!(!rel1.is_equivalent_to(&rel3)); // Different direction
    }
    
    #[test]
    fn test_relationship_validity() {
        let rel = Relationship::new(Uuid::new_v4(), Uuid::new_v4(), "knows", None);
        
        assert!(rel.is_valid_at(Utc::now()));
        
        let mut archived = rel.clone();
        archived.archive("duplicate");
        assert!(!archived.is_valid_at(Utc::now()));
    }
    
    #[test]
    fn test_relationship_type() {
        let rel_type = RelationshipType::new("works_at", "works at")
            .with_inverse("employs")
            .with_description("Employment relationship");
        
        assert_eq!(rel_type.name, "works_at");
        assert_eq!(rel_type.label, "works at");
        assert_eq!(rel_type.inverse, Some("employs".to_string()));
        assert!(!rel_type.symmetric);
    }
    
    #[test]
    fn test_symmetric_relationship() {
        let rel_type = RelationshipType::new("knows", "knows").symmetric();
        
        assert!(rel_type.symmetric);
        assert!(rel_type.inverse.is_none());
    }
    
    #[test]
    fn test_relationship_inverse() {
        let source_id = Uuid::new_v4();
        let target_id = Uuid::new_v4();
        
        let rel = Relationship::new(source_id, target_id, "works_at", None)
            .with_confidence(0.9);
        
        let inverse = rel.inverse("employs");
        
        assert_eq!(inverse.source_id, target_id);
        assert_eq!(inverse.target_id, source_id);
        assert_eq!(inverse.relation_type, "employs");
        assert_eq!(inverse.extraction_confidence, 0.9);
    }
}
