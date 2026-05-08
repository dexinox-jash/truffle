//! WikiNode Model
//!
//! The WikiNode represents compiled, structured knowledge extracted from RawArtifacts.
//! It forms the nodes of the knowledge graph with bidirectional linking.
//!
//! Per Section 2.1.2 of the Enterprise Specification:
//! - Markdown content with YAML frontmatter
//! - WikiLinks for bidirectional navigation
//! - Provenance tracking for audit trail
//! - CRDT vector clock for conflict resolution

use serde::{Deserialize, Serialize};
use uuid::Uuid;
use chrono::{DateTime, Utc};
use std::collections::HashMap;

use super::VectorClock;

/// Type of wiki node
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Default)]
#[serde(rename_all = "snake_case")]
pub enum WikiNodeType {
    /// A specific entity (person, place, organization)
    #[default]
    Entity,
    /// An abstract concept or category
    Concept,
    /// A chronological entry (date, event, timeline)
    Chronology,
    /// An index or table of contents
    Index,
}

impl WikiNodeType {
    /// Get the directory name for this node type
    pub fn directory(&self) -> &'static str {
        match self {
            WikiNodeType::Entity => "entities",
            WikiNodeType::Concept => "concepts",
            WikiNodeType::Chronology => "chronology",
            WikiNodeType::Index => "index",
        }
    }
    
    /// Get the template filename for this node type
    pub fn template(&self) -> &'static str {
        match self {
            WikiNodeType::Entity => "entity_template.md",
            WikiNodeType::Concept => "concept_template.md",
            WikiNodeType::Chronology => "chronology_template.md",
            WikiNodeType::Index => "index_template.md",
        }
    }
}

/// Privacy classification for access control and encryption
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Default)]
#[serde(rename_all = "snake_case")]
pub enum PrivacyClassification {
    /// Publicly shareable
    Public,
    /// Personal but not sensitive
    #[default]
    Personal,
    /// Sensitive information
    Sensitive,
    /// Financial data (auto-encrypt in sync)
    Financial,
}

impl PrivacyClassification {
    /// Check if this classification requires encryption during sync
    pub fn requires_encryption(&self) -> bool {
        matches!(self, PrivacyClassification::Sensitive | PrivacyClassification::Financial)
    }
    
    /// Get the encryption level required
    pub fn encryption_level(&self) -> &'static str {
        match self {
            PrivacyClassification::Public => "none",
            PrivacyClassification::Personal => "recommended",
            PrivacyClassification::Sensitive => "required",
            PrivacyClassification::Financial => "required",
        }
    }
}

/// Encryption status of the node
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Default)]
#[serde(rename_all = "snake_case")]
pub enum EncryptionStatus {
    /// Stored as plaintext
    #[default]
    Plaintext,
    /// Encrypted with AES-256-GCM
    Aes256Gcm,
}

/// A wiki link (bidirectional reference between nodes)
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub struct WikiLink {
    /// Target node ID
    pub target_id: Uuid,
    /// Target node title (for display)
    pub target_title: String,
    /// Link context (e.g., "mentioned in", "parent of", "related to")
    pub context: Option<String>,
    /// When the link was created
    pub created_at: DateTime<Utc>,
}

impl WikiLink {
    pub fn new(target_id: Uuid, target_title: impl Into<String>) -> Self {
        Self {
            target_id,
            target_title: target_title.into(),
            context: None,
            created_at: Utc::now(),
        }
    }
    
    pub fn with_context(mut self, context: impl Into<String>) -> Self {
        self.context = Some(context.into());
        self
    }
    
    /// Render as wikilink syntax: [[Target Title]]
    pub fn to_wikilink(&self) -> String {
        format!("[[{}]]", self.target_title)
    }
    
    /// Render as wikilink with alias: [[Target Title|Display Text]]
    pub fn to_wikilink_with_alias(&self, alias: impl AsRef<str>) -> String {
        format!("[[{}|{}]]", self.target_title, alias.as_ref())
    }
}

/// Provenance - audit trail for compiled knowledge
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Provenance {
    /// Source artifacts that contributed to this node
    pub source_artifacts: Vec<Uuid>,
    /// When this node was compiled
    pub compiled_at: DateTime<Utc>,
    /// Model version used for compilation
    pub model_version: String,
    /// Confidence score (0.0 - 1.0)
    pub confidence_score: f64,
    /// Compilation duration in milliseconds
    pub compilation_duration_ms: Option<u64>,
    /// Schema version used
    pub schema_version: String,
}

