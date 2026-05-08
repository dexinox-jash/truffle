# Project Truffle: Complete Directory Structure

> **Classification:** AAA Commercial SaaS | Zero-Knowledge Infrastructure  
> **Version:** 2.0  
> **Last Updated:** 2024  

## Overview

This document defines the complete directory structure for Project Truffle - a Local-First Knowledge Compiler. The project follows a monorepo pattern with clear module boundaries and separation of concerns.

```
truffle/
├── truffle-core/          # Rust core library (AI, storage, sync)
├── truffle-desktop/       # Tauri desktop application
├── truffle-mobile/        # iOS/Android mobile companion
├── truffle-relay/         # Cloudflare Workers relay server
├── truffle-infra/         # Terraform infrastructure definitions
├── truffle-docs/          # Documentation and specifications
├── Cargo.toml            # Workspace root
├── package.json          # Node.js workspace root
├── LICENSE
├── README.md
└── .gitignore
```

---

## 1. truffle-core/ - Rust Core Library

The heart of Truffle. All business logic, AI processing, storage, and cryptography lives here.

```
truffle-core/
├── Cargo.toml                    # Crate manifest
├── build.rs                      # Build script (link native deps)
├── src/
│   ├── lib.rs                    # Public API exports
│   ├── prelude.rs                # Common imports
│   │
│   ├── ai/                       # AI/ML Processing Layer
│   │   ├── mod.rs
│   │   ├── gemma/                # Gemma 4 integration
│   │   │   ├── mod.rs
│   │   │   ├── engine.rs         # llama.cpp wrapper
│   │   │   ├── model.rs          # Model management
│   │   │   ├── tokenizer.rs      # Tokenization
│   │   │   └── prompts/          # Compiled-in prompts
│   │   │       ├── system.txt
│   │   │       └── compilation.txt
│   │   ├── embeddings/           # Vector embeddings (MiniLM)
│   │   │   ├── mod.rs
│   │   │   ├── encoder.rs
│   │   │   └── index.rs          # HNSW vector index
│   │   ├── ocr/                  # OCR subsystem
│   │   │   ├── mod.rs
│   │   │   └── tesseract.rs      # Tesseract wrapper
│   │   └── safety/               # Content safety filters
│   │       ├── mod.rs
│   │       ├── classifier.rs     # NSFW/medical/banking detector
│   │       └── rules.rs          # Safety rules
│   │
│   ├── storage/                  # Data Persistence Layer
│   │   ├── mod.rs
│   │   ├── db.rs                 # SQLite connection management
│   │   ├── migrations/           # Database migrations
│   │   │   ├── mod.rs
│   │   │   ├── 001_initial.sql
│   │   │   ├── 002_add_vectors.sql
│   │   │   └── 003_add_crdt.sql
│   │   ├── models/               # Data models
│   │   │   ├── mod.rs
│   │   │   ├── artifact.rs       # RawArtifact
│   │   │   ├── wiki.rs           # WikiNode
│   │   │   ├── link.rs           # WikiLink
│   │   │   └── queue.rs          # Ingestion queue
│   │   ├── repositories/         # Data access layer
│   │   │   ├── mod.rs
│   │   │   ├── artifact_repo.rs
│   │   │   ├── wiki_repo.rs
│   │   │   └── queue_repo.rs
│   │   └── vector/               # Vector search (sqlite-vec)
│   │       ├── mod.rs
│   │       └── search.rs
│   │
│   ├── compilation/              # Knowledge Compilation Pipeline
│   │   ├── mod.rs
│   │   ├── pipeline.rs           # Main pipeline orchestrator
│   │   ├── queue.rs              # Ingestion queue manager
│   │   ├── schema/               # Schema rules engine
│   │   │   ├── mod.rs
│   │   │   ├── parser.rs         # schema.md parser
│   │   │   ├── rules.rs          # Rule definitions
│   │   │   └── engine.rs         # Rule execution engine
│   │   ├── extractors/           # Entity extractors
│   │   │   ├── mod.rs
│   │   │   ├── receipt.rs
│   │   │   ├── contact.rs
│   │   │   ├── travel.rs
│   │   │   └── generic.rs
│   │   └── graph/                # Knowledge graph construction
│   │       ├── mod.rs
│   │       ├── builder.rs
│   │       ├── resolver.rs       # Entity resolution
│   │       └── linker.rs         # Backlink management
│   │
│   ├── crypto/                   # Cryptography Layer
│   │   ├── mod.rs
│   │   ├── keys.rs               # Key generation & management
│   │   ├── x3dh.rs               # X3DH key exchange (Signal)
│   │   ├── kyber.rs              # Post-quantum Kyber-768
│   │   ├── aes.rs                # AES-256-GCM
│   │   ├── chacha.rs             # ChaCha20-Poly1305
│   │   ├── hkdf.rs               # HKDF-SHA256
│   │   └── enclave.rs            # Secure Enclave/TPM integration
│   │
│   ├── sync/                     # Zero-Knowledge Sync Protocol
│   │   ├── mod.rs
│   │   ├── protocol.rs           # ZKS-1 protocol implementation
│   │   ├── crdt.rs               # Yjs CRDT wrapper
│   │   ├── client.rs             # Sync client
│   │   ├── pairing.rs            # Device pairing ceremony
│   │   ├── message.rs            # SyncMessage types
│   │   └── conflict.rs           # Conflict resolution
│   │
│   ├── export/                   # Export functionality
│   │   ├── mod.rs
│   │   ├── markdown.rs           # Markdown/Obsidian export
│   │   ├── git.rs                # Git repository export
│   │   └── json.rs               # JSON dump
│   │
│   ├── fs/                       # Filesystem abstraction
│   │   ├── mod.rs
│   │   ├── watcher.rs            # Screenshot folder watcher
│   │   ├── paths.rs              # Path management
│   │   └── lock.rs               # File locking
│   │
│   └── telemetry/                # Privacy-respecting telemetry
│       ├── mod.rs
│       └── metrics.rs            # Anonymous metrics
│
├── tests/                        # Integration tests
│   ├── integration_tests.rs
│   ├── fixtures/                 # Test data
│   │   ├── screenshots/
│   │   └── schemas/
│   └── helpers.rs
│
├── benches/                      # Performance benchmarks
│   └── compilation_bench.rs
│
└── examples/                     # Example usage
    └── basic_compilation.rs
```

