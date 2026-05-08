# Testing Guide

## Overview

This guide covers all testing practices for the Truffle project, including unit tests, integration tests, and end-to-end tests for both the Rust backend and TypeScript frontend.

## Testing Philosophy

- **Test early, test often** - Write tests alongside code
- **Test behavior, not implementation** - Focus on outcomes
- **Fast feedback** - Unit tests should run in milliseconds
- **Deterministic** - Tests should produce consistent results

## Backend Testing (Rust)

### Running Tests

```bash
# Run all tests
cargo test --workspace

# Run tests for specific crate
cargo test -p truffle-core

# Run specific test
cargo test test_entity_creation

# Run with output
cargo test -- --nocapture

# Run ignored tests
cargo test -- --ignored
```

### Unit Tests

Place unit tests in the same file as the code being tested:

```rust
// src/models/entity.rs
pub struct Entity {
    pub id: Uuid,
    pub name: String,
    pub entity_type: EntityType,
}

impl Entity {
    pub fn new(name: &str, entity_type: EntityType) -> Self {
        Self {
            id: Uuid::new_v4(),
            name: name.to_string(),
            entity_type,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_entity_creation() {
        let entity = Entity::new("Alice", EntityType::Person);
        assert_eq!(entity.name, "Alice");
        assert_eq!(entity.entity_type, EntityType::Person);
    }

    #[test]
    fn test_entity_has_uuid() {
        let entity = Entity::new("Test", EntityType::Organization);
        // UUID should be valid (not nil)
        assert_ne!(entity.id, Uuid::nil());
    }
}
```

### Integration Tests

Place integration tests in `tests/` directory:

```rust
// tests/entity_api_test.rs
use truffle_core::{models::*, database::Database};

#[tokio::test]
async fn test_entity_crud() {
    // Setup
    let db = Database::new_in_memory().await.unwrap();
    
    // Create
    let entity = Entity::new("Test Corp", EntityType::Organization);
    let created = db.create_entity(&entity).await.unwrap();
    assert_eq!(created.name, "Test Corp");
    
    // Read
    let found = db.get_entity(created.id).await.unwrap();
    assert!(found.is_some());
    
    // Update
    let mut updated = created.clone();
    updated.name = "Updated Corp".to_string();
    db.update_entity(&updated).await.unwrap();
    
    // Delete
    db.delete_entity(created.id).await.unwrap();
    let deleted = db.get_entity(created.id).await.unwrap();
    assert!(deleted.is_none());
}
```

### Test Fixtures

Use `rstest` for parameterized tests:

```rust
use rstest::rstest;

#[rstest]
#[case("Alice", EntityType::Person)]
#[case("Acme Corp", EntityType::Organization)]
#[case("Engineering", EntityType::Department)]
fn test_entity_types(#[case] name: &str, #[case] entity_type: EntityType) {
    let entity = Entity::new(name, entity_type);
    assert_eq!(entity.entity_type, entity_type);
}
```

### Async Testing

```rust
#[tokio::test]
async fn test_async_operation() {
    let result = async_operation().await;
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_concurrent_operations() {
    let handles: Vec<_> = (0..10)
        .map(|i| tokio::spawn(async move {
            perform_operation(i).await
        }))
        .collect();
    
    for handle in handles {
        assert!(handle.await.unwrap().is_ok());
    }
}
```

### Mocking

Use `mockall` for mocking traits:

```rust
use mockall::mock;

mock! {
    Database {}
    #[async_trait]
    impl DatabaseTrait for Database {
        async fn get_entity(&self, id: Uuid) -> Result<Option<Entity>>;
        async fn create_entity(&self, entity: &Entity) -> Result<Entity>;
    }
}

#[tokio::test]
async fn test_with_mock() {
    let mut mock = MockDatabase::new();
    mock.expect_get_entity()
        .with(eq(test_id))
        .times(1)
        .returning(|_| Ok(Some(test_entity())));
    
    let service = EntityService::new(mock);
    let result = service.get_entity(test_id).await;
    assert!(result.is_ok());
}
```

## Frontend Testing (TypeScript/React)

### Running Tests

