//! Integration Test Utilities and Setup
//!
//! This module provides shared test utilities, database setup, and builder
//! patterns for creating test data in the Truffle Core integration tests.

use std::sync::Arc;
use tokio::sync::RwLock;
use uuid::Uuid;
use chrono::{DateTime, Utc, Duration};

use truffle_core::{
    Database, DatabaseConfig,
    Entity, EntityType, EntityMetadata, PrivacyLevel,
    Relationship,
    GraphRepository, EntityRepository, RelationshipRepository,
    validation::{Fact, FactValue, FactSource},
};

/// Create a test database with in-memory storage
pub async fn setup_test_db() -> Arc<RwLock<Database>> {
    let db = Database::open_in_memory()
        .expect("Failed to create in-memory database");
    Arc::new(RwLock::new(db))
}

/// Create a test database with a temporary file
pub async fn setup_test_db_file() -> (Arc<RwLock<Database>>, tempfile::TempDir) {
    let temp_dir = tempfile::tempdir().expect("Failed to create temp directory");
    let config = DatabaseConfig::with_data_dir(temp_dir.path());
    let db = Database::open(config)
        .expect("Failed to create database");
    (Arc::new(RwLock::new(db)), temp_dir)
}

/// Builder for creating test entities
pub struct TestEntityBuilder {
    entity_type: EntityType,
    name: String,
    description: Option<String>,
    metadata: EntityMetadata,
    privacy_level: PrivacyLevel,
    confidence: f32,
    verified: bool,
    embedding: Option<Vec<f32>>,
}

impl TestEntityBuilder {
    /// Create a new person entity builder
    pub fn person(name: &str) -> Self {
        Self {
            entity_type: EntityType::Person,
            name: name.to_string(),
            description: None,
            metadata: EntityMetadata {
                title: Some("Employee".to_string()),
                ..Default::default()
            },
            privacy_level: PrivacyLevel::Internal,
            confidence: 0.85,
            verified: false,
            embedding: None,
        }
    }

    /// Create a new organization entity builder
    pub fn organization(name: &str) -> Self {
        Self {
            entity_type: EntityType::Organization,
            name: name.to_string(),
            description: Some(format!("{} organization", name)),
            metadata: EntityMetadata {
                industry: Some("Technology".to_string()),
                website: Some(format!("https://{}.com", name.to_lowercase().replace(" ", ""))),
                ..Default::default()
            },
            privacy_level: PrivacyLevel::Internal,
            confidence: 0.9,
            verified: false,
            embedding: None,
        }
    }

    /// Create a new location entity builder
    pub fn location(name: &str, lat: f64, lon: f64) -> Self {
        Self {
            entity_type: EntityType::Location,
            name: name.to_string(),
            description: None,
            metadata: EntityMetadata {
                latitude: Some(lat),
                longitude: Some(lon),
                ..Default::default()
            },
            privacy_level: PrivacyLevel::Public,
            confidence: 0.95,
            verified: false,
            embedding: None,
        }
    }

    /// Create a new event entity builder
    pub fn event(name: &str, start: DateTime<Utc>, end: DateTime<Utc>) -> Self {
        Self {
            entity_type: EntityType::Event,
            name: name.to_string(),
            description: Some(format!("Event: {}", name)),
            metadata: EntityMetadata {
                event_start: Some(start),
                event_end: Some(end),
                ..Default::default()
            },
            privacy_level: PrivacyLevel::Internal,
            confidence: 0.8,
            verified: false,
            embedding: None,
        }
    }

    /// Create a new product entity builder
    pub fn product(name: &str) -> Self {
        Self {
            entity_type: EntityType::Product,
            name: name.to_string(),
            description: Some(format!("Product: {}", name)),
            metadata: EntityMetadata {
                category: Some("Software".to_string()),
                ..Default::default()
            },
            privacy_level: PrivacyLevel::Public,
            confidence: 0.88,
            verified: false,
            embedding: None,
        }
    }

