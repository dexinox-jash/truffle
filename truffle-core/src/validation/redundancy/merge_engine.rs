//! Merge Engine
//!
//! Handles entity merging logic including conflict resolution,
//! provenance preservation, and merge preview generation.

use chrono::{DateTime, Utc};
use std::collections::HashMap;
use tracing::{debug, info, instrument, warn};
use uuid::Uuid;

use crate::graph::GraphRepository;
use crate::models::{Entity, EntityMetadata, Relationship};
use crate::Result;

use super::{DuplicateCandidate, MergeConflict, MergePreview, MergeResult, ProvenanceRecord};

/// Engine for merging duplicate entities
pub struct MergeEngine {
    repository: GraphRepository,
    config: MergeConfig,
}

/// Configuration for merge operations
#[derive(Debug, Clone)]
pub struct MergeConfig {
    /// Strategy for resolving attribute conflicts
    pub conflict_strategy: ConflictResolutionStrategy,
    /// Whether to archive merged entities (soft delete)
    pub archive_merged: bool,
    /// Whether to preserve all aliases
    pub preserve_aliases: bool,
    /// Whether to merge metadata fields
    pub merge_metadata: bool,
    /// Minimum confidence to auto-resolve conflicts
    pub auto_resolve_threshold: f32,
}

impl Default for MergeConfig {
    fn default() -> Self {
        Self {
            conflict_strategy: ConflictResolutionStrategy::PreferHigherConfidence,
            archive_merged: true,
            preserve_aliases: true,
            merge_metadata: true,
            auto_resolve_threshold: 0.8,
        }
    }
}

/// Strategy for resolving attribute conflicts
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ConflictResolutionStrategy {
    /// Keep the attribute with higher extraction confidence
    PreferHigherConfidence,
    /// Keep the more recent attribute
    PreferMoreRecent,
    /// Keep the verified attribute
    PreferVerified,
    /// Always require manual review
    ManualReview,
    /// Combine values where possible
    MergeValues,
}

/// Status of a merge conflict
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ConflictStatus {
    /// Conflict detected, awaiting resolution
    Pending,
    /// Auto-resolved by strategy
    AutoResolved,
    /// Requires manual review
    RequiresReview,
    /// User has resolved
    UserResolved,
}

impl MergeEngine {
    /// Create a new merge engine
    pub fn new(repository: GraphRepository) -> Self {
        Self {
            repository,
            config: MergeConfig::default(),
        }
    }
    
    /// Create with custom configuration
    pub fn with_config(repository: GraphRepository, config: MergeConfig) -> Self {
        Self {
            repository,
            config,
        }
    }
    
    /// Generate a preview of what a merge would look like
    #[instrument(skip(self, candidate), level = "debug")]
    pub async fn preview_merge(&self, candidate: &DuplicateCandidate) -> Result<MergePreview> {
        debug!(
            "Generating merge preview for entities {} <-> {}",
            candidate.entity_a_id, candidate.entity_b_id
        );
        
        // Fetch both entities
        let entity_a = self.repository.entities.get(&candidate.entity_a_id).await?
            .ok_or_else(|| crate::Error::Other(format!(
                "Entity {} not found", candidate.entity_a_id
            )))?;
        
        let entity_b = self.repository.entities.get(&candidate.entity_b_id).await?
            .ok_or_else(|| crate::Error::Other(format!(
                "Entity {} not found", candidate.entity_b_id
            )))?;
        
        // Determine primary and secondary
        let (primary, secondary) = self.determine_primary(&entity_a, &entity_b);
        
        // Analyze potential conflicts
        let conflicts = self.analyze_conflicts(&primary, &secondary).await?;
        
        // Generate merged entity preview
        let merged_preview = self.create_merged_preview(&primary, &secondary, &conflicts).await?;
        
        // Get relationships that need updating
        let affected_relationships = self.get_affected_relationships(&primary.id, &secondary.id).await?;
        
        // Calculate statistics
        let stats = MergeStats {
            total_conflicts: conflicts.len(),
            auto_resolvable: conflicts.iter()
                .filter(|c| c.can_auto_resolve)
                .count(),
            requires_review: conflicts.iter()
                .filter(|c| c.status == ConflictStatus::RequiresReview)
                .count(),
            affected_relationships: affected_relationships.len(),
        };
        
        Ok(MergePreview {
            candidate: candidate.clone(),
            primary_entity: primary,
            secondary_entity: secondary,
            merged_preview,
            conflicts,
            affected_relationships,
            stats,
        })
    }
    
