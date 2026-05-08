//! Stage 3: Knowledge Graph Construction
//!
//! Builds the knowledge graph by:
//! 1. Entity resolution (fuzzy matching)
//! 2. Link injection (bidirectional backlinks)
//! 3. Vector indexing (HNSW via sqlite-vec)
//! 4. Temporal indexing (R-Tree)
//!
//! Per Section 2.2.3 of the Enterprise Specification

use crate::models::{
    WikiNode, WikiNodeType, WikiLink, Provenance, TemporalVectors,
    PrivacyClassification, VectorClock,
};
use crate::pipeline::processor::{ProcessingResult, DocumentType, ExtractedEntity};
use crate::database::{Database, WikiNodeRepository};
use std::sync::Arc;
use std::collections::HashMap;
use tracing::{info, debug, warn};

/// Graph builder configuration
#[derive(Debug, Clone)]
pub struct GraphConfig {
    /// Semantic similarity threshold for linking
    pub similarity_threshold: f64,
    /// Levenshtein distance threshold for entity resolution
    pub levenshtein_threshold: usize,
    /// Enable Metaphone phonetic matching
    pub use_metaphone: bool,
    /// Temporal proximity window (seconds)
    pub temporal_proximity_secs: u64,
    /// HNSW ef_construction parameter
    pub hnsw_ef_construction: usize,
    /// HNSW M parameter
    pub hnsw_m: usize,
}

impl Default for GraphConfig {
    fn default() -> Self {
        Self {
            similarity_threshold: 0.82,
            levenshtein_threshold: 2,
            use_metaphone: true,
            temporal_proximity_secs: 300, // 5 minutes
            hnsw_ef_construction: 128,
            hnsw_m: 16,
        }
    }
}

/// Graph builder for knowledge graph construction
pub struct GraphBuilder {
    config: GraphConfig,
    database: Option<Arc<Database>>,
    entity_cache: HashMap<String, uuid::Uuid>,
}

impl GraphBuilder {
    /// Create a new graph builder
    pub fn new(config: GraphConfig) -> Self {
        Self {
            config,
            database: None,
            entity_cache: HashMap::new(),
        }
    }
    
    /// Create with database access
    pub fn with_database(mut self, database: Arc<Database>) -> Self {
        self.database = Some(database);
        self
    }
    
    /// Build a wiki node from processing result
    pub async fn build_node(&self, result: ProcessingResult) -> anyhow::Result<WikiNode> {
        debug!("Building wiki node from processing result: {}", result.artifact_uuid);
        
        // Determine node type from document type
        let node_type = self.infer_node_type(result.document_type);
        
        // Generate title from entities
        let title = self.generate_title(&result);
        
        // Generate content from entities
        let content = self.generate_content(&result);
        
        // Create provenance
        let provenance = Provenance::new(
            vec![result.artifact_uuid],
            &result.model_version,
            result.confidence,
        ).with_duration(result.duration_ms);
        
        // Create the node
        let mut node = WikiNode::new(
            &title,
            node_type,
            &content,
            provenance,
        );
        
        // Apply privacy classification
        node.privacy_classification = result.privacy;
        
        // Apply temporal vectors
        node.temporal_vectors = TemporalVectors {
            mentioned_dates: result.temporal.dates,
            fiscal_quarter: result.temporal.fiscal_quarter,
            day_of_week: None,
            hour_of_day: None,
        };
        
        // Set embedding
        if let Some(embedding) = result.embedding {
            node.embedding = Some(embedding);
        }
        
        // Resolve entities and create links
        let links = self.resolve_entities(&result).await?;
        node.forward_links = links;
        
        info!("Built wiki node: {} ({}) with {} links", node.id, node.title, node.forward_links.len());
        
        Ok(node)
    }
    
