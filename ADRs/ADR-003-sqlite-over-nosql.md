# ADR-003: SQLite over NoSQL for Local-First Storage

## Status

**Accepted** | Date: 2024-01-15 | Author: Systems Architect

## Context

Project Truffle requires a local-first storage solution that can:

- Store structured wiki data with complex relationships
- Support full-text search across all content
- Enable vector similarity search for semantic queries
- Provide ACID guarantees for data integrity
- Operate offline indefinitely
- Sync efficiently between devices
- Export to standard formats (Markdown, JSON)
- Run on resource-constrained devices (mobile)

Two primary database approaches were evaluated:
1. **SQLite** (relational with extensions)
2. **Embedded NoSQL** (RocksDB, LMDB, or similar)

## Decision

We will use **SQLite** with the following extensions:
- **sqlite-vec** for vector similarity search (HNSW index)
- **R-Tree** for temporal/spatial indexing
- **FTS5** for full-text search
- **WAL mode** for concurrent read/write

## Consequences

### Positive

| Aspect | SQLite Advantage |
|--------|------------------|
| **ACID Compliance** | Full transactional integrity |
| **Query Power** | SQL for complex joins and aggregations |
| **Observability** | Standard tools (DB Browser, CLI) |
| **Maturity** | 25+ years of production use |
| **Ecosystem** | Extensive documentation and community |
| **Portability** | Single file, cross-platform |
| **Backup** | Litestream for continuous backup |
| **CRDT Integration** | Yjs works well with relational data |

### Negative

| Aspect | SQLite Challenge | Mitigation |
|--------|------------------|------------|
| **Schema Migrations** | Required for schema changes | Automated migration system |
| **Write Concurrency** | Single writer | WAL mode + queue-based writes |
| **Horizontal Scaling** | Not applicable (local-first) | N/A - by design |
| **Nested Documents** | Less natural than NoSQL | JSON columns for flexibility |

## Alternatives Considered

### RocksDB

**Pros:**
- High write throughput
- Used by major projects (Facebook, LinkedIn)
- Good for key-value workloads

**Cons:**
- No built-in query language
- Complex to tune
- No full-text search
- Requires external vector search integration

**Verdict:** Rejected due to complexity and lack of query capabilities.

### LMDB

**Pros:**
- Memory-mapped performance
- ACID transactions
- Small footprint

**Cons:**
- Key-value only
- No built-in indexing
- Limited Windows support

**Verdict:** Rejected due to lack of query and indexing features.

### PouchDB/CouchDB

**Pros:**
- Built-in sync capabilities
- JSON document model
- Good for offline-first

**Cons:**
- Larger bundle size
- Complex conflict resolution
- No vector search
- Requires CouchDB server for sync

**Verdict:** Rejected due to bundle size and sync complexity.

### Custom Storage

**Pros:**
- Optimized for exact use case
- Full control over format

**Cons:**
- Massive development effort
- Bug-prone
- No ecosystem support

**Verdict:** Rejected due to development cost.

## Implementation Details

### Schema Design

```sql
-- Core tables
CREATE TABLE artifacts (
    id BLOB PRIMARY KEY,  -- UUIDv4
    filename TEXT NOT NULL,
    path TEXT NOT NULL,
    captured_at INTEGER NOT NULL,  -- Unix timestamp
    device_id TEXT NOT NULL,
    status INTEGER NOT NULL,  -- 0: pending, 1: processing, 2: completed, 3: failed
    ocr_text TEXT,
    metadata JSON,
    created_at INTEGER DEFAULT (unixepoch()),
    updated_at INTEGER DEFAULT (unixepoch())
);

CREATE TABLE wiki_nodes (
    id BLOB PRIMARY KEY,
    node_type INTEGER NOT NULL,  -- 0: entity, 1: concept, 2: chronology, 3: index
    title TEXT NOT NULL,
    content TEXT NOT NULL,
    privacy INTEGER NOT NULL,  -- 0: public, 1: personal, 2: sensitive, 3: financial
    encryption_status INTEGER NOT NULL,  -- 0: plaintext, 1: aes256-gcm
    schema_version TEXT NOT NULL,
    confidence REAL,
    version_vector JSON,  -- CRDT vector clock
    created_at INTEGER DEFAULT (unixepoch()),
    updated_at INTEGER DEFAULT (unixepoch())
);

CREATE TABLE wiki_links (
    source_id BLOB NOT NULL REFERENCES wiki_nodes(id),
    target_id BLOB NOT NULL REFERENCES wiki_nodes(id),
    link_type INTEGER NOT NULL,  -- 0: wiki_link, 1: backlink, 2: temporal
    created_at INTEGER DEFAULT (unixepoch()),
    PRIMARY KEY (source_id, target_id)
);

CREATE TABLE provenance (
    node_id BLOB NOT NULL REFERENCES wiki_nodes(id),
    artifact_id BLOB NOT NULL REFERENCES artifacts(id),
    compiled_at INTEGER NOT NULL,
    model_version TEXT NOT NULL,
    confidence REAL NOT NULL,
    PRIMARY KEY (node_id, artifact_id)
);

-- Vector search with sqlite-vec
CREATE VIRTUAL TABLE node_embeddings USING vec0(
    node_id BLOB PRIMARY KEY,
    embedding FLOAT[384]  -- MiniLM-L6-v2 dimensions
);

-- Full-text search
CREATE VIRTUAL TABLE wiki_fts USING fts5(
    title,
    content,
    content='wiki_nodes',
    content_rowid='id'
);

-- Temporal indexing with R-Tree
CREATE VIRTUAL TABLE temporal_index USING rtree(
    id,
    min_date, max_date,
    min_lat, max_lat,
    min_lon, max_lon
);
```

