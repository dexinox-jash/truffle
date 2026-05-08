//! GraphQL Schema Types for Truffle Knowledge Graph API
//!
//! This module defines the GraphQL schema types for the knowledge graph API,
//! including entities, relationships, decisions, and action items.

use async_graphql::{ComplexObject, Context, Enum, InputObject, Object, Schema, SimpleObject, ID};
use chrono::{DateTime, Utc};
use uuid::Uuid;

use crate::api::context::ApiContext;
use crate::graph::{EntityFilter, RelationshipFilter, RelationshipDirection};
use crate::models::{
    ActionItem as ActionItemModel, ActionItemStatus as ActionItemStatusModel,
    ContradictionSeverity as ContradictionSeverityModel, Decision as DecisionModel,
    DecisionStatus as DecisionStatusModel, Entity as EntityModel, EntityMetadata as EntityMetadataModel,
    EntityType as EntityTypeModel, Priority as PriorityModel, PrivacyLevel as PrivacyLevelModel,
    Relationship as RelationshipModel,
};

/// The GraphQL schema type
pub type ApiSchema = Schema<QueryRoot, MutationRoot, SubscriptionRoot>;

/// Creates a new GraphQL schema instance
pub fn create_schema() -> ApiSchema {
    Schema::new(QueryRoot, MutationRoot, SubscriptionRoot)
}

/// Query root for the GraphQL schema
pub struct QueryRoot;

#[Object]
impl QueryRoot {
    /// Get an entity by ID
    async fn entity(&self, ctx: &Context<'_>, id: ID) -> async_graphql::Result<Option<EntityType>> {
        let context = ctx.data::<ApiContext>()
            .map_err(|e| async_graphql::Error::new(format!("Failed to get context: {}", e)))?;
        
        let uuid = parse_uuid(&id)?;
        let entity = context.get_entity(&uuid).await
            .map_err(|e| async_graphql::Error::new(format!("Database error: {}", e)))?;
        
        entity.map(EntityType::try_from).transpose()
    }

    /// Get a relationship by ID
    async fn relationship(&self, ctx: &Context<'_>, id: ID) -> async_graphql::Result<Option<RelationshipType>> {
        let context = ctx.data::<ApiContext>()
            .map_err(|e| async_graphql::Error::new(format!("Failed to get context: {}", e)))?;
        
        let uuid = parse_uuid(&id)?;
        let relationship = context.get_relationship(&uuid).await
            .map_err(|e| async_graphql::Error::new(format!("Database error: {}", e)))?;
        
        relationship.map(RelationshipType::try_from).transpose()
    }

    /// Search entities with filters
    async fn entities(
        &self,
        ctx: &Context<'_>,
        filter: Option<EntityFilterInput>,
        sort: Option<EntitySortInput>,
        first: Option<i32>,
        after: Option<String>,
    ) -> async_graphql::Result<EntityConnection> {
        let context = ctx.data::<ApiContext>()
            .map_err(|e| async_graphql::Error::new(format!("Failed to get context: {}", e)))?;
        
        let filter: EntityFilter = filter.map(Into::into).unwrap_or_default();
        let limit = first.map(|n| n.max(1).min(100) as usize).unwrap_or(20);
        let offset = after.and_then(|a| decode_cursor(&a)).unwrap_or(0);
        
        let (entities, total) = context.search_entities(filter, offset, limit).await
            .map_err(|e| async_graphql::Error::new(format!("Database error: {}", e)))?;
        
        let mut nodes = Vec::with_capacity(entities.len());
        for entity in entities {
            nodes.push(EntityType::try_from(entity)?);
        }
        
        let has_next_page = offset + nodes.len() < total;
        let end_cursor = if has_next_page {
            Some(encode_cursor(offset + nodes.len()))
        } else {
            None
        };
        
        Ok(EntityConnection {
            nodes,
            total_count: total,
            page_info: PageInfo {
                has_next_page,
                has_previous_page: offset > 0,
                start_cursor: Some(encode_cursor(offset)),
                end_cursor,
            },
        })
    }

    /// Search relationships with filters
    async fn relationships(
        &self,
        ctx: &Context<'_>,
        filter: Option<RelationshipFilterInput>,
        sort: Option<RelationshipSortInput>,
        first: Option<i32>,
        after: Option<String>,
    ) -> async_graphql::Result<RelationshipConnection> {
        let context = ctx.data::<ApiContext>()
            .map_err(|e| async_graphql::Error::new(format!("Failed to get context: {}", e)))?;
        
        let filter: RelationshipFilter = filter.map(Into::into).unwrap_or_default();
        let limit = first.map(|n| n.max(1).min(100) as usize).unwrap_or(20);
        let offset = after.and_then(|a| decode_cursor(&a)).unwrap_or(0);
        
        let (relationships, total) = context.search_relationships(filter, offset, limit).await
            .map_err(|e| async_graphql::Error::new(format!("Database error: {}", e)))?;
        
        let mut nodes = Vec::with_capacity(relationships.len());
        for rel in relationships {
            nodes.push(RelationshipType::try_from(rel)?);
        }
        
        let has_next_page = offset + nodes.len() < total;
        let end_cursor = if has_next_page {
            Some(encode_cursor(offset + nodes.len()))
        } else {
            None
        };
        
        Ok(RelationshipConnection {
            nodes,
            total_count: total,
            page_info: PageInfo {
                has_next_page,
                has_previous_page: offset > 0,
                start_cursor: Some(encode_cursor(offset)),
                end_cursor,
            },
        })
    }

    /// Get relationships for a specific entity
    async fn entity_relationships(
        &self,
        ctx: &Context<'_>,
        entity_id: ID,
        direction: Option<RelationshipDirectionEnum>,
    ) -> async_graphql::Result<Vec<RelationshipType>> {
        let context = ctx.data::<ApiContext>()
            .map_err(|e| async_graphql::Error::new(format!("Failed to get context: {}", e)))?;
        
        let uuid = parse_uuid(&entity_id)?;
        let dir = direction.map(Into::into);
        
        let relationships = context.get_entity_relationships(&uuid, dir).await
            .map_err(|e| async_graphql::Error::new(format!("Database error: {}", e)))?;
        
        let mut result = Vec::with_capacity(relationships.len());
        for rel in relationships {
            result.push(RelationshipType::try_from(rel)?);
        }
        
        Ok(result)
    }

