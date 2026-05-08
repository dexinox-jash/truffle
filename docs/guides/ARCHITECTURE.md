# Truffle Architecture

## Overview

Truffle is a Local-First Knowledge Compiler (LFKC) that transforms meeting transcripts and documents into structured knowledge graphs using on-device AI.

## Core Principles

### 1. Local-First
- All data stored locally in SQLite
- AI models run on-device (Gemma 4B)
- Works offline indefinitely

### 2. Zero-Knowledge
- End-to-end encryption
- Infrastructure cannot decrypt user data
- Cryptographic guarantees

### 3. Knowledge Graph
- Entities (people, orgs, locations, etc.)
- Relationships between entities
- Temporal validity tracking

## System Architecture

```
┌─────────────────────────────────────────────────────────────┐
│                    Truffle Desktop App                       │
├─────────────────────────────────────────────────────────────┤
│  React UI      │  Tauri Bridge  │  Rust Backend             │
│  ─────────     │  ────────────  │  ────────────             │
│  Components    │  Commands      │  truffle-core             │
│  Hooks         │  Events        │  truffle-crypto           │
│  Stores        │  State         │                           │
└─────────────────────────────────────────────────────────────┘
                              │
                    ┌─────────┴─────────┐
                    ▼                   ▼
              SQLite (WAL)        Local Filesystem
```

## Backend Architecture (truffle-core)

### Layer Structure
```
truzzle-core/src/
├── models/           # Data structures
├── database/         # SQLite persistence
├── pipeline/         # Data processing
│   └── extraction/   # Entity extraction
├── graph/            # Graph operations
├── validation/       # Validation engine
├── api/              # GraphQL API
└── sync/             # CRDT sync
```

### Knowledge Graph Pipeline

```
Meeting Transcript
      ↓
[Preprocessing] → Text cleaning, segmentation
      ↓
[Tokenization] → Sentence/word boundaries
      ↓
[NER] → Named entity recognition (Gemma 4B)
      ↓
[Resolution] → Entity linking, deduplication
      ↓
[Enrichment] → Relationship extraction
      ↓
[Persistence] → SQLite storage
      ↓
[Validation] → Contradiction detection
```

### Database Schema

#### Core Tables
- **entities** - Knowledge graph nodes
- **relationships** - Graph edges
- **meetings** - Meeting transcripts
- **decisions** - Tracked decisions
- **action_items** - Action items
- **contradictions** - Validation issues

#### Full-Text Search
- **entity_search** - FTS5 virtual table
- **meeting_search** - FTS5 virtual table

### Validation Engine

```
Facts
  ↓
[Contradiction Detection] → Temporal/factual conflicts
  ↓
[Confidence Scoring] → Bayesian updating
  ↓
[Duplicate Detection] → Similarity matching
  ↓
[Reports] → Validation summary
```

## Frontend Architecture

### Component Hierarchy
```
App
├── AppLayout
│   ├── Header
│   ├── Navigation
│   ├── Main Content
│   │   ├── EntitiesPage
│   │   │   ├── EntityList
│   │   │   └── EntityDetail
│   │   ├── GraphPage
│   │   │   └── GraphViewer
│   │   ├── ValidationPage
│   │   │   ├── ValidationSummary
│   │   │   └── ContradictionList
│   │   └── MeetingsPage
│   └── StatusBar
└── Modals
```

### State Management

#### Server State (React Query)
- Entity data
- Graph data
- Validation results
- Cached with stale-while-revalidate

#### UI State (Zustand)
- Selected entity
- View mode
- Filter settings
- Modal state

### Data Flow

```
User Action
    ↓
React Component
    ↓
Hook (useEntities, etc.)
    ↓
Tauri Command
    ↓
Rust Backend
    ↓
SQLite Database
```

## Security Architecture

### Encryption Layers
1. **Data at Rest** - SQLite encrypted with SQLCipher
2. **Sync Data** - X3DH + Kyber-768 hybrid encryption
3. **Export Data** - AES-256-GCM with password

### Key Management
- Keys generated on-device
- No server-side key storage
- Hardware-backed when available (Secure Enclave, TPM)

## Performance Considerations

### Database
- WAL mode for concurrent reads/writes
- Indexes on frequently queried columns
- Connection pooling

### Graph Operations
- BFS/DFS with depth limits
- Cached similarity scores
- Lazy relationship loading

### Frontend
- Virtualized lists for large datasets
- Debounced search inputs
- Code splitting by route

## Scalability Limits

| Component | Limit | Notes |
|-----------|-------|-------|
| Entities | 100,000 | SQLite practical limit |
| Relationships | 1,000,000 | With indexing |
| Graph Depth | 10 | Query limit |
| Concurrent Users | 1 | Local-first design |

## Technology Stack

### Backend
- **Language**: Rust 1.70+
- **Database**: SQLite 3.40+ (WAL mode)
- **AI**: llama.cpp (Gemma 4B)
- **Sync**: Yjs/Yrs (CRDTs)

### Frontend
- **Framework**: React 18
- **Build**: Vite 5
- **Desktop**: Tauri 2.0
- **Styling**: Tailwind CSS
- **State**: Zustand + React Query

## Deployment

### Desktop App
- **macOS**: .dmg, .app
- **Windows**: .msi, .exe
- **Linux**: .deb, .rpm, AppImage

### Updates
- Built-in auto-updater (Tauri)
- Delta updates for efficiency
- Rollback capability

## Monitoring

### Metrics
- Entity count
- Query performance
- Validation issues
- Sync status

### Logs
- Structured logging (tracing in Rust)
- Log rotation
- Export capability