### Repository Pattern

```rust
// truffle-core/src/storage/repositories/mod.rs

#[async_trait]
pub trait Repository<T, ID>: Send + Sync {
    async fn find_by_id(&self, id: ID) -> Result<Option<T>, StorageError>;
    async fn find_all(&self, pagination: Pagination) -> Result<Vec<T>, StorageError>;
    async fn save(&self, entity: &T) -> Result<T, StorageError>;
    async fn delete(&self, id: ID) -> Result<(), StorageError>;
}

#[async_trait]
pub trait WikiRepository: Repository<WikiNode, Uuid> {
    async fn search_fulltext(&self, query: &str) -> Result<Vec<WikiNode>, StorageError>;
    async fn search_semantic(&self, embedding: &Embedding, k: usize) -> Result<Vec<WikiNode>, StorageError>;
    async fn find_backlinks(&self, node_id: Uuid) -> Result<Vec<WikiLink>, StorageError>;
}
```

### Migration System

```rust
// truffle-core/src/storage/migrations/mod.rs

pub struct MigrationManager {
    pool: DatabasePool,
}

impl MigrationManager {
    pub async fn run_migrations(&self) -> Result<MigrationReport, MigrationError> {
        let current = self.get_current_version().await?;
        let migrations = Self::load_migrations();
        
        for migration in migrations.iter().skip(current as usize) {
            let tx = self.pool.begin().await?;
            migration.apply(&tx).await?;
            tx.commit().await?;
        }
        
        Ok(MigrationReport { applied: migrations.len() - current as usize })
    }
}
```

### Configuration

```rust
// Database configuration
pub struct DatabaseConfig {
    pub path: PathBuf,
    pub wal_mode: bool = true,
    pub foreign_keys: bool = true,
    pub busy_timeout_ms: u64 = 5000,
    pub journal_mode: JournalMode = JournalMode::Wal,
    pub synchronous: SynchronousMode = SynchronousMode::Normal,
    pub cache_size_kb: i32 = -64000,  // 64MB
    pub temp_store: TempStore = TempStore::Memory,
}
```

## Performance Characteristics

| Metric | Target | Notes |
|--------|--------|-------|
| Query Latency (simple) | <10ms | Indexed lookups |
| Query Latency (complex join) | <50ms | Multi-table joins |
| Full-text Search | <100ms | FTS5 with ranking |
| Vector Search (k=10) | <50ms | HNSW index |
| Write Throughput | 1000 TPS | WAL mode |
| Concurrent Readers | Unlimited | WAL mode |
| Database Size | <10GB | With 100K screenshots |

## Backup Strategy

```rust
// Litestream integration for optional cloud backup
pub struct BackupConfig {
    pub enabled: bool,
    pub destination: BackupDestination,
    pub retention: Duration,
}

pub enum BackupDestination {
    S3 { bucket: String, region: String },
    S3Compatible { endpoint: String, bucket: String },
    Local { path: PathBuf },
}
```

Note: Backup is **opt-in** and user-controlled. Default is local-only.

## Sync Integration

```rust
// CRDT sync with Yjs
pub struct SyncAdapter {
    db: DatabasePool,
    crdt: YjsDoc,
}

impl SyncAdapter {
    pub async fn apply_update(&self, update: &[u8]) -> Result<(), SyncError> {
        // Apply Yjs CRDT update to SQLite
        let tx = self.db.begin().await?;
        
        for change in self.crdt.apply_update(update)? {
            match change {
                CrdtChange::Insert { table, row } => {
                    self.insert_row(&tx, table, row).await?;
                }
                CrdtChange::Update { table, id, fields } => {
                    self.update_row(&tx, table, id, fields).await?;
                }
                CrdtChange::Delete { table, id } => {
                    self.delete_row(&tx, table, id).await?;
                }
            }
        }
        
        tx.commit().await?;
        Ok(())
    }
}
```

## References

- [SQLite Documentation](https://www.sqlite.org/docs.html)
- [sqlite-vec](https://github.com/asg017/sqlite-vec)
- [Litestream](https://litestream.io/)
- [Yjs Documentation](https://docs.yjs.dev/)
- [Project Truffle Database Schema](./schema/database-schema.sql)

---

**Decision Owner:** CTO  
**Stakeholders:** Engineering, Product  
**Review Date:** 2024-07-15 (6 months)