    /// Semantic search for entities
    async fn search_entities(
        &self,
        ctx: &Context<'_>,
        query: String,
        limit: Option<i32>,
    ) -> async_graphql::Result<Vec<EntityType>> {
        let context = ctx.data::<ApiContext>()
            .map_err(|e| async_graphql::Error::new(format!("Failed to get context: {}", e)))?;
        
        let limit = limit.map(|n| n.max(1).min(50) as usize).unwrap_or(10);
        
        let entities = context.semantic_search(&query, limit).await
            .map_err(|e| async_graphql::Error::new(format!("Search error: {}", e)))?;
        
        let mut result = Vec::with_capacity(entities.len());
        for entity in entities {
            result.push(EntityType::try_from(entity)?);
        }
        
        Ok(result)
    }

    /// Get meetings with optional filtering
    async fn meetings(
        &self,
        ctx: &Context<'_>,
        filter: Option<MeetingFilterInput>,
    ) -> async_graphql::Result<Vec<MeetingType>> {
        let _context = ctx.data::<ApiContext>()
            .map_err(|e| async_graphql::Error::new(format!("Failed to get context: {}", e)))?;
        let _filter = filter;
        // Placeholder - would query meeting repository
        Ok(Vec::new())
    }

    /// Get decisions for an entity
    async fn decisions(
        &self,
        ctx: &Context<'_>,
        entity_id: Option<ID>,
    ) -> async_graphql::Result<Vec<DecisionType>> {
        let _context = ctx.data::<ApiContext>()
            .map_err(|e| async_graphql::Error::new(format!("Failed to get context: {}", e)))?;
        let _entity_id = entity_id;
        // Placeholder - would query decision repository
        Ok(Vec::new())
    }

    /// Get action items with optional filtering
    async fn action_items(
        &self,
        ctx: &Context<'_>,
        entity_id: Option<ID>,
        status: Option<ActionItemStatusEnum>,
    ) -> async_graphql::Result<Vec<ActionItemType>> {
        let _context = ctx.data::<ApiContext>()
            .map_err(|e| async_graphql::Error::new(format!("Failed to get context: {}", e)))?;
        let _entity_id = entity_id;
        let _status = status;
        // Placeholder - would query action item repository
        Ok(Vec::new())
    }

    /// Get graph starting from an entity with specified depth
    async fn graph(
        &self,
        ctx: &Context<'_>,
        entity_id: ID,
        depth: Option<i32>,
    ) -> async_graphql::Result<GraphNode> {
        let _context = ctx.data::<ApiContext>()
            .map_err(|e| async_graphql::Error::new(format!("Failed to get context: {}", e)))?;
        let _entity_id = entity_id;
        let _depth = depth.map(|d| d.max(1).min(5)).unwrap_or(2);
        // Placeholder - would use graph traversal
        Err(async_graphql::Error::new("Graph traversal not yet implemented"))
    }

    /// Find path between two entities
    async fn path(
        &self,
        ctx: &Context<'_>,
        from: ID,
        to: ID,
        max_depth: Option<i32>,
    ) -> async_graphql::Result<Vec<EntityType>> {
        let _context = ctx.data::<ApiContext>()
            .map_err(|e| async_graphql::Error::new(format!("Failed to get context: {}", e)))?;
        let _from = from;
        let _to = to;
        let _max_depth = max_depth.map(|d| d.max(1).min(10)).unwrap_or(5);
        // Placeholder - would use path finding
        Ok(Vec::new())
    }
}

/// Mutation root for the GraphQL schema
pub struct MutationRoot;

#[Object]
impl MutationRoot {
    /// Create a new entity
    async fn create_entity(
        &self,
        ctx: &Context<'_>,
        input: CreateEntityInput,
    ) -> async_graphql::Result<EntityType> {
        let context = ctx.data::<ApiContext>()
            .map_err(|e| async_graphql::Error::new(format!("Failed to get context: {}", e)))?;
        
        let entity = context.create_entity(input).await
            .map_err(|e| async_graphql::Error::new(format!("Failed to create entity: {}", e)))?;
        
        EntityType::try_from(entity)
    }

    /// Update an existing entity
    async fn update_entity(
        &self,
        ctx: &Context<'_>,
        id: ID,
        input: UpdateEntityInput,
    ) -> async_graphql::Result<EntityType> {
        let context = ctx.data::<ApiContext>()
            .map_err(|e| async_graphql::Error::new(format!("Failed to get context: {}", e)))?;
        
        let uuid = parse_uuid(&id)?;
        let entity = context.update_entity(&uuid, input).await
            .map_err(|e| async_graphql::Error::new(format!("Failed to update entity: {}", e)))?;
        
        EntityType::try_from(entity)
    }

    /// Delete an entity
    async fn delete_entity(&self, ctx: &Context<'_>, id: ID) -> async_graphql::Result<bool> {
        let context = ctx.data::<ApiContext>()
            .map_err(|e| async_graphql::Error::new(format!("Failed to get context: {}", e)))?;
        
        let uuid = parse_uuid(&id)?;
        context.delete_entity(&uuid).await
            .map_err(|e| async_graphql::Error::new(format!("Failed to delete entity: {}", e)))?;
        
        Ok(true)
    }

    /// Create a new relationship
    async fn create_relationship(
        &self,
        ctx: &Context<'_>,
        input: CreateRelationshipInput,
    ) -> async_graphql::Result<RelationshipType> {
        let context = ctx.data::<ApiContext>()
            .map_err(|e| async_graphql::Error::new(format!("Failed to get context: {}", e)))?;
        
        let relationship = context.create_relationship(input).await
            .map_err(|e| async_graphql::Error::new(format!("Failed to create relationship: {}", e)))?;
        
        RelationshipType::try_from(relationship)
    }

    /// Update an existing relationship
    async fn update_relationship(
        &self,
        ctx: &Context<'_>,
        id: ID,
        input: UpdateRelationshipInput,
    ) -> async_graphql::Result<RelationshipType> {
        let context = ctx.data::<ApiContext>()
            .map_err(|e| async_graphql::Error::new(format!("Failed to get context: {}", e)))?;
        
        let uuid = parse_uuid(&id)?;
        let relationship = context.update_relationship(&uuid, input).await
            .map_err(|e| async_graphql::Error::new(format!("Failed to update relationship: {}", e)))?;
        
        RelationshipType::try_from(relationship)
    }

