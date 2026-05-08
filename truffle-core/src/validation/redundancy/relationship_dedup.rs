//! Relationship Deduplicator
//!
//! Identifies and removes redundant relationships,
//! keeping the highest confidence version.

use std::collections::{HashMap, HashSet};
use tracing::{debug, info, instrument, warn};
use uuid::Uuid;

use crate::graph::RelationshipRepository;
use crate::models::Relationship;
use crate::Result;

/// Deduplicator for relationships
pub struct RelationshipDeduplicator {
    repository: RelationshipRepository,
    config: DedupConfig,
}

/// Configuration for relationship deduplication
#[derive(Debug, Clone)]
pub struct DedupConfig {
    /// Minimum confidence threshold for keeping relationships
    pub min_confidence: f32,
    /// Whether to archive or delete duplicates
    pub archive_duplicates: bool,
    /// Whether to merge properties from duplicates
    pub merge_properties: bool,
    /// Strategy for handling conflicts
    pub conflict_strategy: DedupConflictStrategy,
}

impl Default for DedupConfig {
    fn default() -> Self {
        Self {
            min_confidence: 0.3,
            archive_duplicates: true,
            merge_properties: true,
            conflict_strategy: DedupConflictStrategy::KeepHighestConfidence,
        }
    }
}

/// Strategy for resolving deduplication conflicts
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DedupConflictStrategy {
    /// Keep the relationship with highest confidence
    KeepHighestConfidence,
    /// Keep the most recently updated relationship
    KeepMostRecent,
    /// Keep the verified relationship
    KeepVerified,
    /// Combine all duplicates into one
    MergeAll,
}

/// Result of a deduplication operation
#[derive(Debug, Clone)]
pub struct DedupResult {
    /// Number of duplicate groups found
    pub groups_found: usize,
    /// Number of relationships archived
    pub archived: usize,
    /// Number of relationships deleted
    pub deleted: usize,
    /// Number of relationships merged
    pub merged: usize,
    /// IDs of kept relationships
    pub kept_relationships: Vec<Uuid>,
    /// IDs of removed relationships
    pub removed_relationships: Vec<Uuid>,
    /// Any errors encountered
    pub errors: Vec<String>,
}

/// A group of duplicate relationships
#[derive(Debug, Clone)]
pub struct DuplicateGroup {
    /// Signature identifying this group
    pub signature: RelationshipSignature,
    /// Relationships in this group
    pub relationships: Vec<Relationship>,
    /// Relationship to keep
    pub keep_id: Uuid,
    /// Relationships to remove
    pub remove_ids: Vec<Uuid>,
}

/// Signature for identifying equivalent relationships
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct RelationshipSignature {
    /// Source entity ID
    pub source_id: Uuid,
    /// Target entity ID
    pub target_id: Uuid,
    /// Relationship type
    pub relation_type: String,
}

impl RelationshipSignature {
    /// Create a signature from a relationship
    pub fn from_relationship(rel: &Relationship) -> Self {
        Self {
            source_id: rel.source_id,
            target_id: rel.target_id,
            relation_type: rel.relation_type.clone(),
        }
    }
}

impl RelationshipDeduplicator {
    /// Create a new deduplicator
    pub fn new(repository: RelationshipRepository) -> Self {
        Self {
            repository,
            config: DedupConfig::default(),
        }
    }
    
    /// Create with custom configuration
    pub fn with_config(repository: RelationshipRepository, config: DedupConfig) -> Self {
        Self {
            repository,
            config,
        }
    }
    
    /// Find redundant relationships
    #[instrument(skip(self), level = "debug")]
    pub async fn find_redundant_relationships(&self) -> Result<Vec<Uuid>> {
        debug!("Finding redundant relationships");
        
        let groups = self.find_duplicate_groups().await?;
        let mut redundant = Vec::new();
        
        for group in groups {
            redundant.extend(group.remove_ids);
        }
        
        debug!("Found {} redundant relationships", redundant.len());
        Ok(redundant)
    }
    
