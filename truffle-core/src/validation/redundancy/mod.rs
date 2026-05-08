//! Redundancy Detection Module
//!
//! Semantic duplicate detection and merge candidate identification
//! for the Truffle knowledge graph.
//!
//! # Overview
//!
//! This module provides tools for detecting and resolving redundancy
//! in the knowledge graph:
//!
//! - **DuplicateDetector**: Finds potential duplicate entities using
//!   combined similarity metrics (name, embedding, relationships)
//! - **MergeEngine**: Handles entity merging with conflict resolution
//!   and provenance preservation
//! - **RelationshipDeduplicator**: Identifies and removes redundant
//!   relationships between entities
//!
//! # Example Usage
//!
//! ```rust,no_run
//! use truffle_core::validation::redundancy::{
//!     DuplicateDetector, MergeEngine, RelationshipDeduplicator,
//!     SimilarityConfig, MergeConfig
//! };
//!
//! # async fn example() -> anyhow::Result<()> {
//! // Create detector with default configuration
//! let detector = DuplicateDetector::new(
//!     graph_query_engine,
//!     relationship_repository,
//! );
//!
//! // Find duplicates with 0.8 similarity threshold
//! let candidates = detector.find_duplicate_entities(0.8).await?;
//!
//! // Preview and execute merge
//! let merge_engine = MergeEngine::new(repository);
//! for candidate in candidates {
//!     let preview = merge_engine.preview_merge(&candidate).await?;
//!     if preview.stats.requires_review == 0 {
//!         let result = merge_engine.execute_merge(&candidate).await?;
//!     }
//! }
//! # Ok(())
//! # }
//! ```

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

// Module exports
mod detector;
mod merge_engine;
mod relationship_dedup;

// Public re-exports
pub use detector::{
    DuplicateDetector,
    SimilarityWeights,
    DEFAULT_NAME_WEIGHT,
    DEFAULT_EMBEDDING_WEIGHT,
    DEFAULT_RELATIONSHIP_WEIGHT,
    DEFAULT_DUPLICATE_THRESHOLD,
};

pub use merge_engine::{
    MergeEngine,
    MergeConfig,
    ConflictResolutionStrategy,
    ConflictStatus,
    MergeStats,
};

pub use relationship_dedup::{
    RelationshipDeduplicator,
    DedupConfig,
    DedupConflictStrategy,
    DedupResult,
    DuplicateGroup,
    RelationshipSignature,
    RelationshipStats,
    BatchDeduplicator,
};

/// Configuration for similarity calculation
#[derive(Debug, Clone, Copy)]
pub struct SimilarityConfig {
    /// Similarity threshold for duplicate detection
    pub threshold: f32,
    /// Weights for different similarity components
    pub weights: SimilarityWeights,
    /// Minimum confidence for entities to be considered
    pub min_confidence: f32,
    /// Skip verified entities in detection
    pub skip_verified: bool,
    /// Include embedding similarity in calculation
    pub use_embeddings: bool,
    /// Include relationship overlap in calculation
    pub use_relationships: bool,
}

impl Default for SimilarityConfig {
    fn default() -> Self {
        Self {
            threshold: 0.75,
            weights: SimilarityWeights::default(),
            min_confidence: 0.5,
            skip_verified: true,
            use_embeddings: true,
            use_relationships: true,
        }
    }
}

/// Potential duplicate entity pair
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DuplicateCandidate {
    /// First entity ID
    pub entity_a_id: Uuid,
    /// Second entity ID
    pub entity_b_id: Uuid,
    /// Combined similarity score (0.0 - 1.0)
    pub similarity_score: f32,
    /// Name similarity component
    pub name_similarity: f32,
    /// Embedding similarity component
    pub embedding_similarity: f32,
    /// Relationship overlap component
    pub relationship_overlap: f32,
    /// When this candidate was detected
    pub detected_at: DateTime<Utc>,
}

impl DuplicateCandidate {
    /// Create a new duplicate candidate
    pub fn new(
        entity_a_id: Uuid,
        entity_b_id: Uuid,
        similarity_score: f32,
    ) -> Self {
        Self {
            entity_a_id,
            entity_b_id,
            similarity_score: similarity_score.clamp(0.0, 1.0),
            name_similarity: 0.0,
            embedding_similarity: 0.0,
            relationship_overlap: 0.0,
            detected_at: Utc::now(),
        }
    }
    
