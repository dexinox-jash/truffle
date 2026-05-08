//! Schema Loader
//! 
//! Loads and parses schema.md files for the AI Constitution.

use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
use tracing::{info, warn, debug};

use crate::schema::{Schema, CompilationRule, TriggerCondition, CompilationAction, LinkTarget, PrivacyLevel, ProhibitedExtraction, ProhibitedAction, LinkingRules, EntityResolution, FieldType, DataType, NormalizationRule};
use crate::{AiError, AiResult};

/// Configuration for schema loading
#[derive(Debug, Clone)]
pub struct SchemaConfig {
    /// Path to schema.md file
    pub schema_path: PathBuf,
    /// Enable user customization
    pub allow_customization: bool,
    /// Auto-reload on changes
    pub auto_reload: bool,
}

impl Default for SchemaConfig {
    fn default() -> Self {
        Self {
            schema_path: PathBuf::from("schema.md"),
            allow_customization: true,
            auto_reload: false,
        }
    }
}

/// Loads and manages schema definitions
pub struct SchemaLoader {
    config: SchemaConfig,
    schema: Schema,
    loaded_at: chrono::DateTime<chrono::Utc>,
}

impl SchemaLoader {
    /// Create a new schema loader
    pub fn new(config: &SchemaConfig) -> AiResult<Self> {
        let mut loader = Self {
            config: config.clone(),
            schema: Schema::default(),
            loaded_at: chrono::Utc::now(),
        };
        
        // Try to load from file, fall back to defaults
        if config.schema_path.exists() {
            loader.load_from_file(&config.schema_path)?;
        } else {
            warn!("Schema file not found, using defaults: {:?}", config.schema_path);
        }
        
        Ok(loader)
    }
    
    /// Load schema from a markdown file
    fn load_from_file(&mut self, path: &Path) -> AiResult<()> {
        info!("Loading schema from {:?}", path);
        
        let content = fs::read_to_string(path)
            .map_err(|e| AiError::Io(e))?;
        
        self.schema = self.parse_schema_md(&content)?;
        self.loaded_at = chrono::Utc::now();
        
        info!("Schema loaded: {} compilation rules", self.schema.compilation_rules.len());
        
        Ok(())
    }
    
    /// Parse schema.md content
    /// 
    /// Schema.md format:
    /// ```markdown
    /// # Schema v2.0
    /// 
    /// ## Compilation Rules
    /// 
    /// ### Receipt Rule
    /// - trigger: concept_detected('receipt')
    /// - action: create_entity(merchant_name)
    /// - extract: [total_amount, currency, items_list]
    /// - privacy: financial
    /// ```
    fn parse_schema_md(&self, content: &str) -> AiResult<Schema> {
        let mut schema = Schema::default();
        
        // Parse version from header
        if let Some(version_line) = content.lines().find(|l| l.contains("Schema v")) {
            if let Some(version) = version_line.split("v").nth(1) {
                schema.version = version.trim().to_string();
            }
        }
        
        // Parse compilation rules
        let mut current_rule: Option<PartialRule> = None;
        
        for line in content.lines() {
            let line = line.trim();
            
            // Start of a new rule
            if line.starts_with("### ") && line.contains("Rule") {
                // Save previous rule if exists
                if let Some(rule) = current_rule.take() {
                    if let Ok(compiled) = self.compile_rule(rule) {
                        schema.compilation_rules.push(compiled);
                    }
                }
                
                // Start new rule
                let rule_name = line.trim_start_matches("### ").trim_end_matches(" Rule").to_lowercase();
                current_rule = Some(PartialRule {
                    id: rule_name,
                    trigger: None,
                    action: None,
                    extract: Vec::new(),
                    link_to: Vec::new(),
                    privacy: PrivacyLevel::Personal,
                    temporal_awareness: false,
                });
            }
            
            // Parse rule properties
            if let Some(ref mut rule) = current_rule {
                if line.starts_with("- trigger:") {
                    rule.trigger = Some(self.parse_trigger(&line[10..].trim()));
                } else if line.starts_with("- action:") {
                    rule.action = Some(self.parse_action(&line[9..].trim()));
                } else if line.starts_with("- extract:") {
                    rule.extract = self.parse_extract(&line[10..].trim());
                } else if line.starts_with("- link_to:") {
                    rule.link_to = self.parse_link_to(&line[9..].trim());
                } else if line.starts_with("- privacy:") {
                    rule.privacy = self.parse_privacy(&line[10..].trim());
                } else if line.starts_with("- temporal_awareness:") {
                    rule.temporal_awareness = line.contains("true");
                }
            }
        }
        
        // Save last rule
        if let Some(rule) = current_rule {
            if let Ok(compiled) = self.compile_rule(rule) {
                schema.compilation_rules.push(compiled);
            }
        }
        
        Ok(schema)
    }
    