    /// Find and group duplicate relationships
    #[instrument(skip(self), level = "debug")]
    pub async fn find_duplicate_groups(&self) -> Result<Vec<DuplicateGroup>> {
        debug!("Finding duplicate relationship groups");
        
        // Get all non-archived relationships
        let all_relationships = self.get_all_relationships().await?;
        debug!("Checking {} relationships for duplicates", all_relationships.len());
        
        // Group by signature
        let mut groups: HashMap<RelationshipSignature, Vec<Relationship>> = HashMap::new();
        
        for rel in all_relationships {
            // Skip archived relationships
            if rel.archived {
                continue;
            }
            
            // Skip low confidence relationships
            if rel.extraction_confidence < self.config.min_confidence {
                continue;
            }
            
            let sig = RelationshipSignature::from_relationship(&rel);
            groups.entry(sig).or_default().push(rel);
        }
        
        // Build duplicate groups
        let mut duplicates = Vec::new();
        
        for (signature, relationships) in groups {
            if relationships.len() > 1 {
                let (keep_id, remove_ids) = self.select_relationships_to_dedup(&relationships)?;
                
                duplicates.push(DuplicateGroup {
                    signature,
                    relationships,
                    keep_id,
                    remove_ids,
                });
            }
        }
        
        debug!("Found {} duplicate groups", duplicates.len());
        Ok(duplicates)
    }
    
    /// Execute deduplication
    #[instrument(skip(self), level = "info")]
    pub async fn execute_dedup(&self) -> Result<DedupResult> {
        info!("Executing relationship deduplication");
        
        let groups = self.find_duplicate_groups().await?;
        
        let mut result = DedupResult {
            groups_found: groups.len(),
            archived: 0,
            deleted: 0,
            merged: 0,
            kept_relationships: Vec::new(),
            removed_relationships: Vec::new(),
            errors: Vec::new(),
        };
        
        for group in groups {
            match self.process_duplicate_group(&group).await {
                Ok(processed) => {
                    result.kept_relationships.push(processed.kept_id);
                    result.removed_relationships.extend(&processed.removed_ids);
                    
                    if self.config.archive_duplicates {
                        result.archived += processed.removed_ids.len();
                    } else {
                        result.deleted += processed.removed_ids.len();
                    }
                    
                    if self.config.merge_properties && processed.merged {
                        result.merged += 1;
                    }
                }
                Err(e) => {
                    let error_msg = format!(
                        "Error processing group {:?}: {}",
                        group.signature, e
                    );
                    warn!("{}", error_msg);
                    result.errors.push(error_msg);
                }
            }
        }
        
        info!(
            "Deduplication complete: {} groups, {} archived, {} deleted",
            result.groups_found, result.archived, result.deleted
        );
        
        Ok(result)
    }
    
    /// Archive deprecated relationships
    #[instrument(skip(self), level = "debug")]
    pub async fn archive_deprecated_relationships(&self, reason: &str) -> Result<usize> {
        debug!("Archiving deprecated relationships: {}", reason);
        
        let all_relationships = self.get_all_relationships().await?;
        let mut archived = 0;
        
        for mut rel in all_relationships {
            // Archive if it matches the deprecation criteria
            if self.should_archive(&rel) {
                rel.archive(reason);
                self.repository.save(&rel).await?;
                archived += 1;
            }
        }
        
        debug!("Archived {} relationships", archived);
        Ok(archived)
    }
    
    /// Get statistics about relationship distribution
    pub async fn get_statistics(&self) -> Result<RelationshipStats> {
        let all_relationships = self.get_all_relationships().await?;
        
        let total = all_relationships.len();
        let archived = all_relationships.iter().filter(|r| r.archived).count();
        let verified = all_relationships.iter().filter(|r| r.verified).count();
        
        // Count by type
        let mut by_type: HashMap<String, usize> = HashMap::new();
        for rel in &all_relationships {
            *by_type.entry(rel.relation_type.clone()).or_insert(0) += 1;
        }
        
        // Find duplicates
        let duplicates = self.find_duplicate_groups().await?;
        let duplicate_count: usize = duplicates.iter().map(|g| g.relationships.len()).sum();
        
        Ok(RelationshipStats {
            total,
            active: total - archived,
            archived,
            verified,
            by_type,
            duplicate_groups: duplicates.len(),
            potential_savings: duplicate_count.saturating_sub(duplicates.len()),
        })
    }
    