    /// Create a new concept entity builder
    pub fn concept(name: &str) -> Self {
        Self {
            entity_type: EntityType::Concept,
            name: name.to_string(),
            description: Some(format!("Concept: {}", name)),
            metadata: EntityMetadata::default(),
            privacy_level: PrivacyLevel::Public,
            confidence: 0.75,
            verified: false,
            embedding: None,
        }
    }

    /// Set the description
    pub fn with_description(mut self, description: impl Into<String>) -> Self {
        self.description = Some(description.into());
        self
    }

    /// Set metadata
    pub fn with_metadata(mut self, metadata: EntityMetadata) -> Self {
        self.metadata = metadata;
        self
    }

    /// Set role (for Person type)
    pub fn with_role(mut self, role: &str) -> Self {
        if self.entity_type == EntityType::Person {
            self.metadata.title = Some(role.to_string());
        }
        self
    }

    /// Set email (for Person type)
    pub fn with_email(mut self, email: &str) -> Self {
        if self.entity_type == EntityType::Person {
            self.metadata.email = Some(email.to_string());
        }
        self
    }

    /// Set industry (for Organization type)
    pub fn with_industry(mut self, industry: &str) -> Self {
        if self.entity_type == EntityType::Organization {
            self.metadata.industry = Some(industry.to_string());
        }
        self
    }

    /// Set coordinates (for Location type)
    pub fn with_coordinates(mut self, lat: f64, lon: f64) -> Self {
        if self.entity_type == EntityType::Location {
            self.metadata.latitude = Some(lat);
            self.metadata.longitude = Some(lon);
        }
        self
    }

    /// Set privacy level
    pub fn with_privacy(mut self, level: PrivacyLevel) -> Self {
        self.privacy_level = level;
        self
    }

    /// Set confidence score
    pub fn with_confidence(mut self, confidence: f32) -> Self {
        self.confidence = confidence.clamp(0.0, 1.0);
        self
    }

    /// Mark as verified
    pub fn verified(mut self) -> Self {
        self.verified = true;
        self
    }

    /// Set embedding vector
    pub fn with_embedding(mut self, embedding: Vec<f32>) -> Self {
        self.embedding = Some(embedding);
        self
    }

    /// Build the entity (in memory only, not persisted)
    pub fn build(self) -> Entity {
        let mut entity = Entity::new(
            self.name,
            self.entity_type,
            None, // source_meeting_id
        );

        if let Some(desc) = self.description {
            entity = entity.with_description(desc);
        }

        entity = entity
            .with_metadata(self.metadata)
            .with_privacy(self.privacy_level)
            .with_confidence(self.confidence);

        if self.verified {
            entity.verify();
        }

        if let Some(emb) = self.embedding {
            entity = entity.with_embedding(emb);
        }

        entity
    }
}

/// Builder for creating test relationships
pub struct TestRelationshipBuilder {
    source_id: Uuid,
    target_id: Uuid,
    relation_type: String,
    properties: Option<serde_json::Value>,
    confidence: f32,
    valid_from: Option<DateTime<Utc>>,
    valid_until: Option<DateTime<Utc>>,
    verified: bool,
}

impl TestRelationshipBuilder {
    /// Create a new relationship builder
    pub fn new(source_id: Uuid, target_id: Uuid, relation_type: impl Into<String>) -> Self {
        Self {
            source_id,
            target_id,
            relation_type: relation_type.into(),
            properties: None,
            confidence: 0.8,
            valid_from: None,
            valid_until: None,
            verified: false,
        }
    }

    /// Create a "works_at" relationship
    pub fn works_at(person_id: Uuid, org_id: Uuid) -> Self {
        Self::new(person_id, org_id, "works_at")
            .with_confidence(0.9)
    }

    /// Create a "knows" relationship
    pub fn knows(person_a_id: Uuid, person_b_id: Uuid) -> Self {
        Self::new(person_a_id, person_b_id, "knows")
            .with_confidence(0.75)
    }

    /// Create a "located_in" relationship
    pub fn located_in(entity_id: Uuid, location_id: Uuid) -> Self {
        Self::new(entity_id, location_id, "located_in")
            .with_confidence(0.95)
    }

