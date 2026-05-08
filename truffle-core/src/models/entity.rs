//! Entity Model
//!
//! Knowledge graph entities (nodes) representing people, organizations,
//! locations, events, products, concepts, decisions, and action items.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// An entity in the knowledge graph
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Entity {
    /// Unique identifier (content-addressable UUID)
    pub id: Uuid,
    
    /// Entity type
    pub entity_type: EntityType,
    
    /// Display name
    pub name: String,
    
    /// URL-friendly identifier
    pub slug: String,
    
    /// Description
    pub description: Option<String>,
    
    /// Type-specific metadata
    pub metadata: EntityMetadata,
    
    /// Vector embedding for semantic search (384-dim MiniLM)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub embedding: Option<Vec<f32>>,
    
    /// When entity was first observed
    pub first_seen_at: DateTime<Utc>,
    
    /// When entity was last updated
    pub last_updated_at: DateTime<Utc>,
    
    /// Source meeting ID
    #[serde(skip_serializing_if = "Option::is_none")]
    pub source_meeting_id: Option<Uuid>,
    
    /// Source chunk ID within transcript
    #[serde(skip_serializing_if = "Option::is_none")]
    pub source_chunk_id: Option<Uuid>,
    
    /// Extraction confidence (0.0 - 1.0)
    pub extraction_confidence: f32,
    
    /// Whether entity has been manually verified
    pub verified: bool,
    
    /// Privacy classification
    pub privacy_level: PrivacyLevel,
}

impl Entity {
    /// Create a new entity
    pub fn new(
        name: impl Into<String>,
        entity_type: EntityType,
        source_meeting_id: Option<Uuid>,
    ) -> Self {
        let name = name.into();
        let slug = slugify(&name);
        let id = crate::models::content_addressable_uuid(format!("{}:{}", entity_type.as_str(), &name).as_bytes());
        
        Self {
            id,
            entity_type,
            name,
            slug,
            description: None,
            metadata: EntityMetadata::default(),
            embedding: None,
            first_seen_at: Utc::now(),
            last_updated_at: Utc::now(),
            source_meeting_id,
            source_chunk_id: None,
            extraction_confidence: 0.8,
            verified: false,
            privacy_level: PrivacyLevel::Internal,
        }
    }
    
    /// Generate a slug from name
    pub fn generate_slug(name: &str) -> String {
        slugify(name)
    }
    
    /// Update entity with verification
    pub fn verify(&mut self) {
        self.verified = true;
        self.last_updated_at = Utc::now();
    }
    
    /// Set description
    pub fn with_description(mut self, description: impl Into<String>) -> Self {
        self.description = Some(description.into());
        self
    }
    
    /// Set metadata
    pub fn with_metadata(mut self, metadata: EntityMetadata) -> Self {
        self.metadata = metadata;
        self
    }
    
    /// Set embedding
    pub fn with_embedding(mut self, embedding: Vec<f32>) -> Self {
        self.embedding = Some(embedding);
        self
    }
    
    /// Set confidence
    pub fn with_confidence(mut self, confidence: f32) -> Self {
        self.extraction_confidence = confidence.clamp(0.0, 1.0);
        self
    }
    
    /// Set privacy level
    pub fn with_privacy(mut self, level: PrivacyLevel) -> Self {
        self.privacy_level = level;
        self
    }
}

/// Entity types
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Hash)]
#[serde(rename_all = "snake_case")]
pub enum EntityType {
    Person,
    Organization,
    Location,
    Event,
    Product,
    Concept,
    Decision,
    ActionItem,
}

impl EntityType {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Person => "person",
            Self::Organization => "organization",
            Self::Location => "location",
            Self::Event => "event",
            Self::Product => "product",
            Self::Concept => "concept",
            Self::Decision => "decision",
            Self::ActionItem => "action_item",
        }
    }
    
    pub fn from_str(s: &str) -> Option<Self> {
        match s {
            "person" => Some(Self::Person),
            "organization" => Some(Self::Organization),
            "location" => Some(Self::Location),
            "event" => Some(Self::Event),
            "product" => Some(Self::Product),
            "concept" => Some(Self::Concept),
            "decision" => Some(Self::Decision),
            "action_item" => Some(Self::ActionItem),
            _ => None,
        }
    }
    
    /// Get display name
    pub fn display_name(&self) -> &'static str {
        match self {
            Self::Person => "Person",
            Self::Organization => "Organization",
            Self::Location => "Location",
            Self::Event => "Event",
            Self::Product => "Product",
            Self::Concept => "Concept",
            Self::Decision => "Decision",
            Self::ActionItem => "Action Item",
        }
    }
    
    /// Get icon for UI
    pub fn icon(&self) -> &'static str {
        match self {
            Self::Person => "👤",
            Self::Organization => "🏢",
            Self::Location => "📍",
            Self::Event => "📅",
            Self::Product => "📦",
            Self::Concept => "💡",
            Self::Decision => "✓",
            Self::ActionItem => "⚡",
        }
    }
}

impl Default for EntityType {
    fn default() -> Self {
        Self::Concept
    }
}

/// Privacy levels for entities
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum PrivacyLevel {
    Public,
    Internal,
    Confidential,
    Restricted,
}

impl PrivacyLevel {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Public => "public",
            Self::Internal => "internal",
            Self::Confidential => "confidential",
            Self::Restricted => "restricted",
        }
    }
}