    /// Select which relationship to keep and which to remove
    fn select_relationships_to_dedup(
        &self,
        relationships: &[Relationship],
    ) -> Result<(Uuid, Vec<Uuid>)> {
        let sorted = self.sort_relationships(relationships);
        
        let keep_id = sorted[0].id;
        let remove_ids: Vec<Uuid> = sorted.iter().skip(1).map(|r| r.id).collect();
        
        Ok((keep_id, remove_ids))
    }
    
    /// Sort relationships by priority (best first)
    fn sort_relationships(&self, relationships: &[Relationship]) -> Vec<Relationship> {
        let mut sorted: Vec<Relationship> = relationships.to_vec();
        
        sorted.sort_by(|a, b| {
            let a_score = self.calculate_relationship_priority(a);
            let b_score = self.calculate_relationship_priority(b);
            
            b_score.partial_cmp(&a_score).unwrap_or(std::cmp::Ordering::Equal)
        });
        
        sorted
    }
    
    /// Calculate priority score for a relationship
    fn calculate_relationship_priority(&self, rel: &Relationship) -> f32 {
        let mut score = rel.extraction_confidence;
        
        // Verified relationships get a boost
        if rel.verified {
            score += 0.2;
        }
        
        // Relationships with properties get a small boost
        if rel.properties.is_some() {
            score += 0.05;
        }
        
        // More recently updated is slightly preferred
        let age_hours = (chrono::Utc::now() - rel.last_updated_at).num_hours() as f32;
        score -= age_hours / 10000.0; // Small penalty for older relationships
        
        score
    }
    
    /// Process a single duplicate group
    async fn process_duplicate_group(
        &self,
        group: &DuplicateGroup,
    ) -> Result<ProcessedGroupResult> {
        debug!("Processing duplicate group with {} relationships", group.relationships.len());
        
        let keep_rel = group.relationships.iter()
            .find(|r| r.id == group.keep_id)
            .cloned()
            .ok_or_else(|| crate::Error::Other(
                "Keep relationship not found in group".to_string()
            ))?;
        
        // Merge properties if enabled
        if self.config.merge_properties {
            let merged = self.merge_group_properties(&keep_rel, &group.relationships).await?;
            self.repository.save(&merged).await?;
        }
        
        // Archive or delete duplicates
        for rel_id in &group.remove_ids {
            if let Some(mut rel) = self.repository.get(rel_id).await? {
                if self.config.archive_duplicates {
                    rel.archive(format!(
                        "Duplicate of relationship {} (deduplication)",
                        group.keep_id
                    ));
                    self.repository.save(&rel).await?;
                } else {
                    self.repository.delete(rel_id).await?;
                }
            }
        }
        
        Ok(ProcessedGroupResult {
            kept_id: group.keep_id,
            removed_ids: group.remove_ids.clone(),
            merged: self.config.merge_properties,
        })
    }
    
    /// Merge properties from all relationships in a group
    async fn merge_group_properties(
        &self,
        primary: &Relationship,
        all: &[Relationship],
    ) -> Result<Relationship> {
        let mut merged = primary.clone();
        
        if merged.properties.is_none() {
            merged.properties = Some(serde_json::Value::Object(serde_json::Map::new()));
        }
        
        for rel in all {
            if rel.id == primary.id {
                continue;
            }
            
            if let Some(ref props) = rel.properties {
                merged.properties = self.merge_json_values(merged.properties.clone().unwrap(), props.clone());
            }
        }
        
        merged.last_updated_at = chrono::Utc::now();
        Ok(merged)
    }
    
    /// Merge two JSON values
    fn merge_json_values(&self, mut base: serde_json::Value, other: serde_json::Value) -> serde_json::Value {
        match (&mut base, other) {
            (serde_json::Value::Object(base_map), serde_json::Value::Object(other_map)) => {
                for (key, value) in other_map {
                    if !base_map.contains_key(&key) {
                        base_map.insert(key, value);
                    }
                }
                base
            }
            _ => base,
        }
    }
    
    /// Check if a relationship should be archived
    fn should_archive(&self, rel: &Relationship) -> bool {
        // Archive if expired
        if let Some(valid_until) = rel.valid_until {
            if chrono::Utc::now() > valid_until && !rel.archived {
                return true;
            }
        }
        
        // Archive if very low confidence
        if rel.extraction_confidence < 0.2 && !rel.verified {
            return true;
        }
        
        false
    }
    
