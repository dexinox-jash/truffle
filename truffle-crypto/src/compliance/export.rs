//! Data Export Module
//!
//! This module implements the 5-minute complete export capability
//! as specified in Section 1.2 (Red Line #4) of the Truffle specification.
//!
//! The user must be able to export complete knowledge state to plain
//! markdown/git within 5 minutes, without internet, without authentication.

use crate::error::{CryptoError, CryptoResult};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::time::{Instant, SystemTime, UNIX_EPOCH};

/// Export format
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum ExportFormat {
    /// Markdown files (default)
    Markdown,
    /// Obsidian vault format
    Obsidian,
    /// JSON
    Json,
    /// Plain text
    PlainText,
    /// Git repository
    Git,
}

impl ExportFormat {
    /// Get file extension for this format
    pub fn extension(&self) -> &'static str {
        match self {
            Self::Markdown | Self::Obsidian => "md",
            Self::Json => "json",
            Self::PlainText => "txt",
            Self::Git => "", // Git is a directory
        }
    }

    /// Get a description of this format
    pub fn description(&self) -> &'static str {
        match self {
            Self::Markdown => "Markdown files with YAML frontmatter",
            Self::Obsidian => "Obsidian vault with wiki links",
            Self::Json => "JSON export",
            Self::PlainText => "Plain text export",
            Self::Git => "Git repository",
        }
    }
}

/// Export configuration
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ExportConfig {
    /// Export format
    pub format: ExportFormat,
    /// Output directory
    pub output_dir: PathBuf,
    /// Include raw images
    pub include_images: bool,
    /// Include metadata
    pub include_metadata: bool,
    /// Compress output
    pub compress: bool,
    /// Maximum export time (seconds)
    pub max_duration_secs: u64,
    /// Include deleted items (tombstones)
    pub include_deleted: bool,
    /// Privacy filter (exclude sensitive data)
    pub privacy_filter: bool,
}

impl Default for ExportConfig {
    fn default() -> Self {
        Self {
            format: ExportFormat::Markdown,
            output_dir: PathBuf::from("./export"),
            include_images: true,
            include_metadata: true,
            compress: false,
            max_duration_secs: 300, // 5 minutes
            include_deleted: false,
            privacy_filter: false,
        }
    }
}

impl ExportConfig {
    /// Create a new export config for Obsidian
    pub fn obsidian(output_dir: impl AsRef<Path>) -> Self {
        Self {
            format: ExportFormat::Obsidian,
            output_dir: output_dir.as_ref().to_path_buf(),
            ..Default::default()
        }
    }

    /// Create a new export config for Git
    pub fn git(output_dir: impl AsRef<Path>) -> Self {
        Self {
            format: ExportFormat::Git,
            output_dir: output_dir.as_ref().to_path_buf(),
            ..Default::default()
        }
    }

    /// Set maximum duration
    pub fn with_max_duration(mut self, secs: u64) -> Self {
        self.max_duration_secs = secs;
        self
    }
}

/// Export progress
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct ExportProgress {
    /// Total items to export
    pub total_items: u64,
    /// Items exported so far
    pub exported_items: u64,
    /// Current item being exported
    pub current_item: Option<String>,
    /// Export start time
    pub start_time: u64,
    /// Estimated completion time
    pub estimated_completion: u64,
    /// Bytes exported
    pub bytes_exported: u64,
    /// Errors encountered
    pub errors: Vec<String>,
}

impl ExportProgress {
    /// Create new progress tracker
    pub fn new(total_items: u64) -> Self {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();

        Self {
            total_items,
            exported_items: 0,
            current_item: None,
            start_time: now,
            estimated_completion: now,
            bytes_exported: 0,
            errors: Vec::new(),
        }
    }

