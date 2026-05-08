# Project Truffle: Comprehensive Test Plan
## QA/Security Validation Document v1.0

**Classification:** AAA Commercial SaaS | Zero-Knowledge Infrastructure  
**Compliance Target:** SOC 2 Type II, ISO 27001, GDPR/CCPA Certified  
**Document Version:** 1.0.0  
**Last Updated:** 2024  

---

## Table of Contents

1. [Executive Summary](#1-executive-summary)
2. [Testing Strategy Overview](#2-testing-strategy-overview)
3. [Unit Test Specifications](#3-unit-test-specifications)
4. [Integration Test Specifications](#4-integration-test-specifications)
5. [E2E Test Specifications](#5-e2e-test-specifications)
6. [Performance Test Specifications](#6-performance-test-specifications)
7. [Security Test Specifications](#7-security-test-specifications)
8. [Testing Gates & CI/CD Integration](#8-testing-gates--cicd-integration)
9. [Test Data Management](#9-test-data-management)
10. [Defect Management](#10-defect-management)

---

## 1. Executive Summary

### 1.1 Purpose

This document defines the comprehensive testing strategy for Project Truffle, a Local-First Knowledge Compiler (LFKC) that transforms unstructured visual data into structured knowledge graphs using on-device multimodal AI.

### 1.2 Testing Philosophy

> **"Test what cannot fail. Verify what must be true."**

Our testing approach prioritizes:
- **Zero-Knowledge Validation:** Cryptographic guarantees must be mathematically verified
- **Red Lines Enforcement:** Non-negotiable constraints (Section 1.2) are continuously validated
- **Local-First Assurance:** Offline functionality is never compromised
- **Enterprise Readiness:** SOC 2 Type II and GDPR compliance through automated verification

### 1.3 Critical Success Metrics

| Metric | Target | Verification Method |
|--------|--------|---------------------|
| Unit Test Coverage | ≥80% | Istanbul/nyc + cargo-tarpaulin |
| Integration Test Pass Rate | 100% | CI gate blocking |
| E2E Test Pass Rate | ≥98% | Nightly + release blocking |
| Security Scan Clean | Zero critical/high | Snyk + truffleHog |
| Performance Budget | <100MB desktop | Bundle analyzer |
| Red Lines Compliance | 100% | Automated checker |

---

## 2. Testing Strategy Overview

### 2.1 Testing Pyramid

```
                    /\
                   /  \
                  / E2E \           <- 5% of tests (Playwright/Detox)
                 /--------\
                /          \
               / Integration \      <- 15% of tests (SQLite + Gemma 4)
              /----------------\
             /                  \
            /     Unit Tests      \   <- 80% of tests (Jest/Cargo)
           /------------------------\
```

### 2.2 Testing Layers Mapping

| Layer | Technology | Coverage Target | Execution |
|-------|------------|-----------------|-----------|
| Unit - TypeScript | Jest 29+ | 80% | Every commit |
| Unit - Rust | Cargo test | 80% | Every commit |
| Integration | SQLite + llama.cpp | 100% scenarios | PR merge |
| E2E Desktop | Playwright | Critical paths | Nightly |
| E2E Mobile | Detox | Critical paths | Nightly |
| Security | Custom + Snyk | All vectors | Every commit |
| Performance | Lighthouse + custom | Budgets | Release |

### 2.3 Red Lines Testing

Every test suite MUST validate the five non-negotiable constraints:

1. **Sovereignty:** Data remains on device - no network calls for raw images
2. **Zero-Knowledge:** Infrastructure cannot decrypt - cryptographic verification
3. **Survival Mode:** Offline functionality - network isolation tests
4. **Exit Capability:** Export completes in <5 minutes - performance + completeness
5. **Economic Viability:** Bundle size <100MB - build verification

---

## 3. Unit Test Specifications

### 3.1 TypeScript Unit Tests (Jest)

**Location:** `test-suites/unit/ts/`
**Framework:** Jest 29.x with TypeScript
**Coverage Tool:** Istanbul/nyc

#### 3.1.1 Test Configuration

```typescript
// jest.config.ts
export default {
  preset: 'ts-jest',
  testEnvironment: 'node',
  coverageThreshold: {
    global: {
      branches: 80,
      functions: 80,
      lines: 80,
      statements: 80
    }
  },
  collectCoverageFrom: [
    'src/**/*.ts',
    '!src/**/*.d.ts',
    '!src/**/index.ts'
  ],
  testMatch: ['**/*.test.ts'],
  moduleNameMapping: {
    '^@/(.*)$': '<rootDir>/src/$1'
  }
};
```

#### 3.1.2 Core Module Test Suites

##### A. Cryptographic Module Tests (`crypto.test.ts`)

```typescript
describe('X3DH Key Exchange', () => {
  test('SHOULD generate valid Ed25519 identity key', () => {
    // Verify key generation meets RFC 8032
  });
  
  test('SHOULD derive shared secret via X3DH handshake', () => {
    // Verify Signal Protocol implementation
  });
  
  test('SHOULD reject invalid signature', () => {
    // STRIDE: Spoofing mitigation verification
  });
  
  test('SHOULD use Kyber-768 for post-quantum hybrid', () => {
    // NIST FIPS 203 compliance
  });
});

describe('AES-256-GCM Encryption', () => {
  test('SHOULD encrypt with random 96-bit nonce', () => {
    // Verify nonce uniqueness
  });
  
  test('SHOULD detect tampering via authentication tag', () => {
    // STRIDE: Tampering mitigation verification
  });
  
  test('SHOULD use ChaCha20-Poly1305 on mobile', () => {
    // Side-channel resistance
  });
});

describe('Zero-Knowledge Verification', () => {
  test('MUST NOT have plaintext paths to sync payload', () => {
    // Red Line: Zero-Knowledge enforcement
  });
  
  test('MUST verify infrastructure cannot decrypt', () => {
    // Mathematical proof verification
  });
});
```

##### B. CRDT Module Tests (`crdt.test.ts`)

```typescript
describe('Yjs CRDT Integration', () => {
  test('SHOULD generate valid state update', () => {
    // Yjs encodeStateAsUpdate verification
  });
  
  test('SHOULD merge concurrent edits correctly', () => {
    // Automerge conflict resolution
  });
  
  test('SHOULD maintain Lamport timestamps', () => {
    // Vector clock verification
  });
  
  test('SHOULD handle schema migrations', () => {
    // Versioned migration tests
  });
});
```

##### C. Schema Validation Tests (`schema.test.ts`)

```typescript
describe('Schema Compilation Rules', () => {
  test('SHOULD detect receipt and extract financial data', () => {
    // Receipt schema rule verification
  });
  
  test('SHOULD redact password fields', () => {
    // Security: Password extraction blocked
  });
  
  test('SHOULD tokenize credit card numbers', () => {
    // Privacy: Last 4 only
  });
  
  test('SHOULD quarantine SSN detection', () => {
    // Compliance: SSN refusal
  });
  
  test('SHOULD classify privacy level correctly', () => {
    // public | personal | sensitive | financial
  });
});
```

##### D. Wiki Node Tests (`wiki.test.ts`)

```typescript
describe('WikiNode Operations', () => {
  test('SHOULD create entity node from artifact', () => {
    // Node creation verification
  });
  
  test('SHOULD maintain backlink integrity', () => {
    // Bidirectional link verification
  });
  
  test('SHOULD generate valid MarkdownAST', () => {
    // GFM + YAML frontmatter + WikiLinks
  });
  
  test('SHOULD track provenance correctly', () => {
    // Audit trail verification
  });
});
```

##### E. Storage Layer Tests (`storage.test.ts`)

```typescript
describe('SQLite Operations', () => {
  test('SHOULD use WAL mode for transactions', () => {
    // ACID compliance verification
  });
  
  test('SHOULD fsync every transaction', () => {
    // Durability verification
  });
  
  test('SHOULD maintain data sovereignty', () => {
    // Red Line: No external storage
  });
});

describe('Vector Search', () => {
  test('SHOULD generate valid embeddings', () => {
    // MiniLM-L6-v2 verification
  });
  
  test('SHOULD find similar vectors with threshold 0.82', () => {
    // Cosine similarity verification
  });
  
  test('SHOULD use HNSW index correctly', () => {
    // ef_construction=128, M=16
  });
});
```

### 3.2 Rust Unit Tests (Cargo)

**Location:** `test-suites/unit/rust/`
**Framework:** Built-in cargo test
**Coverage Tool:** cargo-tarpaulin

#### 3.2.1 Test Configuration

```toml
# Cargo.toml
[dev-dependencies]
criterion = "0.5"
tempfile = "3.8"
mockall = "0.12"

[[bin]]
name = "truffle-core"

[profile.test]
opt-level = 0
debug = true
```

#### 3.2.2 Core Module Test Suites

##### A. Core Library Tests (`core/`)

```rust
// src/core/artifact.rs
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_artifact_creation() {
        let artifact = RawArtifact::new(
            "test.png".to_string(),
            vec![0u8; 1024],
        );
        assert_eq!(artifact.filename, "test.png");
        assert!(artifact.uuid.is_valid());
    }

    #[test]
    fn test_content_addressable_uuid() {
        // UUID must be SHA256 of content
        let data = b"test content";
        let artifact = RawArtifact::from_bytes(data.to_vec());
        let expected_uuid = sha256(data);
        assert_eq!(artifact.uuid.as_bytes(), &expected_uuid[..16]);
    }

    #[test]
    fn test_device_id_hashing() {
        // Device ID must be privacy-preserving
        let device_id = DeviceFingerprint::from_raw("device-123");
        assert_ne!(device_id.to_string(), "device-123");
        assert!(device_id.verify("device-123"));
    }
}

// src/core/compilation.rs
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_compilation_queue_priority() {
        let mut queue = CompilationQueue::new();
        
        let p0 = QueueItem::new(artifact(), Priority::P0);
        let p1 = QueueItem::new(artifact(), Priority::P1);
        let p2 = QueueItem::new(artifact(), Priority::P2);
        
        queue.push(p2);
        queue.push(p0);
        queue.push(p1);
        
        assert_eq!(queue.pop().unwrap().priority, Priority::P0);
        assert_eq!(queue.pop().unwrap().priority, Priority::P1);
        assert_eq!(queue.pop().unwrap().priority, Priority::P2);
    }

    #[test]
    fn test_retry_limit() {
        let mut item = QueueItem::new(artifact(), Priority::P1);
        
        for _ in 0..3 {
            assert!(item.can_retry());
            item.increment_retry();
        }
        
        assert!(!item.can_retry());
    }
}
```

##### B. Cryptographic Tests (`crypto/`)

```rust
// src/crypto/x3dh.rs
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_key_generation() {
        let identity = IdentityKey::generate();
        assert!(identity.public_key().is_valid());
        
        // Verify Ed25519 format
        assert_eq!(identity.public_key().as_bytes().len(), 32);
    }

    #[test]
    fn test_x3dh_handshake() {
        let alice = DeviceKeys::generate();
        let bob = DeviceKeys::generate();
        
        let alice_shared = alice.x3dh_handshake(bob.public_bundle());
        let bob_shared = bob.x3dh_handshake(alice.public_bundle());
        
        assert_eq!(alice_shared.as_bytes(), bob_shared.as_bytes());
    }

    #[test]
    fn test_kyber_hybrid() {
        // Post-quantum hybrid verification
        let pq_keys = PQKeys::generate();
        let ciphertext = pq_keys.encapsulate();
        let shared = pq_keys.decapsulate(&ciphertext);
        
        assert_eq!(shared.len(), 32);
    }

    #[test]
    fn test_aes_gcm_encryption() {
        let key = EncryptionKey::generate();
        let plaintext = b"test message";
        
        let ciphertext = key.encrypt(plaintext);
        let decrypted = key.decrypt(&ciphertext).unwrap();
        
        assert_eq!(decrypted, plaintext);
    }

    #[test]
    fn test_tamper_detection() {
        let key = EncryptionKey::generate();
        let plaintext = b"test message";
        
        let mut ciphertext = key.encrypt(plaintext);
        ciphertext[0] ^= 0xFF; // Flip bits
        
        assert!(key.decrypt(&ciphertext).is_err());
    }

    #[test]
    fn test_nonce_uniqueness() {
        let key = EncryptionKey::generate();
        let plaintext = b"test";
        
        let ct1 = key.encrypt(plaintext);
        let ct2 = key.encrypt(plaintext);
        
        // Nonces must be different
        assert_ne!(&ct1[0..12], &ct2[0..12]);
    }
}
```

##### C. Sync Protocol Tests (`sync/`)

```rust
// src/sync/protocol.rs
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_crdt_encryption() {
        let update = vec![1u8, 2, 3, 4, 5];
        let key = SyncKey::generate();
        
        let encrypted = encrypt_crdt(&update, &key);
        let decrypted = decrypt_crdt(&encrypted, &key).unwrap();
        
        assert_eq!(decrypted, update);
    }

    #[test]
    fn test_message_authentication() {
        let msg = SyncMessage::new(vec![1, 2, 3]);
        let key = SyncKey::generate();
        
        let signed = msg.sign(&key);
        assert!(signed.verify(&key).is_ok());
        
        let mut tampered = signed;
        tampered.payload[0] ^= 0xFF;
        assert!(tampered.verify(&key).is_err());
    }

    #[test]
    fn test_tombstone_generation() {
        let artifact_id = Uuid::new_v4();
        let tombstone = Tombstone::new(artifact_id);
        
        assert_eq!(tombstone.artifact_id, artifact_id);
        assert!(tombstone.timestamp > 0);
    }
}
```

##### D. Graph Database Tests (`graph/`)

```rust
// src/graph/entity_resolution.rs
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_levenshtein_matching() {
        assert!(fuzzy_match("John Doe", "Jon Doe", 2));
        assert!(!fuzzy_match("John Doe", "Jane Smith", 2));
    }

    #[test]
    fn test_metaphone_matching() {
        assert!(metaphone_match("Robert", "Rupert"));
        assert!(!metaphone_match("John", "Jane"));
    }

    #[test]
    fn test_entity_resolution() {
        let existing = vec![
            Entity::new("John Smith"),
            Entity::new("Jane Doe"),
        ];
        
        let new = Entity::new("Jon Smith");
        let resolved = resolve_entity(&new, &existing);
        
        assert!(resolved.is_some());
        assert_eq!(resolved.unwrap().name, "John Smith");
    }
}
```

---

## 4. Integration Test Specifications

### 4.1 SQLite + Gemma 4 Integration Tests

**Location:** `test-suites/integration/`
**Framework:** Custom test harness with llama.cpp
**Requirements:** GitHub Actions large runner with GPU

#### 4.1.1 Test Environment Setup

```yaml
# .github/workflows/integration-tests.yml
name: Integration Tests

on:
  pull_request:
    branches: [main]
  schedule:
    - cron: '0 2 * * *' # Nightly

jobs:
  integration:
    runs-on: ubuntu-latest-gpu
    steps:
      - uses: actions/checkout@v4
      
      - name: Setup llama.cpp
        run: |
          git clone https://github.com/ggerganov/llama.cpp
          cd llama.cpp && git checkout b2699
          cmake -B build -DLLAMA_CUDA=ON
          cmake --build build --config Release
      
      - name: Download Gemma 4
        run: |
          wget https://huggingface.co/google/gemma-4/resolve/main/gemma-4-2b-it-Q4_K_M.gguf
          echo "a1b2c3... gemma-4-2b-it-Q4_K_M.gguf" | sha256sum -c
      
      - name: Run Integration Tests
        run: cargo test --test integration -- --nocapture
```

#### 4.1.2 Test Scenarios

##### A. End-to-End Compilation Pipeline

```rust
// tests/integration/compilation_pipeline.rs
#[test]
fn test_receipt_compilation() {
    // Given: A receipt screenshot
    let receipt_image = load_test_image("receipt_sample_01.png");
    
    // When: Compilation pipeline runs
    let result = compile_artifact(receipt_image);
    
    // Then: Extracted data matches expected
    assert!(result.is_ok());
    let wiki_node = result.unwrap();
    
    assert_eq!(wiki_node.node_type, "entity");
    assert!(wiki_node.content.contains("merchant_name"));
    assert!(wiki_node.content.contains("total_amount"));
    assert!(wiki_node.privacy_classification == "financial");
    assert!(wiki_node.provenance.confidence_score > 0.85);
}

#[test]
fn test_flight_confirmation_compilation() {
    let flight_image = load_test_image("flight_confirmation.png");
    
    let result = compile_artifact(flight_image);
    let wiki_node = result.unwrap();
    
    assert!(wiki_node.content.contains("airline"));
    assert!(wiki_node.content.contains("flight_number"));
    assert!(wiki_node.content.contains("confirmation_code"));
    assert!(wiki_node.temporal_vectors.mentioned_dates.len() >= 2);
}

#[test]
fn test_contact_card_compilation() {
    let contact_image = load_test_image("contact_card.png");
    
    let result = compile_artifact(contact_image);
    let wiki_node = result.unwrap();
    
    assert!(wiki_node.content.contains("phone"));
    assert!(wiki_node.content.contains("email"));
    assert!(wiki_node.privacy_classification == "personal");
}

#[test]
fn test_sensitive_content_redaction() {
    let sensitive_image = load_test_image("password_screenshot.png");
    
    let result = compile_artifact(sensitive_image);
    let wiki_node = result.unwrap();
    
    // Password should be redacted
    assert!(!wiki_node.content.contains("password123"));
    assert!(wiki_node.content.contains("[REDACTED]"));
}

#[test]
fn test_credit_card_tokenization() {
    let cc_image = load_test_image("credit_card.png");
    
    let result = compile_artifact(cc_image);
    let wiki_node = result.unwrap();
    
    // Full CC number should not appear
    assert!(!wiki_node.content.contains("4532123456789012"));
    // Last 4 may appear
    assert!(wiki_node.content.contains("9012") || !wiki_node.content.contains("card_number"));
}
```

##### B. Knowledge Graph Construction

```rust
// tests/integration/knowledge_graph.rs
#[test]
fn test_entity_linking() {
    // Given: Multiple receipts from same merchant
    let receipt1 = compile_artifact(load_test_image("starbucks_01.png"));
    let receipt2 = compile_artifact(load_test_image("starbucks_02.png"));
    
    // When: Entities are resolved
    let graph = build_knowledge_graph(vec![receipt1, receipt2]);
    
    // Then: Same merchant is linked
    let merchant_nodes: Vec<_> = graph
        .nodes
        .iter()
        .filter(|n| n.title.to_lowercase().contains("starbucks"))
        .collect();
    
    assert_eq!(merchant_nodes.len(), 1);
    assert_eq!(merchant_nodes[0].backlinks.len(), 2);
}

#[test]
fn test_temporal_linking() {
    // Given: Screenshots within 5 minutes
    let img1 = load_test_image("screenshot_12:00.png");
    let img2 = load_test_image("screenshot_12:03.png");
    
    // When: Temporal proximity linking applied
    let graph = build_knowledge_graph(vec![
        compile_artifact(img1),
        compile_artifact(img2),
    ]);
    
    // Then: Nodes are linked via temporal proximity
    let links = graph.temporal_links();
    assert!(!links.is_empty());
}

#[test]
fn test_semantic_similarity_linking() {
    // Given: Semantically similar screenshots
    let img1 = load_test_image("meeting_notes_01.png");
    let img2 = load_test_image("meeting_notes_02.png");
    
    // When: Semantic similarity calculated
    let embedding1 = generate_embedding(&img1);
    let embedding2 = generate_embedding(&img2);
    let similarity = cosine_similarity(&embedding1, &embedding2);
    
    // Then: Similarity exceeds threshold
    assert!(similarity >= 0.82);
}
```

##### C. Vector Search Integration

```rust
// tests/integration/vector_search.rs
#[test]
fn test_hnsw_index_creation() {
    let db = setup_test_database();
    
    // Insert test embeddings
    for i in 0..1000 {
        let embedding = random_embedding(384);
        db.insert_embedding(format!("doc_{}", i), embedding);
    }
    
    // Create HNSW index
    db.create_hnsw_index(ef_construction: 128, M: 16);
    
    // Verify index exists
    assert!(db.has_hnsw_index());
}

#[test]
fn test_vector_search_accuracy() {
    let db = setup_test_database_with_embeddings();
    
    // Search for similar vectors
    let query = db.get_embedding("doc_50").unwrap();
    let results = db.vector_search(&query, k: 10);
    
    // Top result should be the query itself
    assert_eq!(results[0].id, "doc_50");
    
    // All results should have high similarity
    for result in &results {
        assert!(result.similarity >= 0.82);
    }
}

#[test]
fn test_fulltext_search_integration() {
    let db = setup_test_database_with_wiki();
    
    // Search for term
    let results = db.fulltext_search("Starbucks", limit: 10);
    
    // Results should contain matching nodes
    assert!(!results.is_empty());
    for result in &results {
        assert!(result.content.to_lowercase().contains("starbucks"));
    }
}
```

##### D. CRDT Sync Integration

```rust
// tests/integration/crdt_sync.rs
#[test]
fn test_crdt_merge() {
    // Given: Two devices with divergent state
    let device_a = create_wiki_node("Test Node");
    let device_b = device_a.clone();
    
    // Device A edits
    let mut state_a = YjsDoc::new();
    state_a.insert("title", "Test Node - Edited by A");
    
    // Device B edits concurrently
    let mut state_b = YjsDoc::new();
    state_b.insert("content", "Content added by B");
    
    // When: States are merged
    let merged = crdt_merge(&state_a, &state_b);
    
    // Then: Both edits are preserved
    assert!(merged.get("title").contains("Edited by A"));
    assert!(merged.get("content").contains("added by B"));
}

#[test]
fn test_sync_message_encryption() {
    // Given: A CRDT update
    let update = generate_crdt_update();
    let key = SyncKey::from_pairing("device_a", "device_b");
    
    // When: Encrypted for sync
    let encrypted = encrypt_sync_message(&update, &key);
    
    // Then: Payload is encrypted
    assert_ne!(encrypted.payload, update);
    
    // And: Can be decrypted
    let decrypted = decrypt_sync_message(&encrypted, &key).unwrap();
    assert_eq!(decrypted, update);
}

#[test]
fn test_tombstone_propagation() {
    // Given: A deleted artifact
    let artifact_id = Uuid::new_v4();
    let tombstone = Tombstone::new(artifact_id);
    
    // When: Tombstone is synced
    let sync_msg = create_sync_message(tombstone);
    
    // Then: Receiving device processes deletion
    let received = process_sync_message(sync_msg);
    assert!(received.is_tombstone());
    assert_eq!(received.get_artifact_id(), artifact_id);
}
```

---

## 5. E2E Test Specifications

### 5.1 Desktop E2E Tests (Playwright)

**Location:** `test-suites/e2e-desktop/`
**Framework:** Playwright 1.40+
**Target:** Tauri desktop application

#### 5.1.1 Test Configuration

```typescript
// playwright.config.ts
import { defineConfig, devices } from '@playwright/test';

export default defineConfig({
  testDir: './tests',
  fullyParallel: true,
  forbidOnly: !!process.env.CI,
  retries: process.env.CI ? 2 : 0,
  workers: process.env.CI ? 1 : undefined,
  reporter: 'html',
  use: {
    trace: 'on-first-retry',
    screenshot: 'only-on-failure',
  },
  projects: [
    {
      name: 'chromium',
      use: { ...devices['Desktop Chrome'] },
    },
    {
      name: 'firefox',
      use: { ...devices['Desktop Firefox'] },
    },
    {
      name: 'webkit',
      use: { ...devices['Desktop Safari'] },
    },
  ],
});
```

#### 5.1.2 Critical Path Tests

```typescript
// tests/critical-paths.spec.ts
import { test, expect } from '@playwright/test';

test.describe('Critical User Paths', () => {
  test('user can capture and compile a screenshot', async ({ page }) => {
    // Launch app
    await page.goto('app://localhost');
    
    // Navigate to Raw view
    await page.click('[data-testid="raw-tab"]');
    
    // Simulate screenshot capture
    await page.setInputFiles('[data-testid="drop-zone"]', 'test-assets/receipt.png');
    
    // Wait for compilation
    await page.waitForSelector('[data-testid="compilation-complete"]');
    
    // Verify wiki node created
    await page.click('[data-testid="wiki-tab"]');
    const node = await page.locator('[data-testid="wiki-node"]').first();
    await expect(node).toBeVisible();
  });

  test('user can search across wiki', async ({ page }) => {
    await page.goto('app://localhost');
    
    // Open search
    await page.keyboard.press('Control+k');
    await page.fill('[data-testid="search-input"]', 'Starbucks');
    
    // Verify search results
    const results = await page.locator('[data-testid="search-result"]');
    await expect(results).toHaveCount.greaterThan(0);
  });

  test('user can export to Obsidian', async ({ page }) => {
    await page.goto('app://localhost');
    
    // Trigger export
    await page.keyboard.press('Control+Shift+e');
    await page.click('[data-testid="export-obsidian"]');
    
    // Verify export completes
    await page.waitForSelector('[data-testid="export-complete"]');
    
    // Verify markdown files created
    const exportPath = await page.inputValue('[data-testid="export-path"]');
    expect(exportPath).toContain('.md');
  });

  test('Red Line: export completes within 5 minutes', async ({ page }) => {
    const startTime = Date.now();
    
    await page.goto('app://localhost');
    await page.keyboard.press('Control+Shift+e');
    await page.click('[data-testid="export-all"]');
    await page.waitForSelector('[data-testid="export-complete"]');
    
    const elapsed = Date.now() - startTime;
    expect(elapsed).toBeLessThan(5 * 60 * 1000); // 5 minutes
  });
});

test.describe('Offline Functionality', () => {
  test('Red Line: app functions offline', async ({ page, context }) => {
    // Go offline
    await context.setOffline(true);
    
    await page.goto('app://localhost');
    
    // Verify core functionality works
    await page.click('[data-testid="raw-tab"]');
    await expect(page.locator('[data-testid="raw-grid"]')).toBeVisible();
    
    await page.click('[data-testid="wiki-tab"]');
    await expect(page.locator('[data-testid="wiki-navigator"]')).toBeVisible();
    
    // Search should work locally
    await page.keyboard.press('Control+k');
    await page.fill('[data-testid="search-input"]', 'test');
    await expect(page.locator('[data-testid="search-result"]')).toBeVisible();
  });

  test('sync resumes when coming back online', async ({ page, context }) => {
    await page.goto('app://localhost');
    
    // Go offline, make changes
    await context.setOffline(true);
    await page.click('[data-testid="wiki-node"]').first();
    await page.fill('[data-testid="editor"]', 'Updated content');
    
    // Come back online
    await context.setOffline(false);
    
    // Verify sync indicator
    await page.waitForSelector('[data-testid="sync-syncing"]');
    await page.waitForSelector('[data-testid="sync-complete"]');
  });
});

test.describe('Security Verification', () => {
  test('Red Line: no plaintext data in network requests', async ({ page, context }) => {
    const requests: any[] = [];
    
    context.on('request', request => {
      requests.push({
        url: request.url(),
        postData: request.postData(),
      });
    });
    
    await page.goto('app://localhost');
    
    // Trigger sync
    await page.click('[data-testid="sync-now"]');
    await page.waitForTimeout(2000);
    
    // Verify no plaintext in requests
    for (const req of requests) {
      if (req.url.includes('sync')) {
        // Payload should be encrypted (not valid JSON/UTF-8)
        const postData = req.postData;
        if (postData) {
          expect(isEncrypted(postData)).toBe(true);
        }
      }
    }
  });

  test('device pairing ceremony works end-to-end', async ({ page }) => {
    await page.goto('app://localhost');
    
    // Start pairing
    await page.click('[data-testid="settings"]');
    await page.click('[data-testid="add-device"]');
    
    // Verify QR code displayed
    await expect(page.locator('[data-testid="pairing-qr"]')).toBeVisible();
    
    // Verify fingerprint shown
    const fingerprint = await page.textContent('[data-testid="device-fingerprint"]');
    expect(fingerprint).toMatch(/^[A-F0-9]{16}$/);
  });
});
```

### 5.2 Mobile E2E Tests (Detox)

**Location:** `test-suites/e2e-mobile/`
**Framework:** Detox 20.x
**Target:** iOS/Android mobile apps

#### 5.2.1 Test Configuration

```javascript
// .detoxrc.js
module.exports = {
  apps: {
    'ios.debug': {
      type: 'ios.app',
      binaryPath: 'ios/build/Build/Products/Debug-iphonesimulator/Truffle.app',
      build: 'xcodebuild -workspace ios/Truffle.xcworkspace -scheme Truffle -configuration Debug -sdk iphonesimulator -derivedDataPath ios/build',
    },
    'android.debug': {
      type: 'android.apk',
      binaryPath: 'android/app/build/outputs/apk/debug/app-debug.apk',
      build: 'cd android && ./gradlew assembleDebug assembleAndroidTest -DtestBuildType=debug',
    },
  },
  devices: {
    simulator: {
      type: 'ios.simulator',
      device: {
        type: 'iPhone 15 Pro',
      },
    },
    emulator: {
      type: 'android.emulator',
      device: {
        avdName: 'Pixel_7_API_34',
      },
    },
  },
  configurations: {
    'ios.sim.debug': {
      device: 'simulator',
      app: 'ios.debug',
    },
    'android.emu.debug': {
      device: 'emulator',
      app: 'android.debug',
    },
  },
};
```

#### 5.2.2 Mobile Test Scenarios

```javascript
// e2e/screenshot-capture.test.js
describe('Screenshot Capture Flow', () => {
  beforeAll(async () => {
    await device.launchApp();
  });

  beforeEach(async () => {
    await device.reloadReactNative();
  });

  it('should capture screenshot via share extension', async () => {
    // Simulate share extension trigger
    await device.openURL({
      url: 'truffle://capture?source=screenshot',
    });

    // Verify capture screen
    await expect(element(by.id('capture-screen'))).toBeVisible();

    // Simulate image selection
    await element(by.id('select-image')).tap();

    // Wait for compilation
    await waitFor(element(by.id('compilation-complete')))
      .toBeVisible()
      .withTimeout(30000);

    // Verify success notification
    await expect(element(by.text('Screenshot compiled'))).toBeVisible();
  });

  it('should handle background compilation checkpoint', async () => {
    // Start compilation
    await element(by.id('capture-button')).tap();
    await element(by.id('select-image')).tap();

    // Simulate app backgrounding (iOS 30s limit)
    await device.sendToHome();
    await device.launchApp({ newInstance: false });

    // Verify compilation resumed
    await expect(element(by.id('compilation-progress'))).toBeVisible();
  });

  it('should warn at 5GB storage usage', async () => {
    // Mock storage at 4.9GB
    await device.setStatusBar({
      dataNetwork: 'wifi',
      wifiMode: 'enabled',
    });

    // Add screenshots to reach threshold
    for (let i = 0; i < 50; i++) {
      await addTestScreenshot();
    }

    // Verify warning shown
    await expect(element(by.text('Storage Warning'))).toBeVisible();
    await expect(element(by.text('You are approaching 5GB of storage'))).toBeVisible();
  });

  it('should auto-pause at 10GB storage', async () => {
    // Mock storage at 9.9GB
    await mockStorageUsage(9.9 * 1024 * 1024 * 1024);

    // Attempt capture
    await element(by.id('capture-button')).tap();

    // Verify auto-pause
    await expect(element(by.text('Storage Full'))).toBeVisible();
    await expect(element(by.id('capture-button'))).toBeDisabled();
  });
});

describe('Wiki Browser', () => {
  it('should navigate wiki links', async () => {
    await element(by.id('wiki-tab')).tap();

    // Tap a node
    await element(by.id('wiki-node')).atIndex(0).tap();

    // Verify node detail shown
    await expect(element(by.id('node-detail'))).toBeVisible();

    // Tap a wiki link
    await element(by.id('wiki-link')).tap();

    // Verify navigation
    await expect(element(by.id('node-detail'))).toBeVisible();
  });

  it('should search wiki', async () => {
    await element(by.id('wiki-tab')).tap();

    // Open search
    await element(by.id('search-button')).tap();

    // Type search query
    await element(by.id('search-input')).typeText('Starbucks');

    // Verify results
    await expect(element(by.id('search-result'))).toBeVisible();
  });
});

describe('Sync Functionality', () => {
  it('should sync when charging and on wifi', async () => {
    // Enable charging and wifi
    await device.setStatusBar({
      batteryState: 'charging',
      dataNetwork: 'wifi',
    });

    // Trigger background sync
    await device.sendToHome();
    await device.launchApp({ newInstance: false });

    // Verify sync indicator
    await expect(element(by.id('sync-indicator'))).toBeVisible();
  });

  it('should store keys in Secure Enclave', async () => {
    // Verify key storage
    const keyStored = await device.getPlatformUtil().executeShellCommand(
      'security find-generic-password -s truffle.sync.key'
    );
    expect(keyStored).toContain('truffle.sync.key');
  });
});

describe('Red Lines - Mobile', () => {
  it('MUST keep data on device (Sovereignty)', async () => {
    // Capture screenshot
    await element(by.id('capture-button')).tap();

    // Verify no network requests to external storage
    const networkRequests = await getNetworkRequests();
    const externalStorageRequests = networkRequests.filter(
      req => req.url.includes('s3') || req.url.includes('cloud')
    );
    expect(externalStorageRequests).toHaveLength(0);
  });

  it('MUST function offline (Survival Mode)', async () => {
    // Disable network
    await device.setStatusBar({ dataNetwork: 'none' });

    // Verify app functions
    await element(by.id('wiki-tab')).tap();
    await expect(element(by.id('wiki-list'))).toBeVisible();

    // Search should work
    await element(by.id('search-button')).tap();
    await element(by.id('search-input')).typeText('test');
    await expect(element(by.id('search-result'))).toBeVisible();
  });
});
```

---

## 6. Performance Test Specifications

### 6.1 Bundle Size Tests

```typescript
// tests/performance/bundle-size.test.ts
import { test, expect } from '@playwright/test';
import fs from 'fs';
import path from 'path';

test.describe('Bundle Size Validation', () => {
  test('desktop bundle MUST be under 100MB', () => {
    const bundlePath = path.join(__dirname, '../../dist/Truffle.dmg');
    const stats = fs.statSync(bundlePath);
    const sizeMB = stats.size / (1024 * 1024);
    
    expect(sizeMB).toBeLessThan(100);
  });

  test('mobile bundle MUST be under 50MB', () => {
    const iosBundle = path.join(__dirname, '../../ios/build/Truffle.ipa');
    const androidBundle = path.join(__dirname, '../../android/app/build/outputs/apk/release/app-release.apk');
    
    const iosStats = fs.statSync(iosBundle);
    const androidStats = fs.statSync(androidBundle);
    
    expect(iosStats.size / (1024 * 1024)).toBeLessThan(50);
    expect(androidStats.size / (1024 * 1024)).toBeLessThan(50);
  });

  test('Gemma 4 model MUST use delta updates', () => {
    const deltaPath = path.join(__dirname, '../../models/gemma-4-delta.bsdiff');
    const fullPath = path.join(__dirname, '../../models/gemma-4-full.gguf');
    
    const deltaStats = fs.statSync(deltaPath);
    const fullStats = fs.statSync(fullPath);
    
    // Delta should be ~25% of full size
    expect(deltaStats.size).toBeLessThan(fullStats.size * 0.3);
  });
});
```

### 6.2 Memory Usage Tests

```rust
// tests/performance/memory.rs
#[test]
fn test_memory_ceiling() {
    let initial_memory = get_memory_usage();
    
    // Load Gemma 4 model
    let model = load_model("gemma-4-2b-it-Q4_K_M.gguf");
    let model_memory = get_memory_usage();
    
    // Model should fit in 4GB
    assert!(model_memory - initial_memory < 4 * 1024 * 1024 * 1024);
    
    // Compile 100 screenshots
    for i in 0..100 {
        let image = load_test_image(&format!("test_{}.png", i));
        compile_artifact(image);
    }
    
    let final_memory = get_memory_usage();
    
    // Frontend should stay under 800MB
    // Backend (including model) should stay under 4GB
    assert!(final_memory - model_memory < 800 * 1024 * 1024);
    assert!(final_memory < 4 * 1024 * 1024 * 1024);
}

#[test]
fn test_memory_leak_detection() {
    let baseline = get_memory_usage();
    
    // Repeated operations
    for _ in 0..1000 {
        let image = load_test_image("test.png");
        let result = compile_artifact(image);
        drop(result);
    }
    
    // Force garbage collection
    force_gc();
    
    let final_memory = get_memory_usage();
    
    // Memory should return to near baseline
    assert!(final_memory - baseline < 50 * 1024 * 1024); // 50MB tolerance
}
```

### 6.3 Performance Budgets

```typescript
// tests/performance/budgets.test.ts
test.describe('Performance Budgets', () => {
  test('Time to First Compile < 3 seconds', async ({ page }) => {
    const startTime = Date.now();
    
    await page.goto('app://localhost');
    await page.click('[data-testid="raw-tab"]');
    await page.setInputFiles('[data-testid="drop-zone"]', 'test-assets/receipt.png');
    await page.waitForSelector('[data-testid="compilation-complete"]');
    
    const elapsed = Date.now() - startTime;
    expect(elapsed).toBeLessThan(3000);
  });

  test('Compilation Throughput >= 1 screenshot/second', async ({ page }) => {
    await page.goto('app://localhost');
    
    const startTime = Date.now();
    const count = 10;
    
    for (let i = 0; i < count; i++) {
      await page.setInputFiles('[data-testid="drop-zone"]', `test-assets/receipt_${i}.png`);
    }
    
    await page.waitForSelector('[data-testid="all-compilations-complete"]');
    
    const elapsed = Date.now() - startTime;
    const throughput = count / (elapsed / 1000);
    
    expect(throughput).toBeGreaterThanOrEqual(1);
  });

  test('UI Responsiveness 60fps during compilation', async ({ page }) => {
    await page.goto('app://localhost');
    
    // Start FPS monitoring
    const fpsMeasurements: number[] = [];
    
    page.on('console', msg => {
      if (msg.text().includes('FPS:')) {
        fpsMeasurements.push(parseFloat(msg.text().split(':')[1]));
      }
    });
    
    // Trigger compilation
    await page.setInputFiles('[data-testid="drop-zone"]', 'test-assets/receipt.png');
    
    // Wait for completion
    await page.waitForSelector('[data-testid="compilation-complete"]');
    
    // Verify FPS
    const avgFps = fpsMeasurements.reduce((a, b) => a + b, 0) / fpsMeasurements.length;
    expect(avgFps).toBeGreaterThanOrEqual(60);
  });
});
```

---

## 7. Security Test Specifications

### 7.1 Dependency Audit

```yaml
# .github/workflows/security-audit.yml
name: Security Audit

on:
  push:
    branches: [main]
  pull_request:
    branches: [main]
  schedule:
    - cron: '0 0 * * *' # Daily

jobs:
  audit:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4

      - name: Run Snyk for TypeScript
        uses: snyk/actions/node@master
        env:
          SNYK_TOKEN: ${{ secrets.SNYK_TOKEN }}
        with:
          args: --severity-threshold=high

      - name: Run cargo-audit for Rust
        run: |
          cargo install cargo-audit
          cargo audit --deny warnings

      - name: Check for known vulnerabilities
        run: |
          npm audit --audit-level=high
```

### 7.2 Secrets Scanning

```yaml
# .github/workflows/secrets-scan.yml
name: Secrets Scan

on: [push, pull_request]

jobs:
  scan:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
        with:
          fetch-depth: 0

      - name: Run truffleHog
        uses: trufflesecurity/trufflehog@main
        with:
          path: ./
          base: main
          head: HEAD
          extra_args: --debug --only-verified

      - name: Run GitLeaks
        uses: gitleaks/gitleaks-action@v2
        env:
          GITHUB_TOKEN: ${{ secrets.GITHUB_TOKEN }}
```

### 7.3 Penetration Test Scenarios

```typescript
// tests/security/penetration.test.ts
test.describe('Penetration Tests', () => {
  test('STRIDE: Spoofing - reject invalid signatures', async () => {
    // Attempt to send message with forged signature
    const forgedMessage = createForgedMessage();
    
    const result = await sendSyncMessage(forgedMessage);
    
    // Should be rejected
    expect(result.status).toBe(403);
  });

  test('STRIDE: Tampering - detect bit flips', async () => {
    // Create valid encrypted message
    const message = createEncryptedMessage();
    
    // Flip random bit
    const tampered = flipRandomBit(message);
    
    // Decryption should fail
    expect(() => decrypt(tampered)).toThrow('Authentication failed');
  });

  test('STRIDE: Repudiation - immutable audit logs', async () => {
    // Perform action
    const action = await performAction('create_node');
    
    // Verify log entry
    const logEntry = await getAuditLog(action.id);
    expect(logEntry).toBeDefined();
    
    // Attempt to modify log
    const modified = await attemptLogModification(action.id);
    expect(modified).toBe(false);
    
    // Verify Merkle tree integrity
    const merkleRoot = await getMerkleRoot();
    expect(verifyMerkleTree(merkleRoot)).toBe(true);
  });

  test('STRIDE: Information Disclosure - no plaintext paths', async () => {
    // Monitor all network traffic
    const traffic = await captureNetworkTraffic(async () => {
      await performSync();
    });
    
    // Verify no plaintext
    for (const request of traffic) {
      if (request.body) {
        const isPlaintext = isValidUTF8(request.body) && 
                           JSON.parse(request.body);
        expect(isPlaintext).toBe(false);
      }
    }
  });

  test('STRIDE: Denial of Service - offline resilience', async () => {
    // Block all network
    await blockNetwork();
    
    // App should continue functioning
    const result = await compileArtifact(testImage);
    expect(result.success).toBe(true);
    
    // Local storage should work
    const saved = await saveToLocalStorage(testData);
    expect(saved).toBe(true);
  });

  test('STRIDE: Elevation of Privilege - sandbox enforcement', async () => {
    // Attempt to escape sandbox
    const escapeAttempt = await attemptSandboxEscape();
    expect(escapeAttempt.success).toBe(false);
    
    // Verify macOS sandbox
    const sandboxStatus = await checkSandboxStatus();
    expect(sandboxStatus.enforced).toBe(true);
  });
});
```

### 7.4 Fuzzing Tests

```rust
// tests/security/fuzzing.rs
use libfuzzer_sys::fuzz_target;

fuzz_target!(|data: &[u8]| {
    // Fuzz sync message parsing
    if let Ok(message) = SyncMessage::from_bytes(data) {
        // Should not panic
        let _ = message.validate();
    }
    
    // Fuzz decryption
    if data.len() > 32 {
        let key = &data[0..32];
        let ciphertext = &data[32..];
        let _ = decrypt_aes_gcm(ciphertext, key);
    }
});

#[test]
fn test_schema_fuzzing() {
    use arbitrary::Arbitrary;
    
    for _ in 0..10000 {
        let mut data = vec![0u8; 1024];
        rand::thread_rng().fill(&mut data[..]);
        
        // Fuzz schema input
        let result = std::panic::catch_unwind(|| {
            let schema_input = String::from_utf8_lossy(&data);
            let _ = Schema::parse(&schema_input);
        });
        
        // Should not panic
        assert!(result.is_ok());
    }
}
```

---

## 8. Testing Gates & CI/CD Integration

### 8.1 CI/CD Pipeline Integration

```yaml
# .github/workflows/ci-cd.yml
name: CI/CD Pipeline

on:
  push:
    branches: [main, develop]
  pull_request:
    branches: [main]

jobs:
  # Gate 1: Unit Tests
  unit-tests:
    name: Unit Tests
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      
      - name: TypeScript Unit Tests
        run: |
          npm ci
          npm run test:unit -- --coverage
          npx nyc check-coverage --lines 80 --functions 80 --branches 80
      
      - name: Rust Unit Tests
        run: |
          cargo test --lib
          cargo tarpaulin --out Xml --fail-under 80
      
      - name: Upload Coverage
        uses: codecov/codecov-action@v3

  # Gate 2: Integration Tests
  integration-tests:
    name: Integration Tests
    needs: unit-tests
    runs-on: ubuntu-latest-gpu
    steps:
      - uses: actions/checkout@v4
      
      - name: Setup Test Environment
        run: |
          ./scripts/setup-integration-tests.sh
      
      - name: Run Integration Tests
        run: cargo test --test integration

  # Gate 3: Security Audit
  security-audit:
    name: Security Audit
    needs: unit-tests
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      
      - name: Dependency Audit
        run: |
          npm audit --audit-level=high
          cargo audit --deny warnings
      
      - name: Secrets Scan
        uses: trufflesecurity/trufflehog@main
        with:
          path: ./
          base: main
          head: HEAD
      
      - name: Red Lines Check
        run: npm run test:red-lines

  # Gate 4: Performance Tests
  performance-tests:
    name: Performance Tests
    needs: [unit-tests, integration-tests]
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      
      - name: Bundle Size Check
        run: |
          npm run build
          npm run test:bundle-size
      
      - name: Memory Usage Check
        run: cargo test --test performance

  # Gate 5: E2E Tests
  e2e-tests:
    name: E2E Tests
    needs: [integration-tests, security-audit]
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      
      - name: Desktop E2E
        run: |
          npm run test:e2e:desktop
      
      - name: Mobile E2E
        run: |
          npm run test:e2e:mobile

  # Release Gate
  release:
    name: Release
    needs: [e2e-tests, performance-tests]
    runs-on: ubuntu-latest
    if: github.ref == 'refs/heads/main'
    steps:
      - uses: actions/checkout@v4
      
      - name: Generate SBOM
        run: |
          npm run generate-sbom
          cargo cyclonedx
      
      - name: Create Release
        uses: softprops/action-gh-release@v1
        with:
          files: |
            dist/*
            sbom/*
```

### 8.2 Testing Gates Summary

| Gate | Tests | Coverage | Blocking | Execution |
|------|-------|----------|----------|-----------|
| 1 | Unit (TS + Rust) | 80% | Yes | Every commit |
| 2 | Integration (SQLite + Gemma 4) | 100% scenarios | Yes | PR merge |
| 3 | Security (Snyk + truffleHog) | Zero critical | Yes | Every commit |
| 4 | Performance (Bundle + Memory) | Budgets met | Yes | Release |
| 5 | E2E (Playwright + Detox) | 98% pass | Yes | Nightly + Release |

---

## 9. Test Data Management

### 9.1 Test Assets

```
test-assets/
├── images/
│   ├── receipts/
│   │   ├── starbucks_01.png
│   │   ├── starbucks_02.png
│   │   ├── grocery_receipt.png
│   │   └── restaurant_receipt.png
│   ├── travel/
│   │   ├── flight_confirmation.png
│   │   └── hotel_booking.png
│   ├── contacts/
│   │   └── business_card.png
│   └── sensitive/
│       ├── password_screenshot.png
│       └── credit_card.png
├── schemas/
│   ├── test-schema-v1.md
│   └── test-schema-v2.md
└── fixtures/
    ├── wiki-nodes.json
    ├── crdt-updates.bin
    └── sync-messages.json
```

### 9.2 Synthetic Data Generation

```typescript
// scripts/generate-test-data.ts
import { faker } from '@faker-js/faker';

export function generateReceiptImage(): Buffer {
  // Generate synthetic receipt image
  const canvas = createCanvas(800, 1200);
  const ctx = canvas.getContext('2d');
  
  // Draw receipt background
  ctx.fillStyle = '#FFFFFF';
  ctx.fillRect(0, 0, 800, 1200);
  
  // Add merchant name
  ctx.font = 'bold 32px Arial';
  ctx.fillStyle = '#000000';
  ctx.fillText(faker.company.name(), 50, 100);
  
  // Add items
  let y = 200;
  for (let i = 0; i < 5; i++) {
    ctx.font = '24px Arial';
    ctx.fillText(faker.commerce.productName(), 50, y);
    ctx.fillText(`$${faker.commerce.price()}`, 600, y);
    y += 50;
  }
  
  // Add total
  ctx.font = 'bold 28px Arial';
  ctx.fillText('TOTAL:', 400, y + 50);
  ctx.fillText(`$${faker.commerce.price()}`, 600, y + 50);
  
  return canvas.toBuffer('image/png');
}

export function generateTestWikiNode(): WikiNode {
  return {
    id: faker.string.uuid(),
    node_type: 'entity',
    title: faker.company.name(),
    content: faker.lorem.paragraphs(3),
    backlinks: [],
    forward_links: [],
    provenance: {
      source_artifacts: [faker.string.uuid()],
      compiled_at: faker.date.recent().toISOString(),
      model_version: 'gemma-4-e2b-v1.2.0',
      confidence_score: faker.number.float({ min: 0.85, max: 0.99 }),
    },
    temporal_vectors: {
      mentioned_dates: [faker.date.recent().toISOString()],
    },
    privacy_classification: 'personal',
    encryption_status: 'aes256-gcm',
    version: generateVectorClock(),
  };
}
```

---

## 10. Defect Management

### 10.1 Severity Levels

| Level | Description | Response Time | Example |
|-------|-------------|---------------|---------|
| S1 | Red Line Violation | Immediate | Data leaves device |
| S2 | Security Vulnerability | 24 hours | Cryptographic weakness |
| S3 | Data Loss Risk | 48 hours | Sync corruption |
| S4 | Functional Defect | 1 week | UI glitch |
| S5 | Enhancement | Next sprint | Feature request |

### 10.2 Bug Report Template

```markdown
## Bug Report

**Severity:** [S1-S5]
**Component:** [Crypto/Sync/UI/etc]
**Environment:** [OS/Browser/Version]

### Description
[Clear description of the issue]

### Steps to Reproduce
1. [Step 1]
2. [Step 2]
3. [Step 3]

### Expected Behavior
[What should happen]

### Actual Behavior
[What actually happens]

### Evidence
- Screenshots:
- Logs:
- Test output:

### Red Line Impact
[If applicable, which Red Line is affected]
```

---

## Appendix A: Test Execution Checklist

### Pre-Release Checklist

- [ ] All unit tests pass (80%+ coverage)
- [ ] All integration tests pass
- [ ] All E2E tests pass (≥98%)
- [ ] Security audit clean (zero critical/high)
- [ ] Performance budgets met
- [ ] Red Lines verified
- [ ] SBOM generated
- [ ] Changelog updated
- [ ] Documentation updated

### Red Lines Verification

- [ ] **Sovereignty:** No raw image data leaves device
- [ ] **Zero-Knowledge:** Infrastructure cannot decrypt
- [ ] **Survival Mode:** App functions 100% offline
- [ ] **Exit Capability:** Export completes in <5 minutes
- [ ] **Economic Viability:** Bundle <100MB desktop, <50MB mobile

---

*Document Classification: Confidential*  
*Review Cycle: Weekly during MVP, Monthly post-launch*  
*Owner: QA/Security Auditor*