    /// Delete a relationship
    async fn delete_relationship(&self, ctx: &Context<'_>, id: ID) -> async_graphql::Result<bool> {
        let context = ctx.data::<ApiContext>()
            .map_err(|e| async_graphql::Error::new(format!("Failed to get context: {}", e)))?;
        
        let uuid = parse_uuid(&id)?;
        context.delete_relationship(&uuid).await
            .map_err(|e| async_graphql::Error::new(format!("Failed to delete relationship: {}", e)))?;
        
        Ok(true)
    }

    /// Verify an entity
    async fn verify_entity(&self, ctx: &Context<'_>, id: ID) -> async_graphql::Result<EntityType> {
        let context = ctx.data::<ApiContext>()
            .map_err(|e| async_graphql::Error::new(format!("Failed to get context: {}", e)))?;
        
        let uuid = parse_uuid(&id)?;
        let entity = context.verify_entity(&uuid).await
            .map_err(|e| async_graphql::Error::new(format!("Failed to verify entity: {}", e)))?;
        
        EntityType::try_from(entity)
    }

    /// Verify a relationship
    async fn verify_relationship(&self, ctx: &Context<'_>, id: ID) -> async_graphql::Result<RelationshipType> {
        let context = ctx.data::<ApiContext>()
            .map_err(|e| async_graphql::Error::new(format!("Failed to get context: {}", e)))?;
        
        let uuid = parse_uuid(&id)?;
        let relationship = context.verify_relationship(&uuid).await
            .map_err(|e| async_graphql::Error::new(format!("Failed to verify relationship: {}", e)))?;
        
        RelationshipType::try_from(relationship)
    }
}

/// Subscription root for the GraphQL schema
pub struct SubscriptionRoot;

#[Object]
impl SubscriptionRoot {
    /// Subscribe to entity updates (placeholder for future implementation)
    async fn entity_updates(&self) -> async_graphql::Result<String> {
        Ok("Subscriptions not yet implemented".to_string())
    }
}

/// Entity type enum for GraphQL
#[derive(Enum, Copy, Clone, Eq, PartialEq, Debug)]
pub enum EntityTypeEnum {
    /// A person
    Person,
    /// An organization
    Organization,
    /// A location
    Location,
    /// An event
    Event,
    /// A product
    Product,
    /// A concept
    Concept,
    /// A decision
    Decision,
    /// An action item
    ActionItem,
}

impl From<EntityTypeModel> for EntityTypeEnum {
    fn from(ty: EntityTypeModel) -> Self {
        match ty {
            EntityTypeModel::Person => Self::Person,
            EntityTypeModel::Organization => Self::Organization,
            EntityTypeModel::Location => Self::Location,
            EntityTypeModel::Event => Self::Event,
            EntityTypeModel::Product => Self::Product,
            EntityTypeModel::Concept => Self::Concept,
            EntityTypeModel::Decision => Self::Decision,
            EntityTypeModel::ActionItem => Self::ActionItem,
        }
    }
}

impl From<EntityTypeEnum> for EntityTypeModel {
    fn from(ty: EntityTypeEnum) -> Self {
        match ty {
            EntityTypeEnum::Person => Self::Person,
            EntityTypeEnum::Organization => Self::Organization,
            EntityTypeEnum::Location => Self::Location,
            EntityTypeEnum::Event => Self::Event,
            EntityTypeEnum::Product => Self::Product,
            EntityTypeEnum::Concept => Self::Concept,
            EntityTypeEnum::Decision => Self::Decision,
            EntityTypeEnum::ActionItem => Self::ActionItem,
        }
    }
}

/// Decision status enum for GraphQL
#[derive(Enum, Copy, Clone, Eq, PartialEq, Debug)]
pub enum DecisionStatusEnum {
    /// Proposed status
    Proposed,
    /// Approved status
    Approved,
    /// Rejected status
    Rejected,
    /// Superseded status
    Superseded,
    /// Implemented status
    Implemented,
    /// Abandoned status
    Abandoned,
}

impl From<DecisionStatusModel> for DecisionStatusEnum {
    fn from(status: DecisionStatusModel) -> Self {
        match status {
            DecisionStatusModel::Proposed => Self::Proposed,
            DecisionStatusModel::Approved => Self::Approved,
            DecisionStatusModel::Rejected => Self::Rejected,
            DecisionStatusModel::Superseded => Self::Superseded,
            DecisionStatusModel::Implemented => Self::Implemented,
            DecisionStatusModel::Abandoned => Self::Abandoned,
        }
    }
}

/// Action item status enum for GraphQL
#[derive(Enum, Copy, Clone, Eq, PartialEq, Debug)]
pub enum ActionItemStatusEnum {
    /// Open status
    Open,
    /// In progress status
    InProgress,
    /// Blocked status
    Blocked,
    /// Completed status
    Completed,
    /// Cancelled status
    Cancelled,
    /// Overdue status
    Overdue,
}

impl From<ActionItemStatusModel> for ActionItemStatusEnum {
    fn from(status: ActionItemStatusModel) -> Self {
        match status {
            ActionItemStatusModel::Open => Self::Open,
            ActionItemStatusModel::InProgress => Self::InProgress,
            ActionItemStatusModel::Blocked => Self::Blocked,
            ActionItemStatusModel::Completed => Self::Completed,
            ActionItemStatusModel::Cancelled => Self::Cancelled,
            ActionItemStatusModel::Overdue => Self::Overdue,
        }
    }
}

impl From<ActionItemStatusEnum> for ActionItemStatusModel {
    fn from(status: ActionItemStatusEnum) -> Self {
        match status {
            ActionItemStatusEnum::Open => Self::Open,
            ActionItemStatusEnum::InProgress => Self::InProgress,
            ActionItemStatusEnum::Blocked => Self::Blocked,
            ActionItemStatusEnum::Completed => Self::Completed,
            ActionItemStatusEnum::Cancelled => Self::Cancelled,
            ActionItemStatusEnum::Overdue => Self::Overdue,
        }
    }
}

