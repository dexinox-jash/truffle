//! Schema Ruleset Model
//!
//! The Schema defines the AI Constitution - rules for how RawArtifacts are
//! compiled into WikiNodes. It includes compilation rules, privacy settings,
//! and prohibited extractions.
//!
//! Per Section 2.1.3 of the Enterprise Specification:
//! - schema.md distributed with app updates, user editable
//! - Trigger-based compilation rules
//! - Privacy classification per rule
//! - Prohibited extractions for sensitive data

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Schema version for migration tracking
pub const CURRENT_SCHEMA_VERSION: &str = "2.0";

/// Privacy levels for extracted data
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum PrivacyLevel {
    /// Publicly shareable
    Public,
    /// Personal information
    Personal,
    /// Sensitive data
    Sensitive,
    /// Financial data (auto-encrypt)
    Financial,
}

impl PrivacyLevel {
    /// Convert to PrivacyClassification
    pub fn to_classification(&self) -> super::PrivacyClassification {
        match self {
            PrivacyLevel::Public => super::PrivacyClassification::Public,
            PrivacyLevel::Personal => super::PrivacyClassification::Personal,
            PrivacyLevel::Sensitive => super::PrivacyClassification::Sensitive,
            PrivacyLevel::Financial => super::PrivacyClassification::Financial,
        }
    }
}

/// Trigger conditions for compilation rules
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum Trigger {
    /// Concept detected in content (e.g., "receipt")
    ConceptDetected { concept: String },
    /// Entity type detected
    EntityType { entity_type: String },
    /// Specific context detected
    Context { context: String },
    /// Pattern matched in content
    Pattern { pattern: String },
    /// Entity type AND context both match
    EntityTypeAndContext { entity_type: String, context: String },
    /// OCR text contains keyword
    OcrContains { keyword: String },
    /// App bundle ID matches
    AppBundle { bundle_id: String },
    /// Custom trigger with multiple conditions
    Custom { conditions: HashMap<String, String> },
}

impl Trigger {
    /// Evaluate if this trigger matches the given context
    pub fn evaluate(&self, ctx: &TriggerContext) -> bool {
        match self {
            Trigger::ConceptDetected { concept } => {
                ctx.detected_concepts.iter().any(|c| 
                    c.to_lowercase() == concept.to_lowercase()
                )
            }
            Trigger::EntityType { entity_type } => {
                ctx.entity_types.iter().any(|e| 
                    e.to_lowercase() == entity_type.to_lowercase()
                )
            }
            Trigger::Context { context } => {
                ctx.contexts.iter().any(|c| 
                    c.to_lowercase() == context.to_lowercase()
                )
            }
            Trigger::Pattern { pattern } => {
                ctx.ocr_text.as_ref()
                    .map(|text| text.contains(pattern))
                    .unwrap_or(false)
            }
            Trigger::EntityTypeAndContext { entity_type, context } => {
                self.evaluate_entity_type_and_context(ctx, entity_type, context)
            }
            Trigger::OcrContains { keyword } => {
                ctx.ocr_text.as_ref()
                    .map(|text| text.to_lowercase().contains(&keyword.to_lowercase()))
                    .unwrap_or(false)
            }
            Trigger::AppBundle { bundle_id } => {
                ctx.app_bundle.as_ref()
                    .map(|b| b.to_lowercase() == bundle_id.to_lowercase())
                    .unwrap_or(false)
            }
            Trigger::Custom { conditions } => {
                conditions.iter().all(|(key, value)| {
                    ctx.custom.get(key)
                        .map(|v| v.to_lowercase() == value.to_lowercase())
                        .unwrap_or(false)
                })
            }
        }
    }
    
    fn evaluate_entity_type_and_context(
        &self,
        ctx: &TriggerContext,
        entity_type: &str,
        context: &str,
    ) -> bool {
        let entity_match = ctx.entity_types.iter().any(|e| 
            e.to_lowercase() == entity_type.to_lowercase()
        );
        let context_match = ctx.contexts.iter().any(|c| 
            c.to_lowercase() == context.to_lowercase()
        );
        entity_match && context_match
    }
    
