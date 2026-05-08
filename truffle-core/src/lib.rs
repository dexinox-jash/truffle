//! Truffle Core - Local-First Knowledge Compiler
//!
//! The Rust backend for Project Truffle, implementing:
//! - Data models (RawArtifact, WikiNode, Schema)
//! - SQLite database with WAL mode
//! - Compilation pipeline (4 stages)
//! - CRDT-based sync with Yjs/Yrs
//!
//! # Architecture
//!
//! ```text
//! ┌─────────────────────────────────────────────────────────────┐
//! │                    Truffle Core Library                      │
//! ├─────────────────────────────────────────────────────────────┤
//! │  Models    │  Database   │  Pipeline   │  Sync             │
//! │  ───────── │  ─────────  │  ─────────  │  ────             │
//! │  RawArtifact│  Connection│  Ingestion  │  CRDT (Yrs)       │
//! │  WikiNode  │  Migrations │  Processor  │  Delta            │
//! │  Schema    │  Queries    │  Graph      │  Crypto           │
//! │            │             │  Persistence│                   │
//! └─────────────────────────────────────────────────────────────┘
//!                              │
//!                    ┌─────────┴─────────┐
//!                    ▼                   ▼
//!              SQLite (WAL)        Local Filesystem
//! ```
//!
//! # Example Usage
//!
//! ```rust,no_run
//! use truffle_core::{Database, DatabaseConfig, CompilationPipeline, PipelineConfig};
//! use std::sync::Arc;
//!
//! #[tokio::main]
//! async fn main() -> anyhow::Result<()> {
//!     // Initialize database
//!     let config = DatabaseConfig::with_data_dir("~/.truffle");
//!     let db = Arc::new(Database::open(config)?);
//!
//!     // Create and start pipeline
//!     let pipeline_config = PipelineConfig::default();
//!     let mut pipeline = CompilationPipeline::new(pipeline_config, db)?;
//!     pipeline.start().await?;
//!
//!     // Submit artifact for processing
//!     // pipeline.submit(artifact, 0).await?;
//!
//!     Ok(())
//! }
//! ```

#![warn(missing_docs)]
#![warn(rust_2018_idioms)]

pub mod models;
pub mod database;
pub mod pipeline;
pub mod sync;
pub mod collaboration;
pub mod graph;
pub mod validation;
pub mod api;

// Re-export commonly used types
pub use models::{
    RawArtifact, RawArtifactMetadata, AppContext, IngestionStatus, IngestionQueueEntry,
    WikiNode, WikiNodeType, WikiLink, Provenance, TemporalVectors,
    PrivacyClassification, EncryptionStatus, VectorClock,
    SchemaRuleset, CompilationRule, Trigger, Action, PrivacyLevel as SchemaPrivacyLevel,
    content_addressable_uuid, DeviceFingerprint, Geohash, BlobPointer,
    // Phase 2: Knowledge Graph models
    Entity, EntityType, EntityMetadata, EntityAlias, AliasType, PrivacyLevel as EntityPrivacyLevel,
    Relationship, RelationshipDirection, RelationshipType, relation_types,
    Decision, DecisionStatus,
    ActionItem, Priority, ActionItemStatus,
    Meeting, MeetingType,
};

// Re-export extraction pipeline
pub use pipeline::extraction::{
    ExtractionPipeline, ExtractionConfig, ExtractionResult,
    Token,
};

// Re-export graph query API
pub use graph::{
    GraphQuery, QueryResult, EntityFilter, RelationshipFilter,
    GraphRepository, EntityRepository, RelationshipRepository,
    GraphTraversal, PathFinder, TraversalConfig,
};

// Re-export validation and confidence types (Phase 3)
pub use validation::{
    // Configuration
    ValidationConfig, ConfidenceConfig, ContradictionConfig, RedundancyConfig,
    // Engine
    ValidationEngine, EntityValidationResult, EngineValidationSummary as ValidationSummary,
    // Core models
    Fact, FactValue, FactSource,
    Contradiction, ContradictionType, ContradictionSeverity, ContradictionSummary,
    ContradictionResolution, ResolutionType, ResolutionPolicy, ResolutionStats,
    ValidationBatchId, ValidationRequest, ValidationResult, ValidationStatus,
    // Contradiction checking
    ContradictionChecker, ValidationReport,
    // Confidence
    ConfidenceScorer, ConfidenceScore, ConfidenceFactor,
    FactorCategory,
    BayesianUpdater, BayesianUpdateResult, Evidence, EvidenceType,
    SourceReliability, SourceRegistry, SourceType,
    // Redundancy
    DuplicateCandidate, DuplicateDetector, MergeEngine, MergeConfig,
    ConflictResolutionStrategy, MergePreview, MergeResult,
};