---

## 2. truffle-desktop/ - Tauri Desktop Application

Primary user interface built with Tauri v2 (Rust backend + Web frontend).

```
truffle-desktop/
├── src-tauri/                    # Rust Tauri backend
│   ├── Cargo.toml
│   ├── build.rs
│   ├── src/
│   │   ├── main.rs               # Application entry point
│   │   ├── lib.rs                # Library exports for testing
│   │   ├── commands/             # Tauri command handlers
│   │   │   ├── mod.rs
│   │   │   ├── artifact.rs       # Screenshot commands
│   │   │   ├── wiki.rs           # Wiki commands
│   │   │   ├── compilation.rs    # Compilation commands
│   │   │   ├── sync.rs           # Sync commands
│   │   │   ├── search.rs         # Search commands
│   │   │   ├── export.rs         # Export commands
│   │   │   └── settings.rs       # Settings commands
│   │   ├── state/                # Application state
│   │   │   ├── mod.rs
│   │   │   └── app_state.rs
│   │   ├── menu/                 # Native menus
│   │   │   ├── mod.rs
│   │   │   └── app_menu.rs
│   │   ├── tray/                 # System tray
│   │   │   ├── mod.rs
│   │   │   └── system_tray.rs
│   │   └── error.rs              # Error handling
│   ├── capabilities/             # Tauri v2 capabilities
│   │   └── default.json
│   └── icons/                    # App icons
│
├── src/                          # Frontend (React + TypeScript)
│   ├── main.tsx                  # Entry point
│   ├── App.tsx                   # Root component
│   ├── index.html
│   ├── styles/
│   │   ├── global.css            # Global styles (Darkroom theme)
│   │   ├── variables.css         # CSS custom properties
│   │   └── animations.css        # Motion definitions
│   │
│   ├── components/               # React components
│   │   ├── ui/                   # Primitive UI components
│   │   │   ├── Button/
│   │   │   ├── Card/
│   │   │   ├── Input/
│   │   │   ├── Modal/
│   │   │   ├── Tooltip/
│   │   │   ├── Progress/
│   │   │   └── index.ts
│   │   │
│   │   ├── layout/               # Layout components
│   │   │   ├── AppLayout.tsx     # Main 3-pane layout
│   │   │   ├── Sidebar.tsx
│   │   │   ├── Header.tsx
│   │   │   └── StatusBar.tsx
│   │   │
│   │   ├── raw/                  # Raw Waterfall components
│   │   │   ├── RawWaterfall.tsx
│   │   │   ├── ThumbnailGrid.tsx
│   │   │   ├── ArtifactCard.tsx
│   │   │   └── DropZone.tsx
│   │   │
│   │   ├── compilation/          # Compilation Preview components
│   │   │   ├── CompilationPreview.tsx
│   │   │   ├── BeforeAfter.tsx
│   │   │   ├── GemmaLogs.tsx
│   │   │   └── ProgressBar.tsx   # Chemical development animation
│   │   │
│   │   ├── wiki/                 # Wiki Navigator components
│   │   │   ├── WikiNavigator.tsx
│   │   │   ├── TreeView.tsx      # Obsidian-style tree
│   │   │   ├── NodeEditor.tsx    # Milkdown editor
│   │   │   ├── Backlinks.tsx
│   │   │   ├── SearchPanel.tsx
│   │   │   └── WikiLink.tsx      # Custom wiki link component
│   │   │
│   │   ├── sync/                 # Sync components
│   │   │   ├── SyncStatus.tsx
│   │   │   ├── DevicePairing.tsx
│   │   │   └── QRCode.tsx
│   │   │
│   │   └── search/               # Search components
│   │       ├── CommandPalette.tsx
│   │       ├── SearchResults.tsx
│   │       └── FilterPanel.tsx
│   │
│   ├── hooks/                    # Custom React hooks
│   │   ├── useArtifacts.ts
│   │   ├── useWiki.ts
│   │   ├── useCompilation.ts
│   │   ├── useSync.ts
│   │   ├── useSearch.ts
│   │   ├── useSettings.ts
│   │   └── useKeyboard.ts        # Keyboard shortcuts
│   │
│   ├── stores/                   # Zustand state stores
│   │   ├── appStore.ts
│   │   ├── artifactStore.ts
│   │   ├── wikiStore.ts
│   │   ├── compilationStore.ts
│   │   ├── syncStore.ts
│   │   └── settingsStore.ts
│   │
│   ├── lib/                      # Utility libraries
│   │   ├── tauri.ts              # Tauri API wrapper
│   │   ├── queryClient.ts        # TanStack Query setup
│   │   ├── flexsearch.ts         # Full-text search
│   │   └── utils.ts
│   │
│   ├── types/                    # TypeScript type definitions
│   │   ├── artifact.ts
│   │   ├── wiki.ts
│   │   ├── sync.ts
│   │   └── index.ts
│   │
│   └── plugins/                  # Editor plugins
│       └── milkdown/
│           ├── WikiLinkPlugin.ts
│           └── index.ts
│
├── public/                       # Static assets
│   ├── fonts/
│   │   ├── Inter-Variable.ttf
│   │   ├── JetBrainsMono.ttf
│   │   └── SourceSerif4.ttf
│   └── images/
│       └── logo.svg
│
├── package.json
├── tsconfig.json
├── vite.config.ts
├── tailwind.config.js            # Darkroom theme colors
├── tauri.conf.json               # Tauri configuration
└── eslint.config.js
```