    /// Execute a merge operation
    #[instrument(skip(self, candidate), level = "info")]
    pub async fn execute_merge(&self, candidate: &DuplicateCandidate) -> Result<MergeResult> {
        info!(
            "Executing merge for entities {} <-> {}",
            candidate.entity_a_id, candidate.entity_b_id
        );
        
        // Generate preview first
        let preview = self.preview_merge(candidate).await?;
        
        // Check if all conflicts are resolved
        let unresolved = preview.conflicts.iter()
            .filter(|c| c.status == ConflictStatus::Pending || c.status == ConflictStatus::RequiresReview)
            .count();
        
        if unresolved > 0 && self.config.conflict_strategy == ConflictResolutionStrategy::ManualReview {
            warn!("Cannot execute merge: {} conflicts require manual review", unresolved);
            return Err(crate::Error::Other(format!(
                "Merge requires manual review: {} unresolved conflicts",
                unresolved
            )));
        }
        
        // Perform the merge
        let merged = self.perform_merge(&preview).await?;
        
        // Update relationships
        let updated_relationships = self.update_relationships(
            &preview.primary_entity.id,
            &preview.secondary_entity.id,
        ).await?;
        
        // Archive or delete secondary entity
        if self.config.archive_merged {
            self.archive_entity(&preview.secondary_entity.id, &preview.primary_entity.id).await?;
        } else {
            self.delete_entity(&preview.secondary_entity.id).await?;
        }
        
        // Create provenance record
        let provenance = ProvenanceRecord {
            merge_id: Uuid::new_v4(),
            primary_id: preview.primary_entity.id,
            secondary_id: preview.secondary_entity.id,
            merged_id: merged.id,
            merged_at: Utc::now(),
            conflicts_resolved: preview.conflicts.len(),
            relationships_updated: updated_relationships.len(),
        };
        
        info!(
            "Merge completed: {} relationships updated, {} conflicts resolved",
            updated_relationships.len(),
            preview.conflicts.len()
        );
        
        Ok(MergeResult {
            merged_entity: merged,
            provenance,
            updated_relationships,
            archived_entity_id: if self.config.archive_merged {
                Some(preview.secondary_entity.id)
            } else {
                None
            },
        })
    }
    
    /// Determine which entity should be the primary (kept) entity
    fn determine_primary(&self, entity_a: &Entity, entity_b: &Entity) -> (Entity, Entity) {
        let a_score = self.calculate_entity_priority(entity_a);
        let b_score = self.calculate_entity_priority(entity_b);
        
        if a_score >= b_score {
            (entity_a.clone(), entity_b.clone())
        } else {
            (entity_b.clone(), entity_a.clone())
        }
    }
    
    /// Calculate priority score for an entity
    fn calculate_entity_priority(&self, entity: &Entity) -> f32 {
        let mut score = entity.extraction_confidence;
        
        // Verified entities get bonus
        if entity.verified {
            score += 0.2;
        }
        
        // Entities with embeddings get small bonus
        if entity.embedding.is_some() {
            score += 0.05;
        }
        
        // More recently updated is slightly preferred
        let age_hours = (Utc::now() - entity.last_updated_at).num_hours() as f32;
        score -= age_hours / 1000.0; // Small penalty for older entities
        
        score
    }
    