impl Provenance {
    pub fn new(
        source_artifacts: Vec<Uuid>,
        model_version: impl Into<String>,
        confidence_score: f64,
    ) -> Self {
        Self {
            source_artifacts,
            compiled_at: Utc::now(),
            model_version: model_version.into(),
            confidence_score: confidence_score.clamp(0.0, 1.0),
            compilation_duration_ms: None,
            schema_version: "2.0".to_string(),
        }
    }
    
    pub fn with_duration(mut self, duration_ms: u64) -> Self {
        self.compilation_duration_ms = Some(duration_ms);
        self
    }
    
    /// Check if confidence meets threshold
    pub fn meets_confidence_threshold(&self, threshold: f64) -> bool {
        self.confidence_score >= threshold
    }
    
    /// Get confidence level as string
    pub fn confidence_level(&self) -> &'static str {
        match self.confidence_score {
            s if s >= 0.95 => "very_high",
            s if s >= 0.85 => "high",
            s if s >= 0.70 => "medium",
            s if s >= 0.50 => "low",
            _ => "very_low",
        }
    }
}

/// Temporal vectors for time-based queries
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct TemporalVectors {
    /// Dates mentioned in the content
    pub mentioned_dates: Vec<DateTime<Utc>>,
    /// Fiscal quarter (e.g., "Q1-2026")
    pub fiscal_quarter: Option<String>,
    /// Day of week (0 = Sunday, 6 = Saturday)
    pub day_of_week: Option<u8>,
    /// Time of day (hour, 0-23)
    pub hour_of_day: Option<u8>,
}

impl TemporalVectors {
    pub fn new() -> Self {
        Self::default()
    }
    
    pub fn with_mentioned_dates(mut self, dates: Vec<DateTime<Utc>>) -> Self {
        self.mentioned_dates = dates;
        self
    }
    
    pub fn with_fiscal_quarter(mut self, quarter: impl Into<String>) -> Self {
        self.fiscal_quarter = Some(quarter.into());
        self
    }
    
    /// Calculate fiscal quarter from date
    pub fn calculate_fiscal_quarter(date: DateTime<Utc>, fiscal_year_start: u8) -> String {
        let month = date.month() as i32;
        let year = date.year();
        
        // Adjust month based on fiscal year start
        let adjusted_month = ((month - fiscal_year_start as i32 + 12) % 12) + 1;
        let quarter = ((adjusted_month - 1) / 3) + 1;
        
        // Adjust year if we're in a different fiscal year
        let fiscal_year = if month < fiscal_year_start as u32 {
            year
        } else {
            year + 1
        };
        
        format!("Q{}-{}", quarter, fiscal_year)
    }
}

/// Markdown AST node types for structured content
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum MarkdownNode {
    /// Document root
    Document { children: Vec<MarkdownNode> },
    /// Heading (h1-h6)
    Heading { level: u8, text: String },
    /// Paragraph
    Paragraph { text: String },
    /// Bullet list
    List { items: Vec<String>, ordered: bool },
    /// Code block
    CodeBlock { language: Option<String>, code: String },
    /// WikiLink
    WikiLink { target: String, alias: Option<String> },
    /// Regular link
    Link { url: String, text: String },
    /// Blockquote
    Blockquote { text: String },
    /// Horizontal rule
    HorizontalRule,
    /// Table
    Table { headers: Vec<String>, rows: Vec<Vec<String>> },
}

/// YAML frontmatter for wiki nodes
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Frontmatter {
    /// Node title
    pub title: String,
    /// Node type
    pub node_type: WikiNodeType,
    /// Creation date
    pub created_at: DateTime<Utc>,
    /// Last modified date
    pub updated_at: DateTime<Utc>,
    /// Privacy classification
    pub privacy: PrivacyClassification,
    /// Tags
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub tags: Vec<String>,
    /// Custom metadata
    #[serde(flatten)]
    pub custom: HashMap<String, serde_json::Value>,
}