/// Priority enum for GraphQL
#[derive(Enum, Copy, Clone, Eq, PartialEq, Debug)]
pub enum PriorityEnum {
    /// Low priority
    Low,
    /// Medium priority
    Medium,
    /// High priority
    High,
    /// Urgent priority
    Urgent,
}

impl From<PriorityModel> for PriorityEnum {
    fn from(priority: PriorityModel) -> Self {
        match priority {
            PriorityModel::Low => Self::Low,
            PriorityModel::Medium => Self::Medium,
            PriorityModel::High => Self::High,
            PriorityModel::Urgent => Self::Urgent,
        }
    }
}

/// Privacy level enum for GraphQL
#[derive(Enum, Copy, Clone, Eq, PartialEq, Debug)]
pub enum PrivacyLevelEnum {
    /// Public privacy level
    Public,
    /// Internal privacy level
    Internal,
    /// Confidential privacy level
    Confidential,
    /// Restricted privacy level
    Restricted,
}

impl From<PrivacyLevelModel> for PrivacyLevelEnum {
    fn from(level: PrivacyLevelModel) -> Self {
        match level {
            PrivacyLevelModel::Public => Self::Public,
            PrivacyLevelModel::Internal => Self::Internal,
            PrivacyLevelModel::Confidential => Self::Confidential,
            PrivacyLevelModel::Restricted => Self::Restricted,
        }
    }
}

impl From<PrivacyLevelEnum> for PrivacyLevelModel {
    fn from(level: PrivacyLevelEnum) -> Self {
        match level {
            PrivacyLevelEnum::Public => Self::Public,
            PrivacyLevelEnum::Internal => Self::Internal,
            PrivacyLevelEnum::Confidential => Self::Confidential,
            PrivacyLevelEnum::Restricted => Self::Restricted,
        }
    }
}

/// Contradiction severity enum for GraphQL
#[derive(Enum, Copy, Clone, Eq, PartialEq, Debug, PartialOrd, Ord)]
pub enum ContradictionSeverityEnum {
    /// Low severity
    Low,
    /// Medium severity
    Medium,
    /// High severity
    High,
    /// Critical severity
    Critical,
}

impl From<ContradictionSeverityModel> for ContradictionSeverityEnum {
    fn from(severity: ContradictionSeverityModel) -> Self {
        match severity {
            ContradictionSeverityModel::Low => Self::Low,
            ContradictionSeverityModel::Medium => Self::Medium,
            ContradictionSeverityModel::High => Self::High,
            ContradictionSeverityModel::Critical => Self::Critical,
        }
    }
}

impl From<ContradictionSeverityEnum> for ContradictionSeverityModel {
    fn from(severity: ContradictionSeverityEnum) -> Self {
        match severity {
            ContradictionSeverityEnum::Low => Self::Low,
            ContradictionSeverityEnum::Medium => Self::Medium,
            ContradictionSeverityEnum::High => Self::High,
            ContradictionSeverityEnum::Critical => Self::Critical,
        }
    }
}

/// Relationship direction enum for GraphQL
#[derive(Enum, Copy, Clone, Eq, PartialEq, Debug)]
pub enum RelationshipDirectionEnum {
    /// Outgoing from source
    Outgoing,
    /// Incoming to target
    Incoming,
    /// Both directions
    Both,
}

impl From<RelationshipDirectionEnum> for RelationshipDirection {
    fn from(dir: RelationshipDirectionEnum) -> Self {
        match dir {
            RelationshipDirectionEnum::Outgoing => Self::Outgoing,
            RelationshipDirectionEnum::Incoming => Self::Incoming,
            RelationshipDirectionEnum::Both => Self::Both,
        }
    }
}

/// Entity object type for GraphQL
#[derive(SimpleObject, Clone, Debug)]
#[graphql(complex)]
pub struct EntityType {
    /// Unique identifier
    pub id: ID,
    /// Entity type
    pub entity_type: EntityTypeEnum,
    /// Display name
    pub name: String,
    /// URL-friendly identifier
    pub slug: String,
    /// Description
    pub description: Option<String>,
    /// Type-specific metadata
    pub metadata: EntityMetadataType,
    /// Extraction confidence (0.0 - 1.0)
    pub extraction_confidence: f32,
    /// Privacy classification
    pub privacy_level: PrivacyLevelEnum,
    /// Whether entity has been manually verified
    pub verified: bool,
    /// When entity was first observed
    pub created_at: DateTime<Utc>,
    /// When entity was last updated
    pub updated_at: DateTime<Utc>,
}

#[ComplexObject]
impl EntityType {
    /// Get related entities
    async fn relationships(&self, ctx: &Context<'_>) -> async_graphql::Result<Vec<RelationshipType>> {
        let context = ctx.data::<ApiContext>()
            .map_err(|e| async_graphql::Error::new(format!("Failed to get context: {}", e)))?;
        
        let uuid = parse_uuid(&self.id)?;
        let relationships = context.get_entity_relationships(&uuid, None).await
            .map_err(|e| async_graphql::Error::new(format!("Database error: {}", e)))?;
        
        let mut result = Vec::with_capacity(relationships.len());
        for rel in relationships {
            result.push(RelationshipType::try_from(rel)?);
        }
        
        Ok(result)
    }
}

impl TryFrom<EntityModel> for EntityType {
    type Error = async_graphql::Error;

    fn try_from(entity: EntityModel) -> Result<Self, Self::Error> {
        Ok(Self {
            id: ID::from(entity.id.to_string()),
            entity_type: entity.entity_type.into(),
            name: entity.name,
            slug: entity.slug,
            description: entity.description,
            metadata: EntityMetadataType::from(entity.metadata),
            extraction_confidence: entity.extraction_confidence,
            privacy_level: entity.privacy_level.into(),
            verified: entity.verified,
            created_at: entity.first_seen_at,
            updated_at: entity.last_updated_at,
        })
    }
}

