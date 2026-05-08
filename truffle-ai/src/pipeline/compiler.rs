//! Main Compilation Engine
//! 
//! Orchestrates the entire compilation pipeline from screenshot to WikiNode.
//! 
//! # Pipeline Flow
//! 
//! ```text
//! 1. Receive image path
//! 2. Run safety classifier (MiniLM)
//! 3. Load appropriate prompt template
//! 4. Preprocess image (resize to 896px)
//! 5. Build prompt with schema context
//! 6. Run Gemma 4 inference (JSON mode)
//! 7. Parse and validate JSON output
//! 8. Extract WikiNode fields
//! 9. Return structured result
//! ```

use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::time::Instant;
use tokio::sync::RwLock;
use tracing::{info, warn, error, debug, instrument};

use crate::model::{Gemma4Engine, ModelConfig, ModelManager, ModelVerifier};
use crate::pipeline::{
    PromptManager, PromptTemplate, JsonParser, ValidationResult,
    SafetyClassifier, SafetyResult, ContentCategory,
    CompilationJob, CompilationStatus, CompilationPriority,
    PipelineStats, QueueConfig, AppContext,
};
use crate::schema::{SchemaLoader, SchemaMatcher, SchemaConfig};
use crate::{AiError, AiResult, PipelineConfig, COMPILATION_TIMEOUT_SECS};

/// Main AI Pipeline for screenshot compilation
pub struct AiPipeline {
    /// Model configuration
    config: PipelineConfig,
    /// Gemma 4 inference engine
    engine: Arc<RwLock<Option<Gemma4Engine>>>,
    /// Prompt manager
    prompt_manager: PromptManager,
    /// Safety classifier
    safety_classifier: SafetyClassifier,
    /// Schema loader
    schema_loader: SchemaLoader,
    /// JSON parser
    parser: JsonParser,
    /// Pipeline statistics
    stats: Arc<RwLock<PipelineStats>>,
    /// Model manager
    model_manager: ModelManager,
    /// Model verifier
    model_verifier: ModelVerifier,
}

/// Options for compilation
#[derive(Debug, Clone)]
pub struct CompilationOptions {
    /// Force recompilation even if cached
    pub force_recompile: bool,
    /// Skip safety check (use with caution)
    pub skip_safety: bool,
    /// Custom timeout override
    pub timeout_secs: Option<u64>,
    /// Priority level
    pub priority: CompilationPriority,
    /// App context for context-aware compilation
    pub app_context: Option<AppContext>,
}

impl Default for CompilationOptions {
    fn default() -> Self {
        Self {
            force_recompile: false,
            skip_safety: false,
            timeout_secs: None,
            priority: CompilationPriority::Normal,
            app_context: None,
        }
    }
}

/// Result of a compilation operation
#[derive(Debug, Clone)]
pub struct CompilationResult {
    /// Unique result ID
    pub id: uuid::Uuid,
    /// Compilation status
    pub status: CompilationStatus,
    /// JSON output (if successful)
    pub json_output: Option<String>,
    /// Parsed WikiNode (if successful)
    pub wiki_node: Option<WikiNode>,
    /// Error message (if failed)
    pub error: Option<String>,
    /// Safety classification
    pub safety_result: SafetyResult,
    /// Compilation time in milliseconds
    pub compilation_time_ms: u64,
    /// Model confidence score
    pub confidence: f32,
    /// Schema rules applied
    pub schema_rules: Vec<String>,
}

/// WikiNode output structure (SPEC v2.0 Section 2.1.2)
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct WikiNode {
    /// Node ID
    pub id: uuid::Uuid,
    /// Node type
    pub node_type: NodeType,
    /// Human-readable title
    pub title: String,
    /// Markdown content
    pub content: String,
    /// Backlinks from other nodes
    pub backlinks: Vec<WikiLink>,
    /// Forward links to other nodes
    pub forward_links: Vec<WikiLink>,
    /// Provenance information
    pub provenance: Provenance,
    /// Temporal vectors
    pub temporal_vectors: TemporalVectors,
    /// Privacy classification
    pub privacy_classification: PrivacyLevel,
    /// Encryption status
    pub encryption_status: EncryptionStatus,
}