    /// Set component scores
    pub fn with_scores(
        mut self,
        name: f32,
        embedding: f32,
        relationship: f32,
    ) -> Self {
        self.name_similarity = name.clamp(0.0, 1.0);
        self.embedding_similarity = embedding.clamp(0.0, 1.0);
        self.relationship_overlap = relationship.clamp(0.0, 1.0);
        self
    }
    
    /// Get the similarity breakdown as a formatted string
    pub fn similarity_breakdown(&self) -> String {
        format!(
            "Combined: {:.2} (name: {:.2}, embedding: {:.2}, relationships: {:.2})",
            self.similarity_score,
            self.name_similarity,
            self.embedding_similarity,
            self.relationship_overlap
        )
    }
    
    /// Check if this candidate meets the threshold for merging
    pub fn meets_threshold(&self, threshold: f32) -> bool {
        self.similarity_score >= threshold
    }
    
    /// Check if all component scores are above their respective thresholds
    pub fn all_components_above(&self, name: f32, embedding: f32, relationship: f32) -> bool {
        self.name_similarity >= name
            && self.embedding_similarity >= embedding
            && self.relationship_overlap >= relationship
    }
}

/// Duplicate relationship pair
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RelationshipDuplicate {
    /// First relationship ID
    pub relationship_a_id: Uuid,
    /// Second relationship ID
    pub relationship_b_id: Uuid,
    /// Relationship to keep
    pub keep_id: Uuid,
    /// Relationship to remove
    pub remove_id: Uuid,
    /// Confidence difference between the two
    pub confidence_delta: f32,
}

/// Preview of a merge operation
#[derive(Debug, Clone)]
pub struct MergePreview {
    /// The duplicate candidate being merged
    pub candidate: DuplicateCandidate,
    /// Entity that will be kept (primary)
    pub primary_entity: crate::models::Entity,
    /// Entity that will be merged/removed (secondary)
    pub secondary_entity: crate::models::Entity,
    /// Preview of the merged entity
    pub merged_preview: crate::models::Entity,
    /// Detected conflicts
    pub conflicts: Vec<MergeConflict>,
    /// Relationships that will be affected
    pub affected_relationships: Vec<crate::models::Relationship>,
    /// Merge statistics
    pub stats: MergeStats,
}

/// Conflict detected during merge preview
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MergeConflict {
    /// Field with conflict
    pub field: String,
    /// Value from primary entity
    pub primary_value: Option<String>,
    /// Value from secondary entity
    pub secondary_value: Option<String>,
    /// Current resolution status
    #[serde(skip)]
    pub status: ConflictStatus,
    /// Whether this conflict can be auto-resolved
    #[serde(skip)]
    pub can_auto_resolve: bool,
    /// Resolved value (if any)
    #[serde(skip)]
    pub resolved_value: Option<String>,
}

impl MergeConflict {
    /// Create a new conflict
    pub fn new(
        field: impl Into<String>,
        primary: Option<String>,
        secondary: Option<String>,
    ) -> Self {
        Self {
            field: field.into(),
            primary_value: primary,
            secondary_value: secondary,
            status: ConflictStatus::Pending,
            can_auto_resolve: false,
            resolved_value: None,
        }
    }
    
    /// Mark as resolved with a specific value
    pub fn resolve_with(mut self, value: impl Into<String>) -> Self {
        self.resolved_value = Some(value.into());
        self.status = ConflictStatus::UserResolved;
        self
    }
    
    /// Check if this conflict involves different values
    pub fn has_different_values(&self) -> bool {
        match (&self.primary_value, &self.secondary_value) {
            (Some(a), Some(b)) => a != b,
            (None, Some(_)) | (Some(_), None) => true,
            (None, None) => false,
        }
    }
    
    /// Get a description of the conflict
    pub fn description(&self) -> String {
        format!(
            "{}: '{}' vs '{}'",
            self.field,
            self.primary_value.as_deref().unwrap_or("(none)"),
            self.secondary_value.as_deref().unwrap_or("(none)")
        )
    }
}