/// The WikiNode - compiled knowledge entity
/// 
/// This is the primary unit of knowledge in Truffle. RawArtifacts are compiled
/// into WikiNodes through the AI pipeline, forming an interconnected graph.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WikiNode {
    /// Unique identifier
    pub id: Uuid,
    /// Node type
    pub node_type: WikiNodeType,
    /// Human-readable title (URL slugified)
    pub title: String,
    /// Content as Markdown AST
    pub content: Vec<MarkdownNode>,
    /// Raw markdown content (for storage)
    pub raw_content: String,
    /// Backlinks (nodes that reference this one)
    pub backlinks: Vec<WikiLink>,
    /// Forward links (nodes this one references)
    pub forward_links: Vec<WikiLink>,
    /// Provenance for audit trail
    pub provenance: Provenance,
    /// Temporal vectors for time queries
    #[serde(default)]
    pub temporal_vectors: TemporalVectors,
    /// Privacy classification
    pub privacy_classification: PrivacyClassification,
    /// Encryption status
    pub encryption_status: EncryptionStatus,
    /// CRDT vector clock for sync
    pub version: VectorClock,
    /// When created
    pub created_at: DateTime<Utc>,
    /// When last modified
    pub updated_at: DateTime<Utc>,
    /// Semantic embedding for similarity search
    #[serde(skip_serializing_if = "Option::is_none")]
    pub embedding: Option<Vec<f32>>,
}

impl WikiNode {
    /// Create a new wiki node
    pub fn new(
        title: impl Into<String>,
        node_type: WikiNodeType,
        content: impl Into<String>,
        provenance: Provenance,
    ) -> Self {
        let title = title.into();
        let raw_content = content.into();
        let now = Utc::now();
        
        // Parse markdown to AST
        let content_ast = parse_markdown_to_ast(&raw_content);
        
        // Extract forward links from content
        let forward_links = extract_wikilinks(&raw_content);
        
        Self {
            id: Uuid::new_v4(),
            node_type,
            title: title.clone(),
            content: content_ast,
            raw_content,
            backlinks: Vec::new(),
            forward_links,
            provenance,
            temporal_vectors: TemporalVectors::new(),
            privacy_classification: PrivacyClassification::Personal,
            encryption_status: EncryptionStatus::Plaintext,
            version: VectorClock::new(),
            created_at: now,
            updated_at: now,
            embedding: None,
        }
    }
    
    /// Set privacy classification
    pub fn with_privacy(mut self, classification: PrivacyClassification) -> Self {
        self.privacy_classification = classification;
        if classification.requires_encryption() {
            self.encryption_status = EncryptionStatus::Aes256Gcm;
        }
        self
    }
    
    /// Set temporal vectors
    pub fn with_temporal_vectors(mut self, vectors: TemporalVectors) -> Self {
        self.temporal_vectors = vectors;
        self
    }
    
    /// Set embedding vector
    pub fn with_embedding(mut self, embedding: Vec<f32>) -> Self {
        self.embedding = Some(embedding);
        self
    }
    
    /// Add a backlink from another node
    pub fn add_backlink(&mut self, link: WikiLink) {
        // Prevent duplicates
        if !self.backlinks.iter().any(|l| l.target_id == link.target_id) {
            self.backlinks.push(link);
            self.updated_at = Utc::now();
        }
    }
    
    /// Remove a backlink
    pub fn remove_backlink(&mut self, target_id: Uuid) {
        self.backlinks.retain(|l| l.target_id != target_id);
        self.updated_at = Utc::now();
    }
    
    /// Update forward links from content
    pub fn update_forward_links(&mut self) {
        self.forward_links = extract_wikilinks(&self.raw_content);
        self.updated_at = Utc::now();
    }
    
    /// Update content and re-parse
    pub fn update_content(&mut self, new_content: impl Into<String>) {
        self.raw_content = new_content.into();
        self.content = parse_markdown_to_ast(&self.raw_content);
        self.update_forward_links();
        self.updated_at = Utc::now();
    }
    
    /// Increment version for a device
    pub fn increment_version(&mut self, device_id: &str) {
        self.version.increment(device_id);
        self.updated_at = Utc::now();
    }
    
    /// Merge with another version (CRDT)
    pub fn merge(&mut self, other: &WikiNode) {
        // Merge vector clocks
        self.version.merge(&other.version);
        
        // For now, last-write-wins for content
        // TODO: Implement proper Yjs merge
        if other.updated_at > self.updated_at {
            self.raw_content = other.raw_content.clone();
            self.content = other.content.clone();
            self.forward_links = other.forward_links.clone();
        }
        
        // Merge backlinks (union)
        for link in &other.backlinks {
            self.add_backlink(link.clone());
        }
        
        self.updated_at = Utc::now();
    }
    
