# Project Truffle: Interface Specifications

> **Classification:** AAA Commercial SaaS | Zero-Knowledge Infrastructure  
> **Version:** 2.0  
> **Last Updated:** 2024  

## Table of Contents

1. [Rust Core ↔ Tauri Frontend Interface](#1-rust-core--tauri-frontend-interface)
2. [Desktop ↔ Mobile Sync Interface](#2-desktop--mobile-sync-interface)
3. [Device ↔ Relay Server Interface](#3-device--relay-server-interface)
4. [AI Pipeline Interfaces](#4-ai-pipeline-interfaces)
5. [Storage Layer Interfaces](#5-storage-layer-interfaces)
6. [Export Interfaces](#6-export-interfaces)

---

## 1. Rust Core ↔ Tauri Frontend Interface

### Command Pattern

All interactions between the Tauri frontend and Rust core use Tauri's command system with type-safe IPC.

```typescript
// truffle-desktop/src/types/tauri.d.ts

// Base response wrapper
interface TauriResponse<T> {
  success: boolean;
  data?: T;
  error?: {
    code: string;
    message: string;
    details?: Record<string, unknown>;
  };
}

// Error codes
enum ErrorCode {
  NOT_FOUND = 'NOT_FOUND',
  VALIDATION_ERROR = 'VALIDATION_ERROR',
  COMPILATION_ERROR = 'COMPILATION_ERROR',
  SYNC_ERROR = 'SYNC_ERROR',
  CRYPTO_ERROR = 'CRYPTO_ERROR',
  IO_ERROR = 'IO_ERROR',
  PERMISSION_DENIED = 'PERMISSION_DENIED',
}
```

### Artifact Commands

```rust
// truffle-desktop/src-tauri/src/commands/artifact.rs

use serde::{Deserialize, Serialize};
use ts_rs::TS;

/// Request: List artifacts with optional filters
#[derive(Debug, Deserialize, TS)]
#[ts(export)]
pub struct ListArtifactsRequest {
    pub status: Option<ArtifactStatus>,
    pub limit: Option<usize>,
    pub offset: Option<usize>,
    pub search_query: Option<String>,
}

/// Response: Artifact list
#[derive(Debug, Serialize, TS)]
#[ts(export)]
pub struct ListArtifactsResponse {
    pub artifacts: Vec<ArtifactDto>,
    pub total: usize,
    pub has_more: bool,
}

/// DTO: Artifact for frontend consumption
#[derive(Debug, Serialize, TS)]
#[ts(export)]
pub struct ArtifactDto {
    pub id: String,
    pub filename: String,
    pub thumbnail_path: String,
    pub captured_at: String, // ISO8601
    pub status: ArtifactStatus,
    pub ocr_text: Option<String>,
    pub metadata: ArtifactMetadataDto,
}

/// Tauri command: List artifacts
#[tauri::command]
pub async fn list_artifacts(
    state: State<'_, AppState>,
    request: ListArtifactsRequest,
) -> Result<ListArtifactsResponse, TruffleError> {
    // Implementation
}

/// Tauri command: Import screenshot from path
#[tauri::command]
pub async fn import_screenshot(
    state: State<'_, AppState>,
    source_path: String,
) -> Result<ArtifactDto, TruffleError> {
    // Implementation
}

/// Tauri command: Delete artifact
#[tauri::command]
pub async fn delete_artifact(
    state: State<'_, AppState>,
    artifact_id: String,
) -> Result<(), TruffleError> {
    // Implementation
}
```

### Wiki Commands

```rust
// truffle-desktop/src-tauri/src/commands/wiki.rs

/// Request: Create or update wiki node
#[derive(Debug, Deserialize, TS)]
#[ts(export)]
pub struct SaveNodeRequest {
    pub id: Option<String>, // None for create
    pub title: String,
    pub content: String, // Markdown
    pub node_type: NodeType,
    pub privacy: PrivacyClassification,
}

/// Response: Saved node
#[derive(Debug, Serialize, TS)]
#[ts(export)]
pub struct NodeDto {
    pub id: String,
    pub title: String,
    pub content: String,
    pub node_type: NodeType,
    pub backlinks: Vec<BacklinkDto>,
    pub forward_links: Vec<WikiLinkDto>,
    pub provenance: ProvenanceDto,
    pub privacy: PrivacyClassification,
    pub updated_at: String,
}

/// Tauri command: Get node by ID
#[tauri::command]
pub async fn get_node(
    state: State<'_, AppState>,
    node_id: String,
) -> Result<NodeDto, TruffleError> {
    // Implementation
}

/// Tauri command: Save node
#[tauri::command]
pub async fn save_node(
    state: State<'_, AppState>,
    request: SaveNodeRequest,
) -> Result<NodeDto, TruffleError> {
    // Implementation
}

/// Tauri command: Get backlinks
#[tauri::command]
pub async fn get_backlinks(
    state: State<'_, AppState>,
    node_id: String,
) -> Result<Vec<BacklinkDto>, TruffleError> {
    // Implementation
}

/// Tauri command: Search wiki
#[tauri::command]
pub async fn search_wiki(
    state: State<'_, AppState>,
    query: String,
    search_type: SearchType, // FullText | Semantic | Both
) -> Result<SearchResultsDto, TruffleError> {
    // Implementation
}
```

### Compilation Commands

```rust
// truffle-desktop/src-tauri/src/commands/compilation.rs

/// Request: Compile artifacts
#[derive(Debug, Deserialize, TS)]
#[ts(export)]
pub struct CompileRequest {
    pub artifact_ids: Vec<String>,
    pub priority: CompilationPriority, // P0 | P1 | P2
}

/// Response: Compilation status
#[derive(Debug, Serialize, TS)]
#[ts(export)]
pub struct CompilationStatusDto {
    pub job_id: String,
    pub status: JobStatus, // Pending | Running | Completed | Failed
    pub progress: f32, // 0.0 - 1.0
    pub processed: usize,
    pub total: usize,
    pub current_artifact: Option<String>,
    pub logs: Vec<CompilationLogDto>,
    pub errors: Vec<CompilationErrorDto>,
}

/// Tauri command: Start compilation
#[tauri::command]
pub async fn compile_artifacts(
    state: State<'_, AppState>,
    request: CompileRequest,
) -> Result<CompilationStatusDto, TruffleError> {
    // Implementation
}

/// Tauri command: Get compilation status
#[tauri::command]
pub async fn get_compilation_status(
    state: State<'_, AppState>,
    job_id: String,
) -> Result<CompilationStatusDto, TruffleError> {
    // Implementation
}

/// Tauri command: Cancel compilation
#[tauri::command]
pub async fn cancel_compilation(
    state: State<'_, AppState>,
    job_id: String,
) -> Result<(), TruffleError> {
    // Implementation
}
```

### Sync Commands

```rust
// truffle-desktop/src-tauri/src/commands/sync.rs

/// Response: Sync status
#[derive(Debug, Serialize, TS)]
#[ts(export)]
pub struct SyncStatusDto {
    pub enabled: bool,
    pub status: SyncState, // Disconnected | Connecting | Connected | Error
    pub last_sync: Option<String>,
    pub pending_outbound: usize,
    pub pending_inbound: usize,
    pub connected_devices: Vec<DeviceDto>,
}

/// Tauri command: Get sync status
#[tauri::command]
pub async fn get_sync_status(
    state: State<'_, AppState>,
) -> Result<SyncStatusDto, TruffleError> {
    // Implementation
}

/// Tauri command: Initiate device pairing
#[tauri::command]
pub async fn start_pairing(
    state: State<'_, AppState>,
) -> Result<PairingSessionDto, TruffleError> {
    // Implementation
}

/// Tauri command: Complete pairing with SAS verification
#[tauri::command]
pub async fn verify_pairing(
    state: State<'_, AppState>,
    session_id: String,
    sas_code: String,
) -> Result<DeviceDto, TruffleError> {
    // Implementation
}

/// Tauri command: Remove paired device
#[tauri::command]
pub async fn remove_device(
    state: State<'_, AppState>,
    device_id: String,
) -> Result<(), TruffleError> {
    // Implementation
}
```

### Export Commands

```rust
// truffle-desktop/src-tauri/src/commands/export.rs

/// Request: Export wiki to format
#[derive(Debug, Deserialize, TS)]
#[ts(export)]
pub struct ExportRequest {
    pub format: ExportFormat, // Markdown | Obsidian | JSON | Git
    pub destination_path: String,
    pub options: ExportOptions,
}

#[derive(Debug, Deserialize, TS)]
#[ts(export)]
pub struct ExportOptions {
    pub include_raw: bool,
    pub include_attachments: bool,
    pub date_range: Option<(String, String)>,
    pub privacy_filter: Option<Vec<PrivacyClassification>>,
}

/// Response: Export result
#[derive(Debug, Serialize, TS)]
#[ts(export)]
pub struct ExportResultDto {
    pub export_id: String,
    pub status: ExportStatus,
    pub destination_path: String,
    pub exported_nodes: usize,
    pub exported_artifacts: usize,
    pub errors: Vec<String>,
}

/// Tauri command: Export wiki
#[tauri::command]
pub async fn export_wiki(
    state: State<'_, AppState>,
    request: ExportRequest,
) -> Result<ExportResultDto, TruffleError> {
    // Implementation
}
```

### Events (Backend → Frontend)

```rust
// truffle-desktop/src-tauri/src/events.rs

/// Event: Compilation progress update
#[derive(Debug, Clone, Serialize)]
pub struct CompilationProgressEvent {
    pub job_id: String,
    pub progress: f32,
    pub current_artifact: Option<String>,
    pub log: Option<String>,
}

/// Event: Sync status changed
#[derive(Debug, Clone, Serialize)]
pub struct SyncStatusEvent {
    pub status: SyncState,
    pub message: Option<String>,
}

/// Event: New artifact detected
#[derive(Debug, Clone, Serialize)]
pub struct NewArtifactEvent {
    pub artifact_id: String,
    pub filename: String,
}

/// Event: Wiki node updated
#[derive(Debug, Clone, Serialize)]
pub struct WikiUpdateEvent {
    pub node_id: String,
    pub change_type: ChangeType, // Created | Updated | Deleted
}

// Emit events from Rust
// app.emit_all("compilation:progress", event)?;
// app.emit_all("sync:status", event)?;
// app.emit_all("artifact:new", event)?;
// app.emit_all("wiki:update", event)?;
```

---

## 2. Desktop ↔ Mobile Sync Interface

### Sync Protocol (ZKS-1)

The sync protocol uses Yjs CRDTs wrapped in AES-256-GCM encryption over WebSocket.

```rust
// truffle-core/src/sync/protocol.rs

/// ZKS-1 Protocol version
pub const PROTOCOL_VERSION: u8 = 1;

/// Sync message envelope
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SyncMessage {
    pub header: SyncHeader,
    pub payload: EncryptedPayload,
    pub mac: [u8; 32], // HMAC-SHA256
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SyncHeader {
    pub protocol_version: u8,
    pub device_id: DeviceFingerprint,
    pub timestamp: u64, // Unix millis
    pub nonce: [u8; 12], // AES-GCM nonce
}

#[derive(Debug, Clone)]
pub struct EncryptedPayload {
    pub ciphertext: Vec<u8>,
    pub tag: [u8; 16], // AES-GCM auth tag
}

/// Decrypted payload structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SyncPayload {
    pub crdt_update: Vec<u8>, // Yjs binary
    pub schema_version: String,
    pub deleted_artifacts: Vec<Uuid>, // Tombstones
}

/// Device pairing request
#[derive(Debug, Serialize, Deserialize)]
pub struct PairingRequest {
    pub ephemeral_key: [u8; 32], // X25519 public
    pub identity_key: [u8; 32],  // Ed25519 public
    pub one_time_token: String,
}

/// Device pairing response
#[derive(Debug, Serialize, Deserialize)]
pub struct PairingResponse {
    pub ephemeral_key: [u8; 32],
    pub identity_key: [u8; 32],
    pub signature: [u8; 64], // Ed25519 signature
}

/// SAS (Short Authentication String) for verification
#[derive(Debug, Clone)]
pub struct SasVerification {
    pub code: String, // 6-digit numeric
    pub method: SasMethod, // Decimal | Emoji
}
```

### Mobile FFI Interface

```rust
// truffle-mobile/src/services/core/CoreModule.rs (bridge)

use jni::JNIEnv;
use jni::objects::JString;
use jni::signature::JavaType;

/// Initialize the core library
/// Java: com.truffle.mobile.CoreModule.init(String dataDir)
#[no_mangle]
pub extern "C" fn Java_com_truffle_mobile_CoreModule_init(
    env: JNIEnv,
    _class: JClass,
    data_dir: JString,
) -> jint {
    // Implementation
}

/// Import screenshot from share extension
/// Java: com.truffle.mobile.CoreModule.importScreenshot(String path)
#[no_mangle]
pub extern "C" fn Java_com_truffle_mobile_CoreModule_importScreenshot(
    mut env: JNIEnv,
    _class: JClass,
    path: JString,
) -> jstring {
    // Implementation
}

/// Get wiki nodes list
/// Java: com.truffle.mobile.CoreModule.getWikiNodes(int limit, int offset)
#[no_mangle]
pub extern "C" fn Java_com_truffle_mobile_CoreModule_getWikiNodes(
    mut env: JNIEnv,
    _class: JClass,
    limit: jint,
    offset: jint,
) -> jstring {
    // Implementation
}

/// Search wiki
/// Java: com.truffle.mobile.CoreModule.searchWiki(String query)
#[no_mangle]
pub extern "C" fn Java_com_truffle_mobile_CoreModule_searchWiki(
    mut env: JNIEnv,
    _class: JClass,
    query: JString,
) -> jstring {
    // Implementation
}

/// Get sync status
/// Java: com.truffle.mobile.CoreModule.getSyncStatus()
#[no_mangle]
pub extern "C" fn Java_com_truffle_mobile_CoreModule_getSyncStatus(
    mut env: JNIEnv,
    _class: JClass,
) -> jstring {
    // Implementation
}

/// Start device pairing
/// Java: com.truffle.mobile.CoreModule.startPairing()
#[no_mangle]
pub extern "C" fn Java_com_truffle_mobile_CoreModule_startPairing(
    mut env: JNIEnv,
    _class: JClass,
) -> jstring {
    // Implementation
}

/// iOS equivalent using UniFFI
/// See: truffle-core/src/ffi/ios.rs
```

---

## 3. Device ↔ Relay Server Interface

### REST API

```typescript
// truffle-relay/src/types.ts

// Base API response
interface ApiResponse<T> {
  success: boolean;
  data?: T;
  error?: {
    code: string;
    message: string;
  };
}

// Rate limit headers
interface RateLimitHeaders {
  'X-RateLimit-Limit': number;
  'X-RateLimit-Remaining': number;
  'X-RateLimit-Reset': number;
}
```

### Endpoints

```typescript
// POST /v1/pairing/initiate
// Initiate device pairing session
interface InitiatePairingRequest {
  device_fingerprint: string;
  public_key: string; // Base64 encoded Ed25519
}

interface InitiatePairingResponse {
  session_id: string;
  websocket_url: string;
  expires_at: string; // ISO8601
}

// POST /v1/pairing/complete
// Complete pairing with X3DH handshake
interface CompletePairingRequest {
  session_id: string;
  ephemeral_key: string; // Base64 encoded X25519
  signature: string;     // Base64 encoded Ed25519 signature
}

interface CompletePairingResponse {
  paired_device_id: string;
  shared_secret_fingerprint: string;
}

// POST /v1/messages
// Store encrypted message for relay
interface StoreMessageRequest {
  device_id: string;
  recipient_id: string;
  payload: string; // Base64 encoded encrypted blob
  timestamp: number;
}

interface StoreMessageResponse {
  message_id: string;
  expires_at: string;
}

// GET /v1/messages?since={timestamp}
// Retrieve pending messages
interface GetMessagesResponse {
  messages: Array<{
    id: string;
    sender_id: string;
    payload: string; // Base64 encoded
    timestamp: number;
  }>;
  has_more: boolean;
}

// DELETE /v1/messages/{message_id}
// Acknowledge and delete message

// GET /v1/health
// Health check
interface HealthResponse {
  status: 'healthy' | 'degraded';
  version: string;
  timestamp: string;
}
```

### WebSocket Protocol

```typescript
// WebSocket URL: wss://relay.truffle.io/v1/sync

// Client → Server messages
interface WsClientMessage {
  type: 'subscribe' | 'unsubscribe' | 'ping' | 'ack';
  payload?: unknown;
}

interface SubscribeMessage {
  type: 'subscribe';
  device_id: string;
  auth_token: string; // HMAC of timestamp with auth_key
}

interface AckMessage {
  type: 'ack';
  message_ids: string[];
}

// Server → Client messages
interface WsServerMessage {
  type: 'message' | 'notification' | 'pong' | 'error';
  payload?: unknown;
}

interface MessageNotification {
  type: 'message';
  message_id: string;
  sender_id: string;
  timestamp: number;
  // Client then fetches via REST API
}

interface SyncNotification {
  type: 'notification';
  event: 'device_connected' | 'device_disconnected' | 'sync_complete';
  device_id?: string;
}
```

---

## 4. AI Pipeline Interfaces

### Gemma 4 Engine Interface

```rust
// truffle-core/src/ai/gemma/mod.rs

/// Gemma 4 model configuration
#[derive(Debug, Clone)]
pub struct GemmaConfig {
    pub model_path: PathBuf,
    pub context_size: usize,      // Default: 8192
    pub threads: usize,           // Default: num_cpus
    pub batch_size: usize,        // Default: 512
    pub temperature: f32,         // Default: 0.7
    pub top_p: f32,              // Default: 0.9
    pub top_k: usize,            // Default: 40
    pub repeat_penalty: f32,     // Default: 1.1
    pub gpu_layers: usize,       // Default: 0 (CPU only)
}

/// Compilation request
#[derive(Debug, Clone)]
pub struct CompilationRequest {
    pub image_path: PathBuf,
    pub schema: SchemaRules,      // Parsed schema.md
    pub artifact_id: Uuid,
}

/// Compilation result
#[derive(Debug, Clone)]
pub struct CompilationResult {
    pub nodes: Vec<WikiNode>,
    pub confidence: f32,          // 0.0 - 1.0
    pub processing_time_ms: u64,
    pub model_version: String,
    pub logs: Vec<CompilationLog>,
}

#[derive(Debug, Clone)]
pub struct CompilationLog {
    pub level: LogLevel,
    pub message: String,
    pub timestamp: DateTime<Utc>,
}

/// Gemma engine trait
#[async_trait]
pub trait GemmaEngine: Send + Sync {
    /// Initialize the engine with config
    async fn init(config: GemmaConfig) -> Result<Self, AiError>
    where
        Self: Sized;
    
    /// Compile a screenshot into wiki nodes
    async fn compile(&self, request: CompilationRequest) -> Result<CompilationResult, AiError>;
    
    /// Get model info
    fn model_info(&self) -> ModelInfo;
    
    /// Check if model is loaded
    fn is_ready(&self) -> bool;
    
    /// Unload model to free memory
    async fn unload(&self) -> Result<(), AiError>;
}

/// Model information
#[derive(Debug, Clone, Serialize)]
pub struct ModelInfo {
    pub name: String,
    pub version: String,
    pub parameters: String,       // "2B" | "9B"
    pub quantization: String,     // "Q4_K_M"
    pub context_size: usize,
    pub sha256: String,
}
```

### Schema Rules Engine Interface

```rust
// truffle-core/src/compilation/schema/mod.rs

/// Parsed schema rules
#[derive(Debug, Clone)]
pub struct SchemaRules {
    pub version: String,
    pub compilation_rules: Vec<CompilationRule>,
    pub linking_rules: LinkingRules,
    pub prohibited_extractions: Vec<ProhibitedExtraction>,
}

#[derive(Debug, Clone)]
pub struct CompilationRule {
    pub trigger: RuleTrigger,
    pub action: RuleAction,
    pub extract: Vec<ExtractionField>,
    pub link_to: Vec<LinkTarget>,
    pub privacy: PrivacyClassification,
}

#[derive(Debug, Clone)]
pub enum RuleTrigger {
    ConceptDetected(String),       // e.g., "receipt"
    EntityType(String),            // e.g., "person"
    Pattern(String),               // e.g., "flight_confirmation"
    And(Vec<RuleTrigger>),
    Or(Vec<RuleTrigger>),
}

#[derive(Debug, Clone)]
pub enum RuleAction {
    CreateEntity(String),          // Template name
    MergeOrCreate(String),         // Entity type
    CreateTravelItinerary,
    Custom(String),
}

#[derive(Debug, Clone)]
pub struct LinkingRules {
    pub temporal_proximity_seconds: u64,  // Default: 300
    pub semantic_similarity_threshold: f32, // Default: 0.82
    pub entity_resolution: EntityResolutionConfig,
}

/// Schema parser trait
pub trait SchemaParser {
    fn parse(schema_md: &str) -> Result<SchemaRules, SchemaError>;
    fn validate(&self, rules: &SchemaRules) -> Result<(), Vec<ValidationError>>;
}

/// Rule engine trait
#[async_trait]
pub trait RuleEngine: Send + Sync {
    /// Evaluate rules against compilation context
    async fn evaluate(
        &self,
        rules: &SchemaRules,
        context: &CompilationContext,
    ) -> Result<Vec<RuleApplication>, RuleError>;
}
```

### Embedding Interface

```rust
// truffle-core/src/ai/embeddings/mod.rs

/// Embedding configuration
#[derive(Debug, Clone)]
pub struct EmbeddingConfig {
    pub model_path: PathBuf,
    pub dimensions: usize,        // Default: 384 (MiniLM-L6-v2)
    pub max_length: usize,        // Default: 256 tokens
}

/// Embedding engine trait
#[async_trait]
pub trait EmbeddingEngine: Send + Sync {
    /// Generate embedding for text
    async fn embed(&self, text: &str) -> Result<Embedding, EmbeddingError>;
    
    /// Generate embeddings for multiple texts (batch)
    async fn embed_batch(&self, texts: &[&str]) -> Result<Vec<Embedding>, EmbeddingError>;
    
    /// Calculate cosine similarity
    fn similarity(a: &Embedding, b: &Embedding) -> f32;
}

/// Embedding vector
#[derive(Debug, Clone, PartialEq)]
pub struct Embedding {
    pub vector: Vec<f32>,
    pub dimensions: usize,
}

/// HNSW index configuration
#[derive(Debug, Clone)]
pub struct HnswConfig {
    pub ef_construction: usize,   // Default: 128
    pub m: usize,                // Default: 16
    pub ef_search: usize,        // Default: 64
}

/// Vector index trait
#[async_trait]
pub trait VectorIndex: Send + Sync {
    /// Insert embedding with associated ID
    async fn insert(&self, id: Uuid, embedding: &Embedding) -> Result<(), IndexError>;
    
    /// Search for nearest neighbors
    async fn search(
        &self,
        query: &Embedding,
        k: usize,
    ) -> Result<Vec<SearchResult>, IndexError>;
    
    /// Delete embedding
    async fn delete(&self, id: Uuid) -> Result<(), IndexError>;
}

#[derive(Debug, Clone)]
pub struct SearchResult {
    pub id: Uuid,
    pub similarity: f32,          // Cosine similarity
}
```

---

## 5. Storage Layer Interfaces

### Repository Pattern

```rust
// truffle-core/src/storage/repositories/mod.rs

/// Generic repository trait
#[async_trait]
pub trait Repository<T, ID>: Send + Sync {
    async fn find_by_id(&self, id: ID) -> Result<Option<T>, StorageError>;
    async fn find_all(&self, pagination: Pagination) -> Result<Vec<T>, StorageError>;
    async fn save(&self, entity: &T) -> Result<T, StorageError>;
    async fn delete(&self, id: ID) -> Result<(), StorageError>;
    async fn count(&self) -> Result<usize, StorageError>;
}

#[derive(Debug, Clone)]
pub struct Pagination {
    pub limit: usize,
    pub offset: usize,
}

/// Artifact repository
#[async_trait]
pub trait ArtifactRepository: Repository<Artifact, Uuid> {
    async fn find_by_status(
        &self,
        status: ArtifactStatus,
        pagination: Pagination,
    ) -> Result<Vec<Artifact>, StorageError>;
    
    async fn find_pending_compilation(
        &self,
        priority: CompilationPriority,
        limit: usize,
    ) -> Result<Vec<Artifact>, StorageError>;
    
    async fn update_status(
        &self,
        id: Uuid,
        status: ArtifactStatus,
    ) -> Result<(), StorageError>;
}

/// Wiki repository
#[async_trait]
pub trait WikiRepository: Repository<WikiNode, Uuid> {
    async fn find_by_title(&self, title: &str) -> Result<Option<WikiNode>, StorageError>;
    
    async fn search_by_content(
        &self,
        query: &str,
        pagination: Pagination,
    ) -> Result<Vec<WikiNode>, StorageError>;
    
    async fn find_backlinks(&self, node_id: Uuid) -> Result<Vec<WikiLink>, StorageError>;
    
    async fn find_by_provenance(
        &self,
        artifact_id: Uuid,
    ) -> Result<Vec<WikiNode>, StorageError>;
    
    async fn update_backlinks(
        &self,
        node_id: Uuid,
        backlinks: Vec<WikiLink>,
    ) -> Result<(), StorageError>;
}
```

### Database Interface

```rust
// truffle-core/src/storage/db.rs

/// Database configuration
#[derive(Debug, Clone)]
pub struct DatabaseConfig {
    pub path: PathBuf,
    pub wal_mode: bool,           // Default: true
    pub foreign_keys: bool,       // Default: true
    pub busy_timeout_ms: u64,     // Default: 5000
}

/// Database connection pool
pub struct DatabasePool {
    inner: Pool<SqliteConnectionManager>,
}

impl DatabasePool {
    pub fn new(config: DatabaseConfig) -> Result<Self, StorageError>;
    pub fn get(&self) -> Result<PooledConnection, StorageError>;
    pub async fn migrate(&self) -> Result<(), MigrationError>;
}

/// Transaction trait
#[async_trait]
pub trait Transaction: Send + Sync {
    async fn commit(self) -> Result<(), StorageError>;
    async fn rollback(self) -> Result<(), StorageError>;
}

/// Migration manager
#[async_trait]
pub trait MigrationManager: Send + Sync {
    async fn run_migrations(&self) -> Result<MigrationReport, MigrationError>;
    async fn get_current_version(&self) -> Result<i64, MigrationError>;
    async fn validate(&self) -> Result<(), Vec<MigrationError>>;
}
```

---

## 6. Export Interfaces

### Markdown Export

```rust
// truffle-core/src/export/markdown.rs

/// Markdown export configuration
#[derive(Debug, Clone)]
pub struct MarkdownExportConfig {
    pub output_dir: PathBuf,
    pub include_frontmatter: bool,     // Default: true
    pub wiki_link_style: WikiLinkStyle, // [[Title]] or [Title](Title.md)
    pub include_attachments: bool,      // Default: true
    pub date_format: String,           // Default: "%Y-%m-%d"
}

#[derive(Debug, Clone)]
pub enum WikiLinkStyle {
    Obsidian,    // [[Title]]
    GitLab,      // [Title](Title.md)
    GitHub,      // [Title](./Title.md)
}

/// Markdown exporter trait
#[async_trait]
pub trait MarkdownExporter: Send + Sync {
    async fn export_node(&self, node: &WikiNode) -> Result<ExportedFile, ExportError>;
    async fn export_all(&self, filter: ExportFilter) -> Result<ExportReport, ExportError>;
}

#[derive(Debug, Clone)]
pub struct ExportedFile {
    pub path: PathBuf,
    pub content: String,
    pub attachments: Vec<PathBuf>,
}

#[derive(Debug, Clone)]
pub struct ExportReport {
    pub exported_nodes: usize,
    pub exported_attachments: usize,
    pub output_path: PathBuf,
    pub errors: Vec<ExportError>,
    pub duration_ms: u64,
}
```

### Git Export

```rust
// truffle-core/src/export/git.rs

/// Git export configuration
#[derive(Debug, Clone)]
pub struct GitExportConfig {
    pub repo_path: PathBuf,
    pub remote_url: Option<String>,
    pub commit_message_template: String, // "Update wiki: {date}"
    pub author_name: String,
    pub author_email: String,
    pub branch: String,                // Default: "main"
}

/// Git exporter trait
#[async_trait]
pub trait GitExporter: Send + Sync {
    /// Initialize git repository
    async fn init(&self) -> Result<(), ExportError>;
    
    /// Export wiki as git commit
    async fn export_commit(&self, message: Option<String>) -> Result<GitCommit, ExportError>;
    
    /// Push to remote (if configured)
    async fn push(&self) -> Result<(), ExportError>;
    
    /// Get export history
    async fn log(&self, limit: usize) -> Result<Vec<GitCommit>, ExportError>;
}

#[derive(Debug, Clone)]
pub struct GitCommit {
    pub hash: String,
    pub message: String,
    pub author: String,
    pub timestamp: DateTime<Utc>,
    pub files_changed: usize,
}
```

### JSON Export

```rust
// truffle-core/src/export/json.rs

/// JSON export configuration
#[derive(Debug, Clone)]
pub struct JsonExportConfig {
    pub output_path: PathBuf,
    pub pretty_print: bool,           // Default: true
    pub include_embeddings: bool,     // Default: false
    pub include_raw_refs: bool,       // Default: true
}

/// Complete export structure
#[derive(Debug, Serialize)]
pub struct CompleteExport {
    pub export_metadata: ExportMetadata,
    pub artifacts: Vec<ArtifactExport>,
    pub wiki_nodes: Vec<WikiNodeExport>,
    pub links: Vec<LinkExport>,
}

#[derive(Debug, Serialize)]
pub struct ExportMetadata {
    pub version: String,
    pub exported_at: DateTime<Utc>,
    pub device_id: String,
    pub schema_version: String,
}

#[async_trait]
pub trait JsonExporter: Send + Sync {
    async fn export_complete(&self) -> Result<CompleteExport, ExportError>;
    async fn export_incremental(
        &self,
        since: DateTime<Utc>,
    ) -> Result<CompleteExport, ExportError>;
}
```

---

## Interface Versioning

All interfaces follow semantic versioning:

| Interface | Current Version | Compatibility |
|-----------|-----------------|---------------|
| Tauri Commands | 1.0.0 | Backward compatible |
| Sync Protocol (ZKS-1) | 1.0.0 | Forward compatible |
| Relay REST API | 1.0.0 | Backward compatible |
| WebSocket Protocol | 1.0.0 | Forward compatible |
| AI Pipeline | 1.0.0 | Internal only |
| Storage Layer | 1.0.0 | Internal only |

---

**Document Owner:** Systems Architect  
**Review Cycle:** Monthly  
**Classification:** Internal Use