    /// Create an "involves" relationship
    pub fn involves(event_id: Uuid, entity_id: Uuid) -> Self {
        Self::new(event_id, entity_id, "involves")
            .with_confidence(0.8)
    }

    /// Set properties
    pub fn with_properties(mut self, properties: serde_json::Value) -> Self {
        self.properties = Some(properties);
        self
    }

    /// Set confidence
    pub fn with_confidence(mut self, confidence: f32) -> Self {
        self.confidence = confidence.clamp(0.0, 1.0);
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

    /// Set validity period as employment dates
    pub fn with_employment_period(self, start: DateTime<Utc>, end: Option<DateTime<Utc>>) -> Self {
        self.with_validity(start, end)
            .with_properties(serde_json::json!({
                "employment_type": "full_time"
            }))
    }

    /// Mark as verified
    pub fn verified(mut self) -> Self {
        self.verified = true;
        self
    }

    /// Build the relationship
    pub fn build(self) -> Relationship {
        let mut rel = Relationship::new(
            self.source_id,
            self.target_id,
            self.relation_type,
            None,
        );

        rel = rel.with_confidence(self.confidence);

        if let Some(props) = self.properties {
            rel = rel.with_properties(props);
        }

        if let Some(from) = self.valid_from {
            rel = rel.with_validity(from, self.valid_until);
        }

        if self.verified {
            rel.verify();
        }

        rel
    }
}

/// Builder for creating test facts
pub struct TestFactBuilder {
    entity_id: Uuid,
    attribute: String,
    value: FactValue,
    source: FactSource,
    confidence: f32,
    valid_from: Option<DateTime<Utc>>,
    valid_until: Option<DateTime<Utc>>,
}

impl TestFactBuilder {
    /// Create a new fact builder
    pub fn new(entity_id: Uuid, attribute: impl Into<String>, value: FactValue) -> Self {
        Self {
            entity_id,
            attribute: attribute.into(),
            value,
            source: FactSource::manual(Uuid::new_v4()),
            confidence: 0.8,
            valid_from: None,
            valid_until: None,
        }
    }

    /// Create an employer fact
    pub fn employer(entity_id: Uuid, employer_name: impl Into<String>) -> Self {
        Self::new(entity_id, "employer", FactValue::text(employer_name))
    }

    /// Create a title fact
    pub fn title(entity_id: Uuid, title: impl Into<String>) -> Self {
        Self::new(entity_id, "title", FactValue::text(title))
    }

    /// Create a location fact
    pub fn location(entity_id: Uuid, location: impl Into<String>) -> Self {
        Self::new(entity_id, "location", FactValue::text(location))
    }

    /// Create a salary fact
    pub fn salary(entity_id: Uuid, amount: f64) -> Self {
        Self::new(entity_id, "salary", FactValue::number(amount))
    }

    /// Set source
    pub fn with_source(mut self, source: FactSource) -> Self {
        self.source = source;
        self
    }

    /// Set as extraction source
    pub fn from_extraction(mut self, pipeline_id: Uuid) -> Self {
        self.source = FactSource::extraction(pipeline_id);
        self
    }

    /// Set as verified manual source
    pub fn verified(mut self, user_id: Uuid) -> Self {
        self.source = FactSource::manual_verified(user_id);
        self.confidence = 1.0;
        self
    }

    /// Set confidence
    pub fn with_confidence(mut self, confidence: f32) -> Self {
        self.confidence = confidence.clamp(0.0, 1.0);
        self
    }

    /// Set validity range
    pub fn with_valid_range(
        mut self,
        from: Option<DateTime<Utc>>,
        until: Option<DateTime<Utc>>,
    ) -> Self {
        self.valid_from = from;
        self.valid_until = until;
        self
    }