    /// Generate URL-friendly slug from title
    pub fn slug(&self) -> String {
        slugify(&self.title)
    }
    
    /// Get file path for storage
    pub fn file_path(&self) -> String {
        format!("{}/{}.md", self.node_type.directory(), self.slug())
    }
    
    /// Generate YAML frontmatter
    pub fn generate_frontmatter(&self) -> String {
        let frontmatter = Frontmatter {
            title: self.title.clone(),
            node_type: self.node_type,
            created_at: self.created_at,
            updated_at: self.updated_at,
            privacy: self.privacy_classification,
            tags: Vec::new(),
            custom: HashMap::new(),
        };
        
        match serde_yaml::to_string(&frontmatter) {
            Ok(yaml) => format!("---\n{}---\n\n", yaml),
            Err(_) => String::new(),
        }
    }
    
    /// Export to full markdown with frontmatter
    pub fn to_markdown(&self) -> String {
        format!("{}{}", self.generate_frontmatter(), self.raw_content)
    }
    
    /// Get all linked node IDs (both directions)
    pub fn all_links(&self) -> Vec<Uuid> {
        let mut links: Vec<Uuid> = self.forward_links.iter()
            .map(|l| l.target_id)
            .chain(self.backlinks.iter().map(|l| l.target_id))
            .collect();
        links.dedup();
        links
    }
    
    /// Check if this node links to another
    pub fn links_to(&self, node_id: Uuid) -> bool {
        self.forward_links.iter().any(|l| l.target_id == node_id)
    }
    
    /// Check if this node is linked from another
    pub fn linked_from(&self, node_id: Uuid) -> bool {
        self.backlinks.iter().any(|l| l.target_id == node_id)
    }
    
    /// Get word count
    pub fn word_count(&self) -> usize {
        self.raw_content.split_whitespace().count()
    }
    
    /// Get summary (first N characters)
    pub fn summary(&self, max_chars: usize) -> String {
        if self.raw_content.len() <= max_chars {
            self.raw_content.clone()
        } else {
            format!("{}...", &self.raw_content[..max_chars])
        }
    }
}

/// Parse markdown content to AST
fn parse_markdown_to_ast(content: &str) -> Vec<MarkdownNode> {
    use pulldown_cmark::{Parser, Event, Tag, TagEnd, HeadingLevel};
    
    let parser = Parser::new(content);
    let mut nodes = Vec::new();
    let mut current_text = String::new();
    let mut in_code_block = false;
    let mut code_language = None;
    let mut code_content = String::new();
    
    for event in parser {
        match event {
            Event::Start(tag) => {
                match tag {
                    Tag::CodeBlock(lang) => {
                        in_code_block = true;
                        code_language = lang.as_ref().map(|s| s.to_string());
                    }
                    _ => {}
                }
            }
            Event::End(tag_end) => {
                match tag_end {
                    TagEnd::CodeBlock => {
                        in_code_block = false;
                        nodes.push(MarkdownNode::CodeBlock {
                            language: code_language.take(),
                            code: code_content.clone(),
                        });
                        code_content.clear();
                    }
                    TagEnd::Heading(level) => {
                        if !current_text.is_empty() {
                            nodes.push(MarkdownNode::Heading {
                                level: level as u8,
                                text: current_text.trim().to_string(),
                            });
                            current_text.clear();
                        }
                    }
                    TagEnd::Paragraph => {
                        if !current_text.is_empty() {
                            nodes.push(MarkdownNode::Paragraph {
                                text: current_text.trim().to_string(),
                            });
                            current_text.clear();
                        }
                    }
                    _ => {}
                }
            }
            Event::Text(text) => {
                if in_code_block {
                    code_content.push_str(&text);
                } else {
                    current_text.push_str(&text);
                }
            }
            Event::Code(code) => {
                current_text.push('`');
                current_text.push_str(&code);
                current_text.push('`');
            }
            Event::Html(html) => {
                current_text.push_str(&html);
            }
            _ => {}
        }
    }
    
    // Handle any remaining text
    if !current_text.is_empty() {
        nodes.push(MarkdownNode::Paragraph {
            text: current_text.trim().to_string(),
        });
    }
    
    nodes
}