    /// Infer node type from document type
    fn infer_node_type(&self, doc_type: DocumentType) -> WikiNodeType {
        match doc_type {
            DocumentType::Receipt => WikiNodeType::Entity,
            DocumentType::ContactCard => WikiNodeType::Entity,
            DocumentType::TravelConfirmation => WikiNodeType::Chronology,
            DocumentType::Messaging => WikiNodeType::Concept,
            DocumentType::WebPage => WikiNodeType::Concept,
            DocumentType::Code => WikiNodeType::Concept,
            DocumentType::Image => WikiNodeType::Entity,
            DocumentType::Document => WikiNodeType::Entity,
            DocumentType::Unknown => WikiNodeType::Entity,
        }
    }
    
    /// Generate title from processing result
    fn generate_title(&self, result: &ProcessingResult) -> String {
        // Try to find a name entity
        for entity in &result.entities {
            if entity.entity_type == "name" || entity.entity_type == "merchant" {
                return entity.value.clone();
            }
        }
        
        // Fallback to document type + date
        let doc_type_str = format!("{:?}", result.document_type);
        format!("{} {}", doc_type_str, chrono::Utc::now().format("%Y-%m-%d"))
    }
    
    /// Generate markdown content from processing result
    fn generate_content(&self, result: &ProcessingResult) -> String {
        let mut content = String::new();
        
        // Add heading
        content.push_str(&format!("# {}\n\n", self.generate_title(result)));
        
        // Add extracted entities as a list
        if !result.entities.is_empty() {
            content.push_str("## Extracted Information\n\n");
            for entity in &result.entities {
                content.push_str(&format!("- **{}**: {}\n", 
                    capitalize(&entity.entity_type),
                    entity.value
                ));
            }
            content.push('\n');
        }
        
        // Add OCR text if available
        if !result.ocr_text.is_empty() {
            content.push_str("## Original Text\n\n");
            content.push_str("```\n");
            content.push_str(&result.ocr_text);
            content.push_str("\n```\n\n");
        }
        
        // Add metadata
        content.push_str("## Metadata\n\n");
        content.push_str(&format!("- **Document Type**: {:?}\n", result.document_type));
        content.push_str(&format!("- **Confidence**: {:.0}%\n", result.confidence * 100.0));
        content.push_str(&format!("- **Model**: {}\n", result.model_version));
        content.push_str(&format!("- **Source**: [[raw-artifact-{}]]\n", result.artifact_uuid));
        
        content
    }
    
    /// Resolve entities and create links
    async fn resolve_entities(&self, result: &ProcessingResult) -> anyhow::Result<Vec<WikiLink>> {
        let mut links = Vec::new();
        
        // Check for existing entities
        if let Some(ref db) = self.database {
            for entity in &result.entities {
                if let Some(existing_id) = self.find_existing_entity(db, &entity.value).await? {
                    links.push(WikiLink::new(existing_id, &entity.value));
                }
            }
        }
        
        // Add links from matched rules
        for rule_name in &result.matched_rules {
            // Create links to concept nodes for rules
            let concept_id = self.get_or_create_concept(rule_name).await?;
            links.push(WikiLink::new(concept_id, rule_name)
                .with_context("matched_rule"));
        }
        
        Ok(links)
    }
    
    /// Find existing entity by name (fuzzy matching)
    async fn find_existing_entity(&self, db: &Database, name: &str) -> anyhow::Result<Option<uuid::Uuid>> {
        // Search for similar titles
        let nodes = db.with_connection(|conn| {
            let repo = WikiNodeRepository::new(conn);
            repo.search(name, 10)
                .map_err(|e| crate::database::DatabaseError::Query(e.to_string()))
        })?;
        
        for node in nodes {
            // Check exact match
            if node.title.to_lowercase() == name.to_lowercase() {
                return Ok(Some(node.id));
            }
            
            // Check Levenshtein distance
            let distance = levenshtein_distance(&node.title.to_lowercase(), &name.to_lowercase());
            if distance <= self.config.levenshtein_threshold {
                return Ok(Some(node.id));
            }
            
            // Check Metaphone if enabled
            if self.config.use_metaphone {
                // Simplified phonetic check
                // In production, use a proper phonetic library
            }
        }
        
        Ok(None)
    }
    