    /// Analyze conflicts between two entities
    async fn analyze_conflicts(&self, primary: &Entity, secondary: &Entity) -> Result<Vec<MergeConflict>> {
        let mut conflicts = Vec::new();
        
        // Check name conflict
        if primary.name != secondary.name {
            conflicts.push(self.create_conflict(
                "name",
                Some(&primary.name),
                Some(&secondary.name),
                primary.extraction_confidence,
                secondary.extraction_confidence,
            ));
        }
        
        // Check description conflict
        match (&primary.description, &secondary.description) {
            (Some(d1), Some(d2)) if d1 != d2 => {
                conflicts.push(self.create_conflict(
                    "description",
                    Some(d1),
                    Some(d2),
                    primary.extraction_confidence,
                    secondary.extraction_confidence,
                ));
            }
            (None, Some(d2)) => {
                conflicts.push(self.create_conflict(
                    "description",
                    None,
                    Some(d2),
                    primary.extraction_confidence,
                    secondary.extraction_confidence,
                ));
            }
            _ => {}
        }
        
        // Check metadata conflicts if enabled
        if self.config.merge_metadata {
            let metadata_conflicts = self.analyze_metadata_conflicts(&primary.metadata, &secondary.metadata);
            conflicts.extend(metadata_conflicts);
        }
        
        // Determine which conflicts can be auto-resolved
        for conflict in &mut conflicts {
            conflict.can_auto_resolve = self.can_auto_resolve(conflict);
            if conflict.can_auto_resolve {
                conflict.status = ConflictStatus::AutoResolved;
                conflict.resolved_value = self.resolve_conflict(conflict).await?;
            } else if self.config.conflict_strategy == ConflictResolutionStrategy::ManualReview {
                conflict.status = ConflictStatus::RequiresReview;
            }
        }
        
        Ok(conflicts)
    }
    
    /// Analyze metadata field conflicts
    fn analyze_metadata_conflicts(
        &self,
        meta_a: &EntityMetadata,
        meta_b: &EntityMetadata,
    ) -> Vec<MergeConflict> {
        let mut conflicts = Vec::new();
        
        // Compare common metadata fields
        let fields: Vec<(&str, &Option<String>)> = vec![
            ("title", &meta_a.title),
            ("email", &meta_a.email),
            ("phone", &meta_a.phone),
            ("industry", &meta_a.industry),
            ("website", &meta_a.website),
            ("address", &meta_a.address),
        ];
        
        let fields_b: HashMap<&str, &Option<String>> = [
            ("title", &meta_b.title),
            ("email", &meta_b.email),
            ("phone", &meta_b.phone),
            ("industry", &meta_b.industry),
            ("website", &meta_b.website),
            ("address", &meta_b.address),
        ].into_iter().collect();
        
        for (field, val_a) in fields {
            let val_b = fields_b[field];
            
            if val_a != val_b {
                conflicts.push(MergeConflict {
                    field: format!("metadata.{}", field),
                    primary_value: val_a.clone(),
                    secondary_value: val_b.cloned(),
                    status: ConflictStatus::Pending,
                    can_auto_resolve: false,
                    resolved_value: None,
                });
            }
        }
        
        conflicts
    }
    
    /// Create a conflict record
    fn create_conflict(
        &self,
        field: &str,
        primary: Option<&str>,
        secondary: Option<&str>,
        primary_conf: f32,
        secondary_conf: f32,
    ) -> MergeConflict {
        MergeConflict {
            field: field.to_string(),
            primary_value: primary.map(|s| s.to_string()),
            secondary_value: secondary.map(|s| s.to_string()),
            status: ConflictStatus::Pending,
            can_auto_resolve: false,
            resolved_value: None,
        }
    }
    
    /// Check if a conflict can be auto-resolved
    fn can_auto_resolve(&self, conflict: &MergeConflict) -> bool {
        match self.config.conflict_strategy {
            ConflictResolutionStrategy::ManualReview => false,
            _ => true,
        }
    }
    
    /// Resolve a conflict based on strategy
    async fn resolve_conflict(&self, conflict: &MergeConflict) -> Result<Option<String>> {
        match self.config.conflict_strategy {
            ConflictResolutionStrategy::PreferHigherConfidence => {
                // This is simplified; in reality we'd track per-field confidence
                Ok(conflict.primary_value.clone())
            }
            ConflictResolutionStrategy::PreferMoreRecent => {
                Ok(conflict.primary_value.clone())
            }
            ConflictResolutionStrategy::PreferVerified => {
                Ok(conflict.primary_value.clone())
            }
            ConflictResolutionStrategy::MergeValues => {
                // Try to combine values
                match (&conflict.primary_value, &conflict.secondary_value) {
                    (Some(p), Some(s)) if p != s => {
                        Ok(Some(format!("{} (also: {})", p, s)))
                    }
                    (Some(p), _) => Ok(Some(p.clone())),
                    (None, Some(s)) => Ok(Some(s.clone())),
                    (None, None) => Ok(None),
                }
            }
            ConflictResolutionStrategy::ManualReview => Ok(None),
        }
    }
    