impl Default for PrivacyLevel {
    fn default() -> Self {
        Self::Internal
    }
}

/// Entity metadata (type-specific attributes)
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
pub struct EntityMetadata {
    /// For Person: job title, email, phone
    #[serde(skip_serializing_if = "Option::is_none")]
    pub title: Option<String>,
    
    #[serde(skip_serializing_if = "Option::is_none")]
    pub email: Option<String>,
    
    #[serde(skip_serializing_if = "Option::is_none")]
    pub phone: Option<String>,
    
    /// For Organization: industry, size, website
    #[serde(skip_serializing_if = "Option::is_none")]
    pub industry: Option<String>,
    
    #[serde(skip_serializing_if = "Option::is_none")]
    pub website: Option<String>,
    
    /// For Location: address, coordinates
    #[serde(skip_serializing_if = "Option::is_none")]
    pub address: Option<String>,
    
    #[serde(skip_serializing_if = "Option::is_none")]
    pub latitude: Option<f64>,
    
    #[serde(skip_serializing_if = "Option::is_none")]
    pub longitude: Option<f64>,
    
    /// For Event: start/end times
    #[serde(skip_serializing_if = "Option::is_none")]
    pub event_start: Option<DateTime<Utc>>,
    
    #[serde(skip_serializing_if = "Option::is_none")]
    pub event_end: Option<DateTime<Utc>>,
    
    /// For Product: category, price
    #[serde(skip_serializing_if = "Option::is_none")]
    pub category: Option<String>,
    
    #[serde(skip_serializing_if = "Option::is_none")]
    pub price: Option<String>,
    
    /// For Decision: status, impact
    #[serde(skip_serializing_if = "Option::is_none")]
    pub decision_status: Option<String>,
    
    #[serde(skip_serializing_if = "Option::is_none")]
    pub impact: Option<String>,
    
    /// For ActionItem: deadline, assignee
    #[serde(skip_serializing_if = "Option::is_none")]
    pub deadline: Option<DateTime<Utc>>,
    
    #[serde(skip_serializing_if = "Option::is_none")]
    pub assignee: Option<String>,
    
    /// Additional custom fields
    #[serde(flatten)]
    pub extra: std::collections::HashMap<String, serde_json::Value>,
}

impl EntityMetadata {
    /// Create metadata for a person
    pub fn person(title: impl Into<String>) -> Self {
        Self {
            title: Some(title.into()),
            ..Default::default()
        }
    }
    
    /// Create metadata for an organization
    pub fn organization(industry: impl Into<String>) -> Self {
        Self {
            industry: Some(industry.into()),
            ..Default::default()
        }
    }
    
    /// Create metadata for a location
    pub fn location(lat: f64, lon: f64) -> Self {
        Self {
            latitude: Some(lat),
            longitude: Some(lon),
            ..Default::default()
        }
    }
}

/// Entity alias for resolution
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct EntityAlias {
    pub id: i64,
    pub entity_id: Uuid,
    pub alias: String,
    pub alias_type: AliasType,
    pub confidence: f32,
    pub created_at: DateTime<Utc>,
}

/// Types of aliases
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum AliasType {
    Acronym,
    Nickname,
    FullName,
    Misspelling,
    Translation,
}

impl AliasType {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Acronym => "acronym",
            Self::Nickname => "nickname",
            Self::FullName => "full_name",
            Self::Misspelling => "misspelling",
            Self::Translation => "translation",
        }
    }
}

/// Helper function to create a URL-friendly slug
fn slugify(name: &str) -> String {
    name.to_lowercase()
        .replace(|c: char| !c.is_alphanumeric() && c != ' ', "")
        .replace(' ', "-")
        .replace("--", "-")
        .trim_matches('-')
        .to_string()
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_entity_creation() {
        let entity = Entity::new("Alice Smith", EntityType::Person, None);
        
        assert_eq!(entity.name, "Alice Smith");
        assert_eq!(entity.slug, "alice-smith");
        assert_eq!(entity.entity_type, EntityType::Person);
        assert!(!entity.verified);
    }
    
    #[test]
    fn test_slugify() {
        assert_eq!(slugify("Hello World"), "hello-world");
        assert_eq!(slugify("  Spaces  "), "spaces");
        assert_eq!(slugify("Special!@#Chars"), "specialchars");
    }
    
    #[test]
    fn test_entity_type_roundtrip() {
        let types = vec![
            EntityType::Person,
            EntityType::Organization,
            EntityType::Location,
            EntityType::Event,
            EntityType::Product,
            EntityType::Concept,
            EntityType::Decision,
            EntityType::ActionItem,
        ];
        
        for ty in types {
            let s = ty.as_str();
            let parsed = EntityType::from_str(s).unwrap();
            assert_eq!(ty, parsed);
        }
    }
    
    #[test]
    fn test_entity_builder() {
        let entity = Entity::new("Test Corp", EntityType::Organization, None)
            .with_description("A test company")
            .with_confidence(0.95)
            .with_privacy(PrivacyLevel::Public);
        
        assert_eq!(entity.description, Some("A test company".to_string()));
        assert_eq!(entity.extraction_confidence, 0.95);
        assert_eq!(entity.privacy_level, PrivacyLevel::Public);
    }
    
    #[test]
    fn test_entity_verify() {
        let mut entity = Entity::new("Test", EntityType::Concept, None);
        assert!(!entity.verified);
        
        entity.verify();
        assert!(entity.verified);
    }
}