---

## 3. truffle-mobile/ - Mobile Companion (iOS/Android)

React Native application for screenshot capture and read-only wiki access.

```
truffle-mobile/
├── src/
│   ├── App.tsx                   # Entry point
│   ├── index.js
│   ├── navigation/               # Navigation setup
│   │   ├── AppNavigator.tsx
│   │   ├── MainTabs.tsx
│   │   └── types.ts
│   │
│   ├── screens/                  # Screen components
│   │   ├── HomeScreen.tsx
│   │   ├── CaptureScreen.tsx
│   │   ├── WikiBrowserScreen.tsx
│   │   ├── WikiNodeScreen.tsx
│   │   ├── SearchScreen.tsx
│   │   ├── SyncScreen.tsx
│   │   ├── PairingScreen.tsx
│   │   └── SettingsScreen.tsx
│   │
│   ├── components/               # Shared components
│   │   ├── ui/                   # Primitive components
│   │   │   ├── Button.tsx
│   │   │   ├── Card.tsx
│   │   │   ├── Input.tsx
│   │   │   └── index.ts
│   │   ├── ArtifactThumbnail.tsx
│   │   ├── WikiNodeCard.tsx
│   │   ├── SearchBar.tsx
│   │   ├── SyncStatusBadge.tsx
│   │   └── QRScanner.tsx
│   │
│   ├── hooks/                    # Custom hooks
│   │   ├── useArtifacts.ts
│   │   ├── useWiki.ts
│   │   ├── useSync.ts
│   │   └── useBackgroundSync.ts
│   │
│   ├── services/                 # Native module bridges
│   │   ├── core/                 # truffle-core FFI
│   │   │   ├── CoreModule.ts     # React Native module
│   │   │   └── CoreBridge.ts     # TypeScript wrapper
│   │   ├── share/                # Share extension
│   │   │   ├── ShareExtension.ts
│   │   │   └── ShareViewController.swift
│   │   ├── storage/              # Storage management
│   │   │   └── StorageManager.ts
│   │   └── background/           # Background processing
│   │       └── BackgroundTask.ts
│   │
│   ├── stores/                   # State management
│   │   ├── appStore.ts
│   │   ├── artifactStore.ts
│   │   └── syncStore.ts
│   │
│   ├── utils/                    # Utilities
│   │   ├── constants.ts
│   │   ├── formatting.ts
│   │   └── permissions.ts
│   │
│   └── types/                    # TypeScript types
│       ├── artifact.ts
│       ├── wiki.ts
│       └── index.ts
│
├── ios/                          # iOS native code
│   ├── TruffleMobile/
│   │   ├── AppDelegate.swift
│   │   ├── SceneDelegate.swift
│   │   ├── Info.plist
│   │   └── TruffleMobile.entitlements
│   ├── TruffleMobile.xcodeproj/
│   ├── TruffleShareExtension/    # iOS Share Extension
│   │   ├── ShareViewController.swift
│   │   ├── Info.plist
│   │   └── ShareExtension.entitlements
│   └── Podfile
│
├── android/                      # Android native code
│   ├── app/
│   │   ├── src/main/
│   │   │   ├── java/com/truffle/mobile/
│   │   │   │   ├── MainActivity.kt
│   │   │   │   ├── MainApplication.kt
│   │   │   │   ├── CoreModule.kt           # RN bridge
│   │   │   │   └── CorePackage.kt
│   │   │   ├── res/
│   │   │   └── AndroidManifest.xml
│   │   └── build.gradle
│   ├── build.gradle
│   └── settings.gradle
│
├── share-extension/              # Cross-platform share ext
│   ├── index.js
│   └── config.js
│
├── package.json
├── tsconfig.json
├── babel.config.js
├── metro.config.js
├── react-native.config.js
└── Gemfile
```

