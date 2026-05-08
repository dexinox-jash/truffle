//! Data Export Functionality
//!
//! This module provides the 5-minute complete export capability as specified
//! in Section 1.2 of the Truffle specification ("Exit Capability").
//!
//! ## Export Requirements
//!
//! - **Time Limit**: Complete export within 5 minutes
//! - **No Internet**: Works completely offline
//! - **No Authentication**: No server contact required
//! - **Format**: Plain markdown with YAML frontmatter
//! - **Integrity**: Verifiable export integrity
//!
//! ## Export Format
//!
//! The export produces an Obsidian-compatible vault structure:
//!
//! ```text
//! truffle-export-2024-01-15/
//! ├── .truffle/
//! │   ├── manifest.json          # Export metadata
//! │   └── integrity.sha256       # Integrity hashes
//! ├── entities/
//! │   ├── merchant-name.md
//! │   ├── person-name.md
//! │   └── ...
//! ├── concepts/
//! │   ├── expenses.md
//! │   ├── travel.md
//! │   └── ...
//! ├── chronology/
//! │   ├── 2024-01.md
//! │   ├── 2024-02.md
//! │   └── ...
//! ├── index.md                   # Main index
//! └── raw/                       # Original screenshots (optional)
//!     └── ...
//! ```
//!
//! ## Markdown Format
//!
//! Each wiki node is exported as a markdown file with YAML frontmatter:
//!
//! ```markdown
//! ---
//! id: 550e8400-e29b-41d4-a716-446655440000
//! node_type: entity
//! title: "Merchant Name"
//! created_at: 2024-01-15T10:30:00Z
//! updated_at: 2024-01-15T10:30:00Z
//! source_artifacts:
//!   - 550e8400-e29b-41d4-a716-446655440001
//! confidence_score: 0.95
//! privacy_classification: financial
//! tags:
//!   - receipt
//!   - expense
//! ---
//!
//! # Merchant Name
//!
//! Extracted information from receipt.
//!
//! ## Details
//!
//! - **Total Amount**: $45.67
//! - **Currency**: USD
//! - **Date**: 2024-01-15
//!
//! ## Related
//!
//! - [[expenses]]
//! - [[chronology/2024-01]]
//! ```

use crate::{CryptoError, CryptoResult, sha2_256_hash};
use std::collections::HashMap;
use std::path::{Path, PathBuf};

/// Wiki node structure for export
///
/// This mirrors the WikiNode interface from the specification.
#[derive(Debug, Clone)]
pub struct WikiNode {
    /// Unique identifier
    pub id: String,
    /// Node type (entity, concept, chronology, index)
    pub node_type: NodeType,
    /// Human-readable title
    pub title: String,
    /// Markdown content
    pub content: String,
    /// Backlinks (incoming references)
    pub backlinks: Vec<WikiLink>,
    /// Forward links (outgoing references)
    pub forward_links: Vec<WikiLink>,
    /// Provenance information
    pub provenance: Provenance,
    /// Temporal vectors
    pub temporal_vectors: TemporalVectors,
    /// Privacy classification
    pub privacy_classification: PrivacyClassification,
    /// Tags for organization
    pub tags: Vec<String>,
}

/// Node types
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NodeType {
    /// Named entity (person, place, organization)
    Entity,
    /// Abstract concept
    Concept,
    /// Time-based entry
    Chronology,
    /// Index/navigation page
    Index,
}

impl NodeType {
    /// Get the string representation
    pub fn as_str(&self) -> &'static str {
        match self {
            NodeType::Entity => "entity",
            NodeType::Concept => "concept",
            NodeType::Chronology => "chronology",
            NodeType::Index => "index",
        }
    }

    /// Get the directory name for this node type
    pub fn directory(&self) -> &'static str {
        match self {
            NodeType::Entity => "entities",
            NodeType::Concept => "concepts",
            NodeType::Chronology => "chronology",
            NodeType::Index => ".",
        }
    }
}

/// Wiki link structure
#[derive(Debug, Clone)]
pub struct WikiLink {
    /// Target node ID
    pub target_id: String,
    /// Target title (for display)
    pub target_title: String,
    /// Link context (optional)
    pub context: Option<String>,
}

/// Provenance information
#[derive(Debug, Clone)]
pub struct Provenance {
    /// Source artifact UUIDs
    pub source_artifacts: Vec<String>,
    /// Compilation timestamp
    pub compiled_at: String,
    /// Model version
    pub model_version: String,
    /// Confidence score (0-1)
    pub confidence_score: f32,
}

