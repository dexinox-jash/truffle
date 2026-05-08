# Truffle Core

Local-First Knowledge Compiler - Core Rust Backend

## Overview

Truffle Core is the Rust backend for Project Truffle, implementing a local-first knowledge compilation system that transforms screenshots into structured, interlinked knowledge graphs using on-device AI.

## Architecture

```
┌─────────────────────────────────────────────────────────────┐
│                    Truffle Core Library                      │
├─────────────────────────────────────────────────────────────┤
│  Models    │  Database   │  Pipeline   │  Sync             │
│  ───────── │  ─────────  │  ─────────  │  ────             │
│  RawArtifact│  Connection│  Ingestion  │  CRDT (Yrs)       │
│  WikiNode  │  Migrations │  Processor  │  Delta            │
│  Schema    │  Queries    │  Graph      │  Crypto           │
│            │             │  Persistence│                   │
└─────────────────────────────────────────────────────────────┘
```

## Features

### Data Models (Section 2.1)

- **RawArtifact**: Immutable capture of visual data with content-addressable UUIDs (SHA256)
- **WikiNode**: Compiled, structured knowledge with bidirectional WikiLinks
- **SchemaRuleset**: AI compilation rules with privacy classifications

### Database (SQLite with WAL)

- WAL mode for concurrent reads/writes
- ACID transactions
- Full-text search (FTS5)
- Vector search (sqlite-vec)
- Temporal indexing (R-Tree)

### Compilation Pipeline (Section 2.2)

1. **Ingestion Queue**: Priority-based queue (P0-P2)
2. **Multimodal Processing**: AI-powered content analysis interface
3. **Knowledge Graph Construction**: Entity resolution and linking
4. **Persistence & Sync Trigger**: ACID persistence with CRDT generation

### CRDT Sync (Section 2.3)

- Yjs/Yrs integration (YATA algorithm)
- Zero-knowledge encryption (AES-256-GCM)
- X3DH key exchange
- Device pairing with SAS verification

## Quick Start

```rust
use truffle_core::*;
use std::sync::Arc;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Initialize library
    truffle_core::init();
    
    // Open database
    let config = DatabaseConfig::with_data_dir("~/.truffle");
    let db = Arc::new(Database::open(config)?);
    
    // Create and start pipeline
    let pipeline_config = PipelineConfig::default();
    let mut pipeline = CompilationPipeline::new(pipeline_config, db.clone())?;
    pipeline.start().await?;
    
    // Create a raw artifact
    let artifact = RawArtifact::new(
        "screenshot.png",
        &image_bytes,
        storage_path,
        DeviceFingerprint::new(&device_info, b"salt"),
        chrono::Utc::now(),
    )?;
    
    // Submit for processing
    let entry_id = pipeline.submit(artifact, 0).await?;
    
    Ok(())
}
```

## Project Structure

```
truffle-core/
├── Cargo.toml              # Dependencies
├── src/
│   ├── lib.rs              # Library exports
│   ├── models/
│   │   ├── mod.rs          # Common types
│   │   ├── raw_artifact.rs # RawArtifact model
│   │   ├── wiki_node.rs    # WikiNode model
│   │   └── schema.rs       # SchemaRuleset model
│   ├── database/
│   │   ├── mod.rs          # Database module
│   │   ├── connection.rs   # SQLite connection
│   │   ├── migrations.rs   # Schema migrations
│   │   └── queries.rs      # CRUD operations
│   ├── pipeline/
│   │   ├── mod.rs          # Pipeline module
│   │   ├── ingestion.rs    # Stage 1: Ingestion Queue
│   │   ├── processor.rs    # Stage 2: Multimodal Processing
│   │   ├── graph.rs        # Stage 3: Knowledge Graph
│   │   └── persistence.rs  # Stage 4: Persistence & Sync
│   └── sync/
│       ├── mod.rs          # Sync module
│       ├── crdt.rs         # Yjs/Yrs CRDT
│       ├── delta.rs        # Delta generation
│       └── crypto.rs       # Encryption & key exchange
└── tests/
    └── integration_tests.rs # Integration tests
```

## Dependencies

### Core
- `tokio` - Async runtime
- `rusqlite` - SQLite bindings
- `yrs` - Yjs CRDT for Rust
- `serde` - Serialization

### Cryptography
- `aes-gcm` - AES-256-GCM encryption
- `ed25519-dalek` - Ed25519 signatures
- `sha2` / `sha3` - Hashing
- `hkdf` - Key derivation

### Data Processing
- `uuid` - UUID generation
- `chrono` - Date/time handling
- `regex` - Pattern matching

## Testing

Run all tests:

```bash
cargo test
```

Run with coverage:

```bash
cargo tarpaulin --out Html
```

## Compliance

- **SOC 2**: Logical access controls, encryption, key management
- **GDPR**: Pseudonymization, encryption, data residency
- **Local-First**: All data stays on device

## License

AGPL-3.0

## Specification Compliance

This implementation follows the **Enterprise Master Specification v2.0** for Project Truffle:

- Section 2.1: Data Model (RawArtifact, WikiNode, Schema)
- Section 2.2: Compilation Pipeline (4 stages)
- Section 2.3: Zero-Knowledge Sync Protocol (ZKS-1)