---

## 4. truffle-relay/ - Cloudflare Workers Relay

Zero-knowledge relay server for encrypted sync message store-and-forward.

```
truffle-relay/
├── src/
│   ├── index.ts                  # Worker entry point
│   ├── types.ts                  # TypeScript types
│   │
│   ├── handlers/                 # Request handlers
│   │   ├── websocket.ts          # WebSocket connection handler
│   │   ├── message.ts            # Message store/retrieve
│   │   ├── pairing.ts            # Device pairing endpoints
│   │   └── health.ts             # Health check endpoint
│   │
│   ├── storage/                  # R2 storage abstraction
│   │   ├── mod.ts
│   │   ├── message_store.ts      # Message blob storage
│   │   └── rate_limit.ts         # Rate limiting
│   │
│   ├── crypto/                   # Server-side crypto (metadata only)
│   │   ├── mod.ts
│   │   ├── fingerprint.ts        # Device fingerprinting
│   │   └── validation.ts         # Request validation
│   │
│   └── middleware/               # Middleware
│       ├── auth.ts               # Authentication
│       ├── rate_limit.ts         # Rate limiting
│       └── cors.ts               # CORS handling
│
├── tests/
│   ├── websocket.test.ts
│   ├── message.test.ts
│   └── pairing.test.ts
│
├── wrangler.toml                 # Cloudflare Workers config
├── wrangler.prod.toml            # Production config
├── package.json
├── tsconfig.json
├── jest.config.js
└── README.md
```

