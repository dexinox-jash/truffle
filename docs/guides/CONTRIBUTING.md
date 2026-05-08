# Contributing to Truffle

## Code of Conduct
- Be respectful and inclusive
- Focus on constructive feedback
- Respect privacy and security

## How to Contribute

### Reporting Bugs
1. Check existing issues
2. Provide reproduction steps
3. Include system information
4. Attach logs if applicable

### Suggesting Features
1. Open a discussion first
2. Explain use case
3. Consider implementation approach

### Pull Requests

#### Process
1. Fork the repository
2. Create feature branch (`git checkout -b feature/amazing-feature`)
3. Make changes
4. Run tests (`cargo test`, `pnpm test`)
5. Commit with clear message
6. Push to fork
7. Open PR against `main`

#### PR Requirements
- [ ] Tests pass
- [ ] Code formatted (`cargo fmt`, `prettier`)
- [ ] No clippy warnings
- [ ] Documentation updated
- [ ] CHANGELOG.md updated

### Code Standards

#### Rust
- Follow Rust API Guidelines
- Document public APIs
- Use `?` operator, not `unwrap()`
- Write unit tests

#### TypeScript
- Strict mode enabled
- Explicit return types
- Functional components
- Hooks for state

### Commit Message Format
```
type(scope): description

[optional body]

[optional footer]
```

Types: feat, fix, docs, style, refactor, test, chore

Example:
```
feat(graph): add bidirectional pathfinding

Implements bidirectional BFS for faster path finding
between entities in the knowledge graph.

Closes #123
```

### Testing

#### Unit Tests
```rust
#[test]
fn test_entity_creation() {
    let entity = Entity::new("Test", EntityType::Person, None);
    assert_eq!(entity.name, "Test");
}
```

#### Integration Tests
```rust
#[tokio::test]
async fn test_entity_crud() {
    let db = setup_test_db().await;
    // Test operations
}
```

### Documentation
- Update relevant docs
- Add code examples
- Keep README current

## Development Setup
See [Getting Started](./GETTING_STARTED.md)

## Questions?
- Open a discussion
- Join Discord (link)
- Email maintainers