/// Entity metadata type for GraphQL
#[derive(SimpleObject, Clone, Debug, Default)]
pub struct EntityMetadataType {
    /// For Person: job title
    pub title: Option<String>,
    /// For Person: email
    pub email: Option<String>,
    /// For Person: phone
    pub phone: Option<String>,
    /// For Organization: industry
    pub industry: Option<String>,
    /// For Organization: website
    pub website: Option<String>,
    /// For Location: address
    pub address: Option<String>,
    /// For Location: latitude
    pub latitude: Option<f64>,
    /// For Location: longitude
    pub longitude: Option<f64>,
    /// For Event: start time
    pub event_start: Option<DateTime<Utc>>,
    /// For Event: end time
    pub event_end: Option<DateTime<Utc>>,
    /// For Product: category
    pub category: Option<String>,
    /// For Product: price
    pub price: Option<String>,
    /// For Decision: status
    pub decision_status: Option<String>,
    /// For Decision: impact
    pub impact: Option<String>,
    /// For ActionItem: deadline
    pub deadline: Option<DateTime<Utc>>,
    /// For ActionItem: assignee
    pub assignee: Option<String>,
}

impl From<EntityMetadataModel> for EntityMetadataType {
    fn from(metadata: EntityMetadataModel) -> Self {
        Self {
            title: metadata.title,
            email: metadata.email,
            phone: metadata.phone,
            industry: metadata.industry,
            website: metadata.website,
            address: metadata.address,
            latitude: metadata.latitude,
            longitude: metadata.longitude,
            event_start: metadata.event_start,
            event_end: metadata.event_end,
            category: metadata.category,
            price: metadata.price,
            decision_status: metadata.decision_status,
            impact: metadata.impact,
            deadline: metadata.deadline,
            assignee: metadata.assignee,
        }
    }
}

/// Relationship object type for GraphQL
#[derive(SimpleObject, Clone, Debug)]
pub struct RelationshipType {
    /// Unique identifier
    pub id: ID,
    /// Source entity ID
    pub source_id: ID,
    /// Target entity ID
    pub target_id: ID,
    /// Relationship type
    pub relation_type: String,
    /// Additional properties
    pub properties: Option<String>,
    /// Extraction confidence (0.0 - 1.0)
    pub confidence: f32,
    /// Validity period start
    pub valid_from: Option<DateTime<Utc>>,
    /// Validity period end
    pub valid_until: Option<DateTime<Utc>>,
    /// Whether relationship has been manually verified
    pub verified: bool,
    /// Whether this relationship is deprecated/archived
    pub archived: bool,
    /// When the relationship was first observed
    pub created_at: DateTime<Utc>,
    /// When the relationship was last updated
    pub updated_at: DateTime<Utc>,
}

impl TryFrom<RelationshipModel> for RelationshipType {
    type Error = async_graphql::Error;

    fn try_from(rel: RelationshipModel) -> Result<Self, Self::Error> {
        let properties = rel.properties
            .map(|p| serde_json::to_string(&p))
            .transpose()
            .map_err(|e| async_graphql::Error::new(format!("Failed to serialize properties: {}", e)))?;

        Ok(Self {
            id: ID::from(rel.id.to_string()),
            source_id: ID::from(rel.source_id.to_string()),
            target_id: ID::from(rel.target_id.to_string()),
            relation_type: rel.relation_type,
            properties,
            confidence: rel.extraction_confidence,
            valid_from: rel.valid_from,
            valid_until: rel.valid_until,
            verified: rel.verified,
            archived: rel.archived,
            created_at: rel.first_seen_at,
            updated_at: rel.last_updated_at,
        })
    }
}

/// Relationship metadata type for GraphQL
#[derive(SimpleObject, Clone, Debug, Default)]
pub struct RelationshipMetadataType {
    /// Additional properties as JSON string
    pub properties: Option<String>,
}

/// Create entity input type
#[derive(InputObject, Clone, Debug)]
pub struct CreateEntityInput {
    /// Entity type
    pub entity_type: EntityTypeEnum,
    /// Display name
    pub name: String,
    /// Description (optional)
    pub description: Option<String>,
    /// Type-specific metadata (optional)
    pub metadata: Option<EntityMetadataInput>,
    /// Privacy level (optional, defaults to Internal)
    pub privacy_level: Option<PrivacyLevelEnum>,
    /// Extraction confidence (optional, defaults to 0.8)
    pub extraction_confidence: Option<f32>,
}

/// Update entity input type
#[derive(InputObject, Clone, Debug, Default)]
pub struct UpdateEntityInput {
    /// Display name (optional)
    pub name: Option<String>,
    /// Description (optional)
    pub description: Option<String>,
    /// Type-specific metadata (optional)
    pub metadata: Option<EntityMetadataInput>,
    /// Privacy level (optional)
    pub privacy_level: Option<PrivacyLevelEnum>,
    /// Extraction confidence (optional)
    pub extraction_confidence: Option<f32>,
}

/// Entity metadata input type
#[derive(InputObject, Clone, Debug, Default)]
pub struct EntityMetadataInput {
    /// For Person: job title
    pub title: Option<String>,
    /// For Person: email
    pub email: Option<String>,
    /// For Person: phone
    pub phone: Option<String>,
    /// For Organization: industry
    pub industry: Option<String>,
    /// For Organization: website
    pub website: Option<String>,
    /// For Location: address
    pub address: Option<String>,
    /// For Location: latitude
    pub latitude: Option<f64>,
    /// For Location: longitude
    pub longitude: Option<f64>,
    /// For Event: start time
    pub event_start: Option<DateTime<Utc>>,
    /// For Event: end time
    pub event_end: Option<DateTime<Utc>>,
    /// For Product: category
    pub category: Option<String>,
    /// For Product: price
    pub price: Option<String>,
    /// For Decision: status
    pub decision_status: Option<String>,
    /// For Decision: impact
    pub impact: Option<String>,
    /// For ActionItem: deadline
    pub deadline: Option<DateTime<Utc>>,
    /// For ActionItem: assignee
    pub assignee: Option<String>,
}

impl From<EntityMetadataInput> for EntityMetadataModel {
    fn from(input: EntityMetadataInput) -> Self {
        Self {
            title: input.title,
            email: input.email,
            phone: input.phone,
            industry: input.industry,
            website: input.website,
            address: input.address,
            latitude: input.latitude,
            longitude: input.longitude,
            event_start: input.event_start,
            event_end: input.event_end,
            category: input.category,
            price: input.price,
            decision_status: input.decision_status,
            impact: input.impact,
            deadline: input.deadline,
            assignee: input.assignee,
            extra: std::collections::HashMap::new(),
        }
    }
}

