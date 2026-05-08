//! Validation Resolvers
//!
//! GraphQL resolvers for validation queries (contradictions, confidence scores, duplicates).
//! Note: These queries are also available on the main QueryRoot for convenience.
//! This module can be extended for dedicated validation endpoints.

pub use crate::api::schema::QueryRoot as ValidationQueryRoot;