    /// Get human-readable description
    pub fn description(&self) -> String {
        match self {
            Trigger::ConceptDetected { concept } => {
                format!("Concept '{}' detected", concept)
            }
            Trigger::EntityType { entity_type } => {
                format!("Entity type '{}' detected", entity_type)
            }
            Trigger::Context { context } => {
                format!("Context '{}' detected", context)
            }
            Trigger::Pattern { pattern } => {
                format!("Pattern '{}' matched", pattern)
            }
            Trigger::EntityTypeAndContext { entity_type, context } => {
                format!("Entity '{}' in context '{}'", entity_type, context)
            }
            Trigger::OcrContains { keyword } => {
                format!("OCR contains '{}'", keyword)
            }
            Trigger::AppBundle { bundle_id } => {
                format!("App bundle '{}'", bundle_id)
            }
            Trigger::Custom { conditions } => {
                format!("Custom: {:?}", conditions)
            }
        }
    }
}

/// Context for trigger evaluation
#[derive(Debug, Clone, Default)]
pub struct TriggerContext {
    /// Concepts detected by AI
    pub detected_concepts: Vec<String>,
    /// Entity types detected
    pub entity_types: Vec<String>,
    /// Contexts detected
    pub contexts: Vec<String>,
    /// OCR text content
    pub ocr_text: Option<String>,
    /// App bundle ID
    pub app_bundle: Option<String>,
    /// Custom fields
    pub custom: HashMap<String, String>,
}

impl TriggerContext {
    pub fn new() -> Self {
        Self::default()
    }
    
    pub fn with_concepts(mut self, concepts: Vec<String>) -> Self {
        self.detected_concepts = concepts;
        self
    }
    
    pub fn with_entity_types(mut self, types: Vec<String>) -> Self {
        self.entity_types = types;
        self
    }
    
    pub fn with_ocr_text(mut self, text: impl Into<String>) -> Self {
        self.ocr_text = Some(text.into());
        self
    }
    
    pub fn with_app_bundle(mut self, bundle: impl Into<String>) -> Self {
        self.app_bundle = Some(bundle.into());
        self
    }
}

/// Actions to take when trigger matches
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum Action {
    /// Create a new entity node
    CreateEntity { name_field: String },
    /// Merge with existing or create new
    MergeOrCreate { path_template: String },
    /// Create travel itinerary
    CreateTravelItinerary,
    /// Create expense entry
    CreateExpense,
    /// Create contact card
    CreateContact,
    /// Create event
    CreateEvent,
    /// Custom action
    Custom { action_type: String, params: HashMap<String, serde_json::Value> },
}

impl Action {
    /// Get the target node type for this action
    pub fn target_node_type(&self) -> super::WikiNodeType {
        match self {
            Action::CreateEntity { .. } => super::WikiNodeType::Entity,
            Action::MergeOrCreate { .. } => super::WikiNodeType::Entity,
            Action::CreateTravelItinerary => super::WikiNodeType::Chronology,
            Action::CreateExpense => super::WikiNodeType::Entity,
            Action::CreateContact => super::WikiNodeType::Entity,
            Action::CreateEvent => super::WikiNodeType::Chronology,
            Action::Custom { action_type, .. } => {
                match action_type.as_str() {
                    "chronology" => super::WikiNodeType::Chronology,
                    "concept" => super::WikiNodeType::Concept,
                    "index" => super::WikiNodeType::Index,
                    _ => super::WikiNodeType::Entity,
                }
            }
        }
    }
    
    /// Get human-readable description
    pub fn description(&self) -> String {
        match self {
            Action::CreateEntity { name_field } => {
                format!("Create entity from field '{}'", name_field)
            }
            Action::MergeOrCreate { path_template } => {
                format!("Merge or create at '{}'", path_template)
            }
            Action::CreateTravelItinerary => "Create travel itinerary".to_string(),
            Action::CreateExpense => "Create expense entry".to_string(),
            Action::CreateContact => "Create contact card".to_string(),
            Action::CreateEvent => "Create event".to_string(),
            Action::Custom { action_type, .. } => {
                format!("Custom action: {}", action_type)
            }
        }
    }
}

/// A single compilation rule
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompilationRule {
    /// Rule name/identifier
    pub name: String,
    /// Trigger condition
    pub trigger: Trigger,
    /// Action to take
    pub action: Action,
    /// Fields to extract
    #[serde(default)]
    pub extract: Vec<String>,
    /// Nodes to link to
    #[serde(default)]
    pub link_to: Vec<String>,
    /// Privacy classification
    #[serde(default)]
    pub privacy: Option<PrivacyLevel>,
    /// Enable temporal awareness
    #[serde(default)]
    pub temporal_awareness: bool,
    /// Required confidence threshold
    #[serde(default = "default_confidence")]
    pub confidence_threshold: f64,
    /// Whether this rule is enabled
    #[serde(default = "default_true")]
    pub enabled: bool,
}

