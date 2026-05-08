//! Duplicate Detector
//!
//! Finds potential duplicate entities using semantic similarity,
//! name fuzzy matching, and relationship overlap analysis.

use chrono::{DateTime, Utc};
use std::collections::HashSet;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{debug, instrument, trace, warn};
use uuid::Uuid;

use crate::graph::{GraphQuery, RelationshipRepository};
use crate::models::{Entity, Relationship};
use crate::Result;

use super::{DuplicateCandidate, RelationshipDuplicate, SimilarityConfig};

/// Default similarity weights
pub const DEFAULT_NAME_WEIGHT: f32 = 0.4;
pub const DEFAULT_EMBEDDING_WEIGHT: f32 = 0.4;
pub const DEFAULT_RELATIONSHIP_WEIGHT: f32 = 0.2;

/// Minimum similarity threshold for duplicate detection
pub const DEFAULT_DUPLICATE_THRESHOLD: f32 = 0.75;

/// Duplicate detection engine
pub struct DuplicateDetector<Q: GraphQuery> {
    query_engine: Arc<Q>,
    relationship_repo: Arc<RelationshipRepository>,
    config: SimilarityConfig,
}

/// Configuration for similarity calculation
#[derive(Debug, Clone, Copy)]
pub struct SimilarityWeights {
    /// Weight for name similarity (Jaro-Winkler)
    pub name_weight: f32,
    /// Weight for embedding similarity (cosine)
    pub embedding_weight: f32,
    /// Weight for relationship overlap
    pub relationship_weight: f32,
}

impl Default for SimilarityWeights {
    fn default() -> Self {
        Self {
            name_weight: DEFAULT_NAME_WEIGHT,
            embedding_weight: DEFAULT_EMBEDDING_WEIGHT,
            relationship_weight: DEFAULT_RELATIONSHIP_WEIGHT,
        }
    }
}

impl SimilarityWeights {
    /// Create custom weights
    pub fn new(name: f32, embedding: f32, relationship: f32) -> Self {
        // Normalize weights to sum to 1.0
        let total = name + embedding + relationship;
        if total == 0.0 {
            return Self::default();
        }
        
        Self {
            name_weight: name / total,
            embedding_weight: embedding / total,
            relationship_weight: relationship / total,
        }
    }
    
    /// Validate that weights sum to approximately 1.0
    pub fn is_valid(&self) -> bool {
        let sum = self.name_weight + self.embedding_weight + self.relationship_weight;
        (sum - 1.0).abs() < 0.001
    }
}

impl<Q: GraphQuery> DuplicateDetector<Q> {
    /// Create a new duplicate detector
    pub fn new(
        query_engine: Arc<Q>,
        relationship_repo: Arc<RelationshipRepository>,
    ) -> Self {
        Self {
            query_engine,
            relationship_repo,
            config: SimilarityConfig::default(),
        }
    }
    
    /// Create with custom configuration
    pub fn with_config(
        query_engine: Arc<Q>,
        relationship_repo: Arc<RelationshipRepository>,
        config: SimilarityConfig,
    ) -> Self {
        Self {
            query_engine,
            relationship_repo,
            config,
        }
    }
    