/// Node type for WikiNode
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum NodeType {
    /// Concrete entity (person, place, thing)
    Entity,
    /// Abstract concept
    Concept,
    /// Chronological entry
    Chronology,
    /// Index/list node
    Index,
}

/// WikiLink for bidirectional linking
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct WikiLink {
    /// Target node ID
    pub target_id: uuid::Uuid,
    /// Link text
    pub text: String,
    /// Link context
    pub context: Option<String>,
}

/// Provenance information
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct Provenance {
    /// Source artifact UUIDs
    pub source_artifacts: Vec<uuid::Uuid>,
    /// Compilation timestamp
    pub compiled_at: chrono::DateTime<chrono::Utc>,
    /// Model version used
    pub model_version: String,
    /// Confidence score
    pub confidence_score: f32,
}

/// Temporal vectors for time-based queries
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct TemporalVectors {
    /// Dates mentioned in the content
    pub mentioned_dates: Vec<chrono::DateTime<chrono::Utc>>,
    /// Fiscal quarter (if applicable)
    pub fiscal_quarter: Option<String>,
}

/// Privacy classification levels
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PrivacyLevel {
    /// Public information
    Public,
    /// Personal information
    Personal,
    /// Sensitive information
    Sensitive,
    /// Financial information
    Financial,
}

/// Encryption status
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum EncryptionStatus {
    /// Not encrypted
    Plaintext,
    /// AES-256-GCM encrypted
    Aes256Gcm,
}

impl AiPipeline {
    /// Create a new AI pipeline
    /// 
    /// # Arguments
    /// 
    /// * `config` - Pipeline configuration
    /// 
    /// # Errors
    /// 
    /// Returns an error if initialization fails.
    pub async fn new(config: PipelineConfig) -> AiResult<Self> {
        info!("Initializing AI Pipeline v{}", crate::PIPELINE_VERSION);
        
        // Initialize model manager
        let model_manager = ModelManager::new(config.model.clone())?;
        
        // Ensure model is downloaded
        let model_path = model_manager.ensure_model().await?;
        
        // Verify model integrity
        let model_verifier = ModelVerifier::new(config.model.clone());
        model_verifier.verify_on_startup(&model_path)?;
        
        // Initialize prompt manager
        let prompt_manager = PromptManager::new(&config.prompts_dir)?;
        
        // Initialize safety classifier
        let safety_classifier = SafetyClassifier::new()?;
        
        // Initialize schema loader
        let schema_loader = SchemaLoader::new(&config.schema)?;
        
        // Initialize JSON parser
        let parser = JsonParser::new();
        
        info!("AI Pipeline initialized successfully");
        
        Ok(Self {
            config,
            engine: Arc::new(RwLock::new(None)),
            prompt_manager,
            safety_classifier,
            schema_loader,
            parser,
            stats: Arc::new(RwLock::new(PipelineStats::default())),
            model_manager,
            model_verifier,
        })
    }
    
    /// Initialize the inference engine (lazy loading)
    async fn ensure_engine(&self) -> AiResult<()> {
        let mut engine_guard = self.engine.write().await;
        
        if engine_guard.is_none() {
            let model_path = self.config.model.model_path();
            let engine = Gemma4Engine::new(self.config.model.clone(), &model_path).await?;
            *engine_guard = Some(engine);
            info!("Gemma 4 engine initialized");
        }
        
        Ok(())
    }
    