fn default_confidence() -> f64 {
    0.7
}

fn default_true() -> bool {
    true
}

impl CompilationRule {
    /// Create a new compilation rule
    pub fn new(name: impl Into<String>, trigger: Trigger, action: Action) -> Self {
        Self {
            name: name.into(),
            trigger,
            action,
            extract: Vec::new(),
            link_to: Vec::new(),
            privacy: None,
            temporal_awareness: false,
            confidence_threshold: 0.7,
            enabled: true,
        }
    }
    
    /// Add fields to extract
    pub fn with_extract(mut self, fields: Vec<String>) -> Self {
        self.extract = fields;
        self
    }
    
    /// Add link targets
    pub fn with_link_to(mut self, targets: Vec<String>) -> Self {
        self.link_to = targets;
        self
    }
    
    /// Set privacy level
    pub fn with_privacy(mut self, privacy: PrivacyLevel) -> Self {
        self.privacy = Some(privacy);
        self
    }
    
    /// Enable temporal awareness
    pub fn with_temporal_awareness(mut self) -> Self {
        self.temporal_awareness = true;
        self
    }
    
    /// Set confidence threshold
    pub fn with_confidence_threshold(mut self, threshold: f64) -> Self {
        self.confidence_threshold = threshold.clamp(0.0, 1.0);
        self
    }
    
    /// Evaluate this rule against a context
    pub fn evaluate(&self, ctx: &TriggerContext) -> bool {
        if !self.enabled {
            return false;
        }
        self.trigger.evaluate(ctx)
    }
}

/// Prohibited extraction rule for sensitive data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProhibitedExtraction {
    /// Field name to protect
    pub field: String,
    /// Action to take when detected
    pub action: RedactionAction,
    /// Whether to alert the user
    #[serde(default = "default_true")]
    pub alert_user: bool,
}

/// Actions for prohibited extractions
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum RedactionAction {
    /// Redact and log locally
    RedactAndLog,
    /// Tokenize (keep last 4 only)
    TokenizeLast4,
    /// Refuse and quarantine
    RefuseAndQuarantine,
    /// Hash the value
    HashOnly,
    /// Delete the field
    Delete,
}

/// Linking rules for knowledge graph construction
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LinkingRules {
    /// Temporal proximity threshold in seconds (screenshots within this window considered related)
    #[serde(default = "default_temporal_proximity")]
    pub temporal_proximity: u64,
    /// Semantic similarity threshold (cosine similarity)
    #[serde(default = "default_semantic_threshold")]
    pub semantic_similarity_threshold: f64,
    /// Entity resolution: Levenshtein distance threshold
    #[serde(default = "default_levenshtein_threshold")]
    pub entity_resolution_levenshtein: usize,
    /// Enable Metaphone for phonetic matching
    #[serde(default = "default_true")]
    pub use_metaphone: bool,
}

fn default_temporal_proximity() -> u64 {
    300 // 5 minutes
}

fn default_semantic_threshold() -> f64 {
    0.82
}

fn default_levenshtein_threshold() -> usize {
    2
}

impl Default for LinkingRules {
    fn default() -> Self {
        Self {
            temporal_proximity: default_temporal_proximity(),
            semantic_similarity_threshold: default_semantic_threshold(),
            entity_resolution_levenshtein: default_levenshtein_threshold(),
            use_metaphone: true,
        }
    }
}

/// Schema configuration options
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SchemaConfig {
    /// Default model version for compilation
    #[serde(default = "default_model_version")]
    pub default_model_version: String,
    /// Maximum compilation time per artifact (seconds)
    #[serde(default = "default_max_compilation_time")]
    pub max_compilation_time_secs: u64,
    /// Maximum memory usage (MB)
    #[serde(default = "default_max_memory_mb")]
    pub max_memory_mb: u64,
    /// Enable hallucination detection
    #[serde(default = "default_true")]
    pub hallucination_detection: bool,
    /// Hallucination threshold (amounts above this require high confidence)
    #[serde(default = "default_hallucination_threshold")]
    pub hallucination_amount_threshold: f64,
}

fn default_model_version() -> String {
    "gemma-4-2b-it-Q4_K_M".to_string()
}