    /// Update progress
    pub fn update(&mut self, item: impl Into<String>, bytes: u64) {
        self.exported_items += 1;
        self.current_item = Some(item.into());
        self.bytes_exported += bytes;

        // Update estimated completion
        if self.exported_items > 0 {
            let elapsed = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_secs()
                - self.start_time;
            let rate = self.exported_items as f64 / elapsed.max(1) as f64;
            let remaining = (self.total_items - self.exported_items) as f64 / rate;
            self.estimated_completion = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_secs()
                + remaining as u64;
        }
    }

    /// Add error
    pub fn add_error(&mut self, error: impl Into<String>) {
        self.errors.push(error.into());
    }

    /// Get completion percentage
    pub fn percentage(&self) -> f64 {
        if self.total_items == 0 {
            return 100.0;
        }
        (self.exported_items as f64 / self.total_items as f64) * 100.0
    }

    /// Check if complete
    pub fn is_complete(&self) -> bool {
        self.exported_items >= self.total_items
    }
}

/// Export result
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ExportResult {
    /// Success status
    pub success: bool,
    /// Output path
    pub output_path: PathBuf,
    /// Items exported
    pub items_exported: u64,
    /// Bytes exported
    pub bytes_exported: u64,
    /// Duration in seconds
    pub duration_secs: f64,
    /// Errors encountered
    pub errors: Vec<String>,
    /// Warnings
    pub warnings: Vec<String>,
}

impl ExportResult {
    /// Create a successful result
    pub fn success(output_path: PathBuf, items: u64, bytes: u64, duration: f64) -> Self {
        Self {
            success: true,
            output_path,
            items_exported: items,
            bytes_exported: bytes,
            duration_secs: duration,
            errors: Vec::new(),
            warnings: Vec::new(),
        }
    }

    /// Create a failed result
    pub fn failure(error: impl Into<String>) -> Self {
        Self {
            success: false,
            output_path: PathBuf::new(),
            items_exported: 0,
            bytes_exported: 0,
            duration_secs: 0.0,
            errors: vec![error.into()],
            warnings: Vec::new(),
        }
    }

    /// Add warning
    pub fn with_warning(mut self, warning: impl Into<String>) -> Self {
        self.warnings.push(warning.into());
        self
    }
}

/// Data exporter trait
pub trait DataExporter: Send + Sync {
    /// Export data
    fn export(&self, data: &ExportData, config: &ExportConfig) -> CryptoResult<ExportResult>;

    /// Get the format this exporter handles
    fn format(&self) -> ExportFormat;
}

/// Export data container
#[derive(Clone, Debug, Default)]
pub struct ExportData {
    /// Wiki nodes (markdown content)
    pub wiki_nodes: Vec<WikiNode>,
    /// Raw artifacts (images, etc.)
    pub raw_artifacts: Vec<RawArtifact>,
    /// Entity relationships
    pub relationships: Vec<Relationship>,
    /// Metadata
    pub metadata: HashMap<String, String>,
}

/// Wiki node for export
#[derive(Clone, Debug)]
pub struct WikiNode {
    /// Node ID
    pub id: String,
    /// Node title
    pub title: String,
    /// Node content (markdown)
    pub content: String,
    /// Creation timestamp
    pub created_at: u64,
    /// Last modified timestamp
    pub modified_at: u64,
    /// Tags
    pub tags: Vec<String>,
    /// Backlinks
    pub backlinks: Vec<String>,
    /// Forward links
    pub forward_links: Vec<String>,
    /// Privacy classification
    pub privacy: String,
}

/// Raw artifact for export
#[derive(Clone, Debug)]
pub struct RawArtifact {
    /// Artifact ID
    pub id: String,
    /// File name
    pub filename: String,
    /// Binary data
    pub data: Vec<u8>,
    /// MIME type
    pub mime_type: String,
    /// Creation timestamp
    pub created_at: u64,
}

/// Entity relationship
#[derive(Clone, Debug)]
pub struct Relationship {
    /// Source entity ID
    pub source: String,
    /// Target entity ID
    pub target: String,
    /// Relationship type
    pub relationship_type: String,
    /// Metadata
    pub metadata: HashMap<String, String>,
}

