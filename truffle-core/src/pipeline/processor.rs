//! Stage 2: Multimodal Processing
//!
//! AI-powered content analysis that transforms RawArtifacts into
//! structured data. This is an interface layer - actual AI processing
//! is handled by external systems (Gemma 4 via llama.cpp).
//!
//! Per Section 2.2.2 of the Enterprise Specification:
//! - Gemma 4 E2B integration
//! - 30 second timeout per image
//! - Content safety filtering
//! - Memory cap at 4GB

use crate::models::{
    RawArtifact, WikiNode, WikiNodeType, Provenance, TemporalVectors,
    PrivacyClassification, TriggerContext, SchemaRuleset, CompilationRule,
};
use crate::database::{Database, RawArtifactRepository};
use std::sync::Arc;
use std::time::Duration;
use tokio::time::timeout;
use serde::{Deserialize, Serialize};
use tracing::{info, debug, warn, error};

/// Processor configuration
#[derive(Debug, Clone)]
pub struct ProcessorConfig {
    /// Model version to use
    pub model_version: String,
    /// Maximum processing time per image (seconds)
    pub timeout_secs: u64,
    /// Maximum memory usage (MB)
    pub max_memory_mb: u64,
    /// Enable content safety filtering
    pub enable_safety_filter: bool,
    /// Confidence threshold for extraction
    pub confidence_threshold: f64,
    /// Schema ruleset path
    pub schema_path: Option<std::path::PathBuf>,
}

impl Default for ProcessorConfig {
    fn default() -> Self {
        Self {
            model_version: "gemma-4-2b-it-Q4_K_M".to_string(),
            timeout_secs: 30,
            max_memory_mb: 4096,
            enable_safety_filter: true,
            confidence_threshold: 0.7,
            schema_path: None,
        }
    }
}

/// Processing error types
#[derive(Debug, thiserror::Error)]
pub enum ProcessingError {
    #[error("Artifact not found: {0}")]
    ArtifactNotFound(uuid::Uuid),
    
    #[error("Processing timeout after {0}s")]
    Timeout(u64),
    
    #[error("Memory limit exceeded: {0}MB > {1}MB")]
    MemoryExceeded(u64, u64),
    
    #[error("Content safety violation: {0}")]
    SafetyViolation(String),
    
    #[error("Model error: {0}")]
    ModelError(String),
    
    #[error("Schema validation failed: {0}")]
    SchemaValidation(String),
    
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    
    #[error("Database error: {0}")]
    Database(#[from] crate::database::DatabaseError),
}

/// Processing result from AI analysis
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProcessingResult {
    /// Source artifact UUID
    pub artifact_uuid: uuid::Uuid,
    /// Detected document type
    pub document_type: DocumentType,
    /// Extracted entities
    pub entities: Vec<ExtractedEntity>,
    /// OCR text
    pub ocr_text: String,
    /// Confidence score (0.0 - 1.0)
    pub confidence: f64,
    /// Processing duration (ms)
    pub duration_ms: u64,
    /// Model version used
    pub model_version: String,
    /// Schema rules that matched
    pub matched_rules: Vec<String>,
    /// Privacy classification
    pub privacy: PrivacyClassification,
    /// Temporal data extracted
    pub temporal: TemporalExtraction,
    /// Semantic embedding (384-dim)
    pub embedding: Option<Vec<f32>>,
}

/// Document types recognized by the processor
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum DocumentType {
    /// Unknown/unclassified
    Unknown,
    /// Receipt/invoice
    Receipt,
    /// Contact card
    ContactCard,
    /// Flight/travel confirmation
    TravelConfirmation,
    /// Screenshot of messaging
    Messaging,
    /// Web page capture
    WebPage,
    /// Code/screenshot
    Code,
    /// General image
    Image,
    /// Document/PDF
    Document,
}

/// Extracted entity from processing
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExtractedEntity {
    /// Entity type
    pub entity_type: String,
    /// Entity value
    pub value: String,
    /// Confidence (0.0 - 1.0)
    pub confidence: f64,
    /// Normalized/formatted value
    pub normalized: Option<String>,
}

/// Temporal data extraction
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct TemporalExtraction {
    /// Dates mentioned in content
    pub dates: Vec<chrono::DateTime<chrono::Utc>>,
    /// Times mentioned
    pub times: Vec<String>,
    /// Fiscal quarter (if applicable)
    pub fiscal_quarter: Option<String>,
}

/// Content safety classification
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum SafetyClassification {
    /// Safe for processing
    Safe,
    /// NSFW content (process with caution)
    Nsfw,
    /// Medical content (privacy-sensitive)
    Medical,
    /// Banking/financial (high security)
    Banking,
    /// Quarantine (do not process)
    Quarantine,
}

/// Multimodal Processor
pub struct Processor {
    config: ProcessorConfig,
    schema: Arc<SchemaRuleset>,
    database: Option<Arc<Database>>,
}

impl Processor {
    /// Create a new processor
    pub fn new(config: ProcessorConfig) -> Self {
        // Load schema ruleset
        let schema = if let Some(ref path) = config.schema_path {
            // Load from file
            match std::fs::read_to_string(path) {
                Ok(content) => {
                    match SchemaRuleset::from_toml(&content) {
                        Ok(s) => s,
                        Err(e) => {
                            warn!("Failed to load schema from file, using default: {}", e);
                            SchemaRuleset::default_ruleset()
                        }
                    }
                }
                Err(e) => {
                    warn!("Failed to read schema file, using default: {}", e);
                    SchemaRuleset::default_ruleset()
                }
            }
        } else {
            SchemaRuleset::default_ruleset()
        };
        
        Self {
            config,
            schema: Arc::new(schema),
            database: None,
        }
    }
    