    /// Build the fact
    pub fn build(self) -> Fact {
        let mut fact = Fact::new(
            self.entity_id,
            self.attribute,
            self.value,
            self.source,
        )
        .with_confidence(self.confidence);

        fact.valid_from = self.valid_from;
        fact.valid_until = self.valid_until;

        fact
    }
}

/// Generate a simple embedding vector for testing
pub fn generate_test_embedding(seed: u64, dim: usize) -> Vec<f32> {
    use std::f32::consts::PI;
    
    (0..dim)
        .map(|i| {
            let angle = 2.0 * PI * (seed as f32 + i as f32) / dim as f32;
            angle.sin()
        })
        .collect()
}

/// Generate similar embeddings with small variations
pub fn generate_similar_embeddings(base_seed: u64, count: usize, dim: usize, variance: f32) -> Vec<Vec<f32>> {
    let base = generate_test_embedding(base_seed, dim);
    
    (0..count)
        .map(|i| {
            base.iter()
                .enumerate()
                .map(|(j, &v)| {
                    let noise = ((i + j) as f32 * 0.1).sin() * variance;
                    (v + noise).clamp(-1.0, 1.0)
                })
                .collect()
        })
        .collect()
}

/// Assert that two entities are similar (for fuzzy matching tests)
pub fn assert_entities_similar(a: &Entity, b: &Entity, min_similarity: f32) {
    let name_sim = calculate_name_similarity(&a.name, &b.name);
    assert!(
        name_sim >= min_similarity,
        "Expected entities to be similar ({} >= {}), but {} vs {} = {}",
        name_sim, min_similarity, a.name, b.name, name_sim
    );
}

/// Calculate name similarity using Jaro-Winkler
fn calculate_name_similarity(name_a: &str, name_b: &str) -> f32 {
    let a = name_a.to_lowercase();
    let b = name_b.to_lowercase();

    if a == b {
        return 1.0;
    }

    // Simple substring match
    if a.contains(&b) || b.contains(&a) {
        let ratio = a.len().min(b.len()) as f32 / a.len().max(b.len()) as f32;
        return 0.8 + (0.2 * ratio);
    }

    // For more sophisticated matching, use jaro_winkler from the library
    // This is a simplified version for test assertions
    jaro_winkler_simple(&a, &b)
}

/// Simplified Jaro-Winkler similarity
fn jaro_winkler_simple(s1: &str, s2: &str) -> f32 {
    if s1 == s2 {
        return 1.0;
    }
    if s1.is_empty() || s2.is_empty() {
        return 0.0;
    }

    let s1_chars: Vec<char> = s1.chars().collect();
    let s2_chars: Vec<char> = s2.chars().collect();
    let len1 = s1_chars.len();
    let len2 = s2_chars.len();

    let match_window = (len1.max(len2) / 2).saturating_sub(1);
    let mut s1_matches = vec![false; len1];
    let mut s2_matches = vec![false; len2];

    let mut matches = 0;
    for i in 0..len1 {
        let start = i.saturating_sub(match_window);
        let end = (i + match_window + 1).min(len2);
        for j in start..end {
            if !s2_matches[j] && s1_chars[i] == s2_chars[j] {
                s1_matches[i] = true;
                s2_matches[j] = true;
                matches += 1;
                break;
            }
        }
    }

    if matches == 0 {
        return 0.0;
    }

    let mut transpositions = 0;
    let mut k = 0;
    for i in 0..len1 {
        if !s1_matches[i] {
            continue;
        }
        while !s2_matches[k] {
            k += 1;
        }
        if s1_chars[i] != s2_chars[k] {
            transpositions += 1;
        }
        k += 1;
    }

    let matches_f = matches as f32;
    let jaro = ((matches_f / len1 as f32)
        + (matches_f / len2 as f32)
        + ((matches_f - transpositions as f32 / 2.0) / matches_f))
        / 3.0;

    let prefix_len = s1_chars
        .iter()
        .zip(s2_chars.iter())
        .take(4)
        .take_while(|(a, b)| a == b)
        .count();

    jaro + (prefix_len as f32 * 0.1 * (1.0 - jaro))
}

/// Test assertion helpers
pub mod assertions {
    use super::*;

