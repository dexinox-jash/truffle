# ADR-001: Tauri over Electron for Desktop Application

## Status

**Accepted** | Date: 2024-01-15 | Author: Systems Architect

## Context

Project Truffle requires a cross-platform desktop application that serves as the primary user interface. The application must:

- Run on macOS, Windows, and Linux
- Provide native performance for AI processing integration
- Maintain a small bundle size for faster downloads and updates
- Minimize memory footprint (critical for running alongside Gemma 4)
- Support secure, sandboxed execution
- Enable seamless integration with the Rust-based AI core

Two primary options were evaluated:
1. **Electron** (Node.js + Chromium)
2. **Tauri** (Rust + WebKit/WebView2)

## Decision

We will use **Tauri v2** as the desktop application framework.

## Consequences

### Positive

| Aspect | Tauri Advantage |
|--------|-----------------|
| **Bundle Size** | ~3-5MB vs Electron's ~150MB |
| **Memory Usage** | ~50MB idle vs Electron's ~300MB+ |
| **Startup Time** | <1 second vs Electron's 3-5 seconds |
| **Security** | Rust memory safety, OS-native sandbox |
| **AI Integration** | Direct FFI to truffle-core, zero IPC overhead |
| **Code Signing** | Native support for all platforms |
| **Auto-Update** | Built-in, secure update mechanism |
| **Mobile Path** | Tauri v2 supports iOS/Android (future expansion) |

### Negative

| Aspect | Tauri Challenge | Mitigation |
|--------|-----------------|------------|
| **Ecosystem Size** | Smaller than Electron's | Core needs covered; custom plugins acceptable |
| **Web APIs** | Limited to WebKit/WebView2 | Feature detection, polyfills where needed |
| **Debugging** | Less mature DevTools | Use Safari/Edge DevTools; Rust debugging via VS Code |
| **Learning Curve** | Team needs Rust knowledge | Training budget allocated; Rust expertise hired |

## Alternatives Considered

### Electron

**Pros:**
- Mature ecosystem with extensive plugin library
- Large community and documentation
- Familiar to JavaScript developers

**Cons:**
- Bundle size: 150MB+ (unacceptable for our distribution strategy)
- Memory usage: 300MB+ baseline (conflicts with Gemma 4's 4GB requirement)
- Security: Node.js integration increases attack surface
- Performance: V8 overhead for compute-intensive tasks

**Verdict:** Rejected due to resource constraints and security concerns.

### Flutter Desktop

**Pros:**
- Native performance
- Single codebase for mobile and desktop
- Google's backing

**Cons:**
- Dart learning curve for team
- Rust integration requires complex FFI
- Larger bundle size than Tauri
- Less mature desktop support

**Verdict:** Rejected due to FFI complexity and team expertise.

### Native (Swift/C#)

**Pros:**
- Maximum performance
- Native look and feel

**Cons:**
- Separate codebases per platform
- 3x development effort
- Harder to maintain consistency

**Verdict:** Rejected due to resource constraints.

## Implementation Notes

### Frontend Stack

```
Tauri v2
├── React 18 (UI framework)
├── TypeScript 5.3 (type safety)
├── Zustand (state management)
├── TanStack Query (data fetching)
├── Milkdown (Markdown editor)
├── FlexSearch (full-text search)
└── Tailwind CSS (styling)
```

### Rust Integration

```rust
// Direct FFI to truffle-core
#[tauri::command]
async fn compile_artifact(
    state: State<'_, AppState>,
    artifact_id: String,
) -> Result<CompilationResult, Error> {
    // Zero-copy access to core library
    state.core.compile(&artifact_id).await
}
```

### Security Configuration

```json
// tauri.conf.json
{
  "security": {
    "csp": "default-src 'self'; img-src 'self' blob:;",
    "dangerousDisableAssetCspModification": false,
    "freezePrototype": true
  },
  "capabilities": {
    "default": {
      "permissions": [
        "fs:allow-read",
        "fs:allow-write",
        "dialog:allow-open"
      ]
    }
  }
}
```

## Performance Budgets

| Metric | Target | Electron Comparison |
|--------|--------|---------------------|
| Bundle Size | <10MB | 15x smaller |
| Memory (Idle) | <100MB | 3x smaller |
| Memory (Compilation) | <500MB | 2x smaller |
| Startup Time | <2s | 2x faster |
| Time to First Compile | <3s | Comparable |

## Migration Path

If Tauri proves unsuitable:
1. Frontend React code is framework-agnostic
2. Core logic remains in Rust (truffle-core)
3. Can migrate to native platforms incrementally
4. Electron remains fallback option (with performance trade-offs)

## References

- [Tauri Documentation](https://tauri.app/)
- [Tauri v2 Migration Guide](https://beta.tauri.app/)
- [Electron vs Tauri Comparison](https://tauri.app/about/comparisons/electron/)
- [Project Truffle Performance Benchmarks](./benchmarks/desktop-performance.md)

---

**Decision Owner:** CTO  
**Stakeholders:** Engineering, Product, Security  
**Review Date:** 2024-07-15 (6 months)