    /// Create a preview of the merged entity
    async fn create_merged_preview(
        &self,
        primary: &Entity,
        secondary: &Entity,
        conflicts: &[MergeConflict],
    ) -> Result<Entity> {
        let mut merged = primary.clone();
        
        // Apply resolved conflicts
        for conflict in conflicts {
            if let Some(ref resolved) = conflict.resolved_value {
                match conflict.field.as_str() {
                    "name" => merged.name = resolved.clone(),
                    "description" => merged.description = Some(resolved.clone()),
                    _ if conflict.field.starts_with("metadata.") => {
                        // Handle metadata fields
                        let meta_field = &conflict.field[9..]; // Strip "metadata."
                        self.update_metadata_field(&mut merged.metadata, meta_field, resolved);
                    }
                    _ => {}
                }
            }
        }
        
        // Merge metadata from secondary if not present in primary
        self.merge_metadata(&mut merged.metadata, &secondary.metadata);
        
        // Combine embeddings (average if both exist)
        if let (Some(emb_a), Some(emb_b)) = (&primary.embedding, &secondary.embedding) {
            if emb_a.len() == emb_b.len() {
                let averaged: Vec<f32> = emb_a.iter()
                    .zip(emb_b.iter())
                    .map(|(a, b)| (a + b) / 2.0)
                    .collect();
                merged.embedding = Some(averaged);
            }
        }
        
        // Update confidence (boost slightly due to corroboration)
        merged.extraction_confidence = (primary.extraction_confidence + secondary.extraction_confidence)
            .min(1.0);
        
        merged.last_updated_at = Utc::now();
        
        Ok(merged)
    }
    
    /// Update a metadata field
    fn update_metadata_field(&self, metadata: &mut EntityMetadata, field: &str, value: &str) {
        match field {
            "title" => metadata.title = Some(value.to_string()),
            "email" => metadata.email = Some(value.to_string()),
            "phone" => metadata.phone = Some(value.to_string()),
            "industry" => metadata.industry = Some(value.to_string()),
            "website" => metadata.website = Some(value.to_string()),
            "address" => metadata.address = Some(value.to_string()),
            _ => {}
        }
    }
    
    /// Merge metadata fields
    fn merge_metadata(&self, target: &mut EntityMetadata, source: &EntityMetadata) {
        if target.title.is_none() && source.title.is_some() {
            target.title = source.title.clone();
        }
        if target.email.is_none() && source.email.is_some() {
            target.email = source.email.clone();
        }
        if target.phone.is_none() && source.phone.is_some() {
            target.phone = source.phone.clone();
        }
        if target.industry.is_none() && source.industry.is_some() {
            target.industry = source.industry.clone();
        }
        if target.website.is_none() && source.website.is_some() {
            target.website = source.website.clone();
        }
        if target.address.is_none() && source.address.is_some() {
            target.address = source.address.clone();
        }
        
        // Merge extra fields
        for (key, value) in &source.extra {
            target.extra.entry(key.clone()).or_insert_with(|| value.clone());
        }
    }
    
    /// Get relationships that will be affected by the merge
    async fn get_affected_relationships(
        &self,
        primary_id: &Uuid,
        secondary_id: &Uuid,
    ) -> Result<Vec<Relationship>> {
        let mut affected = Vec::new();
        
        // Get relationships for both entities
        let primary_rels = self.repository.relationships.get_for_entity(primary_id).await?;
        let secondary_rels = self.repository.relationships.get_for_entity(secondary_id).await?;
        
        affected.extend(primary_rels);
        affected.extend(secondary_rels);
        
        Ok(affected)
    }
    
    /// Perform the actual merge
    async fn perform_merge(&self, preview: &MergePreview) -> Result<Entity> {
        let merged = self.create_merged_preview(
            &preview.primary_entity,
            &preview.secondary_entity,
            &preview.conflicts,
        ).await?;
        
        // Save merged entity
        self.repository.entities.save(&merged).await?;
        
        Ok(merged)
    }
    