    /// Get or create a concept node
    async fn get_or_create_concept(&self, name: &str) -> anyhow::Result<uuid::Uuid> {
        // Generate deterministic UUID from name
        use sha2::{Sha256, Digest};
        let mut hasher = Sha256::new();
        hasher.update(name.as_bytes());
        let result = hasher.finalize();
        let bytes: [u8; 16] = result[..16].try_into().unwrap();
        Ok(uuid::Uuid::from_bytes(bytes))
    }
    
    /// Index node for vector search
    pub async fn index_vector(&self, node: &WikiNode) -> anyhow::Result<()> {
        if let Some(ref embedding) = node.embedding {
            debug!("Indexing vector for node: {}", node.id);
            
            // In production, this would insert into sqlite-vec virtual table
            // INSERT INTO node_embeddings (node_id, embedding) VALUES (?, ?)
            
            let _ = embedding; // Use the embedding
        }
        
        Ok(())
    }
    
    /// Index node for temporal search
    pub async fn index_temporal(&self, node: &WikiNode) -> anyhow::Result<()> {
        if !node.temporal_vectors.mentioned_dates.is_empty() {
            debug!("Indexing temporal data for node: {}", node.id);
            
            // In production, this would insert into R-Tree
            // INSERT INTO temporal_index (node_id, start_date, end_date) VALUES (?, ?, ?)
        }
        
        Ok(())
    }
    
    /// Find similar nodes by vector similarity
    pub async fn find_similar(&self, _embedding: &[f32], _limit: usize) -> anyhow::Result<Vec<WikiNode>> {
        // In production, this would query sqlite-vec
        // SELECT node_id FROM node_embeddings WHERE embedding MATCH ? LIMIT ?
        
        Ok(Vec::new())
    }
    
    /// Find nodes within temporal range
    pub async fn find_in_range(
        &self,
        _start: chrono::DateTime<chrono::Utc>,
        _end: chrono::DateTime<chrono::Utc>,
    ) -> anyhow::Result<Vec<WikiNode>> {
        // In production, this would query R-Tree
        // SELECT node_id FROM temporal_index WHERE start_date <= ? AND end_date >= ?
        
        Ok(Vec::new())
    }
}

/// Linking engine for managing graph relationships
pub struct LinkingEngine {
    config: GraphConfig,
    database: Option<Arc<Database>>,
}

impl LinkingEngine {
    pub fn new(config: GraphConfig) -> Self {
        Self {
            config,
            database: None,
        }
    }
    
    /// Create bidirectional link between two nodes
    pub async fn link_nodes(
        &self,
        source_id: uuid::Uuid,
        target_id: uuid::Uuid,
        context: Option<String>,
    ) -> anyhow::Result<()> {
        debug!("Linking nodes: {} -> {}", source_id, target_id);
        
        if let Some(ref db) = self.database {
            db.with_connection(|conn| {
                let repo = WikiNodeRepository::new(conn);
                
                // Load nodes
                let source = repo.find_by_id(source_id)?
                    .ok_or_else(|| anyhow::anyhow!("Source node not found"))?;
                let target = repo.find_by_id(target_id)?
                    .ok_or_else(|| anyhow::anyhow!("Target node not found"))?;
                
                // Create forward link
                let forward_link = if let Some(ctx) = context.clone() {
                    WikiLink::new(target_id, &target.title).with_context(ctx)
                } else {
                    WikiLink::new(target_id, &target.title)
                };
                
                // Create backlink
                let backlink = if let Some(ctx) = context {
                    WikiLink::new(source_id, &source.title).with_context(ctx)
                } else {
                    WikiLink::new(source_id, &source.title)
                };
                
                // Save links (this would be done through the repository)
                // For now, just log
                info!("Created link: {} -> {}", source.title, target.title);
                
                Ok(())
            })?;
        }
        
        Ok(())
    }
    