---

## 5. truffle-infra/ - Terraform Infrastructure

Infrastructure-as-Code for all cloud resources.

```
truffle-infra/
├── environments/
│   ├── dev/
│   │   ├── main.tf
│   │   ├── variables.tf
│   │   ├── outputs.tf
│   │   └── terraform.tfvars
│   ├── staging/
│   │   ├── main.tf
│   │   ├── variables.tf
│   │   ├── outputs.tf
│   │   └── terraform.tfvars
│   └── prod/
│       ├── main.tf
│       ├── variables.tf
│       ├── outputs.tf
│       └── terraform.tfvars
│
├── modules/
│   ├── relay/                    # Cloudflare Workers + R2
│   │   ├── main.tf
│   │   ├── variables.tf
│   │   ├── outputs.tf
│   │   └── README.md
│   │
│   ├── cdn/                      # Cloudflare CDN config
│   │   ├── main.tf
│   │   ├── variables.tf
│   │   ├── outputs.tf
│   │   └── README.md
│   │
│   ├── dns/                      # DNS management
│   │   ├── main.tf
│   │   ├── variables.tf
│   │   └── outputs.tf
│   │
│   ├── monitoring/               # Grafana Cloud
│   │   ├── main.tf
│   │   ├── variables.tf
│   │   ├── outputs.tf
│   │   └── dashboards/
│   │       ├── sync-dashboard.json
│   │       └── relay-dashboard.json
│   │
│   └── iam/                      # Access management
│       ├── main.tf
│       ├── variables.tf
│       └── outputs.tf
│
├── global/
│   ├── main.tf                   # Global resources
│   ├── variables.tf
│   └── outputs.tf
│
├── scripts/
│   ├── init-backend.sh           # Initialize Terraform backend
│   ├── plan.sh                   # Run terraform plan
│   └── apply.sh                  # Run terraform apply
│
├── .terraformignore
├── backend.tf                    # Terraform Cloud backend config
├── providers.tf                  # Provider configurations
├── versions.tf                   # Provider version constraints
└── README.md
```

---

## 6. truffle-docs/ - Documentation

Complete project documentation.