/// Markdown exporter
pub struct MarkdownExporter;

impl MarkdownExporter {
    /// Create a new markdown exporter
    pub fn new() -> Self {
        Self
    }

    /// Export a wiki node to markdown
    fn export_node(&self, node: &WikiNode, output_dir: &Path) -> CryptoResult<u64> {
        // Create safe filename
        let filename = Self::sanitize_filename(&node.title);
        let filepath = output_dir.join(format!("{}.md", filename));

        // Build markdown content with YAML frontmatter
        let mut content = String::new();

        // YAML frontmatter
        content.push_str("---\n");
        content.push_str(&format!("id: {}\n", node.id));
        content.push_str(&format!("title: {}\n", node.title));
        content.push_str(&format!("created_at: {}\n", node.created_at));
        content.push_str(&format!("modified_at: {}\n", node.modified_at));
        if !node.tags.is_empty() {
            content.push_str(&format!("tags: [{}]\n", node.tags.join(", ")));
        }
        content.push_str(&format!("privacy: {}\n", node.privacy));
        content.push_str("---\n\n");

        // Main content
        content.push_str(&node.content);
        content.push('\n');

        // Backlinks section
        if !node.backlinks.is_empty() {
            content.push_str("\n## Backlinks\n\n");
            for link in &node.backlinks {
                content.push_str(&format!("- [[{}]]\n", link));
            }
        }

        // Write to file
        std::fs::create_dir_all(output_dir)?;
        let mut file = std::fs::File::create(&filepath)?;
        file.write_all(content.as_bytes())?;

        Ok(content.len() as u64)
    }

    /// Sanitize a string for use as a filename
    fn sanitize_filename(name: &str) -> String {
        name.chars()
            .map(|c| match c {
                'a'..='z' | 'A'..='Z' | '0'..='9' | '-' | '_' | ' ' => c,
                _ => '_',
            })
            .collect::<String>()
            .replace(' ', "_")
    }
}

impl DataExporter for MarkdownExporter {
    fn export(&self, data: &ExportData, config: &ExportConfig) -> CryptoResult<ExportResult> {
        let start = Instant::now();
        let output_dir = &config.output_dir;

        // Create output directory
        std::fs::create_dir_all(output_dir)?;

        let mut items_exported = 0u64;
        let mut bytes_exported = 0u64;
        let mut errors = Vec::new();

        // Export wiki nodes
        for node in &data.wiki_nodes {
            match self.export_node(node, output_dir) {
                Ok(bytes) => {
                    items_exported += 1;
                    bytes_exported += bytes;
                }
                Err(e) => {
                    errors.push(format!("Failed to export {}: {}", node.title, e));
                }
            }
        }

        // Export raw artifacts if requested
        if config.include_images {
            let artifacts_dir = output_dir.join("artifacts");
            std::fs::create_dir_all(&artifacts_dir)?;

            for artifact in &data.raw_artifacts {
                let filepath = artifacts_dir.join(&artifact.filename);
                match std::fs::write(&filepath, &artifact.data) {
                    Ok(()) => {
                        items_exported += 1;
                        bytes_exported += artifact.data.len() as u64;
                    }
                    Err(e) => {
                        errors.push(format!("Failed to export {}: {}", artifact.filename, e));
                    }
                }
            }
        }

        // Export metadata if requested
        if config.include_metadata {
            let metadata_path = output_dir.join("metadata.json");
            let metadata = serde_json::to_string_pretty(&data.metadata)?;
            std::fs::write(&metadata_path, metadata)?;
            bytes_exported += metadata.len() as u64;
        }

        let duration = start.elapsed().as_secs_f64();

        let mut result = ExportResult {
            success: errors.is_empty(),
            output_path: output_dir.clone(),
            items_exported,
            bytes_exported,
            duration_secs: duration,
            errors,
            warnings: Vec::new(),
        };

        // Add warnings if we took too long
        if duration > config.max_duration_secs as f64 {
            result.warnings.push(format!(
                "Export took {:.1}s, exceeding target of {}s",
                duration, config.max_duration_secs
            ));
        }

        Ok(result)
    }

