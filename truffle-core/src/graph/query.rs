//! Graph Query Interface
//!
//! Defines query operations for the knowledge graph.

use async_trait::async_trait;
use chrono::{DateTime, Utc};
use uuid::Uuid;

use crate::models::{Entity, Relationship, EntityType, Decision, ActionItem};
use crate::Result;

/// Graph query trait
#[async_trait]
pub trait GraphQuery: Send + Sync {
    /// Find entity by exact name match
    async fn find_by_name(&self, name: &str) -> Result<Option<Entity>>;
    
    /// Find entities by fuzzy name match
    async fn find_by_name_fuzzy(&self, name: &str, threshold: f64) -> Result<Vec<Entity>>;
    
    /// Semantic search using embeddings
    async fn semantic_search(&self, query: &str, limit: usize) -> Result<Vec<Entity>>;
    
    /// Get entity by ID
    async fn get_entity(&self, id: &Uuid) -> Result<Option<Entity>>;
    
    /// Get all relationships for an entity
    async fn get_relationships(&self, entity_id: &Uuid) -> Result<Vec<Relationship>>;
    
    /// Get outgoing relationships
    async fn get_outgoing_relationships(&self, entity_id: &Uuid) -> Result<Vec<Relationship>>;
    
    /// Get incoming relationships
    async fn get_incoming_relationships(&self, entity_id: &Uuid) -> Result<Vec<Relationship>>;
    
    /// Get entities related to a given entity
    async fn get_related(&self, entity_id: &Uuid, relation_type: Option<&str>) -> Result<Vec<Entity>>;
    
    /// Find path between two entities
    async fn find_path(&self, from: &Uuid, to: &Uuid, max_depth: usize) -> Result<Vec<Entity>>;
    
    /// Get entities mentioned in a time range
    async fn in_time_range(&self, start: DateTime<Utc>, end: DateTime<Utc>) -> Result<Vec<Entity>>;
    
    /// Get decisions involving an entity
    async fn get_decisions(&self, entity_id: &Uuid) -> Result<Vec<Decision>>;
    
    /// Get action items for an entity
    async fn get_action_items(&self, entity_id: &Uuid, status: Option<&str>) -> Result<Vec<ActionItem>>;
    
    /// Search entities by filter
    async fn search_entities(&self, filter: EntityFilter) -> Result<QueryResult<Entity>>;
    
    /// Search relationships by filter
    async fn search_relationships(&self, filter: RelationshipFilter) -> Result<QueryResult<Relationship>>;
}

/// Query result with pagination
#[derive(Debug, Clone)]
pub struct QueryResult<T> {
    /// Items
    pub items: Vec<T>,
    /// Total count (before pagination)
    pub total: usize,
    /// Offset
    pub offset: usize,
    /// Limit
    pub limit: usize,
}

impl<T> QueryResult<T> {
    /// Create a new query result
    pub fn new(items: Vec<T>, total: usize, offset: usize, limit: usize) -> Self {
        Self { items, total, offset, limit }
    }
    
    /// Check if there are more results
    pub fn has_more(&self) -> bool {
        self.offset + self.items.len() < self.total
    }
    
    /// Get next page offset
    pub fn next_offset(&self) -> Option<usize> {
        if self.has_more() {
            Some(self.offset + self.limit)
        } else {
            None
        }
    }
}

/// Filter for entity queries
#[derive(Debug, Clone, Default)]
pub struct EntityFilter {
    /// Entity type
    pub entity_type: Option<EntityType>,
    /// Name contains
    pub name_contains: Option<String>,
    /// Created after
    pub created_after: Option<DateTime<Utc>>,
    /// Created before
    pub created_before: Option<DateTime<Utc>>,
    /// Minimum confidence
    pub min_confidence: Option<f32>,
    /// Verified only
    pub verified_only: bool,
    /// Privacy level
    pub privacy_level: Option<String>,
    /// Pagination offset
    pub offset: usize,
    /// Pagination limit
    pub limit: usize,
}

impl EntityFilter {
    /// Create a new filter
    pub fn new() -> Self {
        Self::default()
    }
    
    /// Filter by type
    pub fn with_type(mut self, entity_type: EntityType) -> Self {
        self.entity_type = Some(entity_type);
        self
    }
    
    /// Filter by name contains
    pub fn with_name_contains(mut self, name: impl Into<String>) -> Self {
        self.name_contains = Some(name.into());
        self
    }
    
    /// Filter by date range
    pub fn with_date_range(mut self, after: DateTime<Utc>, before: DateTime<Utc>) -> Self {
        self.created_after = Some(after);
        self.created_before = Some(before);
        self
    }
    
    /// Filter by minimum confidence
    pub fn with_min_confidence(mut self, confidence: f32) -> Self {
        self.min_confidence = Some(confidence);
        self
    }
    
    /// Verified only
    pub fn verified_only(mut self) -> Self {
        self.verified_only = true;
        self
    }
    
    /// Set pagination
    pub fn with_pagination(mut self, offset: usize, limit: usize) -> Self {
        self.offset = offset;
        self.limit = limit;
        self
    }
}

/// Filter for relationship queries
#[derive(Debug, Clone, Default)]
pub struct RelationshipFilter {
    /// Source entity ID
    pub source_id: Option<Uuid>,
    /// Target entity ID
    pub target_id: Option<Uuid>,
    /// Relationship type
    pub relation_type: Option<String>,
    /// Valid at time
    pub valid_at: Option<DateTime<Utc>>,
    /// Minimum confidence
    pub min_confidence: Option<f32>,
    /// Pagination offset
    pub offset: usize,
    /// Pagination limit
    pub limit: usize,
}

impl RelationshipFilter {
    /// Create a new filter
    pub fn new() -> Self {
        Self::default()
    }
    
    /// Filter by source
    pub fn with_source(mut self, source_id: Uuid) -> Self {
        self.source_id = Some(source_id);
        self
    }
    
    /// Filter by target
    pub fn with_target(mut self, target_id: Uuid) -> Self {
        self.target_id = Some(target_id);
        self
    }
    
    /// Filter by type
    pub fn with_type(mut self, relation_type: impl Into<String>) -> Self {
        self.relation_type = Some(relation_type.into());
        self
    }
    
    /// Filter by validity time
    pub fn valid_at(mut self, time: DateTime<Utc>) -> Self {
        self.valid_at = Some(time);
        self
    }
    
    /// Set pagination
    pub fn with_pagination(mut self, offset: usize, limit: usize) -> Self {
        self.offset = offset;
        self.limit = limit;
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_entity_filter_builder() {
        let filter = EntityFilter::new()
            .with_type(EntityType::Person)
            .with_name_contains("Alice")
            .verified_only()
            .with_pagination(0, 10);
        
        assert_eq!(filter.entity_type, Some(EntityType::Person));
        assert_eq!(filter.name_contains, Some("Alice".to_string()));
        assert!(filter.verified_only);
        assert_eq!(filter.limit, 10);
    }
    
    #[test]
    fn test_relationship_filter_builder() {
        let source_id = Uuid::new_v4();
        let filter = RelationshipFilter::new()
            .with_source(source_id)
            .with_type("works_at");
        
        assert_eq!(filter.source_id, Some(source_id));
        assert_eq!(filter.relation_type, Some("works_at".to_string()));
    }
    
    #[test]
    fn test_query_result_pagination() {
        let items = vec![1, 2, 3, 4, 5];
        let result = QueryResult::new(items, 100, 0, 5);
        
        assert!(result.has_more());
        assert_eq!(result.next_offset(), Some(5));
        assert_eq!(result.total, 100);
    }
}