    /// Find potential duplicate entities above similarity threshold
    #[instrument(skip(self), level = "debug")]
    pub async fn find_duplicate_entities(
        &self,
        threshold: f32,
    ) -> Result<Vec<DuplicateCandidate>> {
        let threshold = threshold.max(0.0).min(1.0);
        debug!("Finding duplicate entities with threshold {}", threshold);
        
        // Get all entities (this could be optimized with batching for large datasets)
        let entities = self.get_all_entities().await?;
        trace!("Checking {} entities for duplicates", entities.len());
        
        let mut candidates = Vec::new();
        let checked_pairs = Arc::new(RwLock::new(HashSet::<(Uuid, Uuid)>::new()));
        
        // Compare each entity with others
        for (i, entity_a) in entities.iter().enumerate() {
            // Skip if entity has been marked as duplicate of another
            if self.should_skip_entity(entity_a).await? {
                continue;
            }
            
            for entity_b in entities.iter().skip(i + 1) {
                // Ensure consistent ordering to avoid duplicate pairs
                let pair = if entity_a.id < entity_b.id {
                    (entity_a.id, entity_b.id)
                } else {
                    (entity_b.id, entity_a.id)
                };
                
                // Skip if already checked
                {
                    let checked = checked_pairs.read().await;
                    if checked.contains(&pair) {
                        continue;
                    }
                }
                
                // Check if entities are of compatible types
                if !self.are_types_compatible(&entity_a.entity_type, &entity_b.entity_type) {
                    continue;
                }
                
                // Calculate similarity
                let similarity = self.calculate_similarity(entity_a, entity_b).await?;
                
                {
                    let mut checked = checked_pairs.write().await;
                    checked.insert(pair);
                }
                
                if similarity.combined >= threshold {
                    debug!(
                        "Found duplicate candidate: {} <-> {} (score: {:.3})",
                        entity_a.name, entity_b.name, similarity.combined
                    );
                    
                    candidates.push(DuplicateCandidate {
                        entity_a_id: entity_a.id,
                        entity_b_id: entity_b.id,
                        similarity_score: similarity.combined,
                        name_similarity: similarity.name,
                        embedding_similarity: similarity.embedding,
                        relationship_overlap: similarity.relationship,
                        detected_at: Utc::now(),
                    });
                }
            }
        }
        
        // Sort by similarity score (highest first)
        candidates.sort_by(|a, b| {
            b.similarity_score
                .partial_cmp(&a.similarity_score)
                .unwrap_or(std::cmp::Ordering::Equal)
        });
        
        debug!("Found {} duplicate candidates", candidates.len());
        Ok(candidates)
    }
    
    /// Find duplicate relationships
    #[instrument(skip(self), level = "debug")]
    pub async fn find_duplicate_relationships(&self) -> Result<Vec<RelationshipDuplicate>> {
        debug!("Finding duplicate relationships");
        
        // Get all relationships
        let relationships = self.get_all_relationships().await?;
        trace!("Checking {} relationships for duplicates", relationships.len());
        
        let mut duplicates = Vec::new();
        let mut checked = HashSet::new();
        
        for (i, rel_a) in relationships.iter().enumerate() {
            // Skip archived relationships
            if rel_a.archived {
                continue;
            }
            
            for rel_b in relationships.iter().skip(i + 1) {
                // Skip archived relationships
                if rel_b.archived {
                    continue;
                }
                
                // Create unique pair identifier
                let pair = if rel_a.id < rel_b.id {
                    (rel_a.id, rel_b.id)
                } else {
                    (rel_b.id, rel_a.id)
                };
                
                if checked.contains(&pair) {
                    continue;
                }
                checked.insert(pair);
                
                // Check if relationships are equivalent
                if rel_a.is_equivalent_to(rel_b) {
                    // Determine which relationship to keep (highest confidence)
                    let (keep_id, remove_id) = if rel_a.extraction_confidence >= rel_b.extraction_confidence {
                        (rel_a.id, rel_b.id)
                    } else {
                        (rel_b.id, rel_a.id)
                    };
                    
                    duplicates.push(RelationshipDuplicate {
                        relationship_a_id: rel_a.id,
                        relationship_b_id: rel_b.id,
                        keep_id,
                        remove_id,
                        confidence_delta: (rel_a.extraction_confidence - rel_b.extraction_confidence).abs(),
                    });
                }
            }
        }
        
        debug!("Found {} duplicate relationships", duplicates.len());
        Ok(duplicates)
    }
    
    /// Calculate similarity between two entities
    #[instrument(skip(self), level = "trace")]
    async fn calculate_similarity(
        &self,
        entity_a: &Entity,
        entity_b: &Entity,
    ) -> Result<SimilarityScores> {
        let name_sim = calculate_name_similarity(&entity_a.name, &entity_b.name);
        
        let embedding_sim = if let (Some(emb_a), Some(emb_b)) = (&entity_a.embedding, &entity_b.embedding) {
            calculate_embedding_similarity(emb_a, emb_b)
        } else {
            0.0
        };
        
        let rel_sim = self.calculate_relationship_overlap(&entity_a.id, &entity_b.id).await?;
        
        let combined = name_sim * self.config.weights.name_weight
            + embedding_sim * self.config.weights.embedding_weight
            + rel_sim * self.config.weights.relationship_weight;
        
        Ok(SimilarityScores {
            combined,
            name: name_sim,
            embedding: embedding_sim,
            relationship: rel_sim,
        })
    }
    