pub use database::{
    Database, DatabaseConfig, DatabaseError, DatabaseStats,
    SynchronousMode, TempStore,
    init_database, run_migrations, MIGRATIONS,
    RawArtifactRepository, WikiNodeRepository, IngestionQueueRepository,
};

pub use pipeline::{
    CompilationPipeline, PipelineConfig, PipelineStats,
    IngestionQueue, QueueConfig, QueueStats, PriorityCalculator,
    Processor, ProcessorConfig, ProcessingResult, ProcessingError,
    DocumentType, ExtractedEntity, TemporalExtraction, SafetyClassification,
    GraphBuilder, GraphConfig, LinkingEngine,
    PersistenceLayer, PersistenceConfig, SyncTrigger, CrdtDelta,
};

pub use sync::{
    SyncConfig, DeviceSyncState, SyncMessage,
    CrdtDocument, CrdtManager, SyncDocument,
    DeltaGenerator, DeltaApplier, SyncDelta, ApplyResult,
    SyncCrypto, DeviceKeys, PublicKeyBundle, EncryptedPayload,
    PairingCeremony, PairingState, SasCode,
    CompressionType, ConflictStrategy, ConflictResolution,
};

pub use collaboration::{
    ActivityEvent, ActivityFeed, ActivityType,
    Comment, CommentManager, CommentThread, TextRange,
    LiveEditManager, LiveEditSession, Operation, OperationType,
    CursorPosition, PresenceManager, SelectionRange, UserPresence,
    CollaborationSession, Participant, Permissions, SessionManager,
};
pub use validation::redundancy::{
    DuplicateDetector, DuplicateCandidate, DuplicateGroup, RelationshipDuplicate,
    MergeEngine, MergePreview, MergeResult, MergeConflict, MergeConfig, MergeStats,
    RelationshipDeduplicator, DedupConfig, DedupResult, RelationshipSignature,
    RedundancyService, FullCleanupResult, ProvenanceRecord,
    SimilarityConfig, SimilarityWeights,
    BatchDetectionResult, BatchDeduplicator, RelationshipStats,
};

/// Library version
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

/// Library name
pub const NAME: &str = env!("CARGO_PKG_NAME");

/// Initialize the Truffle core library
///
/// This function initializes logging and other global state.
/// Call this before using any other library functions.
pub fn init() {
    // Initialize tracing subscriber if not already set
    let _ = tracing_subscriber::fmt()
        .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
        .try_init();
}

/// Result type for Truffle operations
pub type Result<T> = std::result::Result<T, Error>;

/// Error type for Truffle operations
#[derive(Debug, thiserror::Error)]
pub enum Error {
    /// Database error
    #[error("Database error: {0}")]
    Database(#[from] DatabaseError),
    
    /// Processing error
    #[error("Processing error: {0}")]
    Processing(#[from] pipeline::processor::ProcessingError),
    
    /// IO error
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    
    /// Other error
    #[error("{0}")]
    Other(String),
}

/// Library information
#[derive(Debug, Clone)]
pub struct LibraryInfo {
    /// Library version
    pub version: String,
    /// Library name
    pub name: String,
    /// SQLite version
    pub sqlite_version: String,
    /// Features enabled
    pub features: Vec<String>,
}

/// Get library information
pub fn info() -> LibraryInfo {
    LibraryInfo {
        version: VERSION.to_string(),
        name: NAME.to_string(),
        sqlite_version: rusqlite::version().to_string(),
        features: vec![
            "sqlite-wal".to_string(),
            "crdt-sync".to_string(),
            "vector-search".to_string(),
            "collaboration".to_string(),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_version() {
        assert!(!VERSION.is_empty());
        assert!(!NAME.is_empty());
    }
    
    #[test]
    fn test_info() {
        let info = crate::info();
        assert_eq!(info.version, VERSION);
        assert_eq!(info.name, NAME);
        assert!(!info.sqlite_version.is_empty());
        assert!(!info.features.is_empty());
    }
    
    #[test]
    fn test_init() {
        // Should not panic
        crate::init();
    }
}