/// Result of a merge operation
#[derive(Debug, Clone)]
pub struct MergeResult {
    /// The merged entity
    pub merged_entity: crate::models::Entity,
    /// Provenance record for the merge
    pub provenance: ProvenanceRecord,
    /// IDs of updated relationships
    pub updated_relationships: Vec<Uuid>,
    /// ID of archived entity (if archiving was enabled)
    pub archived_entity_id: Option<Uuid>,
}

/// Provenance record for merge operations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProvenanceRecord {
    /// Unique merge operation ID
    pub merge_id: Uuid,
    /// Primary entity ID (kept)
    pub primary_id: Uuid,
    /// Secondary entity ID (merged)
    pub secondary_id: Uuid,
    /// Merged entity ID (usually same as primary)
    pub merged_id: Uuid,
    /// When the merge occurred
    pub merged_at: DateTime<Utc>,
    /// Number of conflicts that were resolved
    pub conflicts_resolved: usize,
    /// Number of relationships updated
    pub relationships_updated: usize,
}

impl ProvenanceRecord {
    /// Create a new provenance record
    pub fn new(primary_id: Uuid, secondary_id: Uuid, merged_id: Uuid) -> Self {
        Self {
            merge_id: Uuid::new_v4(),
            primary_id,
            secondary_id,
            merged_id,
            merged_at: Utc::now(),
            conflicts_resolved: 0,
            relationships_updated: 0,
        }
    }
    
    /// Set statistics
    pub fn with_stats(mut self, conflicts: usize, relationships: usize) -> Self {
        self.conflicts_resolved = conflicts;
        self.relationships_updated = relationships;
        self
    }
}

/// Batch detection results
#[derive(Debug, Clone)]
pub struct BatchDetectionResult {
    /// Total entities scanned
    pub entities_scanned: usize,
    /// Pairs compared
    pub pairs_compared: usize,
    /// Candidates found
    pub candidates_found: Vec<DuplicateCandidate>,
    /// Processing duration in milliseconds
    pub duration_ms: u64,
}

impl BatchDetectionResult {
    /// Create a new batch result
    pub fn new(entities_scanned: usize, pairs_compared: usize) -> Self {
        Self {
            entities_scanned,
            pairs_compared,
            candidates_found: Vec::new(),
            duration_ms: 0,
        }
    }
    
    /// Add candidates
    pub fn with_candidates(mut self, candidates: Vec<DuplicateCandidate>) -> Self {
        self.candidates_found = candidates;
        self
    }
    
    /// Set duration
    pub fn with_duration(mut self, ms: u64) -> Self {
        self.duration_ms = ms;
        self
    }
    
    /// Get count of candidates above a threshold
    pub fn count_above_threshold(&self, threshold: f32) -> usize {
        self.candidates_found
            .iter()
            .filter(|c| c.similarity_score >= threshold)
            .count()
    }
    
    /// Get candidates sorted by similarity score
    pub fn sorted_candidates(&self) -> Vec<&DuplicateCandidate> {
        let mut sorted: Vec<_> = self.candidates_found.iter().collect();
        sorted.sort_by(|a, b| {
            b.similarity_score
                .partial_cmp(&a.similarity_score)
                .unwrap_or(std::cmp::Ordering::Equal)
        });
        sorted
    }
}

/// Redundancy service that combines all detection capabilities
pub struct RedundancyService {
    /// Duplicate detector
    pub detector: DuplicateDetector<crate::graph::GraphRepository>,
    /// Merge engine
    pub merge_engine: MergeEngine,
    /// Relationship deduplicator
    pub relationship_dedup: RelationshipDeduplicator,
}

impl RedundancyService {
    /// Create a new redundancy service
    pub fn new(
        repository: crate::graph::GraphRepository,
    ) -> Self {
        let repo_arc = std::sync::Arc::new(repository);
        
        Self {
            detector: DuplicateDetector::new(
                repo_arc.clone(),
                std::sync::Arc::new(crate::graph::RelationshipRepository::new(
                    std::sync::Arc::clone(&repo_arc).entities().db.clone()
                )),
            ),
            merge_engine: MergeEngine::new((*repo_arc).clone()),
            relationship_dedup: RelationshipDeduplicator::new(
                crate::graph::RelationshipRepository::new(
                    repo_arc.entities().db.clone()
                )
            ),
        }
    }
    