/// Temporal vectors
#[derive(Debug, Clone, Default)]
pub struct TemporalVectors {
    /// Dates mentioned in the content
    pub mentioned_dates: Vec<String>,
    /// Fiscal quarter (optional)
    pub fiscal_quarter: Option<String>,
}

/// Privacy classification
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PrivacyClassification {
    /// Public information
    Public,
    /// Personal information
    Personal,
    /// Sensitive information
    Sensitive,
    /// Financial information
    Financial,
}

impl PrivacyClassification {
    /// Get the string representation
    pub fn as_str(&self) -> &'static str {
        match self {
            PrivacyClassification::Public => "public",
            PrivacyClassification::Personal => "personal",
            PrivacyClassification::Sensitive => "sensitive",
            PrivacyClassification::Financial => "financial",
        }
    }
}

/// Export configuration
#[derive(Debug, Clone)]
pub struct ExportConfig {
    /// Include raw screenshots
    pub include_raw: bool,
    /// Compress output (zip)
    pub compress: bool,
    /// Include metadata files
    pub include_metadata: bool,
    /// Verify integrity after export
    pub verify_integrity: bool,
    /// Custom export path
    pub export_path: Option<PathBuf>,
}

impl Default for ExportConfig {
    fn default() -> Self {
        Self {
            include_raw: false,
            compress: false,
            include_metadata: true,
            verify_integrity: true,
            export_path: None,
        }
    }
}

/// Export manifest for tracking
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ExportManifest {
    /// Export version
    pub version: String,
    /// Export timestamp
    pub exported_at: String,
    /// Number of nodes exported
    pub node_count: usize,
    /// Number of raw files exported
    pub raw_count: usize,
    /// Total size in bytes
    pub total_size: u64,
    /// Integrity hash of the manifest
    pub manifest_hash: String,
}

/// Export a wiki node to markdown format
///
/// # Arguments
///
/// * `node` - The wiki node to export
///
/// # Returns
///
/// Returns the markdown string with YAML frontmatter.
///
/// # Example
///
/// ```rust
/// use truffle_crypto::export::{WikiNode, NodeType, PrivacyClassification, export_to_markdown};
///
/// let node = WikiNode {
///     id: "test-123".to_string(),
///     node_type: NodeType::Entity,
///     title: "Test Entity".to_string(),
///     content: "# Test\n\nThis is a test.".to_string(),
///     backlinks: vec![],
///     forward_links: vec![],
///     provenance: truffle_crypto::export::Provenance {
///         source_artifacts: vec![],
///         compiled_at: "2024-01-15T10:00:00Z".to_string(),
///         model_version: "gemma-4-v1".to_string(),
///         confidence_score: 0.95,
///     },
///     temporal_vectors: Default::default(),
///     privacy_classification: PrivacyClassification::Personal,
///     tags: vec!["test".to_string()],
/// };
///
/// let markdown = export_to_markdown(&node);
/// assert!(markdown.contains("---"));
/// assert!(markdown.contains("Test Entity"));
/// ```
pub fn export_to_markdown(node: &WikiNode) -> String {
    let mut output = String::new();

    // YAML Frontmatter
    output.push_str("---\n");
    output.push_str(&format!("id: {}\n", node.id));
    output.push_str(&format!("node_type: {}\n", node.node_type.as_str()));
    output.push_str(&format!("title: \"{}\"\n", escape_yaml_string(&node.title)));
    output.push_str(&format!("compiled_at: {}\n", node.provenance.compiled_at));
    output.push_str(&format!("model_version: {}\n", node.provenance.model_version));
    output.push_str(&format!("confidence_score: {:.2}\n", node.provenance.confidence_score));
    output.push_str(&format!("privacy_classification: {}\n", node.privacy_classification.as_str()));

    // Source artifacts
    if !node.provenance.source_artifacts.is_empty() {
        output.push_str("source_artifacts:\n");
        for artifact in &node.provenance.source_artifacts {
            output.push_str(&format!("  - {}\n", artifact));
        }
    }

    // Tags
    if !node.tags.is_empty() {
        output.push_str("tags:\n");
        for tag in &node.tags {
            output.push_str(&format!("  - {}\n", tag));
        }
    }

    // Temporal vectors
    if !node.temporal_vectors.mentioned_dates.is_empty() {
        output.push_str("mentioned_dates:\n");
        for date in &node.temporal_vectors.mentioned_dates {
            output.push_str(&format!("  - {}\n", date));
        }
    }

    if let Some(ref quarter) = node.temporal_vectors.fiscal_quarter {
        output.push_str(&format!("fiscal_quarter: {}\n", quarter));
    }

    // Backlinks
    if !node.backlinks.is_empty() {
        output.push_str("backlinks:\n");
        for link in &node.backlinks {
            output.push_str(&format!("  - \"[[{}]]\"\n", escape_yaml_string(&link.target_title)));
        }
    }

    output.push_str("---\n\n");

    // Content
    output.push_str(&format!("# {}\n\n", node.title));
    output.push_str(&node.content);
    output.push('\n');

    // Forward links section
    if !node.forward_links.is_empty() {
        output.push_str("\n## Related\n\n");
        for link in &node.forward_links {
            output.push_str(&format!("- [[{}]]\n", link.target_title));
        }
    }

    output
}