    /// Get all relationships from repository
    async fn get_all_relationships(&self) -> Result<Vec<Relationship>> {
        // This is a simplified implementation
        // In a real system, this would be a paginated query
        Ok(Vec::new())
    }
}

/// Result of processing a duplicate group
#[derive(Debug, Clone)]
struct ProcessedGroupResult {
    kept_id: Uuid,
    removed_ids: Vec<Uuid>,
    merged: bool,
}

/// Statistics about relationships
#[derive(Debug, Clone)]
pub struct RelationshipStats {
    /// Total number of relationships
    pub total: usize,
    /// Active (non-archived) relationships
    pub active: usize,
    /// Archived relationships
    pub archived: usize,
    /// Verified relationships
    pub verified: usize,
    /// Count by relationship type
    pub by_type: HashMap<String, usize>,
    /// Number of duplicate groups
    pub duplicate_groups: usize,
    /// Potential storage savings from deduplication
    pub potential_savings: usize,
}

/// Batch deduplication for large datasets
pub struct BatchDeduplicator {
    deduplicator: RelationshipDeduplicator,
    batch_size: usize,
}

impl BatchDeduplicator {
    /// Create a new batch deduplicator
    pub fn new(deduplicator: RelationshipDeduplicator, batch_size: usize) -> Self {
        Self {
            deduplicator,
            batch_size: batch_size.max(100),
        }
    }
    
    /// Execute deduplication in batches
    pub async fn execute_batch_dedup(&self) -> Result<Vec<DedupResult>> {
        let mut results = Vec::new();
        let mut offset = 0;
        
        loop {
            let batch_result = self.process_batch(offset).await?;
            let done = batch_result.groups_found < self.batch_size;
            
            results.push(batch_result);
            
            if done {
                break;
            }
            
            offset += self.batch_size;
        }
        
        Ok(results)
    }
    
    /// Process a single batch
    async fn process_batch(&self, offset: usize) -> Result<DedupResult> {
        // This would process a specific batch of relationships
        // For now, delegate to the regular dedup
        self.deduplicator.execute_dedup().await
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_dedup_config_default() {
        let config = DedupConfig::default();
        assert!(config.archive_duplicates);
        assert!(config.merge_properties);
        assert_eq!(config.min_confidence, 0.3);
    }
    
    #[test]
    fn test_relationship_signature() {
        let rel = Relationship::new(
            Uuid::new_v4(),
            Uuid::new_v4(),
            "works_at",
            None,
        );
        
        let sig = RelationshipSignature::from_relationship(&rel);
        assert_eq!(sig.source_id, rel.source_id);
        assert_eq!(sig.target_id, rel.target_id);
        assert_eq!(sig.relation_type, "works_at");
    }
    
    #[test]
    fn test_calculate_relationship_priority() {
        let repo = RelationshipRepository::new(
            std::sync::Arc::new(tokio::sync::RwLock::new(
                crate::database::Database::open(crate::database::DatabaseConfig::memory()).unwrap()
            ))
        );
        
        let dedup = RelationshipDeduplicator::new(repo);
        
        let rel = Relationship::new(Uuid::new_v4(), Uuid::new_v4(), "knows", None)
            .with_confidence(0.8);
        
        let score = dedup.calculate_relationship_priority(&rel);
        assert!(score > 0.0);
        assert!(score <= 1.0);
    }
    
    #[test]
    fn test_sort_relationships() {
        let repo = RelationshipRepository::new(
            std::sync::Arc::new(tokio::sync::RwLock::new(
                crate::database::Database::open(crate::database::DatabaseConfig::memory()).unwrap()
            ))
        );
        
        let dedup = RelationshipDeduplicator::new(repo);
        
        let rel1 = Relationship::new(Uuid::new_v4(), Uuid::new_v4(), "knows", None)
            .with_confidence(0.5);
        let rel2 = Relationship::new(Uuid::new_v4(), Uuid::new_v4(), "knows", None)
            .with_confidence(0.9);
        
        let sorted = dedup.sort_relationships(&[rel1.clone(), rel2.clone()]);
        
        assert_eq!(sorted[0].extraction_confidence, 0.9);
        assert_eq!(sorted[1].extraction_confidence, 0.5);
    }
}