    /// Parse trigger condition
    fn parse_trigger(&self, s: &str) -> TriggerCondition {
        let s = s.trim();
        
        if s.starts_with("concept_detected(") {
            if let Some(concept) = s.trim_start_matches("concept_detected('")
                .trim_end_matches("')")
                .split("'")
                .next() {
                return TriggerCondition::ConceptDetected { 
                    concept: concept.to_string() 
                };
            }
        }
        
        if s.starts_with("entity_type(") {
            if let Some(entity_type) = s.trim_start_matches("entity_type('")
                .trim_end_matches("')")
                .split("'")
                .next() {
                return TriggerCondition::EntityType { 
                    entity_type: entity_type.to_string() 
                };
            }
        }
        
        if s.starts_with("pattern(") {
            if let Some(pattern) = s.trim_start_matches("pattern('")
                .trim_end_matches("')")
                .split("'")
                .next() {
                return TriggerCondition::Pattern { 
                    pattern: pattern.to_string() 
                };
            }
        }
        
        // Default fallback
        TriggerCondition::ConceptDetected { concept: s.to_string() }
    }
    
    /// Parse action
    fn parse_action(&self, s: &str) -> CompilationAction {
        let s = s.trim();
        
        if s.starts_with("create_entity(") {
            if let Some(entity_type) = s.trim_start_matches("create_entity('")
                .trim_end_matches("')")
                .split("'")
                .next() {
                return CompilationAction::CreateEntity { 
                    entity_type: entity_type.to_string() 
                };
            }
        }
        
        if s.starts_with("merge_or_create(") {
            if let Some(path) = s.trim_start_matches("merge_or_create('")
                .trim_end_matches("')")
                .split("'")
                .next() {
                return CompilationAction::MergeOrCreate { 
                    path_template: path.to_string() 
                };
            }
        }
        
        if s == "create_travel_itinerary" {
            return CompilationAction::CreateTravelItinerary;
        }
        
        // Default fallback
        CompilationAction::CreateEntity { entity_type: s.to_string() }
    }
    
    /// Parse extract fields
    fn parse_extract(&self, s: &str) -> Vec<String> {
        s.trim_start_matches('[')
            .trim_end_matches(']')
            .split(',')
            .map(|f| f.trim().trim_matches('\'').trim_matches('"').to_string())
            .filter(|f| !f.is_empty())
            .collect()
    }
    
    /// Parse link targets
    fn parse_link_to(&self, s: &str) -> Vec<LinkTarget> {
        s.trim_start_matches('[')
            .trim_end_matches(']')
            .split(',')
            .filter_map(|target| {
                let target = target.trim();
                if target.is_empty() {
                    return None;
                }
                
                Some(LinkTarget {
                    node_type: "concept".to_string(),
                    path_template: target.trim_matches('\'').trim_matches('"').to_string(),
                    context: None,
                })
            })
            .collect()
    }
    
    /// Parse privacy level
    fn parse_privacy(&self, s: &str) -> PrivacyLevel {
        match s.trim() {
            "public" => PrivacyLevel::Public,
            "personal" => PrivacyLevel::Personal,
            "sensitive" => PrivacyLevel::Sensitive,
            "financial" => PrivacyLevel::Financial,
            _ => PrivacyLevel::Personal,
        }
    }
    