    /// Compile a single image
    /// 
    /// # Arguments
    /// 
    /// * `image_path` - Path to the screenshot image
    /// * `options` - Compilation options
    /// 
    /// # Returns
    /// 
    /// Returns a `CompilationResult` with the compiled WikiNode or error.
    #[instrument(skip(self, image_path, options))]
    pub async fn compile_image(
        &self,
        image_path: &Path,
        options: CompilationOptions,
    ) -> AiResult<CompilationResult> {
        let start_time = Instant::now();
        let result_id = uuid::Uuid::new_v4();
        
        info!("Starting compilation for {:?}", image_path);
        
        // Initialize engine if needed
        self.ensure_engine().await?;
        
        // Step 1: Safety check
        let safety_result = if options.skip_safety {
            SafetyResult::safe()
        } else {
            self.run_safety_check(image_path).await?
        };
        
        if safety_result.is_unsafe() {
            warn!("Content flagged by safety classifier: {:?}", safety_result.categories);
            
            return Ok(CompilationResult {
                id: result_id,
                status: CompilationStatus::Quarantined,
                json_output: None,
                wiki_node: None,
                error: Some(format!("Content quarantined: {:?}", safety_result.categories)),
                safety_result,
                compilation_time_ms: start_time.elapsed().as_millis() as u64,
                confidence: 0.0,
                schema_rules: vec![],
            });
        }
        
        // Step 2: Load appropriate prompt template
        let prompt_template = self.select_prompt_template(&options).await?;
        
        // Step 3: Build prompts
        let (system_prompt, user_prompt) = self.build_prompts(&prompt_template, image_path).await?;
        
        // Step 4: Run inference
        let timeout_secs = options.timeout_secs.unwrap_or(COMPILATION_TIMEOUT_SECS);
        let engine_guard = self.engine.read().await;
        let engine = engine_guard.as_ref().ok_or_else(|| {
            AiError::Compilation("Engine not initialized".to_string())
        })?;
        
        let compilation_output = match engine.compile_image(
            image_path,
            &system_prompt,
            &user_prompt,
        ).await {
            Ok(output) => output,
            Err(e) => {
                error!("Compilation failed: {}", e);
                return Ok(CompilationResult {
                    id: result_id,
                    status: CompilationStatus::Failed,
                    json_output: None,
                    wiki_node: None,
                    error: Some(e.to_string()),
                    safety_result,
                    compilation_time_ms: start_time.elapsed().as_millis() as u64,
                    confidence: 0.0,
                    schema_rules: vec![],
                });
            }
        };
        
        // Step 5: Parse and validate JSON
        let validation_result = self.parser.parse_and_validate(&compilation_output.json)?;
        
        let wiki_node = match validation_result {
            ValidationResult::Valid(json) => {
                Some(self.json_to_wiki_node(&json, &compilation_output)?)
            }
            ValidationResult::Invalid(errors) => {
                return Ok(CompilationResult {
                    id: result_id,
                    status: CompilationStatus::Failed,
                    json_output: Some(compilation_output.json),
                    wiki_node: None,
                    error: Some(format!("JSON validation failed: {:?}", errors)),
                    safety_result,
                    compilation_time_ms: start_time.elapsed().as_millis() as u64,
                    confidence: compilation_output.confidence,
                    schema_rules: vec![],
                });
            }
        };
        
        let compilation_time = start_time.elapsed();
        
        // Update statistics
        {
            let mut stats = self.stats.write().await;
            stats.successful += 1;
            stats.update_avg_time(compilation_time.as_millis() as u64);
        }
        
        info!(
            "Compilation completed in {:?} with confidence {}",
            compilation_time, compilation_output.confidence
        );
        
        Ok(CompilationResult {
            id: result_id,
            status: CompilationStatus::Completed,
            json_output: Some(compilation_output.json),
            wiki_node,
            error: None,
            safety_result,
            compilation_time_ms: compilation_time.as_millis() as u64,
            confidence: compilation_output.confidence,
            schema_rules: prompt_template.schema_rules.clone(),
        })
    }
    
    /// Run safety classifier on image
    async fn run_safety_check(&self, image_path: &Path) -> AiResult<SafetyResult> {
        debug!("Running safety check on {:?}", image_path);
        self.safety_classifier.classify(image_path).await
    }
    
    /// Select appropriate prompt template based on context
    async fn select_prompt_template(&self, options: &CompilationOptions) -> AiResult<PromptTemplate> {
        // Check app context for hints
        if let Some(ref app_ctx) = options.app_context {
            let app_name_lower = app_ctx.app_name.to_lowercase();
            
            if app_name_lower.contains("mail") || app_name_lower.contains("message") {
                return self.prompt_manager.get_template("contact");
            }
            
            if app_name_lower.contains("travel") || app_name_lower.contains("flight") {
                return self.prompt_manager.get_template("travel");
            }
            
            if app_name_lower.contains("shop") || app_name_lower.contains("store") {
                return self.prompt_manager.get_template("receipt");
            }
        }
        
        // Default to general template
        self.prompt_manager.get_template("system")
    }
    