    /// Assert that an entity has the expected type
    pub fn assert_entity_type(entity: &Entity, expected: EntityType) {
        assert_eq!(
            entity.entity_type, expected,
            "Expected entity '{}' to be {:?}, but was {:?}",
            entity.name, expected, entity.entity_type
        );
    }

    /// Assert that an entity has the expected name
    pub fn assert_entity_name(entity: &Entity, expected: &str) {
        assert_eq!(
            entity.name, expected,
            "Expected entity name to be '{}', but was '{}'",
            expected, entity.name
        );
    }

    /// Assert that a relationship connects the expected entities
    pub fn assert_relationship_connects(
        rel: &Relationship,
        expected_source: Uuid,
        expected_target: Uuid,
    ) {
        assert_eq!(
            rel.source_id, expected_source,
            "Expected relationship source to be {}, but was {}",
            expected_source, rel.source_id
        );
        assert_eq!(
            rel.target_id, expected_target,
            "Expected relationship target to be {}, but was {}",
            expected_target, rel.target_id
        );
    }

    /// Assert that a fact is currently valid
    pub fn assert_fact_valid(fact: &Fact) {
        assert!(
            fact.is_currently_valid(),
            "Expected fact '{}' to be currently valid",
            fact.attribute
        );
    }

    /// Assert that a fact has the expected value
    pub fn assert_fact_value(fact: &Fact, expected: &str) {
        assert_eq!(
            fact.value_string(), expected,
            "Expected fact '{}' to have value '{}', but was '{}'",
            fact.attribute, expected, fact.value_string()
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_entity_builder_person() {
        let entity = TestEntityBuilder::person("Alice Smith")
            .with_role("Engineer")
            .with_email("alice@example.com")
            .verified()
            .build();

        assert_eq!(entity.name, "Alice Smith");
        assert_eq!(entity.entity_type, EntityType::Person);
        assert_eq!(entity.metadata.title, Some("Engineer".to_string()));
        assert_eq!(entity.metadata.email, Some("alice@example.com".to_string()));
        assert!(entity.verified);
    }

    #[test]
    fn test_entity_builder_organization() {
        let entity = TestEntityBuilder::organization("Acme Corp")
            .with_industry("Manufacturing")
            .build();

        assert_eq!(entity.name, "Acme Corp");
        assert_eq!(entity.entity_type, EntityType::Organization);
        assert_eq!(entity.metadata.industry, Some("Manufacturing".to_string()));
    }

    #[test]
    fn test_relationship_builder() {
        let source_id = Uuid::new_v4();
        let target_id = Uuid::new_v4();

        let rel = TestRelationshipBuilder::works_at(source_id, target_id)
            .verified()
            .build();

        assert_eq!(rel.source_id, source_id);
        assert_eq!(rel.target_id, target_id);
        assert_eq!(rel.relation_type, "works_at");
        assert!(rel.verified);
    }

    #[test]
    fn test_fact_builder() {
        let entity_id = Uuid::new_v4();
        let fact = TestFactBuilder::employer(entity_id, "Acme Corp")
            .with_confidence(0.95)
            .build();

        assert_eq!(fact.entity_id, entity_id);
        assert_eq!(fact.attribute, "employer");
        assert_eq!(fact.value_string(), "Acme Corp");
        assert_eq!(fact.confidence, 0.95);
    }

    #[test]
    fn test_generate_test_embedding() {
        let emb1 = generate_test_embedding(42, 384);
        let emb2 = generate_test_embedding(42, 384);
        let emb3 = generate_test_embedding(43, 384);

        assert_eq!(emb1.len(), 384);
        assert_eq!(emb1, emb2); // Same seed = same embedding
        assert_ne!(emb1, emb3); // Different seed = different embedding
    }

    #[test]
    fn test_name_similarity() {
        let sim = calculate_name_similarity("John Doe", "John Doe");
        assert_eq!(sim, 1.0);

        let sim = calculate_name_similarity("John Doe", "john doe");
        assert_eq!(sim, 1.0);

        let sim = calculate_name_similarity("Johnathan", "John");
        assert!(sim > 0.8);
    }
}
