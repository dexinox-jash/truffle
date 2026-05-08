# Project Truffle — AI Development Rules

> **Product**: Local-First Knowledge Compiler (LFKC)  
> **Classification**: AAA Commercial SaaS | Zero-Knowledge Infrastructure  
> **Version**: 0.1.0  

---

## IDENTITY

Project Truffle transforms screenshots into structured knowledge graphs using on-device AI.  
Every line of code must honor **The Five Red Lines** — non-negotiable constraints that define this product.

---

## THE FIVE RED LINES (NEVER VIOLATE)

```
1. SOVEREIGNTY     — User data (images) stays on device. No exceptions for processing, thumbnails, caching.
2. ZERO-KNOWLEDGE  — We (infrastructure operators) CANNOT decrypt user content. Mathematical guarantee.
3. SURVIVAL MODE   — App works 100% offline indefinitely. Only sync degrades.
4. EXIT CAPABILITY — Full export to markdown/git in <5 minutes, no internet, no auth.
5. ECONOMICS       — 85%+ gross margin at $6 ARPU with 50K users (currently 92.07%).
```

If ANY proposed change would violate a Red Line, **STOP and flag it immediately**.

---

## MODULE MAP

| Module | Language | Purpose | Owner Agent |
|--------|----------|---------|-------------|
| `truffle-core` | Rust | Database, models, pipeline, sync | backend |
| `truffle-crypto` | Rust | Zero-knowledge crypto primitives | security |
| `truffle-ai` | Rust | Gemma 4, OCR, embeddings, safety | backend |
| `truffle-desktop` | Rust + React/TS | Tauri desktop application | frontend + backend |
| `truffle-relay` | TypeScript | Cloudflare Workers relay server | devops |
| `truffle-mobile` | React Native | iOS/Android companion | frontend |
| `truffle-infra` | Terraform | Cloud infrastructure | devops |
| `truffle-qa` | Mixed | Testing & compliance | qa |

---

## CODING STANDARDS

### Rust
- `#![warn(missing_docs)]` — All public items must be documented
- `#![warn(rust_2018_idioms)]` — Modern Rust patterns
- **NEVER** use `.unwrap()` or `.expect()` in production code — use `?` or proper error handling
- **NEVER** use `unsafe` without a safety comment and security review
- All crypto code must use `zeroize` for sensitive data
- All crypto comparisons must be constant-time (`subtle::ConstantTimeEq`)
- Prefer `thiserror` for library errors, `anyhow` for application errors
- Test names: `test_<unit>_<scenario>_<expected_result>`

### TypeScript
- Strict mode always (`"strict": true` in tsconfig)
- No `any` — use proper types or `unknown`
- All Tauri commands must have typed request/response DTOs
- React: functional components only, hooks for state
- Import order: react → external → internal → types

### General
- Files under 500 lines (split if longer)
- No secrets, credentials, or `.env` files committed
- Read before edit — always understand context first
- Prefer editing existing files over creating new ones

---

## BUILD COMMANDS

```bash
# Verify workspace compiles (run from project root)
cargo check --workspace

# Run all Rust tests
cargo test --workspace

# Lint Rust
cargo clippy --workspace -- -D warnings

# Relay TypeScript
cd truffle-relay && npm run typecheck

# Desktop TypeScript
cd truffle-desktop && npm run typecheck
```

---

## AGENT COORDINATION

This project uses the **Ruflo agents army** pattern (`.agents/` directory).  
8 specialized agents coordinate via hierarchical topology:

| Agent | Role | Expertise |
|-------|------|-----------|
| `architect` | System design, module boundaries, API design | Architecture, ADRs |
| `security` | Crypto audit, zero-knowledge verification, threat modeling | Cryptography, compliance |
| `backend` | Rust core, database, pipeline, sync implementation | Rust, SQLite, CRDTs |
| `frontend` | React/TypeScript UI, Darkroom design system | React, Tauri, CSS |
| `devops` | CI/CD, builds, deployment, monitoring | GitHub Actions, Cloudflare |
| `qa` | Test plans, integration tests, red line verification | Testing, compliance |
| `reviewer` | Code reviews, standards enforcement | Code quality |
| `researcher` | Requirements analysis, competitive research | Analysis |

### Anti-Drift Rules
- All work must align with a Red Line or stated goal
- No speculative features — build what's needed, nothing more
- Every change must be verifiable (`cargo check`, `cargo test`, `npm run typecheck`)
- Status markers (`✅ COMPLETE`) require passing CI, not just code existing

---

## FILE ORGANIZATION

```
truffle/
├── .agents/          # Agent configurations
├── .code-review/     # Code review graph config
├── .github/          # CI/CD workflows
├── .security/        # Security policies and threat model
├── .skills/          # AI development skills (read-only reference)
├── ADRs/             # Architecture Decision Records
├── monitoring/       # Grafana dashboards
├── scripts/          # Build, deploy, release scripts
├── truffle-ai/       # AI/ML engine (Rust)
├── truffle-core/     # Core library (Rust)
├── truffle-crypto/   # Cryptography (Rust)
├── truffle-desktop/  # Desktop app (Tauri + React)
├── truffle-infra/    # Infrastructure (Terraform)
├── truffle-mobile/   # Mobile app (React Native)
├── truffle-qa/       # Testing & compliance
├── truffle-relay/    # Relay server (Cloudflare Workers)
├── Cargo.toml        # Rust workspace root
├── CLAUDE.md         # THIS FILE — AI development rules
├── STATUS.md         # Honest project status (verified markers only)
└── README.md         # Project overview
```

---

## SECURITY-FIRST DEVELOPMENT

1. **No data leaves device unencrypted** — verified by network isolation tests
2. **Keys generated on-device only** — no key escrow, no server-side key access
3. **Relay is zero-access** — server cannot decrypt, verified by `grep -r "private_key" truffle-relay/`
4. **All crypto operations constant-time** — prevents timing attacks
5. **Dependencies audited** — `cargo audit` and `npm audit` must pass
6. **Secrets scanning** — pre-commit hooks prevent credential leaks

---

## DECISION LOG

All significant technical decisions are recorded in `ADRs/`:
- ADR-001: Tauri over Electron (performance, security, binary size)
- ADR-002: Gemma over Llama (multimodal, license, size)
- ADR-003: SQLite over NoSQL (local-first, WAL, vector search)
- ADR-004: Zero-Knowledge over E2E (mathematical guarantee vs. trust-based)

New decisions require a new ADR before implementation.