/// Create relationship input type
#[derive(InputObject, Clone, Debug)]
pub struct CreateRelationshipInput {
    /// Source entity ID
    pub source_id: ID,
    /// Target entity ID
    pub target_id: ID,
    /// Relationship type
    pub relation_type: String,
    /// Extraction confidence (optional, defaults to 0.8)
    pub confidence: Option<f32>,
    /// Validity period start (optional)
    pub valid_from: Option<DateTime<Utc>>,
    /// Validity period end (optional)
    pub valid_until: Option<DateTime<Utc>>,
    /// Additional properties as JSON string (optional)
    pub properties: Option<String>,
}

/// Update relationship input type
#[derive(InputObject, Clone, Debug, Default)]
pub struct UpdateRelationshipInput {
    /// Relationship type (optional)
    pub relation_type: Option<String>,
    /// Extraction confidence (optional)
    pub confidence: Option<f32>,
    /// Validity period start (optional)
    pub valid_from: Option<DateTime<Utc>>,
    /// Validity period end (optional)
    pub valid_until: Option<DateTime<Utc>>,
    /// Additional properties as JSON string (optional)
    pub properties: Option<String>,
}

/// Entity filter input type
#[derive(InputObject, Clone, Debug, Default)]
pub struct EntityFilterInput {
    /// Filter by entity type
    pub entity_type: Option<EntityTypeEnum>,
    /// Filter by name contains
    pub name_contains: Option<String>,
    /// Filter by created after date
    pub created_after: Option<DateTime<Utc>>,
    /// Filter by created before date
    pub created_before: Option<DateTime<Utc>>,
    /// Filter by minimum confidence
    pub min_confidence: Option<f32>,
    /// Filter by verified status
    pub verified_only: Option<bool>,
    /// Filter by privacy level
    pub privacy_level: Option<PrivacyLevelEnum>,
}

impl From<EntityFilterInput> for EntityFilter {
    fn from(input: EntityFilterInput) -> Self {
        Self {
            entity_type: input.entity_type.map(Into::into),
            name_contains: input.name_contains,
            created_after: input.created_after,
            created_before: input.created_before,
            min_confidence: input.min_confidence,
            verified_only: input.verified_only.unwrap_or(false),
            privacy_level: input.privacy_level.map(|p| p.as_str().to_string()),
            offset: 0,
            limit: 20,
        }
    }
}

impl PrivacyLevelEnum {
    fn as_str(&self) -> &'static str {
        match self {
            Self::Public => "public",
            Self::Internal => "internal",
            Self::Confidential => "confidential",
            Self::Restricted => "restricted",
        }
    }
}

/// Relationship filter input type
#[derive(InputObject, Clone, Debug, Default)]
pub struct RelationshipFilterInput {
    /// Filter by source entity ID
    pub source_id: Option<ID>,
    /// Filter by target entity ID
    pub target_id: Option<ID>,
    /// Filter by relationship type
    pub relation_type: Option<String>,
    /// Filter by valid at time
    pub valid_at: Option<DateTime<Utc>>,
    /// Filter by minimum confidence
    pub min_confidence: Option<f32>,
}

impl From<RelationshipFilterInput> for RelationshipFilter {
    fn from(input: RelationshipFilterInput) -> Self {
        Self {
            source_id: input.source_id.and_then(|id| id.parse::<Uuid>().ok()),
            target_id: input.target_id.and_then(|id| id.parse::<Uuid>().ok()),
            relation_type: input.relation_type,
            valid_at: input.valid_at,
            min_confidence: input.min_confidence,
            offset: 0,
            limit: 20,
        }
    }
}

/// Meeting filter input type
#[derive(InputObject, Clone, Debug, Default)]
pub struct MeetingFilterInput {
    /// Filter by meeting type
    pub meeting_type: Option<String>,
    /// Filter by processing status
    pub processing_status: Option<String>,
    /// Filter by platform
    pub platform: Option<String>,
}

/// Sort direction enum
#[derive(Enum, Copy, Clone, Eq, PartialEq, Debug)]
pub enum SortDirection {
    /// Ascending order
    Asc,
    /// Descending order
    Desc,
}

/// Entity sort field enum
#[derive(Enum, Copy, Clone, Eq, PartialEq, Debug)]
pub enum EntitySortField {
    /// Sort by name
    Name,
    /// Sort by created date
    CreatedAt,
    /// Sort by updated date
    UpdatedAt,
    /// Sort by confidence
    Confidence,
}

/// Entity sort input type
#[derive(InputObject, Clone, Debug)]
pub struct EntitySortInput {
    /// Field to sort by
    pub field: EntitySortField,
    /// Sort direction
    pub direction: SortDirection,
}

/// Relationship sort field enum
#[derive(Enum, Copy, Clone, Eq, PartialEq, Debug)]
pub enum RelationshipSortField {
    /// Sort by relationship type
    RelationType,
    /// Sort by created date
    CreatedAt,
    /// Sort by updated date
    UpdatedAt,
    /// Sort by confidence
    Confidence,
}

/// Relationship sort input type
#[derive(InputObject, Clone, Debug)]
pub struct RelationshipSortInput {
    /// Field to sort by
    pub field: RelationshipSortField,
    /// Sort direction
    pub direction: SortDirection,
}

/// Page info for pagination
#[derive(SimpleObject, Clone, Debug)]
pub struct PageInfo {
    /// Whether there are more results after this page
    pub has_next_page: bool,
    /// Whether there are results before this page
    pub has_previous_page: bool,
    /// Cursor for the first item in this page
    pub start_cursor: Option<String>,
    /// Cursor for the last item in this page
    pub end_cursor: Option<String>,
}

/// Entity connection for pagination
#[derive(SimpleObject, Clone, Debug)]
pub struct EntityConnection {
    /// List of entity nodes
    pub nodes: Vec<EntityType>,
    /// Total count of entities
    pub total_count: usize,
    /// Page information
    pub page_info: PageInfo,
}

/// Relationship connection for pagination
#[derive(SimpleObject, Clone, Debug)]
pub struct RelationshipConnection {
    /// List of relationship nodes
    pub nodes: Vec<RelationshipType>,
    /// Total count of relationships
    pub total_count: usize,
    /// Page information
    pub page_info: PageInfo,
}

