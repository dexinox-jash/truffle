# Getting Started with Truffle

## Prerequisites

### Required Software
- **Rust** 1.70+ (install via [rustup](https://rustup.rs/))
- **Node.js** 18+ (install via [nvm](https://github.com/nvm-sh/nvm))
- **pnpm** 8+ (`npm install -g pnpm`)

### Optional Tools
- **cargo-watch** - Auto-rebuild on changes
- **cargo-tarpaulin** - Code coverage
- **just** - Command runner

## Quick Start

### 1. Clone Repository
```bash
git clone https://github.com/your-org/truffle.git
cd truffle
```

### 2. Install Dependencies
```bash
# Install Rust dependencies
cargo fetch

# Install Node.js dependencies
pnpm install
```

### 3. Set Up Database
```bash
# Database will be created automatically on first run
# Default location: ~/.truffle/data/
```

### 4. Run Development Server
```bash
# Terminal 1: Start Tauri dev server
pnpm tauri dev

# Terminal 2: Start Vite dev server (if not using tauri dev)
pnpm dev
```

### 5. Open Application
The app will open automatically. Default URL: http://localhost:1420

## Development Workflow

### Backend Development
```bash
# Run Rust tests
cargo test --workspace

# Run with hot reload
cargo watch -x test

# Check code formatting
cargo fmt --check

# Run linter
cargo clippy --workspace -- -D warnings
```

### Frontend Development
```bash
# Run TypeScript checks
pnpm typecheck

# Run linter
pnpm lint

# Run tests
pnpm test

# Run with coverage
pnpm test:coverage
```

### Database Migrations
Migrations run automatically. To force a migration:
```bash
cargo run --bin truffle-core -- migrate
```

## Project Structure

```
truffle/
├── truffle-core/        # Rust backend (database, models, API)
├── truffle-desktop/     # Tauri desktop app
│   ├── src/            # React frontend
│   └── src-tauri/      # Rust Tauri commands
├── truffle-crypto/      # Cryptographic primitives
└── docs/               # Documentation
```

## Common Tasks

### Add New Entity Type
1. Update `EntityType` enum in `truffle-core/src/models/entity.rs`
2. Add GraphQL type in `truffle-core/src/api/schema.rs`
3. Update frontend types in `truffle-desktop/src/types/knowledge-graph.ts`
4. Add UI component in `truffle-desktop/src/components/Entity/EntityTypeBadge.tsx`

### Add New GraphQL Query
1. Add resolver in `truffle-core/src/api/resolvers/query.rs`
2. Add field to schema in `truffle-core/src/api/schema.rs`
3. Add Tauri command in `truffle-desktop/src-tauri/src/knowledge_graph.rs`
4. Add hook in `truffle-desktop/src/hooks/useEntities.ts`

## Next Steps
- Read [Architecture Overview](./ARCHITECTURE.md)
- Learn about [Contributing](./CONTRIBUTING.md)
- Explore [API Reference](../api/GRAPHQL_REFERENCE.md)