fn default_max_compilation_time() -> u64 {
    30
}

fn default_max_memory_mb() -> u64 {
    4096
}

fn default_hallucination_threshold() -> f64 {
    100000.0
}

impl Default for SchemaConfig {
    fn default() -> Self {
        Self {
            default_model_version: default_model_version(),
            max_compilation_time_secs: default_max_compilation_time(),
            max_memory_mb: default_max_memory_mb(),
            hallucination_detection: true,
            hallucination_amount_threshold: default_hallucination_threshold(),
        }
    }
}

/// The complete schema ruleset
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SchemaRuleset {
    /// Schema version
    pub schema_version: String,
    /// Compilation rules
    pub compilation_rules: Vec<CompilationRule>,
    /// Prohibited extractions
    #[serde(default)]
    pub prohibited_extractions: Vec<ProhibitedExtraction>,
    /// Linking rules
    #[serde(default)]
    pub linking_rules: LinkingRules,
    /// Schema configuration
    #[serde(default)]
    pub config: SchemaConfig,
}

impl SchemaRuleset {
    /// Create a new empty ruleset
    pub fn new() -> Self {
        Self {
            schema_version: CURRENT_SCHEMA_VERSION.to_string(),
            compilation_rules: Vec::new(),
            prohibited_extractions: Vec::new(),
            linking_rules: LinkingRules::default(),
            config: SchemaConfig::default(),
        }
    }
    
    /// Add a compilation rule
    pub fn add_rule(&mut self, rule: CompilationRule) {
        self.compilation_rules.push(rule);
    }
    
    /// Add a prohibited extraction
    pub fn add_prohibited_extraction(&mut self, field: impl Into<String>, action: RedactionAction) {
        self.prohibited_extractions.push(ProhibitedExtraction {
            field: field.into(),
            action,
            alert_user: true,
        });
    }
    
    /// Find matching rules for a trigger context
    pub fn find_matching_rules(&self, ctx: &TriggerContext) -> Vec<&CompilationRule> {
        self.compilation_rules
            .iter()
            .filter(|rule| rule.evaluate(ctx))
            .collect()
    }
    
    /// Load from TOML string
    pub fn from_toml(toml_str: &str) -> anyhow::Result<Self> {
        let ruleset: SchemaRuleset = toml::from_str(toml_str)?;
        Ok(ruleset)
    }
    
    /// Save to TOML string
    pub fn to_toml(&self) -> anyhow::Result<String> {
        let toml_str = toml::to_string_pretty(self)?;
        Ok(toml_str)
    }
    
    /// Get default ruleset with built-in rules
    pub fn default_ruleset() -> Self {
        let mut ruleset = Self::new();
        
        // Receipt detection rule
        ruleset.add_rule(
            CompilationRule::new(
                "receipt_detection",
                Trigger::ConceptDetected { concept: "receipt".to_string() },
                Action::CreateEntity { name_field: "merchant_name".to_string() },
            )
            .with_extract(vec![
                "total_amount".to_string(),
                "currency".to_string(),
                "tax_amount".to_string(),
                "items_list".to_string(),
                "payment_method".to_string(),
            ])
            .with_link_to(vec![
                "concepts/expenses".to_string(),
                "chronology/{YYYY-MM}".to_string(),
            ])
            .with_privacy(PrivacyLevel::Financial)
        );
        
        // Contact card rule
        ruleset.add_rule(
            CompilationRule::new(
                "contact_card",
                Trigger::EntityTypeAndContext {
                    entity_type: "person".to_string(),
                    context: "contact_card".to_string(),
                },
                Action::MergeOrCreate {
                    path_template: "entities/{firstname-lastname}.md".to_string(),
                },
            )
            .with_extract(vec![
                "phone".to_string(),
                "email".to_string(),
                "company".to_string(),
                "role".to_string(),
            ])
            .with_privacy(PrivacyLevel::Personal)
        );
        
        // Flight confirmation rule
        ruleset.add_rule(
            CompilationRule::new(
                "flight_confirmation",
                Trigger::Pattern { pattern: "flight confirmation".to_string() },
                Action::CreateTravelItinerary,
            )
            .with_extract(vec![
                "airline".to_string(),
                "flight_number".to_string(),
                "departure".to_string(),
                "arrival".to_string(),
                "confirmation_code".to_string(),
            ])
            .with_temporal_awareness()
        );
        
        // Add prohibited extractions
        ruleset.add_prohibited_extraction("password", RedactionAction::RedactAndLog);
        ruleset.add_prohibited_extraction("credit_card_number", RedactionAction::TokenizeLast4);
        ruleset.add_prohibited_extraction("ssn", RedactionAction::RefuseAndQuarantine);
        ruleset.add_prohibited_extraction("social_security_number", RedactionAction::RefuseAndQuarantine);
        
        ruleset
    }
    