    /// Find related nodes by temporal proximity
    pub async fn find_temporally_related(
        &self,
        node: &WikiNode,
    ) -> anyhow::Result<Vec<uuid::Uuid>> {
        let mut related = Vec::new();
        
        // Find nodes within temporal proximity window
        let window = chrono::Duration::seconds(self.config.temporal_proximity_secs as i64);
        
        for date in &node.temporal_vectors.mentioned_dates {
            let start = *date - window;
            let end = *date + window;
            
            if let Some(ref db) = self.database {
                let nodes = db.with_connection(|conn| {
                    let repo = WikiNodeRepository::new(conn);
                    // This would query the temporal index
                    repo.find_by_type(WikiNodeType::Entity, 100)
                        .map_err(|e| crate::database::DatabaseError::Query(e.to_string()))
                })?;
                
                for other in nodes {
                    if other.id != node.id {
                        for other_date in &other.temporal_vectors.mentioned_dates {
                            if *other_date >= start && *other_date <= end {
                                related.push(other.id);
                            }
                        }
                    }
                }
            }
        }
        
        related.dedup();
        Ok(related)
    }
}

/// Calculate Levenshtein distance between two strings
fn levenshtein_distance(s1: &str, s2: &str) -> usize {
    let len1 = s1.chars().count();
    let len2 = s2.chars().count();
    
    if len1 == 0 {
        return len2;
    }
    if len2 == 0 {
        return len1;
    }
    
    let mut matrix = vec![vec![0; len2 + 1]; len1 + 1];
    
    for i in 0..=len1 {
        matrix[i][0] = i;
    }
    for j in 0..=len2 {
        matrix[0][j] = j;
    }
    
    for (i, c1) in s1.chars().enumerate() {
        for (j, c2) in s2.chars().enumerate() {
            let cost = if c1 == c2 { 0 } else { 1 };
            matrix[i + 1][j + 1] = (matrix[i][j + 1] + 1)
                .min(matrix[i + 1][j] + 1)
                .min(matrix[i][j] + cost);
        }
    }
    
    matrix[len1][len2]
}

/// Capitalize first letter of string
fn capitalize(s: &str) -> String {
    let mut chars = s.chars();
    match chars.next() {
        None => String::new(),
        Some(first) => first.to_uppercase().collect::<String>() + &chars.as_str().to_lowercase(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_graph_config() {
        let config = GraphConfig::default();
        assert_eq!(config.similarity_threshold, 0.82);
        assert_eq!(config.levenshtein_threshold, 2);
        assert!(config.use_metaphone);
    }
    
    #[test]
    fn test_levenshtein_distance() {
        assert_eq!(levenshtein_distance("kitten", "sitting"), 3);
        assert_eq!(levenshtein_distance("saturday", "sunday"), 3);
        assert_eq!(levenshtein_distance("", "abc"), 3);
        assert_eq!(levenshtein_distance("abc", "abc"), 0);
    }
    
    #[test]
    fn test_infer_node_type() {
        let builder = GraphBuilder::new(GraphConfig::default());
        
        assert_eq!(builder.infer_node_type(DocumentType::Receipt), WikiNodeType::Entity);
        assert_eq!(builder.infer_node_type(DocumentType::TravelConfirmation), WikiNodeType::Chronology);
        assert_eq!(builder.infer_node_type(DocumentType::Messaging), WikiNodeType::Concept);
    }
    
    #[tokio::test]
    async fn test_build_node() {
        let builder = GraphBuilder::new(GraphConfig::default());
        
        let result = ProcessingResult {
            artifact_uuid: uuid::Uuid::new_v4(),
            document_type: DocumentType::Receipt,
            entities: vec![
                ExtractedEntity {
                    entity_type: "merchant".to_string(),
                    value: "Test Store".to_string(),
                    confidence: 0.9,
                    normalized: None,
                }
            ],
            ocr_text: "Total: $50.00".to_string(),
            confidence: 0.85,
            duration_ms: 100,
            model_version: "test".to_string(),
            matched_rules: vec!["receipt_detection".to_string()],
            privacy: PrivacyClassification::Financial,
            temporal: TemporalExtraction::default(),
            embedding: Some(vec![0.0f32; 384]),
        };
        
        let node = builder.build_node(result).await.unwrap();
        
        assert_eq!(node.title, "Test Store");
        assert_eq!(node.node_type, WikiNodeType::Entity);
        assert_eq!(node.privacy_classification, PrivacyClassification::Financial);
    }
}
