// SPEC v2.0 SECTION 2.1: Data Model Definitions
// Shared types between frontend and backend

use serde::{Deserialize, Serialize};
use uuid::Uuid;
use chrono::{DateTime, Utc};

// ==================== RAW ARTIFACT ====================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RawArtifact {
    pub uuid: String,
    pub filename: String,
    pub path: String,
    pub captured_at: DateTime<Utc>,
    pub device_id: String,
    pub app_context: Option<AppContext>,
    pub geohash: Option<String>,
    pub metadata: ArtifactMetadata,
    pub status: ArtifactStatus,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppContext {
    pub bundle_id: String,
    pub app_name: String,
    pub window_title: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ArtifactMetadata {
    pub size_bytes: u64,
    pub width: u32,
    pub height: u32,
    pub ocr_text: Option<String>,
    pub embedding_vector: Option<Vec<f32>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ArtifactStatus {
    Pending,
    Compiling,
    Compiled,
    Failed,
    Quarantined,
}

// ==================== WIKI NODE ====================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WikiNode {
    pub id: String,
    pub node_type: NodeType,
    pub title: String,
    pub content: String,
    pub backlinks: Vec<WikiLink>,
    pub forward_links: Vec<WikiLink>,
    pub provenance: Provenance,
    pub temporal_vectors: TemporalVectors,
    pub privacy_classification: PrivacyClassification,
    pub encryption_status: EncryptionStatus,
    pub version: u64,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WikiNodeSummary {
    pub id: String,
    pub node_type: NodeType,
    pub title: String,
    pub excerpt: String,
    pub updated_at: DateTime<Utc>,
    pub backlink_count: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum NodeType {
    Entity,
    Concept,
    Chronology,
    Index,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WikiLink {
    pub source_id: String,
    pub target_id: String,
    pub target_title: String,
    pub context: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Provenance {
    pub source_artifacts: Vec<String>,
    pub compiled_at: DateTime<Utc>,
    pub model_version: String,
    pub confidence_score: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TemporalVectors {
    pub mentioned_dates: Vec<DateTime<Utc>>,
    pub fiscal_quarter: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PrivacyClassification {
    Public,
    Personal,
    Sensitive,
    Financial,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum EncryptionStatus {
    Plaintext,
    Aes256Gcm,
}

// ==================== COMPILATION ====================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompilationResult {
    pub artifact_id: String,
    pub success: bool,
    pub nodes_created: Vec<String>,
    pub nodes_updated: Vec<String>,
    pub error: Option<String>,
    pub duration_ms: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BatchCompilationResult {
    pub processed: usize,
    pub successful: usize,
    pub failed: usize,
    pub results: Vec<CompilationResult>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompilationStatus {
    pub is_compiling: bool,
    pub queue_length: usize,
    pub current_artifact: Option<String>,
    pub progress_percent: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompilationLog {
    pub id: String,
    pub artifact_id: Option<String>,
    pub level: LogLevel,
    pub message: String,
    pub timestamp: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum LogLevel {
    Debug,
    Info,
    Warning,
    Error,
}

// ==================== SEARCH ====================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchResults {
    pub query: String,
    pub artifacts: Vec<RawArtifact>,
    pub wiki_nodes: Vec<WikiNodeSummary>,
    pub total_count: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SemanticSearchResult {
    pub node: WikiNodeSummary,
    pub similarity_score: f32,
}

// ==================== SYNC ====================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SyncStatus {
    pub is_syncing: bool,
    pub last_sync: Option<DateTime<Utc>>,
    pub pending_changes: usize,
    pub connected_devices: Vec<DeviceInfo>,
    pub sync_health: SyncHealth,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeviceInfo {
    pub device_id: String,
    pub device_name: String,
    pub last_seen: DateTime<Utc>,
    pub is_online: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SyncHealth {
    Healthy,
    Degraded,
    Offline,
    Error,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SyncResult {
    pub success: bool,
    pub changes_sent: usize,
    pub changes_received: usize,
    pub conflicts: usize,
    pub error: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PairingInfo {
    pub pairing_code: String,
    pub qr_data: String,
    pub expires_at: DateTime<Utc>,
}

// ==================== SCHEMA ====================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SchemaDefinition {
    pub version: String,
    pub content: String,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SchemaValidationResult {
    pub valid: bool,
    pub errors: Vec<String>,
    pub warnings: Vec<String>,
}

// ==================== SYSTEM ====================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StorageInfo {
    pub raw_storage_bytes: u64,
    pub wiki_storage_bytes: u64,
    pub thumbnail_cache_bytes: u64,
    pub total_bytes: u64,
    pub artifact_count: usize,
    pub node_count: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExportResult {
    pub success: bool,
    pub path: String,
    pub files_exported: usize,
    pub error: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppInfo {
    pub version: String,
    pub name: String,
    pub build_date: String,
}

// ==================== KNOWLEDGE GRAPH ====================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Entity {
    pub id: String,
    pub entity_type: EntityType,
    pub name: String,
    pub description: Option<String>,
    pub attributes: serde_json::Value,
    pub confidence_score: f32,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EntitySummary {
    pub id: String,
    pub entity_type: EntityType,
    pub name: String,
    pub confidence_score: f32,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum EntityType {
    Person,
    Organization,
    Project,
    Technology,
    Concept,
    Decision,
    ActionItem,
    Meeting,
    Document,
    Custom(String),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EntityFilter {
    pub entity_type: Option<EntityType>,
    pub name_contains: Option<String>,
    pub min_confidence: Option<f32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateEntityInput {
    pub entity_type: EntityType,
    pub name: String,
    pub description: Option<String>,
    pub attributes: Option<serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateEntityInput {
    pub name: Option<String>,
    pub description: Option<String>,
    pub attributes: Option<serde_json::Value>,
    pub confidence_score: Option<f32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Relationship {
    pub id: String,
    pub source_id: String,
    pub target_id: String,
    pub relationship_type: String,
    pub properties: Option<serde_json::Value>,
    pub confidence_score: f32,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateRelationshipInput {
    pub source_id: String,
    pub target_id: String,
    pub relationship_type: String,
    pub properties: Option<serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GraphNode {
    pub entity: Entity,
    pub relationships: Vec<GraphEdge>,
    pub children: Vec<GraphNode>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GraphEdge {
    pub relationship: Relationship,
    pub direction: EdgeDirection,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum EdgeDirection {
    Outgoing,
    Incoming,
}

// ==================== VALIDATION ====================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationResult {
    pub entity_id: String,
    pub valid: bool,
    pub contradictions: Vec<Contradiction>,
    pub warnings: Vec<String>,
    pub checked_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Contradiction {
    pub id: String,
    pub entity_id: String,
    pub contradiction_type: String,
    pub description: String,
    pub severity: ContradictionSeverity,
    pub related_entity_ids: Vec<String>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ContradictionSeverity {
    Low,
    Medium,
    High,
    Critical,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConfidenceScore {
    pub entity_id: String,
    pub overall_score: f32,
    pub source_reliability: f32,
    pub cross_reference_score: f32,
    pub temporal_consistency: f32,
    pub calculated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DuplicateCandidate {
    pub primary_id: String,
    pub duplicate_id: String,
    pub similarity_score: f32,
    pub match_reasons: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResolutionInput {
    pub resolution_type: ResolutionType,
    pub note: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ResolutionType {
    Accepted,
    Rejected,
    Merged,
    NeedsReview,
}

// ==================== MEETING EXTRACTION ====================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Meeting {
    pub id: String,
    pub title: String,
    pub occurred_at: DateTime<Utc>,
    pub participants: Vec<String>,
    pub source_artifact_id: String,
    pub extracted_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExtractionResult {
    pub meeting_id: String,
    pub entities_extracted: Vec<EntitySummary>,
    pub decisions_extracted: Vec<Decision>,
    pub action_items_extracted: Vec<ActionItem>,
    pub extraction_confidence: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Decision {
    pub id: String,
    pub meeting_id: String,
    pub description: String,
    pub decided_by: Vec<String>,
    pub related_entities: Vec<String>,
    pub confidence_score: f32,
    pub extracted_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ActionItem {
    pub id: String,
    pub meeting_id: String,
    pub description: String,
    pub assignee: Option<String>,
    pub due_date: Option<DateTime<Utc>>,
    pub status: ActionItemStatus,
    pub related_entities: Vec<String>,
    pub extracted_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ActionItemStatus {
    Pending,
    InProgress,
    Completed,
    Cancelled,
}
