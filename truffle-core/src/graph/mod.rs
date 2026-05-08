//! Graph Query API
//!
//! Query and traversal operations for the knowledge graph.

pub mod query;
pub mod repository;
pub mod traversal;

pub use query::{GraphQuery, QueryResult, EntityFilter, RelationshipFilter};
pub use repository::{EntityRepository, RelationshipRepository, GraphRepository};
pub use traversal::{GraphTraversal, PathFinder, TraversalConfig};

use crate::models::{Entity, Relationship};
use crate::Result;

/// Graph module configuration
#[derive(Debug, Clone)]
pub struct GraphConfig {
    /// Maximum traversal depth
    pub max_traversal_depth: usize,
    /// Default result limit
    pub default_limit: usize,
    /// Enable caching
    pub enable_caching: bool,
}

impl Default for GraphConfig {
    fn default() -> Self {
        Self {
            max_traversal_depth: 5,
            default_limit: 100,
            enable_caching: true,
        }
    }
}

/// Graph service
pub struct GraphService {
    config: GraphConfig,
    repository: GraphRepository,
}

impl GraphService {
    /// Create a new graph service
    pub fn new(config: GraphConfig, repository: GraphRepository) -> Self {
        Self { config, repository }
    }
    
    /// Get repository reference
    pub fn repository(&self) -> &GraphRepository {
        &self.repository
    }
    
    /// Get mutable repository reference
    pub fn repository_mut(&mut self) -> &mut GraphRepository {
        &mut self.repository
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_graph_config_default() {
        let config = GraphConfig::default();
        assert_eq!(config.max_traversal_depth, 5);
        assert_eq!(config.default_limit, 100);
        assert!(config.enable_caching);
    }
}
