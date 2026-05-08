//! Prompt Templates with Versioning
//! 
//! Manages versioned prompt templates for different compilation scenarios.
//! 
//! # Versioning Strategy (SPEC v2.0 Section 4.2)
//! 
//! - All prompts are versioned with semantic versioning
//! - A/B testing support via prompt variant IDs
//! - Prompts stored in `prompts/` directory
//! - Hash-identified for performance tracking

use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
use serde::{Deserialize, Serialize};
use sha2::{Sha256, Digest};
use tracing::{info, warn, debug};

use crate::{AiError, AiResult};

/// Semantic version for prompts
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PromptVersion {
    /// Major version (breaking changes)
    pub major: u32,
    /// Minor version (new features)
    pub minor: u32,
    /// Patch version (bug fixes)
    pub patch: u32,
}

impl PromptVersion {
    /// Create a new version
    pub fn new(major: u32, minor: u32, patch: u32) -> Self {
        Self { major, minor, patch }
    }
    
    /// Parse from string (e.g., "1.2.3")
    pub fn parse(s: &str) -> AiResult<Self> {
        let parts: Vec<&str> = s.split('.').collect();
        if parts.len() != 3 {
            return Err(AiError::Other(format!("Invalid version format: {}", s)));
        }
        
        Ok(Self {
            major: parts[0].parse().map_err(|_| AiError::Other("Invalid major version".to_string()))?,
            minor: parts[1].parse().map_err(|_| AiError::Other("Invalid minor version".to_string()))?,
            patch: parts[2].parse().map_err(|_| AiError::Other("Invalid patch version".to_string()))?,
        })
    }
}

impl Default for PromptVersion {
    fn default() -> Self {
        Self::new(1, 0, 0)
    }
}

impl std::fmt::Display for PromptVersion {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}.{}.{}", self.major, self.minor, self.patch)
    }
}

/// A prompt template with metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PromptTemplate {
    /// Template ID (e.g., "receipt", "contact")
    pub id: String,
    /// Semantic version
    pub version: PromptVersion,
    /// Content hash for tracking
    pub hash: String,
    /// Template name
    pub name: String,
    /// Description
    pub description: String,
    /// System prompt template
    pub system_template: String,
    /// User prompt template
    pub user_template: String,
    /// Schema rules this template applies
    pub schema_rules: Vec<String>,
    /// Example outputs for few-shot prompting
    pub examples: Vec<PromptExample>,
    /// A/B test variant ID
    pub variant_id: Option<String>,
    /// Creation timestamp
    pub created_at: chrono::DateTime<chrono::Utc>,
}

/// Example for few-shot prompting
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PromptExample {
    /// Input description
    pub input: String,
    /// Expected output (JSON)
    pub output: String,
    /// Explanation
    pub explanation: Option<String>,
}

/// Manager for prompt templates
pub struct PromptManager {
    /// Directory containing prompt files
    prompts_dir: PathBuf,
    /// Loaded templates
    templates: HashMap<String, PromptTemplate>,
    /// Current A/B test allocation
    ab_allocation: HashMap<String, f64>, // template_id -> percentage
}

impl PromptManager {
    /// Create a new prompt manager
    /// 
    /// # Arguments
    /// 
    /// * `prompts_dir` - Directory containing prompt template files
    pub fn new(prompts_dir: &Path) -> AiResult<Self> {
        let mut manager = Self {
            prompts_dir: prompts_dir.to_path_buf(),
            templates: HashMap::new(),
            ab_allocation: HashMap::new(),
        };
        
        // Load all templates
        manager.load_all_templates()?;
        
        // Set default A/B allocation
        manager.ab_allocation.insert("system".to_string(), 100.0);
        
        Ok(manager)
    }
    