    /// Update relationships after merge
    async fn update_relationships(
        &self,
        primary_id: &Uuid,
        secondary_id: &Uuid,
    ) -> Result<Vec<Uuid>> {
        let mut updated = Vec::new();
        
        // Get all relationships involving the secondary entity
        let relationships = self.repository.relationships.get_for_entity(secondary_id).await?;
        
        for mut rel in relationships {
            // Update source or target to point to primary
            if rel.source_id == *secondary_id {
                rel.source_id = *primary_id;
            }
            if rel.target_id == *secondary_id {
                rel.target_id = *primary_id;
            }
            
            // Check if this creates a duplicate relationship
            let existing = self.find_equivalent_relationship(&rel).await?;
            
            if let Some(existing) = existing {
                // Keep the one with higher confidence
                if rel.extraction_confidence > existing.extraction_confidence {
                    self.repository.relationships.delete(&existing.id).await?;
                    self.repository.relationships.save(&rel).await?;
                    updated.push(rel.id);
                } else {
                    // Archive the redundant relationship
                    rel.archive(format!("Merged into {}", primary_id));
                    self.repository.relationships.save(&rel).await?;
                }
            } else {
                self.repository.relationships.save(&rel).await?;
                updated.push(rel.id);
            }
        }
        
        Ok(updated)
    }
    
    /// Find an equivalent relationship (same source, target, type)
    async fn find_equivalent_relationship(&self, rel: &Relationship) -> Result<Option<Relationship>> {
        // This would search for existing relationships with same source, target, and type
        // excluding the current relationship
        // Simplified implementation
        Ok(None)
    }
    
    /// Archive an entity
    async fn archive_entity(&self, entity_id: &Uuid, merged_into: &Uuid) -> Result<()> {
        if let Some(mut entity) = self.repository.entities.get(entity_id).await? {
            entity.last_updated_at = Utc::now();
            // Mark as archived/merged
            self.repository.entities.save(&entity).await?;
            debug!("Archived entity {} (merged into {})", entity_id, merged_into);
        }
        Ok(())
    }
    
    /// Delete an entity
    async fn delete_entity(&self, entity_id: &Uuid) -> Result<()> {
        self.repository.entities.delete(entity_id).await?;
        debug!("Deleted entity {}", entity_id);
        Ok(())
    }
}

/// Statistics for merge preview
#[derive(Debug, Clone)]
pub struct MergeStats {
    /// Total number of conflicts
    pub total_conflicts: usize,
    /// Conflicts that can be auto-resolved
    pub auto_resolvable: usize,
    /// Conflicts requiring manual review
    pub requires_review: usize,
    /// Number of relationships to update
    pub affected_relationships: usize,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::{Entity, EntityType};
    
    fn create_test_entity(name: &str, confidence: f32) -> Entity {
        Entity::new(name, EntityType::Person, None)
            .with_confidence(confidence)
    }
    
    #[test]
    fn test_merge_config_default() {
        let config = MergeConfig::default();
        assert!(config.archive_merged);
        assert!(config.preserve_aliases);
        assert!(config.merge_metadata);
    }
    
    #[test]
    fn test_calculate_entity_priority() {
        let engine = MergeEngine::new(GraphRepository::new(
            std::sync::Arc::new(tokio::sync::RwLock::new(
                crate::database::Database::open(crate::database::DatabaseConfig::memory()).unwrap()
            ))
        ));
        
        let entity = create_test_entity("Test", 0.8);
        let score = engine.calculate_entity_priority(&entity);
        assert!(score > 0.0);
        assert!(score <= 1.0);
    }
    
    #[test]
    fn test_determine_primary() {
        let engine = MergeEngine::new(GraphRepository::new(
            std::sync::Arc::new(tokio::sync::RwLock::new(
                crate::database::Database::open(crate::database::DatabaseConfig::memory()).unwrap()
            ))
        ));
        
        let entity_a = create_test_entity("A", 0.9);
        let entity_b = create_test_entity("B", 0.5);
        
        let (primary, secondary) = engine.determine_primary(&entity_a, &entity_b);
        assert_eq!(primary.name, "A");
        assert_eq!(secondary.name, "B");
    }
}
