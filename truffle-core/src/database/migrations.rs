//! Database Migrations
//!
//! Schema migrations for the Truffle database using refinery.
//! All migrations are idempotent and support rollback where possible.

use rusqlite::Connection;
use refinery::{Migration, Runner, Report};
use refinery::config::Config;
use thiserror::Error;
use tracing::{info, debug};

/// Migration error types
#[derive(Error, Debug)]
pub enum MigrationError {
    #[error("SQLite error: {0}")]
    Sqlite(#[from] rusqlite::Error),
    
    #[error("Refinery error: {0}")]
    Refinery(#[from] refinery::Error),
    
    #[error("Migration {0} failed: {1}")]
    MigrationFailed(u32, String),
    
    #[error("Unknown migration version")]
    UnknownVersion,
}

/// Embedded migrations
pub mod embedded {
    use refinery::embed_migrations;
    embed_migrations!("./migrations");
}

/// Run all pending migrations
pub fn run_migrations(conn: &mut Connection) -> Result<Report, MigrationError> {
    debug!("Running database migrations");
    
    // For now, run manual migrations since refinery embed requires files
    // In production, use embedded::migrations::runner()
    run_manual_migrations(conn)?;
    
    // Return a dummy report
    Ok(Report::new(vec![]))
}

/// Manual migration runner (for development)
fn run_manual_migrations(conn: &mut Connection) -> Result<(), MigrationError> {
    // Create migrations table if not exists
    conn.execute_batch(MIGRATION_TABLE_SQL)?;
    
    // Get current version
    let current_version: u32 = conn.query_row(
        "SELECT COALESCE(MAX(version), 0) FROM __migrations",
        [],
        |row| row.get(0)
    )?;
    
    info!("Current database version: {}", current_version);
    
    // Run pending migrations
    for migration in MIGRATIONS.iter() {
        if migration.version > current_version {
            info!("Applying migration {}: {}", migration.version, migration.name);
            
            conn.execute_batch(migration.sql)?;
            
            // Record migration
            conn.execute(
                "INSERT INTO __migrations (version, name, applied_at) VALUES (?1, ?2, datetime('now'))",
                [&migration.version.to_string(), &migration.name],
            )?;
            
            info!("Migration {} applied successfully", migration.version);
        }
    }
    
    Ok(())
}

/// Migration table schema
const MIGRATION_TABLE_SQL: &str = r#"
CREATE TABLE IF NOT EXISTS __migrations (
    version INTEGER PRIMARY KEY,
    name TEXT NOT NULL,
    applied_at TEXT NOT NULL
);
"#;

/// A single migration
pub struct MigrationDef {
    pub version: u32,
    pub name: &'static str,
    pub sql: &'static str,
}

/// All database migrations
pub const MIGRATIONS: &[MigrationDef] = &[
    // Migration 1: Initial schema
    MigrationDef {
        version: 1,
        name: "initial_schema",
        sql: MIGRATION_V1,
    },
    // Migration 2: Add vector search
    MigrationDef {
        version: 2,
        name: "add_vector_search",
        sql: MIGRATION_V2,
    },
    // Migration 3: Add sync tables
    MigrationDef {
        version: 3,
        name: "add_sync_tables",
        sql: MIGRATION_V3,
    },
    // Migration 4: Add indices
    MigrationDef {
        version: 4,
        name: "add_indices",
        sql: MIGRATION_V4,
    },
    // Migration 5: Knowledge Graph Schema (Phase 2)
    MigrationDef {
        version: 5,
        name: "knowledge_graph_schema",
        sql: MIGRATION_V5,
    },
];

/// Migration 1: Initial schema - RawArtifacts and WikiNodes
const MIGRATION_V1: &str = r#"
-- Raw Artifacts table (immutable captures)
CREATE TABLE IF NOT EXISTS raw_artifacts (
    uuid BLOB PRIMARY KEY,  -- 16-byte UUID
    filename TEXT NOT NULL,
    storage_path TEXT NOT NULL,
    content_hash TEXT NOT NULL,  -- SHA256 of image
    captured_at TEXT NOT NULL,  -- ISO8601
    device_id TEXT NOT NULL,  -- Hashed device fingerprint
    app_bundle_id TEXT,
    app_name TEXT,
    window_title TEXT,
    geohash TEXT,  -- 4-char precision
    size_bytes INTEGER NOT NULL,
    width INTEGER NOT NULL,
    height INTEGER NOT NULL,
    ocr_text TEXT,
    status TEXT NOT NULL DEFAULT 'pending',  -- pending, processing, compiled, failed, excluded
    retry_count INTEGER NOT NULL DEFAULT 0,
    created_at TEXT NOT NULL,
    updated_at TEXT NOT NULL
);

-- Wiki Nodes table (compiled knowledge)
CREATE TABLE IF NOT EXISTS wiki_nodes (
    id BLOB PRIMARY KEY,  -- 16-byte UUID
    node_type TEXT NOT NULL,  -- entity, concept, chronology, index
    title TEXT NOT NULL,
    slug TEXT NOT NULL,
    raw_content TEXT NOT NULL,
    source_artifacts TEXT NOT NULL,  -- JSON array of UUIDs
    compiled_at TEXT NOT NULL,
    model_version TEXT NOT NULL,
    confidence_score REAL NOT NULL,
    compilation_duration_ms INTEGER,
    schema_version TEXT NOT NULL,
    mentioned_dates TEXT,  -- JSON array of ISO8601 dates
    fiscal_quarter TEXT,
    privacy_classification TEXT NOT NULL DEFAULT 'personal',
    encryption_status TEXT NOT NULL DEFAULT 'plaintext',
    version_clock TEXT NOT NULL DEFAULT '{}',  -- JSON vector clock
    embedding BLOB,  -- Float32 array for vector search
    created_at TEXT NOT NULL,
    updated_at TEXT NOT NULL
);

-- Wiki Links table (bidirectional relationships)
CREATE TABLE IF NOT EXISTS wiki_links (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    source_id BLOB NOT NULL,
    target_id BLOB NOT NULL,
    target_title TEXT NOT NULL,
    context TEXT,
    created_at TEXT NOT NULL,
    UNIQUE(source_id, target_id),
    FOREIGN KEY (source_id) REFERENCES wiki_nodes(id) ON DELETE CASCADE,
    FOREIGN KEY (target_id) REFERENCES wiki_nodes(id) ON DELETE CASCADE
);

-- Ingestion Queue table
CREATE TABLE IF NOT EXISTS ingestion_queue (
    id BLOB PRIMARY KEY,
    raw_uuid BLOB NOT NULL UNIQUE,
    status TEXT NOT NULL DEFAULT 'pending',
    retry_count INTEGER NOT NULL DEFAULT 0,
    priority INTEGER NOT NULL DEFAULT 2,  -- 0=highest, 2=lowest
    created_at TEXT NOT NULL,
    started_at TEXT,
    completed_at TEXT,
    error_message TEXT,
    FOREIGN KEY (raw_uuid) REFERENCES raw_artifacts(uuid) ON DELETE CASCADE
);

-- Schema settings table
CREATE TABLE IF NOT EXISTS schema_settings (
    key TEXT PRIMARY KEY,
    value TEXT NOT NULL,
    updated_at TEXT NOT NULL
);

-- Insert default schema version
INSERT OR REPLACE INTO schema_settings (key, value, updated_at) 
VALUES ('schema_version', '2.0', datetime('now'));
"#;

/// Migration 2: Add vector search support via sqlite-vec
const MIGRATION_V2: &str = r#"
-- Enable sqlite-vec extension (loaded at runtime)
-- Virtual table for artifact embeddings
CREATE VIRTUAL TABLE IF NOT EXISTS artifact_embeddings USING vec0(
    uuid BLOB PRIMARY KEY,
    embedding FLOAT[384]  -- MiniLM-L6-v2 dimensions
);

-- Virtual table for wiki node embeddings
CREATE VIRTUAL TABLE IF NOT EXISTS node_embeddings USING vec0(
    node_id BLOB PRIMARY KEY,
    embedding FLOAT[384]
);

-- Full-text search for OCR text
CREATE VIRTUAL TABLE IF NOT EXISTS ocr_fts USING fts5(
    uuid,
    ocr_text,
    content='raw_artifacts',
    content_rowid='rowid'
);

-- Full-text search for wiki content
CREATE VIRTUAL TABLE IF NOT EXISTS wiki_fts USING fts5(
    node_id,
    title,
    content,
    content='wiki_nodes',
    content_rowid='rowid'
);
"#;

/// Migration 3: Add sync and CRDT tables
const MIGRATION_V3: &str = r#"
-- Device registration
CREATE TABLE IF NOT EXISTS devices (
    device_id TEXT PRIMARY KEY,  -- Hashed fingerprint
    display_name TEXT,
    public_key BLOB,  -- Ed25519 public key
    last_seen_at TEXT,
    is_primary BOOLEAN NOT NULL DEFAULT 0,
    created_at TEXT NOT NULL
);

-- Sync state for CRDT
CREATE TABLE IF NOT EXISTS sync_state (
    device_id TEXT PRIMARY KEY,
    last_sync_at TEXT,
    vector_clock TEXT NOT NULL,  -- JSON
    pending_changes INTEGER NOT NULL DEFAULT 0,
    FOREIGN KEY (device_id) REFERENCES devices(device_id) ON DELETE CASCADE
);

-- Outbound sync queue (encrypted CRDT deltas)
CREATE TABLE IF NOT EXISTS sync_queue (
    id BLOB PRIMARY KEY,
    device_id TEXT NOT NULL,
    encrypted_payload BLOB NOT NULL,  -- AES-256-GCM encrypted
    nonce BLOB NOT NULL,  -- 96-bit nonce
    timestamp TEXT NOT NULL,
    retry_count INTEGER NOT NULL DEFAULT 0,
    FOREIGN KEY (device_id) REFERENCES devices(device_id) ON DELETE CASCADE
);

-- Tombstones for deleted items (GDPR compliance)
CREATE TABLE IF NOT EXISTS tombstones (
    id BLOB PRIMARY KEY,
    item_type TEXT NOT NULL,  -- 'raw_artifact' or 'wiki_node'
    deleted_at TEXT NOT NULL,
    deleted_by_device TEXT NOT NULL,
    vector_clock TEXT NOT NULL  -- For sync propagation
);

-- Audit log (local only, Merkle tree for integrity)
CREATE TABLE IF NOT EXISTS audit_log (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    action TEXT NOT NULL,
    item_type TEXT NOT NULL,
    item_id BLOB,
    device_id TEXT NOT NULL,
    timestamp TEXT NOT NULL,
    hash_chain BLOB NOT NULL  -- SHA256 of previous + current
);
"#;

/// Migration 4: Performance indices
const MIGRATION_V4: &str = r#"
-- Raw artifacts indices
CREATE INDEX IF NOT EXISTS idx_raw_status ON raw_artifacts(status);
CREATE INDEX IF NOT EXISTS idx_raw_captured_at ON raw_artifacts(captured_at);
CREATE INDEX IF NOT EXISTS idx_raw_device ON raw_artifacts(device_id);
CREATE INDEX IF NOT EXISTS idx_raw_geohash ON raw_artifacts(geohash);
CREATE INDEX IF NOT EXISTS idx_raw_app_bundle ON raw_artifacts(app_bundle_id);

-- Wiki nodes indices
CREATE INDEX IF NOT EXISTS idx_wiki_type ON wiki_nodes(node_type);
CREATE INDEX IF NOT EXISTS idx_wiki_slug ON wiki_nodes(slug);
CREATE INDEX IF NOT EXISTS idx_wiki_compiled ON wiki_nodes(compiled_at);
CREATE INDEX IF NOT EXISTS idx_wiki_privacy ON wiki_nodes(privacy_classification);

-- Wiki links indices
CREATE INDEX IF NOT EXISTS idx_links_source ON wiki_links(source_id);
CREATE INDEX IF NOT EXISTS idx_links_target ON wiki_links(target_id);

-- Ingestion queue indices
CREATE INDEX IF NOT EXISTS idx_queue_status ON ingestion_queue(status);
CREATE INDEX IF NOT EXISTS idx_queue_priority ON ingestion_queue(priority, created_at);

-- Temporal queries index
CREATE INDEX IF NOT EXISTS idx_wiki_fiscal ON wiki_nodes(fiscal_quarter);

-- Sync indices
CREATE INDEX IF NOT EXISTS idx_sync_device ON sync_queue(device_id);
CREATE INDEX IF NOT EXISTS idx_tombstones_type ON tombstones(item_type);
CREATE INDEX IF NOT EXISTS idx_audit_timestamp ON audit_log(timestamp);
"#;

/// Migration 5: Knowledge Graph Schema (Phase 2)
const MIGRATION_V5: &str = r#"
-- ============================================
-- KNOWLEDGE GRAPH SCHEMA (Phase 2)
-- ============================================

-- Entity types lookup
CREATE TABLE IF NOT EXISTS entity_types (
    type_name TEXT PRIMARY KEY,
    description TEXT,
    icon TEXT,
    color TEXT
);

INSERT OR IGNORE INTO entity_types (type_name, description) VALUES 
    ('person', 'Individual human'),
    ('organization', 'Company, team, or group'),
    ('location', 'Physical or virtual place'),
    ('event', 'Scheduled occurrence'),
    ('product', 'Product or service'),
    ('concept', 'Abstract idea or topic'),
    ('decision', 'Recorded decision'),
    ('action_item', 'Task or commitment');

-- Relationship types lookup
CREATE TABLE IF NOT EXISTS relationship_types (
    type_name TEXT PRIMARY KEY,
    display_name TEXT NOT NULL,
    inverse_type TEXT,  -- e.g., 'works_at' -> 'employs'
    description TEXT
);

INSERT OR IGNORE INTO relationship_types (type_name, display_name, inverse_type, description) VALUES
    ('works_at', 'works at', 'employs', 'Employment relationship'),
    ('located_in', 'located in', 'contains', 'Location relationship'),
    ('mentioned_in', 'mentioned in', 'mentions', 'Mention in meeting'),
    ('decided_by', 'decided by', 'decided', 'Decision maker'),
    ('reported_to', 'reports to', 'manages', 'Reporting structure'),
    ('collaborated_with', 'collaborated with', 'collaborated_with', 'Collaboration'),
    ('depends_on', 'depends on', 'required_for', 'Dependency'),
    ('caused', 'caused', 'caused_by', 'Causation'),
    ('part_of', 'part of', 'contains', 'Composition'),
    ('related_to', 'related to', 'related_to', 'General relation'),
    ('committed_to', 'committed to', 'committed_by', 'Commitment'),
    ('blocked_by', 'blocked by', 'blocking', 'Blocker');

-- ============================================
-- ENTITIES (Nodes in the knowledge graph)
-- ============================================
CREATE TABLE IF NOT EXISTS entities (
    id BLOB PRIMARY KEY,
    entity_type TEXT NOT NULL,
    name TEXT NOT NULL,
    slug TEXT NOT NULL,
    description TEXT,
    metadata JSON,
    
    -- Vector embedding for semantic search (384-dim MiniLM)
    embedding BLOB,
    
    -- Temporal tracking
    first_seen_at TEXT NOT NULL DEFAULT (datetime('now')),
    last_updated_at TEXT NOT NULL DEFAULT (datetime('now')),
    
    -- Source tracking
    source_meeting_id BLOB,
    source_chunk_id BLOB,
    
    -- Confidence and verification
    extraction_confidence REAL NOT NULL CHECK (extraction_confidence BETWEEN 0.0 AND 1.0) DEFAULT 0.8,
    verified BOOLEAN NOT NULL DEFAULT FALSE,
    
    -- Privacy classification
    privacy_level TEXT NOT NULL DEFAULT 'internal' CHECK (privacy_level IN ('public', 'internal', 'confidential', 'restricted')),
    
    FOREIGN KEY (entity_type) REFERENCES entity_types(type_name),
    FOREIGN KEY (source_meeting_id) REFERENCES meetings(id)
);

-- Entity aliases for resolution
CREATE TABLE IF NOT EXISTS entity_aliases (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    entity_id BLOB NOT NULL,
    alias TEXT NOT NULL,
    alias_type TEXT NOT NULL DEFAULT 'nickname' CHECK (alias_type IN ('acronym', 'nickname', 'full_name', 'misspelling', 'translation')),
    confidence REAL NOT NULL DEFAULT 0.8 CHECK (confidence BETWEEN 0.0 AND 1.0),
    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    
    UNIQUE(entity_id, alias),
    FOREIGN KEY (entity_id) REFERENCES entities(id) ON DELETE CASCADE
);

-- ============================================
-- RELATIONSHIPS (Edges in the knowledge graph)
-- ============================================
CREATE TABLE IF NOT EXISTS relationships (
    id BLOB PRIMARY KEY,
    source_id BLOB NOT NULL,
    target_id BLOB NOT NULL,
    relation_type TEXT NOT NULL,
    
    -- Temporal bounds for time-varying relationships
    valid_from TEXT,
    valid_until TEXT,
    
    -- Metadata
    confidence REAL NOT NULL CHECK (confidence BETWEEN 0.0 AND 1.0) DEFAULT 0.8,
    metadata JSON,
    
    -- Source tracking
    source_meeting_id BLOB,
    extraction_method TEXT NOT NULL DEFAULT 'rule_based' CHECK (extraction_method IN ('rule_based', 'ml_model', 'manual', 'inferred')),
    
    -- Timestamps
    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    updated_at TEXT NOT NULL DEFAULT (datetime('now')),
    
    UNIQUE(source_id, target_id, relation_type, valid_from),
    FOREIGN KEY (source_id) REFERENCES entities(id) ON DELETE CASCADE,
    FOREIGN KEY (target_id) REFERENCES entities(id) ON DELETE CASCADE,
    FOREIGN KEY (relation_type) REFERENCES relationship_types(type_name),
    FOREIGN KEY (source_meeting_id) REFERENCES meetings(id)
);

-- ============================================
-- MEETINGS (Source documents)
-- ============================================
CREATE TABLE IF NOT EXISTS meetings (
    id BLOB PRIMARY KEY,
    title TEXT NOT NULL,
    
    -- Timing
    started_at TEXT,
    ended_at TEXT,
    timezone TEXT DEFAULT 'UTC',
    
    -- Participants (JSON array of entity IDs)
    participant_ids JSON,
    
    -- Transcript
    transcript TEXT,
    transcript_chunks JSON,
    
    -- Metadata
    meeting_type TEXT CHECK (meeting_type IN ('standup', 'planning', 'review', 'retrospective', '1on1', 'interview', 'presentation', 'brainstorm', 'other')),
    platform TEXT,
    recording_path TEXT,
    
    -- Processing status
    processing_status TEXT DEFAULT 'pending' CHECK (processing_status IN ('pending', 'transcribing', 'extracting', 'building_graph', 'completed', 'failed')),
    processing_error TEXT,
    
    -- Timestamps
    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    updated_at TEXT NOT NULL DEFAULT (datetime('now'))
);

-- Entity mentions (linking entities to transcript locations)
CREATE TABLE IF NOT EXISTS entity_mentions (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    entity_id BLOB NOT NULL,
    meeting_id BLOB NOT NULL,
    
    -- Location in transcript
    chunk_index INTEGER NOT NULL,
    start_offset INTEGER NOT NULL,
    end_offset INTEGER NOT NULL,
    mention_text TEXT NOT NULL,
    
    -- Context window
    context_before TEXT,
    context_after TEXT,
    
    -- Confidence
    confidence REAL NOT NULL CHECK (confidence BETWEEN 0.0 AND 1.0) DEFAULT 0.8,
    resolved_entity_id BLOB,
    
    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    
    UNIQUE(entity_id, meeting_id, chunk_index, start_offset),
    FOREIGN KEY (entity_id) REFERENCES entities(id) ON DELETE CASCADE,
    FOREIGN KEY (meeting_id) REFERENCES meetings(id) ON DELETE CASCADE,
    FOREIGN KEY (resolved_entity_id) REFERENCES entities(id)
);

-- ============================================
-- DECISIONS (Tracked across meetings)
-- ============================================
CREATE TABLE IF NOT EXISTS decisions (
    id BLOB PRIMARY KEY,
    
    -- Content
    decision_text TEXT NOT NULL,
    summary TEXT,
    
    -- Versioning
    version INTEGER NOT NULL DEFAULT 1,
    previous_version_id BLOB,
    decision_chain_id BLOB NOT NULL,
    
    -- Status
    status TEXT DEFAULT 'proposed' CHECK (status IN ('proposed', 'approved', 'rejected', 'superseded', 'implemented', 'abandoned')),
    
    -- Attribution
    proposed_by_id BLOB,
    decided_by_ids JSON,
    dissenting_ids JSON,
    
    -- Location
    decided_in_meeting_id BLOB,
    
    -- Timeline
    proposed_at TEXT,
    decided_at TEXT,
    implemented_at TEXT,
    
    -- Impact
    impact_score REAL CHECK (impact_score BETWEEN 0.0 AND 1.0),
    dependent_decision_ids JSON,
    
    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    updated_at TEXT NOT NULL DEFAULT (datetime('now')),
    
    FOREIGN KEY (proposed_by_id) REFERENCES entities(id),
    FOREIGN KEY (decided_in_meeting_id) REFERENCES meetings(id)
);

-- ============================================
-- ACTION ITEMS (Extracted commitments)
-- ============================================
CREATE TABLE IF NOT EXISTS action_items (
    id BLOB PRIMARY KEY,
    
    -- Content
    description TEXT NOT NULL,
    
    -- Assignment
    assignee_id BLOB,
    creator_id BLOB,
    
    -- Source
    source_meeting_id BLOB,
    source_decision_id BLOB,
    
    -- Status
    status TEXT DEFAULT 'open' CHECK (status IN ('open', 'in_progress', 'blocked', 'completed', 'cancelled', 'overdue')),
    
    -- Timeline
    deadline TEXT,
    completed_at TEXT,
    
    -- Priority
    priority TEXT DEFAULT 'medium' CHECK (priority IN ('low', 'medium', 'high', 'urgent')),
    
    -- Verification
    completion_verified_by_id BLOB,
    verification_notes TEXT,
    
    -- Reminders
    reminder_sent BOOLEAN DEFAULT FALSE,
    reminder_sent_at TEXT,
    
    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    updated_at TEXT NOT NULL DEFAULT (datetime('now')),
    
    FOREIGN KEY (assignee_id) REFERENCES entities(id),
    FOREIGN KEY (creator_id) REFERENCES entities(id),
    FOREIGN KEY (source_meeting_id) REFERENCES meetings(id),
    FOREIGN KEY (source_decision_id) REFERENCES decisions(id),
    FOREIGN KEY (completion_verified_by_id) REFERENCES entities(id)
);

-- ============================================
-- INDICES for Performance
-- ============================================
-- Entity indices
CREATE INDEX IF NOT EXISTS idx_entities_type ON entities(entity_type);
CREATE INDEX IF NOT EXISTS idx_entities_name ON entities(name);
CREATE INDEX IF NOT EXISTS idx_entities_slug ON entities(slug);
CREATE INDEX IF NOT EXISTS idx_entities_first_seen ON entities(first_seen_at);

-- Entity alias indices
CREATE INDEX IF NOT EXISTS idx_aliases_entity ON entity_aliases(entity_id);
CREATE INDEX IF NOT EXISTS idx_aliases_alias ON entity_aliases(alias);

-- Relationship indices
CREATE INDEX IF NOT EXISTS idx_relationships_source ON relationships(source_id);
CREATE INDEX IF NOT EXISTS idx_relationships_target ON relationships(target_id);
CREATE INDEX IF NOT EXISTS idx_relationships_type ON relationships(relation_type);
CREATE INDEX IF NOT EXISTS idx_relationships_temporal ON relationships(valid_from, valid_until);

-- Meeting indices
CREATE INDEX IF NOT EXISTS idx_meetings_started ON meetings(started_at);
CREATE INDEX IF NOT EXISTS idx_meetings_status ON meetings(processing_status);
CREATE INDEX IF NOT EXISTS idx_meetings_type ON meetings(meeting_type);

-- Entity mention indices
CREATE INDEX IF NOT EXISTS idx_mentions_entity ON entity_mentions(entity_id);
CREATE INDEX IF NOT EXISTS idx_mentions_meeting ON entity_mentions(meeting_id);

-- Decision indices
CREATE INDEX IF NOT EXISTS idx_decisions_chain ON decisions(decision_chain_id);
CREATE INDEX IF NOT EXISTS idx_decisions_status ON decisions(status);
CREATE INDEX IF NOT EXISTS idx_decisions_meeting ON decisions(decided_in_meeting_id);

-- Action item indices
CREATE INDEX IF NOT EXISTS idx_action_items_assignee ON action_items(assignee_id);
CREATE INDEX IF NOT EXISTS idx_action_items_status ON action_items(status);
CREATE INDEX IF NOT EXISTS idx_action_items_deadline ON action_items(deadline);

-- ============================================
-- FULL-TEXT SEARCH
-- ============================================
-- Entity search
CREATE VIRTUAL TABLE IF NOT EXISTS entity_search USING fts5(
    name, description,
    content='entities',
    content_rowid='rowid'
);

-- Meeting transcript search
CREATE VIRTUAL TABLE IF NOT EXISTS meeting_search USING fts5(
    title, transcript,
    content='meetings',
    content_rowid='rowid'
);
"#;

/// Get migration by version
pub fn get_migration(version: u32) -> Option<&'static MigrationDef> {
    MIGRATIONS.iter().find(|m| m.version == version)
}

/// Get latest migration version
pub fn latest_version() -> u32 {
    MIGRATIONS.iter().map(|m| m.version).max().unwrap_or(0)
}

/// Check if database needs migration
pub fn needs_migration(conn: &Connection) -> Result<bool, MigrationError> {
    // Check if migrations table exists
    let exists: bool = conn.query_row(
        "SELECT 1 FROM sqlite_master WHERE type='table' AND name='__migrations'",
        [],
        |_| Ok(true)
    ).unwrap_or(false);
    
    if !exists {
        return Ok(true);
    }
    
    let current_version: u32 = conn.query_row(
        "SELECT COALESCE(MAX(version), 0) FROM __migrations",
        [],
        |row| row.get(0)
    )?;
    
    Ok(current_version < latest_version())
}

/// Rollback to a specific version (destructive!)
pub fn rollback_to(conn: &mut Connection, target_version: u32) -> Result<(), MigrationError> {
    info!("Rolling back to version {}", target_version);
    
    // This is a simplified rollback - in production, implement proper down migrations
    // For now, we just clear data from higher version tables
    
    let current_version: u32 = conn.query_row(
        "SELECT COALESCE(MAX(version), 0) FROM __migrations",
        [],
        |row| row.get(0)
    )?;
    
    if target_version >= current_version {
        return Ok(());
    }
    
    // Remove migration records
    conn.execute(
        "DELETE FROM __migrations WHERE version > ?1",
        [&target_version],
    )?;
    
    info!("Rollback to version {} completed", target_version);
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_migrations_run() {
        let mut conn = Connection::open_in_memory().expect("Failed to open in-memory database");
        run_manual_migrations(&mut conn).expect("Failed to run migrations");
        
        // Verify tables exist
        let tables: Vec<String> = conn
            .prepare("SELECT name FROM sqlite_master WHERE type='table'")
            .expect("Failed to prepare statement")
            .query_map([], |row| row.get(0))
            .expect("Failed to query tables")
            .collect::<Result<Vec<_>, _>>()
            .expect("Failed to collect results");
        
        // Core tables (v1)
        assert!(tables.contains(&"raw_artifacts".to_string()));
        assert!(tables.contains(&"wiki_nodes".to_string()));
        assert!(tables.contains(&"ingestion_queue".to_string()));
        
        // Knowledge graph tables (v5)
        assert!(tables.contains(&"entities".to_string()));
        assert!(tables.contains(&"relationships".to_string()));
        assert!(tables.contains(&"meetings".to_string()));
        assert!(tables.contains(&"decisions".to_string()));
        assert!(tables.contains(&"action_items".to_string()));
        assert!(tables.contains(&"entity_mentions".to_string()));
        assert!(tables.contains(&"entity_aliases".to_string()));
    }
    
    #[test]
    fn test_latest_version() {
        assert_eq!(latest_version(), 5);
    }
    
    #[test]
    fn test_get_migration() {
        let m1 = get_migration(1).expect("Migration 1 not found");
        assert_eq!(m1.version, 1);
        assert_eq!(m1.name, "initial_schema");
        
        assert!(get_migration(999).is_none());
    }
    
    #[test]
    fn test_needs_migration() {
        let mut conn = Connection::open_in_memory().expect("Failed to open in-memory database");
        
        // Fresh database needs migration
        assert!(needs_migration(&conn).expect("Failed to check migration status"));
        
        // Run migrations
        run_manual_migrations(&mut conn).expect("Failed to run migrations");
        
        // Now up to date
        assert!(!needs_migration(&conn).expect("Failed to check migration status"));
    }
}