/// Meeting type for GraphQL
#[derive(SimpleObject, Clone, Debug)]
pub struct MeetingType {
    /// Unique identifier
    pub id: ID,
    /// Meeting title
    pub title: String,
    /// When meeting started
    pub started_at: Option<DateTime<Utc>>,
    /// When meeting ended
    pub ended_at: Option<DateTime<Utc>>,
    /// Timezone
    pub timezone: String,
    /// Participant entity IDs (JSON array)
    pub participant_ids: Option<String>,
    /// Full transcript text
    pub transcript: Option<String>,
    /// Meeting type
    pub meeting_type: Option<String>,
    /// Platform (zoom, teams, meet, etc.)
    pub platform: Option<String>,
    /// Processing status
    pub processing_status: String,
    /// Created at
    pub created_at: DateTime<Utc>,
    /// Updated at
    pub updated_at: DateTime<Utc>,
}

impl TryFrom<crate::models::Meeting> for MeetingType {
    type Error = async_graphql::Error;

    fn try_from(meeting: crate::models::Meeting) -> Result<Self, Self::Error> {
        Ok(Self {
            id: ID::from(meeting.id.to_string()),
            title: meeting.title,
            started_at: meeting.started_at,
            ended_at: meeting.ended_at,
            timezone: meeting.timezone,
            participant_ids: meeting.participant_ids,
            transcript: meeting.transcript,
            meeting_type: meeting.meeting_type,
            platform: meeting.platform,
            processing_status: meeting.processing_status,
            created_at: meeting.created_at,
            updated_at: meeting.updated_at,
        })
    }
}

/// Decision type for GraphQL
#[derive(SimpleObject, Clone, Debug)]
pub struct DecisionType {
    /// Unique identifier
    pub id: ID,
    /// Decision text
    pub decision_text: String,
    /// Summary
    pub summary: Option<String>,
    /// Version number
    pub version: i32,
    /// Previous version ID
    pub previous_version_id: Option<ID>,
    /// Decision chain ID
    pub decision_chain_id: ID,
    /// Status
    pub status: String,
    /// Person who proposed
    pub proposed_by_id: Option<ID>,
    /// Meeting where decided
    pub decided_in_meeting_id: Option<ID>,
    /// When proposed
    pub proposed_at: Option<DateTime<Utc>>,
    /// When decided
    pub decided_at: Option<DateTime<Utc>>,
    /// Impact score (0.0 - 1.0)
    pub impact_score: Option<f32>,
    /// Created at
    pub created_at: DateTime<Utc>,
    /// Updated at
    pub updated_at: DateTime<Utc>,
}

impl TryFrom<crate::models::Decision> for DecisionType {
    type Error = async_graphql::Error;

    fn try_from(decision: crate::models::Decision) -> Result<Self, Self::Error> {
        Ok(Self {
            id: ID::from(decision.id.to_string()),
            decision_text: decision.decision_text,
            summary: decision.summary,
            version: decision.version,
            previous_version_id: decision.previous_version_id.map(|id| ID::from(id.to_string())),
            decision_chain_id: ID::from(decision.decision_chain_id.to_string()),
            status: decision.status,
            proposed_by_id: decision.proposed_by_id.map(|id| ID::from(id.to_string())),
            decided_in_meeting_id: decision.decided_in_meeting_id.map(|id| ID::from(id.to_string())),
            proposed_at: decision.proposed_at,
            decided_at: decision.decided_at,
            impact_score: decision.impact_score,
            created_at: decision.created_at,
            updated_at: decision.updated_at,
        })
    }
}

/// Action item type for GraphQL
#[derive(SimpleObject, Clone, Debug)]
pub struct ActionItemType {
    /// Unique identifier
    pub id: ID,
    /// Description
    pub description: String,
    /// Assignee entity ID
    pub assignee_id: Option<ID>,
    /// Creator entity ID
    pub creator_id: Option<ID>,
    /// Source meeting ID
    pub source_meeting_id: Option<ID>,
    /// Source decision ID
    pub source_decision_id: Option<ID>,
    /// Status
    pub status: String,
    /// Deadline
    pub deadline: Option<DateTime<Utc>>,
    /// When completed
    pub completed_at: Option<DateTime<Utc>>,
    /// Priority
    pub priority: String,
    /// Created at
    pub created_at: DateTime<Utc>,
    /// Updated at
    pub updated_at: DateTime<Utc>,
}

impl TryFrom<crate::models::ActionItem> for ActionItemType {
    type Error = async_graphql::Error;

    fn try_from(item: crate::models::ActionItem) -> Result<Self, Self::Error> {
        Ok(Self {
            id: ID::from(item.id.to_string()),
            description: item.description,
            assignee_id: item.assignee_id.map(|id| ID::from(id.to_string())),
            creator_id: item.creator_id.map(|id| ID::from(id.to_string())),
            source_meeting_id: item.source_meeting_id.map(|id| ID::from(id.to_string())),
            source_decision_id: item.source_decision_id.map(|id| ID::from(id.to_string())),
            status: item.status,
            deadline: item.deadline,
            completed_at: item.completed_at,
            priority: item.priority,
            created_at: item.created_at,
            updated_at: item.updated_at,
        })
    }
}

/// Contradiction type for GraphQL
#[derive(SimpleObject, Clone, Debug)]
pub struct ContradictionType {
    /// Unique identifier
    pub id: ID,
    /// Entity ID with the contradiction
    pub entity_id: ID,
    /// First fact ID
    pub fact1_id: ID,
    /// Second fact ID
    pub fact2_id: ID,
    /// Contradiction type
    pub contradiction_type: String,
    /// Severity level
    pub severity: ContradictionSeverityEnum,
    /// Description of the contradiction
    pub description: String,
    /// Whether resolved
    pub resolved: bool,
    /// When detected
    pub detected_at: DateTime<Utc>,
}

impl TryFrom<crate::validation::models::Contradiction> for ContradictionType {
    type Error = async_graphql::Error;