    /// Load all templates from the prompts directory
    fn load_all_templates(&mut self) -> AiResult<()> {
        if !self.prompts_dir.exists() {
            warn!("Prompts directory not found: {:?}", self.prompts_dir);
            // Create default templates
            self.create_default_templates()?;
            return Ok(());
        }
        
        // Load system template
        if let Ok(template) = self.load_template_file("system") {
            self.templates.insert("system".to_string(), template);
        }
        
        // Load specialized templates
        for template_id in ["receipt", "contact", "travel"] {
            if let Ok(template) = self.load_template_file(template_id) {
                self.templates.insert(template_id.to_string(), template);
            }
        }
        
        info!("Loaded {} prompt templates", self.templates.len());
        Ok(())
    }
    
    /// Load a single template file
    fn load_template_file(&self, template_id: &str) -> AiResult<PromptTemplate> {
        let file_path = self.prompts_dir.join(format!("{}.txt", template_id));
        
        debug!("Loading template from {:?}", file_path);
        
        let content = fs::read_to_string(&file_path)
            .map_err(|e| AiError::Io(e))?;
        
        self.parse_template(template_id, &content)
    }
    
    /// Parse template content
    fn parse_template(&self, id: &str, content: &str) -> AiResult<PromptTemplate> {
        // Parse header (YAML frontmatter) and body
        let parts: Vec<&str> = content.splitn(3, "---").collect();
        
        if parts.len() < 3 {
            return Err(AiError::Other(format!(
                "Invalid template format for {}", id
            )));
        }
        
        // Parse YAML header
        let header: PromptHeader = serde_yaml::from_str(parts[1].trim())
            .map_err(|e| AiError::Other(format!("Failed to parse template header: {}", e)))?;
        
        // Split body into system and user templates
        let body = parts[2].trim();
        let body_parts: Vec<&str> = body.split("[USER_PROMPT]").collect();
        
        let system_template = body_parts[0].trim().to_string();
        let user_template = body_parts.get(1)
            .map(|s| s.trim().to_string())
            .unwrap_or_default();
        
        // Compute hash
        let mut hasher = Sha256::new();
        hasher.update(content.as_bytes());
        let hash = hex::encode(hasher.finalize())[..16].to_string();
        
        Ok(PromptTemplate {
            id: id.to_string(),
            version: PromptVersion::parse(&header.version)?,
            hash,
            name: header.name,
            description: header.description,
            system_template,
            user_template,
            schema_rules: header.schema_rules,
            examples: header.examples.unwrap_or_default(),
            variant_id: header.variant_id,
            created_at: chrono::Utc::now(),
        })
    }
    
    /// Get a template by ID
    pub fn get_template(&self, id: &str) -> AiResult<PromptTemplate> {
        self.templates
            .get(id)
            .cloned()
            .ok_or_else(|| AiError::Other(format!("Template not found: {}", id)))
    }
    
    /// Get template for A/B testing
    pub fn get_template_for_user(&self, user_id: &str, template_id: &str) -> AiResult<PromptTemplate> {
        // Simple hash-based allocation
        let hash = Self::hash_user_id(user_id);
        let allocation = self.ab_allocation.get(template_id).copied().unwrap_or(100.0);
        
        // If allocation is 100%, always use the main template
        if allocation >= 100.0 {
            return self.get_template(template_id);
        }
        
        // Otherwise, allocate based on hash
        let user_bucket = (hash % 100) as f64;
        
        if user_bucket < allocation {
            self.get_template(template_id)
        } else {
            // Use variant B (would be loaded separately)
            self.get_template(&format!("{}_variant_b", template_id))
        }
    }
    
    /// Hash user ID for consistent A/B allocation
    fn hash_user_id(user_id: &str) -> u64 {
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};
        