    /// Calculate relationship overlap between two entities
    #[instrument(skip(self), level = "trace")]
    async fn calculate_relationship_overlap(
        &self,
        entity_a_id: &Uuid,
        entity_b_id: &Uuid,
    ) -> Result<f32> {
        let rels_a = self.relationship_repo.get_for_entity(entity_a_id).await?;
        let rels_b = self.relationship_repo.get_for_entity(entity_b_id).await?;
        
        if rels_a.is_empty() && rels_b.is_empty() {
            return Ok(0.0); // No relationships to compare
        }
        
        // Create sets of relationship signatures
        let sigs_a: HashSet<String> = rels_a
            .iter()
            .filter(|r| !r.archived)
            .map(|r| format!("{}:{}", r.target_id, r.relation_type))
            .collect();
        
        let sigs_b: HashSet<String> = rels_b
            .iter()
            .filter(|r| !r.archived)
            .map(|r| format!("{}:{}", r.target_id, r.relation_type))
            .collect();
        
        if sigs_a.is_empty() && sigs_b.is_empty() {
            return Ok(0.0);
        }
        
        // Calculate Jaccard similarity
        let intersection: HashSet<_> = sigs_a.intersection(&sigs_b).collect();
        let union: HashSet<_> = sigs_a.union(&sigs_b).collect();
        
        let jaccard = if union.is_empty() {
            0.0
        } else {
            intersection.len() as f32 / union.len() as f32
        };
        
        Ok(jaccard)
    }
    
    /// Check if entity types are compatible for duplicate detection
    fn are_types_compatible(&self, type_a: &crate::models::EntityType, type_b: &crate::models::EntityType) -> bool {
        // Same types are always compatible
        if type_a == type_b {
            return true;
        }
        
        // Allow certain cross-type matches (e.g., Decision and ActionItem might be similar)
        // This is configurable based on use case
        match (type_a, type_b) {
            // For now, only match same types
            _ => false,
        }
    }
    
    /// Check if entity should be skipped in duplicate detection
    async fn should_skip_entity(&self, entity: &Entity) -> Result<bool> {
        // Skip entities with very low confidence
        if entity.extraction_confidence < self.config.min_confidence {
            return Ok(true);
        }
        
        // Skip entities marked as verified (manual review)
        if entity.verified && self.config.skip_verified {
            return Ok(true);
        }
        
        Ok(false)
    }
    
    /// Get all entities from the graph
    async fn get_all_entities(&self) -> Result<Vec<Entity>> {
        // Use a large limit to get all entities
        let filter = crate::graph::EntityFilter::new()
            .with_pagination(0, 10000);
        
        let result = self.query_engine.search_entities(filter).await?;
        Ok(result.items)
    }
    
    /// Get all relationships
    async fn get_all_relationships(&self) -> Result<Vec<Relationship>> {
        // This would ideally be a repository method
        // For now, return empty (would be implemented in real system)
        Ok(Vec::new())
    }
}

/// Individual similarity scores
#[derive(Debug, Clone, Copy)]
struct SimilarityScores {
    combined: f32,
    name: f32,
    embedding: f32,
    relationship: f32,
}

/// Calculate name similarity using Jaro-Winkler algorithm
fn calculate_name_similarity(name_a: &str, name_b: &str) -> f32 {
    let a = name_a.to_lowercase();
    let b = name_b.to_lowercase();
    
    // Exact match
    if a == b {
        return 1.0;
    }
    
    // Check for substring match
    if a.contains(&b) || b.contains(&a) {
        let ratio = a.len().min(b.len()) as f32 / a.len().max(b.len()) as f32;
        return 0.8 + (0.2 * ratio); // Score between 0.8 and 1.0
    }
    
    // Jaro-Winkler similarity
    jaro_winkler_similarity(&a, &b)
}

/// Jaro-Winkler similarity algorithm
fn jaro_winkler_similarity(s1: &str, s2: &str) -> f32 {
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
    
    // Match window
    let match_window = (len1.max(len2) / 2).saturating_sub(1);
    
    let mut s1_matches = vec![false; len1];
    let mut s2_matches = vec![false; len2];
    
    let mut matches = 0;
    let mut transpositions = 0;
    
    // Find matches
    for i in 0..len1 {
        let start = i.saturating_sub(match_window);
        let end = (i + match_window + 1).min(len2);
        
        for j in start..end {
            if s2_matches[j] || s1_chars[i] != s2_chars[j] {
                continue;
            }
            s1_matches[i] = true;
            s2_matches[j] = true;
            matches += 1;
            break;
        }
    }
    
    if matches == 0 {
        return 0.0;
    }
    
    // Count transpositions
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
    
    // Jaro similarity
    let matches_f = matches as f32;
    let jaro = ((matches_f / len1 as f32)
        + (matches_f / len2 as f32)
        + ((matches_f - transpositions as f32 / 2.0) / matches_f))
        / 3.0;
    
    // Jaro-Winkler prefix bonus
    let prefix_len = s1_chars
        .iter()
        .zip(s2_chars.iter())
        .take(4)
        .take_while(|(a, b)| a == b)
        .count();
    
    let prefix_scale = 0.1;
    jaro + (prefix_len as f32 * prefix_scale * (1.0 - jaro))
}