    /// Create processor with database access
    pub fn with_database(mut self, database: Arc<Database>) -> Self {
        self.database = Some(database);
        self
    }
    
    /// Process a raw artifact
    /// 
    /// This is the main entry point for AI processing. In production,
    /// this would interface with llama.cpp running Gemma 4.
    pub async fn process(&self, artifact_uuid: uuid::Uuid) -> Result<ProcessingResult, ProcessingError> {
        debug!("Processing artifact: {}", artifact_uuid);
        
        let start = std::time::Instant::now();
        
        // Load artifact from database
        let artifact = self.load_artifact(artifact_uuid).await?;
        
        // Content safety check
        if self.config.enable_safety_filter {
            let safety = self.check_safety(&artifact).await?;
            if safety == SafetyClassification::Quarantine {
                return Err(ProcessingError::SafetyViolation(
                    "Content quarantined by safety filter".to_string()
                ));
            }
        }
        
        // Perform AI processing (with timeout)
        let result = timeout(
            Duration::from_secs(self.config.timeout_secs),
            self.perform_ai_processing(&artifact)
        ).await.map_err(|_| ProcessingError::Timeout(self.config.timeout_secs))?;
        
        let mut result = result?;
        result.duration_ms = start.elapsed().as_millis() as u64;
        
        info!(
            "Processing complete: {} -> {} (confidence: {:.2}, duration: {}ms)",
            artifact_uuid, result.document_type as i32, result.confidence, result.duration_ms
        );
        
        Ok(result)
    }
    
    /// Load artifact from database
    async fn load_artifact(&self, uuid: uuid::Uuid) -> Result<RawArtifact, ProcessingError> {
        if let Some(ref db) = self.database {
            db.with_connection(|conn| {
                let repo = RawArtifactRepository::new(conn);
                repo.find_by_id(uuid)
                    .map_err(ProcessingError::Database)?
                    .ok_or_else(|| ProcessingError::ArtifactNotFound(uuid))
            })
        } else {
            Err(ProcessingError::ArtifactNotFound(uuid))
        }
    }
    
    /// Check content safety
    async fn check_safety(&self, artifact: &RawArtifact) -> Result<SafetyClassification, ProcessingError> {
        // In production, this would use a local MiniLM classifier
        // For now, return safe
        debug!("Checking content safety for: {}", artifact.uuid);
        Ok(SafetyClassification::Safe)
    }
    
    /// Perform AI processing (placeholder for actual Gemma 4 integration)
    async fn perform_ai_processing(&self, artifact: &RawArtifact) -> Result<ProcessingResult, ProcessingError> {
        debug!("Performing AI processing for: {}", artifact.uuid);
        
        // Build trigger context from artifact
        let ctx = self.build_trigger_context(artifact).await?;
        
        // Find matching schema rules
        let matched_rules = self.schema.find_matching_rules(&ctx);
        let rule_names: Vec<String> = matched_rules.iter()
            .map(|r| r.name.clone())
            .collect();
        
        // Determine document type from rules
        let document_type = if matched_rules.is_empty() {
            DocumentType::Unknown
        } else {
            self.infer_document_type(&matched_rules[0])
        };
        
        // Generate embedding (placeholder)
        let embedding = self.generate_embedding(artifact).await?;
        
        // Extract temporal data
        let temporal = self.extract_temporal(&ctx).await?;
        
        // Determine privacy classification
        let privacy = matched_rules.iter()
            .filter_map(|r| r.privacy)
            .next()
            .map(|p| p.to_classification())
            .unwrap_or(PrivacyClassification::Personal);
        
        // Mock extracted entities based on document type
        let entities = self.mock_extraction(document_type, &ctx);
        
        // Calculate confidence
        let confidence = if matched_rules.is_empty() {
            0.5
        } else {
            matched_rules[0].confidence_threshold
        };
        
        Ok(ProcessingResult {
            artifact_uuid: artifact.uuid,
            document_type,
            entities,
            ocr_text: ctx.ocr_text.unwrap_or_default(),
            confidence,
            duration_ms: 0,
            model_version: self.config.model_version.clone(),
            matched_rules: rule_names,
            privacy,
            temporal,
            embedding,
        })
    }
    