    /// Validate the ruleset
    pub fn validate(&self) -> Vec<String> {
        let mut errors = Vec::new();
        
        // Check for duplicate rule names
        let mut names = std::collections::HashSet::new();
        for rule in &self.compilation_rules {
            if !names.insert(&rule.name) {
                errors.push(format!("Duplicate rule name: {}", rule.name));
            }
        }
        
        // Validate confidence thresholds
        for rule in &self.compilation_rules {
            if rule.confidence_threshold < 0.0 || rule.confidence_threshold > 1.0 {
                errors.push(format!(
                    "Rule '{}' has invalid confidence threshold: {}",
                    rule.name, rule.confidence_threshold
                ));
            }
        }
        
        // Validate semantic similarity threshold
        if self.linking_rules.semantic_similarity_threshold < 0.0 
            || self.linking_rules.semantic_similarity_threshold > 1.0 {
            errors.push("Invalid semantic_similarity_threshold".to_string());
        }
        
        errors
    }
}

impl Default for SchemaRuleset {
    fn default() -> Self {
        Self::default_ruleset()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_trigger_evaluation() {
        let trigger = Trigger::ConceptDetected { concept: "receipt".to_string() };
        
        let ctx = TriggerContext::new()
            .with_concepts(vec!["receipt".to_string(), "invoice".to_string()]);
        
        assert!(trigger.evaluate(&ctx));
        
        let ctx_no_match = TriggerContext::new()
            .with_concepts(vec!["photo".to_string()]);
        
        assert!(!trigger.evaluate(&ctx_no_match));
    }
    
    #[test]
    fn test_compilation_rule_evaluation() {
        let rule = CompilationRule::new(
            "test_rule",
            Trigger::OcrContains { keyword: "Total".to_string() },
            Action::CreateExpense,
        );
        
        let ctx = TriggerContext::new()
            .with_ocr_text("Total: $50.00");
        
        assert!(rule.evaluate(&ctx));
        
        let ctx_no_match = TriggerContext::new()
            .with_ocr_text("Hello world");
        
        assert!(!rule.evaluate(&ctx_no_match));
    }
    
    #[test]
    fn test_default_ruleset() {
        let ruleset = SchemaRuleset::default_ruleset();
        
        assert_eq!(ruleset.schema_version, CURRENT_SCHEMA_VERSION);
        assert!(!ruleset.compilation_rules.is_empty());
        assert!(!ruleset.prohibited_extractions.is_empty());
        
        // Test validation
        let errors = ruleset.validate();
        assert!(errors.is_empty(), "Validation errors: {:?}", errors);
    }
    
    #[test]
    fn test_toml_serialization() {
        let ruleset = SchemaRuleset::default_ruleset();
        
        let toml_str = ruleset.to_toml().unwrap();
        assert!(!toml_str.is_empty());
        
        let parsed = SchemaRuleset::from_toml(&toml_str).unwrap();
        assert_eq!(parsed.schema_version, ruleset.schema_version);
    }
    
    #[test]
    fn test_prohibited_extractions() {
        let ruleset = SchemaRuleset::default_ruleset();
        
        let has_password = ruleset.prohibited_extractions.iter()
            .any(|p| p.field == "password");
        assert!(has_password);
        
        let has_ssn = ruleset.prohibited_extractions.iter()
            .any(|p| p.field == "ssn");
        assert!(has_ssn);
    }
    
    #[test]
    fn test_entity_type_and_context_trigger() {
        let trigger = Trigger::EntityTypeAndContext {
            entity_type: "person".to_string(),
            context: "contact_card".to_string(),
        };
        
        let ctx = TriggerContext::new()
            .with_entity_types(vec!["person".to_string()])
            .with_context(vec!["contact_card".to_string()]);
        
        assert!(trigger.evaluate(&ctx));
        
        let ctx_partial = TriggerContext::new()
            .with_entity_types(vec!["person".to_string()]);
        
        assert!(!trigger.evaluate(&ctx_partial));
    }
}