```bash
# Run all tests
pnpm test

# Run in watch mode
pnpm test:watch

# Run with coverage
pnpm test:coverage

# Run specific test file
pnpm test EntityCard.test.tsx

# Run with UI
pnpm test:ui
```

### Component Testing with Vitest

```typescript
// EntityCard.test.tsx
import { describe, it, expect, vi } from 'vitest';
import { render, screen, fireEvent } from '@testing-library/react';
import { EntityCard } from './EntityCard';
import { Entity, EntityType } from '../types/entity';

const mockEntity: Entity = {
  id: '550e8400-e29b-41d4-a716-446655440000',
  name: 'Alice Johnson',
  type: EntityType.Person,
  createdAt: '2024-01-01T00:00:00Z',
};

describe('EntityCard', () => {
  it('renders entity name', () => {
    render(<EntityCard entity={mockEntity} />);
    expect(screen.getByText('Alice Johnson')).toBeInTheDocument();
  });

  it('renders entity type badge', () => {
    render(<EntityCard entity={mockEntity} />);
    expect(screen.getByText('Person')).toBeInTheDocument();
  });

  it('calls onClick when clicked', () => {
    const handleClick = vi.fn();
    render(<EntityCard entity={mockEntity} onClick={handleClick} />);
    
    fireEvent.click(screen.getByRole('button'));
    expect(handleClick).toHaveBeenCalledWith(mockEntity.id);
  });
});
```

### Hook Testing

```typescript
// useEntities.test.ts
import { describe, it, expect, vi, beforeEach } from 'vitest';
import { renderHook, waitFor } from '@testing-library/react';
import { QueryClient, QueryClientProvider } from '@tanstack/react-query';
import { useEntities } from './useEntities';

const createWrapper = () => {
  const queryClient = new QueryClient({
    defaultOptions: {
      queries: { retry: false },
    },
  });
  return ({ children }: { children: React.ReactNode }) => (
    <QueryClientProvider client={queryClient}>{children}</QueryClientProvider>
  );
};

describe('useEntities', () => {
  beforeEach(() => {
    vi.clearAllMocks();
  });

  it('fetches entities successfully', async () => {
    const mockEntities = [
      { id: '1', name: 'Entity 1' },
      { id: '2', name: 'Entity 2' },
    ];
    
    vi.mocked(fetchEntities).mockResolvedValue(mockEntities);
    
    const { result } = renderHook(() => useEntities(), {
      wrapper: createWrapper(),
    });
    
    await waitFor(() => expect(result.current.isSuccess).toBe(true));
    expect(result.current.data).toEqual(mockEntities);
  });
});
```

### Testing Tauri Commands

```typescript
// commands.test.ts
import { describe, it, expect, vi } from 'vitest';
import { invoke } from '@tauri-apps/api/core';

vi.mock('@tauri-apps/api/core', () => ({
  invoke: vi.fn(),
}));

describe('Tauri Commands', () => {
  it('creates entity via command', async () => {
    const mockEntity = { id: '1', name: 'Test' };
    vi.mocked(invoke).mockResolvedValue(mockEntity);
    
    const result = await invoke('create_entity', {
      name: 'Test',
      entityType: 'Person',
    });
    
    expect(result).toEqual(mockEntity);
    expect(invoke).toHaveBeenCalledWith('create_entity', {
      name: 'Test',
      entityType: 'Person',
    });
  });
});
```

## E2E Testing

### Tauri Driver Setup

```bash
# Install WebDriver
 cargo install tauri-driver

# Run E2E tests
 pnpm test:e2e
```

### E2E Test Example

```typescript
// e2e/app.spec.ts
import { test, expect } from '@playwright/test';

test.describe('Truffle App', () => {
  test('should open main window', async () => {
    const window = await electronApp.firstWindow();
    const title = await window.title();
    expect(title).toBe('Truffle');
  });

  test('should create an entity', async () => {
    const window = await electronApp.firstWindow();
    
    // Click new entity button
    await window.click('[data-testid="new-entity-btn"]');
    
    // Fill form
    await window.fill('[data-testid="entity-name"]', 'Test Entity');
    await window.selectOption('[data-testid="entity-type"]', 'Person');
    
    // Submit
    await window.click('[data-testid="submit-btn"]');
    
    // Verify
    await expect(window.locator('.entity-card')).toContainText('Test Entity');
  });
});
```

