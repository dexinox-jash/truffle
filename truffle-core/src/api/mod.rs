//! GraphQL API Module for Truffle Core
//!
//! This module provides the GraphQL schema and context for the knowledge graph API,
//! enabling flexible querying of entities, relationships, decisions, and action items.
//!
//! # Example
//!
//! ```rust,no_run
//! use truffle_core::api::{create_schema, ApiContext};
//! use std::sync::Arc;
//!
//! # async fn example() -> anyhow::Result<()> {
//! let context = ApiContext::new(/* database */);
//! let schema = create_schema();
//!
//! let response = schema
//!     .execute_with_context("{ entities { id name } }", context)
//!     .await;
//! # Ok(())
//! # }
//! ```

pub mod context;
pub mod resolvers;
pub mod schema;

pub use context::ApiContext;
pub use resolvers::{QueryRoot, MutationRoot, ValidationQueryRoot};
pub use schema::{
    create_schema, ApiSchema, EntityType, EntityMetadataType, EntityMetadataInput,
    RelationshipType, RelationshipMetadataType, CreateEntityInput, UpdateEntityInput,
    CreateRelationshipInput, UpdateRelationshipInput, EntityFilterInput,
    RelationshipFilterInput, EntityTypeEnum, DecisionStatusEnum, ActionItemStatusEnum,
    PriorityEnum, PrivacyLevelEnum, ContradictionSeverityEnum, PageInfo, EntityConnection,
    RelationshipConnection, SortDirection, EntitySortField, EntitySortInput, 
    RelationshipSortField, RelationshipSortInput, MeetingFilterInput,
    MeetingType, DecisionType, ActionItemType, ContradictionType, ConfidenceScoreType,
    DuplicateCandidateType, GraphNode, GraphEdge,
};