    fn try_from(c: crate::validation::models::Contradiction) -> Result<Self, Self::Error> {
        Ok(Self {
            id: ID::from(c.id.to_string()),
            entity_id: ID::from(c.entity_id.to_string()),
            fact1_id: ID::from(c.fact1_id.to_string()),
            fact2_id: ID::from(c.fact2_id.to_string()),
            contradiction_type: format!("{:?}", c.contradiction_type),
            severity: c.severity.into(),
            description: c.description,
            resolved: c.resolved(),
            detected_at: c.detected_at,
        })
    }
}

/// Confidence score type for GraphQL
#[derive(SimpleObject, Clone, Debug)]
pub struct ConfidenceScoreType {
    /// Entity ID
    pub entity_id: ID,
    /// Overall confidence score (0.0 - 1.0)
    pub overall: f32,
    /// Confidence tier
    pub tier: String,
    /// Number of sources
    pub source_count: i32,
    /// Average source reliability
    pub avg_source_reliability: f32,
}

/// Duplicate candidate type for GraphQL
#[derive(SimpleObject, Clone, Debug)]
pub struct DuplicateCandidateType {
    /// Primary entity ID
    pub primary_id: ID,
    /// Duplicate entity ID
    pub duplicate_id: ID,
    /// Similarity score (0.0 - 1.0)
    pub similarity_score: f32,
    /// Matching fields
    pub matching_fields: Vec<String>,
    /// Confidence in duplicate detection
    pub confidence: f32,
}

/// Graph node type for graph traversal
#[derive(SimpleObject, Clone, Debug)]
pub struct GraphNode {
    /// Entity at this node
    pub entity: EntityType,
    /// Depth from starting node
    pub depth: i32,
    /// Relationships from this node
    pub relationships: Vec<GraphEdge>,
    /// Child nodes
    pub children: Vec<GraphNode>,
}

/// Graph edge type for graph traversal
#[derive(SimpleObject, Clone, Debug)]
pub struct GraphEdge {
    /// Relationship ID
    pub id: ID,
    /// Relationship type
    pub relation_type: String,
    /// Target entity ID
    pub target_id: ID,
    /// Confidence
    pub confidence: f32,
}

/// Parse a GraphQL ID into a UUID
fn parse_uuid(id: &ID) -> async_graphql::Result<Uuid> {
    id.parse::<Uuid>()
        .map_err(|e| async_graphql::Error::new(format!("Invalid UUID '{}': {}", id, e)))
}

/// Encode an offset as a cursor
fn encode_cursor(offset: usize) -> String {
    use base64::Engine;
    base64::engine::general_purpose::STANDARD.encode(format!("cursor:{}", offset))
}

/// Decode a cursor to an offset
fn decode_cursor(cursor: &str) -> Option<usize> {
    use base64::Engine;
    let decoded = base64::engine::general_purpose::STANDARD.decode(cursor).ok()?;
    let string = String::from_utf8(decoded).ok()?;
    string.strip_prefix("cursor:")?.parse::<usize>().ok()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_entity_type_enum_roundtrip() {
        let types = vec![
            EntityTypeModel::Person,
            EntityTypeModel::Organization,
            EntityTypeModel::Location,
            EntityTypeModel::Event,
            EntityTypeModel::Product,
            EntityTypeModel::Concept,
            EntityTypeModel::Decision,
            EntityTypeModel::ActionItem,
        ];

        for ty in types {
            let graphql_ty: EntityTypeEnum = ty.into();
            let back_to_model: EntityTypeModel = graphql_ty.into();
            assert_eq!(ty, back_to_model);
        }
    }

    #[test]
    fn test_privacy_level_enum_roundtrip() {
        let levels = vec![
            PrivacyLevelModel::Public,
            PrivacyLevelModel::Internal,
            PrivacyLevelModel::Confidential,
            PrivacyLevelModel::Restricted,
        ];

        for level in levels {
            let graphql_level: PrivacyLevelEnum = level.into();
            let back_to_model: PrivacyLevelModel = graphql_level.into();
            assert_eq!(level, back_to_model);
        }
    }

    #[test]
    fn test_parse_uuid_valid() {
        let id = ID::from("550e8400-e29b-41d4-a716-446655440000");
        let result = parse_uuid(&id);
        assert!(result.is_ok());
    }

    #[test]
    fn test_parse_uuid_invalid() {
        let id = ID::from("not-a-uuid");
        let result = parse_uuid(&id);
        assert!(result.is_err());
    }

    #[test]
    fn test_cursor_encoding_roundtrip() {
        let offset = 42;
        let cursor = encode_cursor(offset);
        let decoded = decode_cursor(&cursor);
        assert_eq!(decoded, Some(offset));
    }

    #[test]
    fn test_decode_invalid_cursor() {
        assert_eq!(decode_cursor("invalid"), None);
        assert_eq!(decode_cursor(""), None);
    }

    #[test]
    fn test_entity_metadata_input_conversion() {
        let input = EntityMetadataInput {
            title: Some("Developer".to_string()),
            email: Some("test@example.com".to_string()),
            phone: None,
            industry: Some("Tech".to_string()),
            website: Some("https://example.com".to_string()),
            address: None,
            latitude: None,
            longitude: None,
            event_start: None,
            event_end: None,
            category: None,
            price: None,
            decision_status: None,
            impact: None,
            deadline: None,
            assignee: None,
        };

        let model: EntityMetadataModel = input.into();
        assert_eq!(model.title, Some("Developer".to_string()));
        assert_eq!(model.email, Some("test@example.com".to_string()));
        assert_eq!(model.industry, Some("Tech".to_string()));
    }

    #[test]
    fn test_entity_filter_conversion() {
        let input = EntityFilterInput {
            entity_type: Some(EntityTypeEnum::Person),
            name_contains: Some("Alice".to_string()),
            created_after: None,
            created_before: None,
            min_confidence: Some(0.8),
            verified_only: Some(true),
            privacy_level: Some(PrivacyLevelEnum::Internal),
        };

        let filter: EntityFilter = input.into();
        assert_eq!(filter.entity_type, Some(EntityTypeModel::Person));
        assert_eq!(filter.name_contains, Some("Alice".to_string()));
        assert_eq!(filter.min_confidence, Some(0.8));
        assert!(filter.verified_only);
        assert_eq!(filter.privacy_level, Some("internal".to_string()));
    }
}