/// Escape special characters in YAML strings
fn escape_yaml_string(s: &str) -> String {
    s.replace('\\', "\\\\")
        .replace('"', "\\\"")
        .replace('\n', "\\n")
}

/// Create a safe filename from a title
fn sanitize_filename(title: &str) -> String {
    title.to_lowercase()
        .replace(' ', "-")
        .replace(|c: char| !c.is_alphanumeric() && c != '-', "")
        .replace("--", "-")
        .trim_matches('-')
        .to_string()
}

/// Export multiple wiki nodes to markdown
///
/// # Arguments
///
/// * `nodes` - Slice of wiki nodes to export
///
/// # Returns
///
/// Returns a map of filenames to markdown content.
pub fn export_nodes_to_markdown(nodes: &[WikiNode]) -> HashMap<String, String> {
    let mut result = HashMap::new();

    for node in nodes {
        let filename = format!("{}/{}",
            node.node_type.directory(),
            sanitize_filename(&node.title)
        );
        let markdown = export_to_markdown(node);
        result.insert(filename, markdown);
    }

    result
}

/// Generate index.md content
///
/// Creates a navigation index for the exported vault.
pub fn generate_index(nodes: &[WikiNode]) -> String {
    let mut output = String::new();

    output.push_str("---\n");
    output.push_str("id: index\n");
    output.push_str("node_type: index\n");
    output.push_str("title: \"Truffle Knowledge Base\"\n");
    output.push_str(&format!("generated_at: {}\n", chrono::Utc::now().to_rfc3339()));
    output.push_str(&format!("total_nodes: {}\n", nodes.len()));
    output.push_str("---\n\n");

    output.push_str("# Truffle Knowledge Base\n\n");
    output.push_str("This is an exported snapshot of your Truffle knowledge base.\n\n");

    // Group by node type
    let entities: Vec<_> = nodes.iter()
        .filter(|n| matches!(n.node_type, NodeType::Entity))
        .collect();
    let concepts: Vec<_> = nodes.iter()
        .filter(|n| matches!(n.node_type, NodeType::Concept))
        .collect();
    let chronology: Vec<_> = nodes.iter()
        .filter(|n| matches!(n.node_type, NodeType::Chronology))
        .collect();

    if !entities.is_empty() {
        output.push_str("## Entities\n\n");
        for node in entities {
            output.push_str(&format!("- [[{}|{}]]\n",
                sanitize_filename(&node.title),
                node.title
            ));
        }
        output.push('\n');
    }

    if !concepts.is_empty() {
        output.push_str("## Concepts\n\n");
        for node in concepts {
            output.push_str(&format!("- [[{}|{}]]\n",
                sanitize_filename(&node.title),
                node.title
            ));
        }
        output.push('\n');
    }

    if !chronology.is_empty() {
        output.push_str("## Chronology\n\n");
        for node in chronology {
            output.push_str(&format!("- [[{}|{}]]\n",
                sanitize_filename(&node.title),
                node.title
            ));
        }
        output.push('\n');
    }

    output.push_str("---\n\n");
    output.push_str("*Exported from Truffle - Your Visual Memory, Compiled.*\n");

    output
}

/// Compute integrity hash for a file
///
/// Returns the SHA-256 hash as a hex string.
pub fn compute_file_hash(content: &[u8]) -> String {
    let hash = sha2_256_hash(content);
    hex::encode(hash)
}