```
truffle-docs/
├── README.md                     # Documentation index
│
├── architecture/                 # Architecture documentation
│   ├── OVERVIEW.md               # High-level architecture
│   ├── DATA_FLOW.md              # Data flow diagrams
│   ├── SECURITY.md               # Security architecture
│   ├── SYNC_PROTOCOL.md          # ZKS-1 protocol spec
│   └── AI_PIPELINE.md            # AI compilation pipeline
│
├── api/                          # API documentation
│   ├── CORE_API.md               # truffle-core Rust API
│   ├── TAURI_COMMANDS.md         # Tauri command reference
│   ├── SYNC_WEBSOCKET.md         # Sync WebSocket protocol
│   └── EXPORT_FORMATS.md         # Export format specs
│
├── development/                  # Developer guides
│   ├── SETUP.md                  # Development environment
│   ├── BUILD.md                  # Build instructions
│   ├── TESTING.md                # Testing guide
│   ├── DEBUGGING.md              # Debugging tips
│   └── CONTRIBUTING.md           # Contribution guidelines
│
├── deployment/                   # Deployment guides
│   ├── DESKTOP_RELEASE.md
│   ├── MOBILE_RELEASE.md
│   ├── RELAY_DEPLOYMENT.md
│   └── ENTERPRISE_SELF_HOST.md
│
├── user/                         # User documentation
│   ├── GETTING_STARTED.md
│   ├── CAPTURE.md
│   ├── COMPILATION.md
│   ├── WIKI.md
│   ├── SYNC.md
│   ├── EXPORT.md
│   └── FAQ.md
│
├── security/                     # Security documentation
│   ├── THREAT_MODEL.md
│   ├── CRYPTOGRAPHY.md
│   ├── COMPLIANCE.md
│   ├── PRIVACY_POLICY.md
│   └── INCIDENT_RESPONSE.md
│
├── adrs/                         # Architecture Decision Records
│   ├── ADR-001-tauri-over-electron.md
│   ├── ADR-002-gemma-over-llama.md
│   ├── ADR-003-sqlite-over-nosql.md
│   ├── ADR-004-zero-knowledge-over-e2e.md
│   └── TEMPLATE.md
│
├── schemas/                      # Schema templates
│   ├── receipts.schema.md
│   ├── contacts.schema.md
│   ├── travel.schema.md
│   ├── design-inspiration.schema.md
│   └── SCHEMA_LANGUAGE.md
│
└── assets/                       # Documentation assets
    ├── diagrams/
    ├── screenshots/
    └── logos/
```

---

## Root Configuration Files

### Cargo.toml (Workspace Root)
```toml
[workspace]
members = [
    "truffle-core",
    "truffle-desktop/src-tauri",
]
resolver = "2"

[workspace.package]
version = "0.1.0"
edition = "2021"
authors = ["Truffle Team <team@truffle.io>"]
license = "AGPL-3.0"
repository = "https://github.com/truffle/truffle"

[workspace.dependencies]
# Core dependencies shared across crates
serde = { version = "1.0", features = ["derive"] }
tokio = { version = "1.35", features = ["full"] }
```

### package.json (Node.js Workspace Root)
```json
{
  "name": "truffle-monorepo",
  "version": "0.1.0",
  "private": true,
  "workspaces": [
    "truffle-desktop",
    "truffle-mobile",
    "truffle-relay"
  ],
  "scripts": {
    "desktop:dev": "cd truffle-desktop && npm run dev",
    "desktop:build": "cd truffle-desktop && npm run build",
    "mobile:ios": "cd truffle-mobile && npx react-native run-ios",
    "mobile:android": "cd truffle-mobile && npx react-native run-android",
    "relay:dev": "cd truffle-relay && npm run dev",
    "relay:deploy": "cd truffle-relay && npm run deploy",
    "test": "npm run test --workspaces",
    "lint": "npm run lint --workspaces"
  },
  "devDependencies": {
    "@types/node": "^20.0.0",
    "typescript": "^5.3.0"
  }
}
```

---

## File Naming Conventions

| Pattern | Usage |
|---------|-------|
| `snake_case.rs` | Rust source files |
| `PascalCase.tsx` | React components |
| `camelCase.ts` | TypeScript utilities, hooks |
| `kebab-case.md` | Documentation files |
| `SCREAMING_SNAKE_CASE` | Constants, environment variables |
| `mod.rs` | Rust module entry points |
| `index.ts` | TypeScript barrel exports |

---

## Module Dependencies

```
                    ┌─────────────────┐
                    │  truffle-relay  │
                    │  (Cloudflare)   │
                    └────────┬────────┘
                             │ HTTPS/WSS
                             ▼
┌──────────────┐    ┌─────────────────┐    ┌──────────────┐
│truffle-mobile│◄──►│  truffle-core   │◄──►│truffle-desktop│
│ (iOS/Android)│ FFI │    (Rust lib)   │ FFI │   (Tauri)    │
└──────────────┘    └─────────────────┘    └──────────────┘
                             │
                             │ SQLite/FS
                             ▼
                    ┌─────────────────┐
                    │  User Device    │
                    │  (Local-First)  │
                    └─────────────────┘
```

---

**Document Owner:** Systems Architect  
**Review Cycle:** Monthly  
**Classification:** Internal Use