## Performance Testing

### Rust Benchmarks

```rust
// benches/entity_benchmark.rs
use criterion::{black_box, criterion_group, criterion_main, Criterion};
use truffle_core::models::Entity;

fn criterion_benchmark(c: &mut Criterion) {
    c.bench_function("entity_creation", |b| {
        b.iter(|| {
            Entity::new(black_box("Test Entity"), black_box(EntityType::Person))
        })
    });
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
```

Run benchmarks:
```bash
cargo bench
```

### Frontend Performance

```typescript
// performance/entity-list.perf.ts
import { measurePerformance } from './utils';

describe('EntityList Performance', () => {
  it('renders 1000 entities in under 100ms', async () => {
    const entities = generateEntities(1000);
    
    const duration = await measurePerformance(() => {
      render(<EntityList entities={entities} />);
    });
    
    expect(duration).toBeLessThan(100);
  });
});
```

## Test Organization

### Directory Structure

```
truffle/
├── truffle-core/
│   ├── src/
│   │   └── models/
│   │       ├── entity.rs
│   │       └── entity_test.rs      # Unit tests
│   └── tests/
│       ├── integration/
│       │   └── entity_api_test.rs  # Integration tests
│       └── fixtures/
│           └── entities.rs         # Test data
├── truffle-desktop/
│   ├── src/
│   │   └── components/
│   │       ├── EntityCard.tsx
│   │       └── EntityCard.test.tsx # Component tests
│   └── tests/
│       ├── e2e/
│       │   └── app.spec.ts         # E2E tests
│       └── utils/
│           └── test-utils.tsx      # Test helpers
```

### Test Data Management

```rust
// tests/fixtures/entities.rs
pub fn test_person() -> Entity {
    Entity::new("Alice Johnson", EntityType::Person)
}

pub fn test_organization() -> Entity {
    Entity::new("Acme Corp", EntityType::Organization)
}

pub fn test_entities() -> Vec<Entity> {
    vec![
        test_person(),
        test_organization(),
        Entity::new("Engineering", EntityType::Department),
    ]
}
```

## CI/CD Testing

### GitHub Actions

```yaml
# .github/workflows/test.yml
name: Test

on: [push, pull_request]

jobs:
  test-rust:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - uses: dtolnay/rust-action@stable
      - run: cargo test --workspace --all-features

  test-frontend:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - uses: pnpm/action-setup@v2
      - run: pnpm install
      - run: pnpm test:coverage

  test-e2e:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - run: pnpm install
      - run: pnpm test:e2e
```

## Coverage

### Rust Coverage

```bash
# Install tarpaulin
cargo install cargo-tarpaulin

# Generate coverage
cargo tarpaulin --out Html

# Open report
open tarpaulin-report.html
```

### Frontend Coverage

```bash
# Vitest coverage
pnpm test:coverage

# Report in ./coverage/
```

## Best Practices

1. **AAA Pattern** - Arrange, Act, Assert
2. **One assertion per test** (when possible)
3. **Descriptive test names** - Explain what and why
4. **Use test helpers** - DRY in tests too
5. **Clean up after tests** - Reset state
6. **Don't test implementation details** - Test behavior
7. **Fast tests** - Avoid I/O in unit tests

## Debugging Tests

### Rust

```bash
# Debug specific test
cargo test test_name -- --nocapture

# Run with debugger
rust-gdb target/debug/deps/test_name
```

### Frontend

```bash
# Debug mode
pnpm test --reporter=verbose

# With debugger
node --inspect-brk node_modules/.bin/vitest
```

## Resources

- [Rust Testing Guide](https://doc.rust-lang.org/book/ch11-00-testing.html)
- [Vitest Documentation](https://vitest.dev/)
- [React Testing Library](https://testing-library.com/docs/react-testing-library/intro/)
- [Tauri Testing](https://tauri.app/v1/guides/testing/)