    /// Run full redundancy detection and cleanup
    pub async fn run_full_cleanup(&self, threshold: f32) -> Result<FullCleanupResult, crate::Error> {
        // Find duplicate entities
        let entity_candidates = self.detector.find_duplicate_entities(threshold).await?;
        
        // Find duplicate relationships
        let rel_duplicates = self.relationship_dedup.find_duplicate_groups().await?;
        
        // Auto-merge high-confidence entity duplicates
        let mut merged_entities = Vec::new();
        for candidate in &entity_candidates {
            if candidate.similarity_score > 0.95 {
                match self.merge_engine.execute_merge(candidate).await {
                    Ok(result) => merged_entities.push(result),
                    Err(e) => {
                        tracing::warn!("Auto-merge failed for {:?}: {}", candidate, e);
                    }
                }
            }
        }
        
        // Execute relationship deduplication
        let dedup_result = self.relationship_dedup.execute_dedup().await?;
        
        Ok(FullCleanupResult {
            entity_candidates_found: entity_candidates.len(),
            entities_auto_merged: merged_entities.len(),
            relationship_groups_found: rel_duplicates.len(),
            relationships_deduplicated: dedup_result.archived + dedup_result.deleted,
            merge_results: merged_entities,
            dedup_result,
        })
    }
}

/// Result of a full cleanup operation
#[derive(Debug, Clone)]
pub struct FullCleanupResult {
    /// Number of entity duplicate candidates found
    pub entity_candidates_found: usize,
    /// Number of entities automatically merged
    pub entities_auto_merged: usize,
    /// Number of relationship duplicate groups found
    pub relationship_groups_found: usize,
    /// Number of relationships deduplicated
    pub relationships_deduplicated: usize,
    /// Results of entity merge operations
    pub merge_results: Vec<MergeResult>,
    /// Relationship deduplication result
    pub dedup_result: DedupResult,
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_duplicate_candidate_creation() {
        let id_a = Uuid::new_v4();
        let id_b = Uuid::new_v4();
        
        let candidate = DuplicateCandidate::new(id_a, id_b, 0.85)
            .with_scores(0.9, 0.8, 0.85);
        
        assert_eq!(candidate.entity_a_id, id_a);
        assert_eq!(candidate.entity_b_id, id_b);
        assert!((candidate.similarity_score - 0.85).abs() < 0.001);
        assert!(candidate.meets_threshold(0.8));
        assert!(!candidate.meets_threshold(0.9));
    }
    
    #[test]
    fn test_merge_conflict() {
        let conflict = MergeConflict::new(
            "name",
            Some("John Doe".to_string()),
            Some("John D.".to_string()),
        );
        
        assert!(conflict.has_different_values());
        assert!(conflict.description().contains("John Doe"));
        
        let resolved = conflict.resolve_with("John Doe");
        assert_eq!(resolved.resolved_value, Some("John Doe".to_string()));
    }
    
    #[test]
    fn test_provenance_record() {
        let id_a = Uuid::new_v4();
        let id_b = Uuid::new_v4();
        let id_merged = Uuid::new_v4();
        
        let record = ProvenanceRecord::new(id_a, id_b, id_merged)
            .with_stats(3, 10);
        
        assert_eq!(record.primary_id, id_a);
        assert_eq!(record.secondary_id, id_b);
        assert_eq!(record.conflicts_resolved, 3);
        assert_eq!(record.relationships_updated, 10);
    }
    
    #[test]
    fn test_batch_detection_result() {
        let mut result = BatchDetectionResult::new(100, 4950);
        
        let candidates = vec![
            DuplicateCandidate::new(Uuid::new_v4(), Uuid::new_v4(), 0.9),
            DuplicateCandidate::new(Uuid::new_v4(), Uuid::new_v4(), 0.7),
        ];
        
        result = result.with_candidates(candidates);
        result = result.with_duration(1500);
        
        assert_eq!(result.entities_scanned, 100);
        assert_eq!(result.pairs_compared, 4950);
        assert_eq!(result.candidates_found.len(), 2);
        assert_eq!(result.count_above_threshold(0.8), 1);
    }
    
    #[test]
    fn test_similarity_config_default() {
        let config = SimilarityConfig::default();
        assert!(config.use_embeddings);
        assert!(config.use_relationships);
        assert!(config.skip_verified);
        assert_eq!(config.min_confidence, 0.5);
    }
}