/// Calculate cosine similarity between two embedding vectors
fn calculate_embedding_similarity(emb_a: &[f32], emb_b: &[f32]) -> f32 {
    if emb_a.len() != emb_b.len() {
        warn!(
            "Embedding dimension mismatch: {} vs {}",
            emb_a.len(),
            emb_b.len()
        );
        return 0.0;
    }
    
    if emb_a.is_empty() {
        return 0.0;
    }
    
    let dot_product: f32 = emb_a.iter().zip(emb_b.iter()).map(|(a, b)| a * b).sum();
    
    let norm_a: f32 = emb_a.iter().map(|x| x * x).sum::<f32>().sqrt();
    let norm_b: f32 = emb_b.iter().map(|x| x * x).sum::<f32>().sqrt();
    
    if norm_a == 0.0 || norm_b == 0.0 {
        return 0.0;
    }
    
    (dot_product / (norm_a * norm_b)).clamp(-1.0, 1.0)
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_jaro_winkler_similarity() {
        // Exact match
        assert!((jaro_winkler_similarity("hello", "hello") - 1.0).abs() < 0.001);
        
        // Similar strings
        let sim = jaro_winkler_similarity("martha", "marhta");
        assert!(sim > 0.9, "Expected high similarity for 'martha'/'marhta', got {}", sim);
        
        // Different strings
        let sim = jaro_winkler_similarity("abc", "xyz");
        assert!(sim < 0.5, "Expected low similarity for 'abc'/'xyz', got {}", sim);
        
        // Empty strings
        assert_eq!(jaro_winkler_similarity("", ""), 1.0);
        assert_eq!(jaro_winkler_similarity("abc", ""), 0.0);
    }
    
    #[test]
    fn test_name_similarity() {
        // Exact match
        assert!((calculate_name_similarity("John Doe", "John Doe") - 1.0).abs() < 0.001);
        
        // Case insensitive
        assert!((calculate_name_similarity("John Doe", "john doe") - 1.0).abs() < 0.001);
        
        // Substring match
        let sim = calculate_name_similarity("Johnathan Doe", "John");
        assert!(sim > 0.8, "Expected high substring similarity, got {}", sim);
    }
    
    #[test]
    fn test_embedding_similarity() {
        // Identical vectors
        let emb = vec![1.0, 0.0, 0.0];
        assert!((calculate_embedding_similarity(&emb, &emb) - 1.0).abs() < 0.001);
        
        // Orthogonal vectors
        let emb_a = vec![1.0, 0.0, 0.0];
        let emb_b = vec![0.0, 1.0, 0.0];
        assert!(calculate_embedding_similarity(&emb_a, &emb_b).abs() < 0.001);
        
        // Opposite vectors
        let emb_a = vec![1.0, 0.0, 0.0];
        let emb_b = vec![-1.0, 0.0, 0.0];
        assert!((calculate_embedding_similarity(&emb_a, &emb_b) - (-1.0)).abs() < 0.001);
    }
    
    #[test]
    fn test_similarity_weights() {
        let weights = SimilarityWeights::default();
        assert!(weights.is_valid());
        assert_eq!(weights.name_weight, DEFAULT_NAME_WEIGHT);
        
        let custom = SimilarityWeights::new(0.5, 0.3, 0.2);
        assert!(custom.is_valid());
        assert!((custom.name_weight - 0.5).abs() < 0.001);
    }
    
    #[test]
    fn test_similarity_weights_normalization() {
        let weights = SimilarityWeights::new(10.0, 20.0, 20.0);
        assert!(weights.is_valid());
        assert!((weights.name_weight - 0.2).abs() < 0.001);
        assert!((weights.embedding_weight - 0.4).abs() < 0.001);
        assert!((weights.relationship_weight - 0.4).abs() < 0.001);
    }
}
