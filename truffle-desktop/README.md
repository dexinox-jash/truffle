# Truffle Desktop

**Local-First Knowledge Compiler** - Transform screenshots into structured, interlinked knowledge graphs.

[![Version](https://img.shields.io/badge/version-0.1.0-FFB800)](https://github.com/truffle/truffle-desktop)
[![License](https://img.shields.io/badge/license-AGPL--3.0-00D4AA)](LICENSE)

## Overview

Truffle is a privacy-first desktop application that uses on-device AI to compile unstructured visual data (screenshots) into structured knowledge graphs. Your data never leaves your device.

## Features

### Core
- **Raw Waterfall**: Browse and manage screenshot artifacts
- **Compilation Preview**: Watch AI transform screenshots into wiki nodes
- **Wiki Navigator**: Obsidian-style tree view for knowledge graph
- **Split Soul Interface**: 3-pane layout (Raw | Compilation | Wiki)

### AI-Powered Compilation
- On-device Gemma 4 inference via llama.cpp
- OCR text extraction
- Entity recognition and linking
- Temporal reference extraction
- Automatic wiki node creation

### Search & Discovery
- Full-text search with FlexSearch
- Semantic search with sqlite-vec
- Backlink navigation
- Related content suggestions

### Sync & Collaboration
- Zero-knowledge sync protocol (ZKS-1)
- Yjs CRDT for conflict-free collaboration
- X3DH key exchange with post-quantum hybrid
- End-to-end encrypted device pairing

### Privacy & Security
- 100% offline capable
- No cloud dependency
- Local-first architecture
- Zero-knowledge infrastructure

## Tech Stack

### Frontend
- **Framework**: React 18 + TypeScript
- **Build Tool**: Vite 5
- **Styling**: TailwindCSS (Darkroom design system)
- **State**: Zustand + TanStack Query
- **Editor**: Milkdown (Markdown WYSIWYM)
- **Search**: FlexSearch

### Backend (Tauri)
- **Runtime**: Rust + Tauri v2
- **Database**: SQLite with sqlite-vec
- **AI**: llama.cpp + Gemma 4
- **Sync**: Yjs CRDT
- **Crypto**: ring (AES-256-GCM, Ed25519, X25519)

## Design System: Darkroom

```
Background:    #0A0A0A (Pure Black OLED)
Surface:       #141414
Primary:       #FFB800 (Amber)
Secondary:     #00D4AA (Teal)
Text Primary:  #E5E5E5
Text Secondary:#737373
```

## Development

### Prerequisites
- Node.js 18+
- Rust 1.70+
- Tauri CLI

### Setup

```bash
# Clone repository
git clone https://github.com/truffle/truffle-desktop.git
cd truffle-desktop

# Install dependencies
npm install

# Run development server
npm run tauri:dev

# Build for production
npm run tauri:build
```

### Project Structure

```
truffle-desktop/
├── src/                    # React frontend
│   ├── components/         # React components
│   │   ├── Layout/         # App layout, header, status bar
│   │   ├── Raw/            # Raw waterfall panel
│   │   ├── Compilation/    # Compilation preview panel
│   │   ├── Wiki/           # Wiki navigator panel
│   │   ├── Editor/         # Milkdown editor
│   │   ├── Search/         # Search components
│   │   └── CommandPalette/ # Cmd+K interface
│   ├── store/              # Zustand stores
│   ├── hooks/              # Custom React hooks
│   ├── types/              # TypeScript types
│   ├── utils/              # Utility functions
│   └── styles/             # Global styles
├── src-tauri/              # Rust backend
│   └── src/
│       ├── main.rs         # Tauri entry point
│       ├── commands.rs     # Command handlers
│       ├── database.rs     # SQLite operations
│       ├── compilation.rs  # AI compilation
│       ├── sync.rs         # CRDT sync
│       ├── crypto.rs       # Cryptographic operations
│       └── schema.rs       # Schema validation
└── public/                 # Static assets
```

## Keyboard Shortcuts

| Shortcut | Action |
|----------|--------|
| `Cmd/Ctrl + K` | Open command palette |
| `Cmd/Ctrl + Shift + F` | Open search |
| `Cmd/Ctrl + 1` | Switch to Raw view |
| `Cmd/Ctrl + 2` | Switch to Split view |
| `Cmd/Ctrl + 3` | Switch to Wiki view |
| `Esc` | Close modals |

## Architecture

### Data Model

```
RawArtifact (Immutable)
├── UUID (content-addressable)
├── Binary (local filesystem)
├── Metadata (OCR, embeddings)
└── Status (pending/compiled/failed)

WikiNode (Compiled Knowledge)
├── ID, Type, Title
├── Content (Markdown + WikiLinks)
├── Backlinks/Forward Links
├── Provenance (audit trail)
└── Version (CRDT vector clock)
```

### Compilation Pipeline

1. **Ingestion**: Queue screenshot for processing
2. **Multimodal Processing**: Gemma 4 analyzes image
3. **Knowledge Graph Construction**: Create/update nodes
4. **Persistence**: Write to SQLite, trigger sync

### Sync Protocol (ZKS-1)

1. **Key Exchange**: X3DH with Kyber-768 hybrid
2. **Encryption**: AES-256-GCM per message
3. **CRDT**: Yjs YATA algorithm
4. **Transport**: WebSocket relay (zero-knowledge)

## License

AGPL-3.0 - See [LICENSE](LICENSE) for details.

## Contributing

We welcome contributions! Please see [CONTRIBUTING.md](CONTRIBUTING.md) for guidelines.

## Security

For security issues, please email security@truffle.io instead of using the issue tracker.

## Acknowledgments

- [Tauri](https://tauri.app/) - Desktop framework
- [Milkdown](https://milkdown.dev/) - Markdown editor
- [Yjs](https://yjs.dev/) - CRDT library
- [llama.cpp](https://github.com/ggerganov/llama.cpp) - LLM inference
- [Gemma](https://ai.google.dev/gemma) - Open models

---

**Your Visual Memory, Compiled.**