        let mut hasher = DefaultHasher::new();
        user_id.hash(&mut hasher);
        hasher.finish()
    }
    
    /// Create default templates if prompts directory doesn't exist
    fn create_default_templates(&mut self) -> AiResult<()> {
        fs::create_dir_all(&self.prompts_dir)
            .map_err(|e| AiError::Io(e))?;
        
        // System template
        let system_template = r#"---
name: "System Prompt"
description: "Main system prompt with schema rules"
version: "1.0.0"
schema_rules:
  - compilation_rules
  - extraction_rules
  - privacy_rules
---
You are Truffle AI, a knowledge compilation engine. Your task is to analyze screenshots and extract structured information.

## Rules
1. Output ONLY valid JSON
2. Follow the schema rules provided
3. Be precise with dates, amounts, and names
4. Classify privacy level appropriately

## Schema Context
{{schema_context}}

## Output Format
{
  "node_type": "entity|concept|chronology|index",
  "title": "Human-readable title",
  "content": "Markdown content with extracted information",
  "privacy_classification": "public|personal|sensitive|financial",
  "confidence_score": 0.0-1.0
}

[USER_PROMPT]
Compile this screenshot according to the schema rules. Output JSON only.

Image: {{image_path}}"#;

        fs::write(self.prompts_dir.join("system.txt"), system_template)
            .map_err(|e| AiError::Io(e))?;
        
        info!("Created default prompt templates");
        Ok(())
    }
    
    /// List all available templates
    pub fn list_templates(&self) -> Vec<&PromptTemplate> {
        self.templates.values().collect()
    }
    
    /// Reload templates from disk
    pub fn reload(&mut self) -> AiResult<()> {
        self.templates.clear();
        self.load_all_templates()
    }
}

/// YAML header for prompt templates
#[derive(Debug, Clone, Deserialize)]
struct PromptHeader {
    name: String,
    description: String,
    version: String,
    #[serde(default)]
    schema_rules: Vec<String>,
    #[serde(default)]
    examples: Option<Vec<PromptExample>>,
    #[serde(default)]
    variant_id: Option<String>,
}

impl PromptTemplate {
    /// Render the system prompt with context
    pub fn render_system(&self, schema_context: &str) -> AiResult<String> {
        let mut rendered = self.system_template.clone();
        rendered = rendered.replace("{{schema_context}}", schema_context);
        Ok(rendered)
    }
    
    /// Render the user prompt with image path
    pub fn render_user(&self, image_path: &Path) -> AiResult<String> {
        let mut rendered = self.user_template.clone();
        rendered = rendered.replace("{{image_path}}", &image_path.to_string_lossy());
        Ok(rendered)
    }
    
    /// Get full prompt with examples for few-shot
    pub fn render_with_examples(&self, schema_context: &str, image_path: &Path) -> AiResult<String> {
        let mut prompt = self.render_system(schema_context)?;
        
        // Add examples if available
        if !self.examples.is_empty() {
            prompt.push_str("\n\n## Examples\n");
            for (i, example) in self.examples.iter().enumerate() {
                prompt.push_str(&format!("\n### Example {}\n", i + 1));
                prompt.push_str(&format!("Input: {}\n", example.input));
                prompt.push_str(&format!("Output: {}\n", example.output));
            }
        }
        
        prompt.push_str("\n");
        prompt.push_str(&self.render_user(image_path)?);
        
        Ok(prompt)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;
    
    #[test]
    fn test_prompt_version() {
        let v = PromptVersion::new(1, 2, 3);
        assert_eq!(v.to_string(), "1.2.3");
        
        let parsed = PromptVersion::parse("2.0.0").unwrap();
        assert_eq!(parsed.major, 2);
    }
    
    #[test]
    fn test_prompt_manager_creation() {
        let temp_dir = TempDir::new().unwrap();
        let manager = PromptManager::new(temp_dir.path());
        assert!(manager.is_ok());
    }
    
    #[test]
    fn test_template_rendering() {
        let template = PromptTemplate {
            id: "test".to_string(),
            version: PromptVersion::default(),
            hash: "abc123".to_string(),
            name: "Test".to_string(),
            description: "Test template".to_string(),
            system_template: "Context: {{schema_context}}".to_string(),
            user_template: "Image: {{image_path}}".to_string(),
            schema_rules: vec![],
            examples: vec![],
            variant_id: None,
            created_at: chrono::Utc::now(),
        };
        
        let system = template.render_system("test context").unwrap();
        assert!(system.contains("test context"));
        
        let user = template.render_user(Path::new("/test/image.png")).unwrap();
        assert!(user.contains("/test/image.png"));
    }
}