    /// Compile a partial rule into a full rule
    fn compile_rule(&self, partial: PartialRule) -> AiResult<CompilationRule> {
        Ok(CompilationRule {
            id: partial.id,
            trigger: partial.trigger.ok_or_else(|| 
                AiError::Schema("Missing trigger condition".to_string())
            )?,
            action: partial.action.ok_or_else(|| 
                AiError::Schema("Missing action".to_string())
            )?,
            extract: partial.extract,
            link_to: partial.link_to,
            privacy: partial.privacy,
            temporal_awareness: partial.temporal_awareness,
        })
    }
    
    /// Load schema context for prompt rendering
    pub async fn load_schema_context(&self) -> AiResult<String> {
        let mut context = String::new();
        
        context.push_str(&format!("# Schema v{}\n\n", self.schema.version));
        
        context.push_str("## Compilation Rules\n\n");
        for rule in &self.schema.compilation_rules {
            context.push_str(&format!("### {} Rule\n", rule.id));
            context.push_str(&format!("- Trigger: {:?}\n", rule.trigger));
            context.push_str(&format!("- Action: {:?}\n", rule.action));
            context.push_str(&format!("- Extract: {:?}\n", rule.extract));
            context.push_str(&format!("- Privacy: {:?}\n\n", rule.privacy));
        }
        
        context.push_str("## Prohibited Extractions\n\n");
        for prohibited in &self.schema.prohibited_extractions {
            context.push_str(&format!("- {}: {:?}\n", prohibited.field, prohibited.action));
        }
        
        Ok(context)
    }
    
    /// Get the loaded schema
    pub fn get_schema(&self) -> &Schema {
        &self.schema
    }
    
    /// Get a rule by ID
    pub fn get_rule(&self, rule_id: &str) -> Option<&CompilationRule> {
        self.schema.compilation_rules.iter().find(|r| r.id == rule_id)
    }
    
    /// Reload schema from disk
    pub fn reload(&mut self) -> AiResult<()> {
        if self.config.schema_path.exists() {
            self.load_from_file(&self.config.schema_path)?;
        }
        Ok(())
    }
    
    /// Get schema load timestamp
    pub fn loaded_at(&self) -> chrono::DateTime<chrono::Utc> {
        self.loaded_at
    }
}

/// Partial rule during parsing
struct PartialRule {
    id: String,
    trigger: Option<TriggerCondition>,
    action: Option<CompilationAction>,
    extract: Vec<String>,
    link_to: Vec<LinkTarget>,
    privacy: PrivacyLevel,
    temporal_awareness: bool,
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;
    
    #[test]
    fn test_schema_loader_default() {
        let config = SchemaConfig::default();
        let loader = SchemaLoader::new(&config);
        assert!(loader.is_ok());
    }
    
    #[test]
    fn test_parse_trigger() {
        let config = SchemaConfig::default();
        let loader = SchemaLoader::new(&config).unwrap();
        
        let trigger = loader.parse_trigger("concept_detected('receipt')");
        match trigger {
            TriggerCondition::ConceptDetected { concept } => {
                assert_eq!(concept, "receipt");
            }
            _ => panic!("Wrong trigger type"),
        }
    }
    
    #[test]
    fn test_parse_extract() {
        let config = SchemaConfig::default();
        let loader = SchemaLoader::new(&config).unwrap();
        
        let fields = loader.parse_extract("[total_amount, currency, items_list]");
        assert_eq!(fields.len(), 3);
        assert!(fields.contains(&"total_amount".to_string()));
    }
    
    #[test]
    fn test_parse_privacy() {
        let config = SchemaConfig::default();
        let loader = SchemaLoader::new(&config).unwrap();
        
        assert_eq!(loader.parse_privacy("financial"), PrivacyLevel::Financial);
        assert_eq!(loader.parse_privacy("public"), PrivacyLevel::Public);
    }
    
    #[tokio::test]
    async fn test_load_schema_context() {
        let config = SchemaConfig::default();
        let loader = SchemaLoader::new(&config).unwrap();
        
        let context = loader.load_schema_context().await.unwrap();
        assert!(context.contains("Schema v"));
        assert!(context.contains("Compilation Rules"));
    }
}