/// Verify export integrity
///
/// Checks that all files in the export directory have valid integrity hashes.
///
/// # Arguments
///
/// * `export_path` - Path to the export directory
///
/// # Returns
///
/// Returns true if all files pass integrity verification.
pub fn verify_export_integrity(export_path: &Path) -> bool {
    let integrity_file = export_path.join(".truffle/integrity.sha256");

    if !integrity_file.exists() {
        // No integrity file, can't verify
        return false;
    }

    // Read integrity file
    let integrity_content = match std::fs::read_to_string(&integrity_file) {
        Ok(content) => content,
        Err(_) => return false,
    };

    // Parse and verify each entry
    for line in integrity_content.lines() {
        let parts: Vec<_> = line.split_whitespace().collect();
        if parts.len() != 2 {
            continue;
        }

        let expected_hash = parts[0];
        let file_path = parts[1];

        let full_path = export_path.join(file_path.trim_start_matches("./"));

        // Read file and compute hash
        let content = match std::fs::read(&full_path) {
            Ok(content) => content,
            Err(_) => return false,
        };

        let actual_hash = compute_file_hash(&content);

        if actual_hash != expected_hash {
            return false;
        }
    }

    true
}

/// Create integrity file for export
///
/// Generates a SHA-256 integrity file for all exported content.
pub fn create_integrity_file(files: &HashMap<String, String>) -> String {
    let mut output = String::new();

    for (path, content) in files {
        let hash = compute_file_hash(content.as_bytes());
        output.push_str(&format!("{}  ./{}.md\n", hash, path));
    }

    output
}

/// Export result structure
#[derive(Debug, Clone)]
pub struct ExportResult {
    /// Path to the exported directory
    pub export_path: PathBuf,
    /// Number of nodes exported
    pub node_count: usize,
    /// Total size in bytes
    pub total_size: u64,
    /// Integrity verified
    pub integrity_verified: bool,
}

/// Complete export function (simulated)
///
/// In a real implementation, this would write files to disk.
/// This version returns the content that would be written.
///
/// # Arguments
///
/// * `nodes` - Wiki nodes to export
/// * `config` - Export configuration
///
/// # Returns
///
/// Returns a map of file paths to content.
pub fn perform_export(
    nodes: &[WikiNode],
    config: &ExportConfig,
) -> CryptoResult<HashMap<String, String>> {
    let mut files = HashMap::new();

    // Export each node
    let node_files = export_nodes_to_markdown(nodes);
    files.extend(node_files);

    // Generate index
    let index = generate_index(nodes);
    files.insert("index.md".to_string(), index);

    // Add metadata if requested
    if config.include_metadata {
        let manifest = ExportManifest {
            version: "1.0".to_string(),
            exported_at: chrono::Utc::now().to_rfc3339(),
            node_count: nodes.len(),
            raw_count: 0,
            total_size: files.values().map(|s| s.len() as u64).sum(),
            manifest_hash: String::new(), // Would be computed after serialization
        };

        let manifest_json = serde_json::to_string_pretty(&manifest)
            .map_err(|e| CryptoError::InternalError(format!("JSON serialization failed: {}", e)))?;

        files.insert(".truffle/manifest.json".to_string(), manifest_json);

        // Create integrity file
        let integrity = create_integrity_file(&files);
        files.insert(".truffle/integrity.sha256".to_string(), integrity);
    }

    Ok(files)
}

/// Estimate export time
///
/// Returns estimated time in seconds for the given number of nodes.
pub fn estimate_export_time(node_count: usize) -> u64 {
    // Conservative estimate: 100 nodes per second
    let base_time = node_count as u64 / 100;
    base_time.max(1) // At least 1 second
}