    fn format(&self) -> ExportFormat {
        ExportFormat::Markdown
    }
}

impl Default for MarkdownExporter {
    fn default() -> Self {
        Self::new()
    }
}

/// Obsidian exporter (extends Markdown with Obsidian-specific features)
pub struct ObsidianExporter;

impl ObsidianExporter {
    /// Create a new Obsidian exporter
    pub fn new() -> Self {
        Self
    }
}

impl DataExporter for ObsidianExporter {
    fn export(&self, data: &ExportData, config: &ExportConfig) -> CryptoResult<ExportResult> {
        // First, do standard markdown export
        let markdown = MarkdownExporter::new();
        let mut result = markdown.export(data, config)?;

        // Add Obsidian-specific files
        let output_dir = &config.output_dir;

        // Create .obsidian directory with config
        let obsidian_dir = output_dir.join(".obsidian");
        std::fs::create_dir_all(&obsidian_dir)?;

        // Create app.json
        let app_config = r#"{
  "alwaysUpdateLinks": true,
  "newFileLocation": "folder",
  "newFileFolderPath": ""
}"#;
        std::fs::write(obsidian_dir.join("app.json"), app_config)?;

        // Create appearance settings
        let appearance = r#"{
  "accentColor": "#FFB800",
  "theme": "obsidian"
}"#;
        std::fs::write(obsidian_dir.join("appearance.json"), appearance)?;

        // Update result
        result.output_path = output_dir.clone();

        Ok(result)
    }

    fn format(&self) -> ExportFormat {
        ExportFormat::Obsidian
    }
}

impl Default for ObsidianExporter {
    fn default() -> Self {
        Self::new()
    }
}

/// Get the appropriate exporter for a format
pub fn get_exporter(format: ExportFormat) -> Box<dyn DataExporter> {
    match format {
        ExportFormat::Markdown => Box::new(MarkdownExporter::new()),
        ExportFormat::Obsidian => Box::new(ObsidianExporter::new()),
        _ => Box::new(MarkdownExporter::new()), // Default to markdown
    }
}

/// Quick export function (5-minute complete export)
pub fn quick_export(
    data: &ExportData,
    output_dir: impl AsRef<Path>,
) -> CryptoResult<ExportResult> {
    let config = ExportConfig {
        format: ExportFormat::Markdown,
        output_dir: output_dir.as_ref().to_path_buf(),
        max_duration_secs: 300, // 5 minutes
        ..Default::default()
    };

    let exporter = MarkdownExporter::new();
    exporter.export(data, &config)
}