    /// Build system and user prompts
    async fn build_prompts(
        &self,
        template: &PromptTemplate,
        image_path: &Path,
    ) -> AiResult<(String, String)> {
        // Load schema context
        let schema_context = self.schema_loader.load_schema_context().await?;
        
        // Build system prompt
        let system_prompt = template.render_system(&schema_context)?;
        
        // Build user prompt
        let user_prompt = template.render_user(image_path)?;
        
        Ok((system_prompt, user_prompt))
    }
    
    /// Convert JSON to WikiNode
    fn json_to_wiki_node(
        &self,
        json: &serde_json::Value,
        output: &crate::model::gemma4::CompilationOutput,
    ) -> AiResult<WikiNode> {
        let id = uuid::Uuid::new_v4();
        
        // Extract fields from JSON
        let node_type = json.get("node_type")
            .and_then(|v| v.as_str())
            .map(|s| match s {
                "entity" => NodeType::Entity,
                "concept" => NodeType::Concept,
                "chronology" => NodeType::Chronology,
                "index" => NodeType::Index,
                _ => NodeType::Entity,
            })
            .unwrap_or(NodeType::Entity);
        
        let title = json.get("title")
            .and_then(|v| v.as_str())
            .unwrap_or("Untitled")
            .to_string();
        
        let content = json.get("content")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string();
        
        let privacy_classification = json.get("privacy_classification")
            .and_then(|v| v.as_str())
            .map(|s| match s {
                "public" => PrivacyLevel::Public,
                "personal" => PrivacyLevel::Personal,
                "sensitive" => PrivacyLevel::Sensitive,
                "financial" => PrivacyLevel::Financial,
                _ => PrivacyLevel::Personal,
            })
            .unwrap_or(PrivacyLevel::Personal);
        
        let confidence = json.get("confidence_score")
            .and_then(|v| v.as_f64())
            .map(|f| f as f32)
            .unwrap_or(output.confidence);
        
        Ok(WikiNode {
            id,
            node_type,
            title,
            content,
            backlinks: vec![],
            forward_links: vec![],
            provenance: Provenance {
                source_artifacts: vec![id],
                compiled_at: chrono::Utc::now(),
                model_version: format!("gemma-4-e2b-{}", crate::PIPELINE_VERSION),
                confidence_score: confidence,
            },
            temporal_vectors: TemporalVectors {
                mentioned_dates: vec![],
                fiscal_quarter: None,
            },
            privacy_classification,
            encryption_status: match privacy_classification {
                PrivacyLevel::Financial | PrivacyLevel::Sensitive => EncryptionStatus::Aes256Gcm,
                _ => EncryptionStatus::Plaintext,
            },
        })
    }
    
    /// Get current pipeline statistics
    pub async fn get_stats(&self) -> PipelineStats {
        self.stats.read().await.clone()
    }
    
    /// Shutdown the pipeline
    pub async fn shutdown(self) {
        info!("Shutting down AI Pipeline");
        
        if let Some(engine) = self.engine.write().await.take() {
            engine.shutdown().await;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_compilation_options_default() {
        let opts = CompilationOptions::default();
        assert!(!opts.force_recompile);
        assert!(!opts.skip_safety);
        assert_eq!(opts.priority, CompilationPriority::Normal);
    }
    
    #[test]
    fn test_wiki_node_creation() {
        let node = WikiNode {
            id: uuid::Uuid::new_v4(),
            node_type: NodeType::Entity,
            title: "Test Node".to_string(),
            content: "Test content".to_string(),
            backlinks: vec![],
            forward_links: vec![],
            provenance: Provenance {
                source_artifacts: vec![],
                compiled_at: chrono::Utc::now(),
                model_version: "test".to_string(),
                confidence_score: 0.95,
            },
            temporal_vectors: TemporalVectors {
                mentioned_dates: vec![],
                fiscal_quarter: None,
            },
            privacy_classification: PrivacyLevel::Personal,
            encryption_status: EncryptionStatus::Plaintext,
        };
        
        assert_eq!(node.title, "Test Node");
        assert!(node.provenance.confidence_score > 0.9);
    }
}