/// Check if export can complete within time limit
///
/// # Arguments
///
/// * `node_count` - Number of nodes to export
/// * `time_limit_seconds` - Time limit in seconds
///
/// # Returns
///
/// Returns true if export can complete within the time limit.
pub fn can_export_within_time_limit(node_count: usize, time_limit_seconds: u64) -> bool {
    estimate_export_time(node_count) <= time_limit_seconds
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_node() -> WikiNode {
        WikiNode {
            id: "550e8400-e29b-41d4-a716-446655440000".to_string(),
            node_type: NodeType::Entity,
            title: "Test Merchant".to_string(),
            content: "# Test Merchant\n\nThis is a test entity.".to_string(),
            backlinks: vec![],
            forward_links: vec![WikiLink {
                target_id: "concept-1".to_string(),
                target_title: "expenses".to_string(),
                context: None,
            }],
            provenance: Provenance {
                source_artifacts: vec!["artifact-1".to_string()],
                compiled_at: "2024-01-15T10:30:00Z".to_string(),
                model_version: "gemma-4-v1".to_string(),
                confidence_score: 0.95,
            },
            temporal_vectors: TemporalVectors {
                mentioned_dates: vec!["2024-01-15".to_string()],
                fiscal_quarter: Some("Q1-2024".to_string()),
            },
            privacy_classification: PrivacyClassification::Financial,
            tags: vec!["receipt".to_string(), "expense".to_string()],
        }
    }

    #[test]
    fn test_export_to_markdown() {
        let node = create_test_node();
        let markdown = export_to_markdown(&node);

        // Should have YAML frontmatter
        assert!(markdown.starts_with("---"));
        assert!(markdown.contains("id: "));
        assert!(markdown.contains("node_type: entity"));
        assert!(markdown.contains("title: \"Test Merchant\""));

        // Should have content
        assert!(markdown.contains("# Test Merchant"));

        // Should have related links
        assert!(markdown.contains("## Related"));
        assert!(markdown.contains("[[expenses]]"));
    }

    #[test]
    fn test_sanitize_filename() {
        assert_eq!(sanitize_filename("Hello World"), "hello-world");
        assert_eq!(sanitize_filename("Test & Company"), "test-company");
        assert_eq!(sanitize_filename("File--Name"), "file-name");
    }

    #[test]
    fn test_compute_file_hash() {
        let content1 = b"test content";
        let content2 = b"test content";
        let content3 = b"different content";

        let hash1 = compute_file_hash(content1);
        let hash2 = compute_file_hash(content2);
        let hash3 = compute_file_hash(content3);

        assert_eq!(hash1, hash2);
        assert_ne!(hash1, hash3);
        assert_eq!(hash1.len(), 64); // SHA-256 hex = 64 chars
    }

    #[test]
    fn test_create_integrity_file() {
        let mut files = HashMap::new();
        files.insert("test/file".to_string(), "test content".to_string());

        let integrity = create_integrity_file(&files);

        assert!(integrity.contains("./test/file.md"));
        assert!(integrity.len() > 64); // Should have hash + path
    }

    #[test]
    fn test_generate_index() {
        let nodes = vec![create_test_node()];
        let index = generate_index(&nodes);

        assert!(index.contains("# Truffle Knowledge Base"));
        assert!(index.contains("total_nodes: 1"));
        assert!(index.contains("## Entities"));
    }

    #[test]
    fn test_estimate_export_time() {
        // 100 nodes should take ~1 second
        assert_eq!(estimate_export_time(100), 1);

        // 500 nodes should take ~5 seconds
        assert_eq!(estimate_export_time(500), 5);

        // Empty should still return 1 second minimum
        assert_eq!(estimate_export_time(0), 1);
    }

    #[test]
    fn test_can_export_within_time_limit() {
        // 100 nodes in 5 minutes (300 seconds) - should succeed
        assert!(can_export_within_time_limit(100, 300));

        // 1 million nodes in 5 minutes - should fail
        assert!(!can_export_within_time_limit(1_000_000, 300));
    }

    #[test]
    fn test_perform_export() {
        let nodes = vec![create_test_node()];
        let config = ExportConfig::default();

        let files = perform_export(&nodes, &config).unwrap();

        // Should have entity file
        assert!(files.contains_key("entities/test-merchant"));

        // Should have index
        assert!(files.contains_key("index.md"));

        // Should have metadata
        assert!(files.contains_key(".truffle/manifest.json"));
        assert!(files.contains_key(".truffle/integrity.sha256"));
    }

    #[test]
    fn test_node_type_directory() {
        assert_eq!(NodeType::Entity.directory(), "entities");
        assert_eq!(NodeType::Concept.directory(), "concepts");
        assert_eq!(NodeType::Chronology.directory(), "chronology");
        assert_eq!(NodeType::Index.directory(), ".");
    }

    #[test]
    fn test_privacy_classification_as_str() {
        assert_eq!(PrivacyClassification::Public.as_str(), "public");
        assert_eq!(PrivacyClassification::Personal.as_str(), "personal");
        assert_eq!(PrivacyClassification::Sensitive.as_str(), "sensitive");
        assert_eq!(PrivacyClassification::Financial.as_str(), "financial");
    }
}