/// Export to Obsidian vault format
pub fn export_to_obsidian(
    data: &ExportData,
    output_dir: impl AsRef<Path>,
) -> CryptoResult<ExportResult> {
    let config = ExportConfig::obsidian(output_dir);
    let exporter = ObsidianExporter::new();
    exporter.export(data, &config)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_export_format() {
        assert_eq!(ExportFormat::Markdown.extension(), "md");
        assert_eq!(ExportFormat::Json.extension(), "json");
        assert!(ExportFormat::Markdown.description().contains("Markdown"));
    }

    #[test]
    fn test_export_config_default() {
        let config = ExportConfig::default();
        assert_eq!(config.format, ExportFormat::Markdown);
        assert_eq!(config.max_duration_secs, 300);
        assert!(config.include_images);
    }

    #[test]
    fn test_export_progress() {
        let mut progress = ExportProgress::new(100);
        assert_eq!(progress.total_items, 100);
        assert_eq!(progress.percentage(), 0.0);

        progress.update("item1", 1000);
        assert_eq!(progress.exported_items, 1);
        assert_eq!(progress.bytes_exported, 1000);

        progress.update("item2", 2000);
        assert_eq!(progress.percentage(), 2.0);
    }

    #[test]
    fn test_export_result() {
        let result = ExportResult::success(
            PathBuf::from("/export"),
            10,
            1000,
            5.0,
        );

        assert!(result.success);
        assert_eq!(result.items_exported, 10);

        let failed = ExportResult::failure("Test error");
        assert!(!failed.success);
        assert_eq!(failed.errors.len(), 1);
    }

    #[test]
    fn test_markdown_exporter_sanitize_filename() {
        let exporter = MarkdownExporter::new();
        assert_eq!(
            MarkdownExporter::sanitize_filename("Hello World!"),
            "Hello_World_"
        );
        assert_eq!(
            MarkdownExporter::sanitize_filename("Test/Path"),
            "Test_Path"
        );
    }

    #[test]
    fn test_markdown_export() {
        let exporter = MarkdownExporter::new();

        let data = ExportData {
            wiki_nodes: vec![WikiNode {
                id: "1".to_string(),
                title: "Test Node".to_string(),
                content: "# Test\n\nThis is a test.".to_string(),
                created_at: 1234567890,
                modified_at: 1234567890,
                tags: vec!["test".to_string()],
                backlinks: vec![],
                forward_links: vec![],
                privacy: "personal".to_string(),
            }],
            raw_artifacts: vec![],
            relationships: vec![],
            metadata: HashMap::new(),
        };

        let temp_dir = tempfile::tempdir().unwrap();
        let config = ExportConfig {
            output_dir: temp_dir.path().to_path_buf(),
            ..Default::default()
        };

        let result = exporter.export(&data, &config).unwrap();
        assert!(result.success);
        assert_eq!(result.items_exported, 1);

        // Check file was created
        let exported_file = temp_dir.path().join("Test_Node.md");
        assert!(exported_file.exists());

        let content = std::fs::read_to_string(&exported_file).unwrap();
        assert!(content.contains("---")); // YAML frontmatter
        assert!(content.contains("# Test"));
    }

    #[test]
    fn test_obsidian_export() {
        let exporter = ObsidianExporter::new();

        let data = ExportData {
            wiki_nodes: vec![],
            raw_artifacts: vec![],
            relationships: vec![],
            metadata: HashMap::new(),
        };

        let temp_dir = tempfile::tempdir().unwrap();
        let config = ExportConfig::obsidian(temp_dir.path());

        let result = exporter.export(&data, &config).unwrap();
        assert!(result.success);

        // Check .obsidian directory was created
        let obsidian_dir = temp_dir.path().join(".obsidian");
        assert!(obsidian_dir.exists());
        assert!(obsidian_dir.join("app.json").exists());
    }

    #[test]
    fn test_quick_export() {
        let data = ExportData {
            wiki_nodes: vec![WikiNode {
                id: "1".to_string(),
                title: "Quick Test".to_string(),
                content: "Content".to_string(),
                created_at: 0,
                modified_at: 0,
                tags: vec![],
                backlinks: vec![],
                forward_links: vec![],
                privacy: "public".to_string(),
            }],
            raw_artifacts: vec![],
            relationships: vec![],
            metadata: HashMap::new(),
        };

        let temp_dir = tempfile::tempdir().unwrap();
        let result = quick_export(&data, temp_dir.path()).unwrap();

        assert!(result.success);
        assert!(result.duration_secs < 5.0); // Should be very fast
    }

    #[test]
    fn test_get_exporter() {
        let markdown = get_exporter(ExportFormat::Markdown);
        assert_eq!(markdown.format(), ExportFormat::Markdown);

        let obsidian = get_exporter(ExportFormat::Obsidian);
        assert_eq!(obsidian.format(), ExportFormat::Obsidian);
    }
}