/// Extract wikilinks from markdown content
fn extract_wikilinks(content: &str) -> Vec<WikiLink> {
    use regex::Regex;
    
    let mut links = Vec::new();
    // Pattern: [[Target]] or [[Target|Alias]]
    let re = Regex::new(r"\[\[([^\]|]+)(?:\|([^\]]+))?\]\]").unwrap();
    
    for cap in re.captures_iter(content) {
        let target = cap[1].trim().to_string();
        // For now, generate a deterministic UUID from target name
        // In production, this would lookup the actual node ID
        let target_id = {
            use sha2::{Sha256, Digest};
            let mut hasher = Sha256::new();
            hasher.update(target.as_bytes());
            let result = hasher.finalize();
            let bytes: [u8; 16] = result[..16].try_into().unwrap();
            Uuid::from_bytes(bytes)
        };
        
        links.push(WikiLink::new(target_id, target));
    }
    
    links
}

/// Convert title to URL-friendly slug
fn slugify(title: &str) -> String {
    title
        .to_lowercase()
        .replace(|c: char| !c.is_alphanumeric() && c != '-', "-")
        .replace("--", "-")
        .trim_matches('-')
        .to_string()
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_wiki_node_creation() {
        let provenance = Provenance::new(
            vec![Uuid::new_v4()],
            "gemma-4-2b-it-Q4_K_M",
            0.92,
        );
        
        let node = WikiNode::new(
            "Test Entity",
            WikiNodeType::Entity,
            "This is test content with [[Another Page]] link.",
            provenance,
        );
        
        assert_eq!(node.title, "Test Entity");
        assert_eq!(node.node_type, WikiNodeType::Entity);
        assert_eq!(node.slug(), "test-entity");
        assert_eq!(node.word_count(), 8);
        assert_eq!(node.forward_links.len(), 1);
    }
    
    #[test]
    fn test_wikilink_extraction() {
        let content = "See [[Getting Started]] and [[Advanced Topics|Advanced]] for more.";
        let links = extract_wikilinks(content);
        
        assert_eq!(links.len(), 2);
        assert_eq!(links[0].target_title, "Getting Started");
        assert_eq!(links[1].target_title, "Advanced Topics");
    }
    
    #[test]
    fn test_backlink_management() {
        let provenance = Provenance::new(vec![Uuid::new_v4()], "test", 0.9);
        let mut node = WikiNode::new("Test", WikiNodeType::Entity, "Content", provenance);
        
        let link = WikiLink::new(Uuid::new_v4(), "Source Page");
        node.add_backlink(link.clone());
        
        assert_eq!(node.backlinks.len(), 1);
        
        // Duplicate should not be added
        node.add_backlink(link);
        assert_eq!(node.backlinks.len(), 1);
    }
    
    #[test]
    fn test_privacy_classification() {
        assert!(PrivacyClassification::Financial.requires_encryption());
        assert!(PrivacyClassification::Sensitive.requires_encryption());
        assert!(!PrivacyClassification::Personal.requires_encryption());
        assert!(!PrivacyClassification::Public.requires_encryption());
    }
    
    #[test]
    fn test_provenance_confidence() {
        let prov = Provenance::new(vec![], "test", 0.96);
        assert!(prov.meets_confidence_threshold(0.95));
        assert!(!prov.meets_confidence_threshold(0.97));
        assert_eq!(prov.confidence_level(), "very_high");
    }
    
    #[test]
    fn test_temporal_fiscal_quarter() {
        use chrono::TimeZone;
        
        // January 2024 with fiscal year starting in April
        let date = Utc.with_ymd_and_hms(2024, 1, 15, 0, 0, 0).unwrap();
        let quarter = TemporalVectors::calculate_fiscal_quarter(date, 4);
        assert_eq!(quarter, "Q4-2023");
        
        // May 2024 with fiscal year starting in April
        let date = Utc.with_ymd_and_hms(2024, 5, 15, 0, 0, 0).unwrap();
        let quarter = TemporalVectors::calculate_fiscal_quarter(date, 4);
        assert_eq!(quarter, "Q1-2024");
    }
    
    #[test]
    fn test_slugify() {
        assert_eq!(slugify("Hello World"), "hello-world");
        assert_eq!(slugify("Test--Entity!!!"), "test-entity");
        assert_eq!(slugify("-Leading Dash"), "leading-dash");
    }
    
    #[test]
    fn test_markdown_parsing() {
        let content = "# Heading\n\nParagraph text.\n\n```rust\ncode\n```";
        let ast = parse_markdown_to_ast(content);
        
        assert!(!ast.is_empty());
        // Should have heading, paragraph, and code block
    }
}