    /// Build trigger context from artifact
    async fn build_trigger_context(&self, artifact: &RawArtifact) -> Result<TriggerContext, ProcessingError> {
        let mut ctx = TriggerContext::new()
            .with_ocr_text(artifact.metadata.ocr_text.clone().unwrap_or_default());
        
        if let Some(ref app_ctx) = artifact.app_context {
            ctx = ctx.with_app_bundle(&app_ctx.bundle_id);
        }
        
        Ok(ctx)
    }
    
    /// Infer document type from rule
    fn infer_document_type(&self, rule: &CompilationRule) -> DocumentType {
        match rule.action {
            crate::models::Action::CreateEntity { .. } => DocumentType::Receipt,
            crate::models::Action::MergeOrCreate { .. } => DocumentType::ContactCard,
            crate::models::Action::CreateTravelItinerary => DocumentType::TravelConfirmation,
            crate::models::Action::CreateExpense => DocumentType::Receipt,
            crate::models::Action::CreateContact => DocumentType::ContactCard,
            crate::models::Action::CreateEvent => DocumentType::TravelConfirmation,
            crate::models::Action::Custom { ref action_type, .. } => {
                match action_type.as_str() {
                    "messaging" => DocumentType::Messaging,
                    "webpage" => DocumentType::WebPage,
                    "code" => DocumentType::Code,
                    _ => DocumentType::Unknown,
                }
            }
        }
    }
    
    /// Generate semantic embedding (placeholder)
    async fn generate_embedding(&self, _artifact: &RawArtifact) -> Result<Option<Vec<f32>>, ProcessingError> {
        // In production, this would use MiniLM-L6-v2
        // Return a 384-dim zero vector as placeholder
        Ok(Some(vec![0.0f32; 384]))
    }
    
    /// Extract temporal data from context
    async fn extract_temporal(&self, ctx: &TriggerContext) -> Result<TemporalExtraction, ProcessingError> {
        use regex::Regex;
        
        let mut temporal = TemporalExtraction::default();
        
        if let Some(ref text) = ctx.ocr_text {
            // Date patterns (simplified)
            let date_re = Regex::new(r"\b(\d{1,2})[/-](\d{1,2})[/-](\d{2,4})\b").unwrap();
            for cap in date_re.captures_iter(text) {
                // Parse date and add to temporal.dates
                // Simplified for demo
                debug!("Found date: {}/{}/{}", &cap[1], &cap[2], &cap[3]);
            }
        }
        
        Ok(temporal)
    }
    
    /// Mock entity extraction based on document type
    fn mock_extraction(&self, doc_type: DocumentType, ctx: &TriggerContext) -> Vec<ExtractedEntity> {
        let mut entities = Vec::new();
        
        match doc_type {
            DocumentType::Receipt => {
                entities.push(ExtractedEntity {
                    entity_type: "merchant".to_string(),
                    value: "Example Store".to_string(),
                    confidence: 0.85,
                    normalized: None,
                });
                entities.push(ExtractedEntity {
                    entity_type: "total_amount".to_string(),
                    value: "$50.00".to_string(),
                    confidence: 0.92,
                    normalized: Some("50.00".to_string()),
                });
            }
            DocumentType::ContactCard => {
                entities.push(ExtractedEntity {
                    entity_type: "name".to_string(),
                    value: "John Doe".to_string(),
                    confidence: 0.95,
                    normalized: None,
                });
            }
            DocumentType::TravelConfirmation => {
                entities.push(ExtractedEntity {
                    entity_type: "airline".to_string(),
                    value: "Example Airlines".to_string(),
                    confidence: 0.88,
                    normalized: None,
                });
            }
            _ => {}
        }
        
        entities
    }
    
    /// Get schema ruleset
    pub fn schema(&self) -> &SchemaRuleset {
        &self.schema
    }
    
    /// Update schema ruleset
    pub fn update_schema(&mut self, schema: SchemaRuleset) {
        self.schema = Arc::new(schema);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_processor_config() {
        let config = ProcessorConfig::default();
        assert_eq!(config.timeout_secs, 30);
        assert_eq!(config.max_memory_mb, 4096);
        assert!(config.enable_safety_filter);
    }
    
    #[test]
    fn test_document_type_serialization() {
        let doc_type = DocumentType::Receipt;
        let json = serde_json::to_string(&doc_type).unwrap();
        assert_eq!(json, "\"receipt\"");
        
        let parsed: DocumentType = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed, DocumentType::Receipt);
    }
    
    #[tokio::test]
    async fn test_processor_creation() {
        let processor = Processor::new(ProcessorConfig::default());
        assert_eq!(processor.config.model_version, "gemma-4-2b-it-Q4_K_M");
    }
}
