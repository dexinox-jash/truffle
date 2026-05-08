//! Entity Resolution
//!
//! Resolves extracted entities to canonical entities in the database.

use crate::pipeline::extraction::ner::{RawEntity, ExtractedEntity};
use crate::models::{Entity, EntityType};
use crate::Result;

/// Entity resolution configuration
#[derive(Debug, Clone)]
pub struct ResolutionConfig {
    /// Similarity threshold for fuzzy matching (0.0 - 1.0)
    pub similarity_threshold: f32,
    /// Enable alias matching
    pub enable_alias_matching: bool,
    /// Enable semantic similarity (requires embeddings)
    pub enable_semantic_similarity: bool,
}

impl Default for ResolutionConfig {
    fn default() -> Self {
        Self {
            similarity_threshold: 0.85,
            enable_alias_matching: true,
            enable_semantic_similarity: false, // Disabled until embeddings ready
        }
    }
}

/// Entity resolver
pub struct EntityResolver {
    config: ResolutionConfig,
}

/// Resolved entity (linked to existing or new)
#[derive(Debug, Clone)]
pub enum ResolvedEntity {
    /// Linked to existing entity
    Linked {
        raw: RawEntity,
        canonical_id: uuid::Uuid,
        confidence: f32,
    },
    /// New entity to create
    New(RawEntity),
}

impl EntityResolver {
    /// Create a new entity resolver
    pub fn new(config: &ResolutionConfig) -> Result<Self> {
        Ok(Self {
            config: config.clone(),
        })
    }
    
    /// Resolve extracted entities to canonical entities
    pub async fn resolve(&self, entities: Vec<ExtractedEntity>) -> Result<Vec<ResolvedEntity>> {
        let mut resolved = Vec::new();
        
        for extracted in entities {
            // For now, create new entities
            // In production, query database for existing entities
            resolved.push(ResolvedEntity::New(extracted.raw));
        }
        
        Ok(resolved)
    }
    
    /// Calculate string similarity (Levenshtein distance normalized)
    fn calculate_similarity(&self, s1: &str, s2: &str) -> f32 {
        let s1_lower = s1.to_lowercase();
        let s2_lower = s2.to_lowercase();
        
        if s1_lower == s2_lower {
            return 1.0;
        }
        
        // Simple Jaccard similarity on character bigrams
        let bigrams1: std::collections::HashSet<_> = s1_lower.chars().collect::<Vec<_>>()
            .windows(2).map(|w| format!("{}{}", w[0], w[1])).collect();
        let bigrams2: std::collections::HashSet<_> = s2_lower.chars().collect::<Vec<_>>()
            .windows(2).map(|w| format!("{}{}", w[0], w[1])).collect();
        
        let intersection: std::collections::HashSet<_> = bigrams1.intersection(&bigrams2).collect();
        let union: std::collections::HashSet<_> = bigrams1.union(&bigrams2).collect();
        
        if union.is_empty() {
            return 0.0;
        }
        
        intersection.len() as f32 / union.len() as f32
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_resolver_creation() {
        let config = ResolutionConfig::default();
        let resolver = EntityResolver::new(&config).unwrap();
        assert!(resolver.config.enable_alias_matching);
    }
    
    #[test]
    fn test_similarity_exact_match() {
        let config = ResolutionConfig::default();
        let resolver = EntityResolver::new(&config).unwrap();
        
        assert_eq!(resolver.calculate_similarity("Alice", "Alice"), 1.0);
        assert_eq!(resolver.calculate_similarity("Alice", "alice"), 1.0);
    }
    
    #[test]
    fn test_similarity_different() {
        let config = ResolutionConfig::default();
        let resolver = EntityResolver::new(&config).unwrap();
        
        let sim = resolver.calculate_similarity("Alice", "Bob");
        assert!(sim < 0.5);
    }
}
