# Tauri Commands Reference

## Overview

The Truffle desktop application exposes a set of Tauri commands for interacting with the local knowledge graph database. These commands provide a Rust-based API that can be invoked from the frontend JavaScript/TypeScript code.

### Architecture

Commands are organized by domain:
- **Entity Commands**: CRUD operations for entities
- **Relationship Commands**: Managing entity connections
- **Graph Commands**: Graph traversal and analysis
- **Validation Commands**: Data quality and consistency checks
- **Search Commands**: Full-text and semantic search
- **Sync Commands**: Data synchronization

---

## Entity Commands

### get_entities

Retrieve a paginated list of entities with optional filtering.

**Parameters:**

| Name | Type | Optional | Description |
|------|------|----------|-------------|
| `filter` | `EntityFilter` | Yes | Filter criteria including type, name, tags |
| `limit` | `number` | Yes | Maximum results (default: 50, max: 1000) |
| `offset` | `number` | Yes | Pagination offset (default: 0) |
| `sort` | `EntitySort` | Yes | Sort field and direction |

**Returns:** `Promise<EntitySummary[]>`

**Example:**

```typescript
import { invoke } from '@tauri-apps/api/tauri';

const entities = await invoke('get_entities', {
  filter: { 
    entityType: 'Person',
    nameContains: 'Smith',
    minConfidence: 0.8
  },
  limit: 50,
  offset: 0,
  sort: { field: 'updatedAt', direction: 'desc' }
});
```

---

### get_entity

Get a single entity by its unique identifier.

**Parameters:**

| Name | Type | Optional | Description |
|------|------|----------|-------------|
| `id` | `string` | No | Entity UUID |
| `includeRelationships` | `boolean` | Yes | Include related entities |
| `relationshipDepth` | `number` | Yes | Depth of relationship traversal |

**Returns:** `Promise<Entity>`

**Example:**

```typescript
const entity = await invoke('get_entity', {
  id: '550e8400-e29b-41d4-a716-446655440000',
  includeRelationships: true,
  relationshipDepth: 1
});
```

---

### create_entity

Create a new entity in the knowledge graph.

**Parameters:**

| Name | Type | Optional | Description |
|------|------|----------|-------------|
| `input` | `CreateEntityInput` | No | Entity creation data |

**Returns:** `Promise<Entity>`

**Example:**

```typescript
const entity = await invoke('create_entity', {
  input: {
    entityType: 'Person',
    name: 'John Doe',
    description: 'Software Engineer',
    metadata: {
      email: 'john@example.com',
      department: 'Engineering'
    },
    privacyLevel: 'Private',
    source: 'manual_entry'
  }
});
```

---

### update_entity

Update an existing entity's properties.

**Parameters:**

| Name | Type | Optional | Description |
|------|------|----------|-------------|
| `id` | `string` | No | Entity UUID |
| `input` | `UpdateEntityInput` | No | Update data (partial) |

**Returns:** `Promise<Entity>`

**Example:**

```typescript
const updated = await invoke('update_entity', {
  id: '550e8400-e29b-41d4-a716-446655440000',
  input: {
    name: 'Johnathan Doe',
    description: 'Senior Software Engineer',
    metadata: {
      title: 'Senior Engineer'
    }
  }
});
```

---

### delete_entity

Delete an entity (soft delete by default).

**Parameters:**

| Name | Type | Optional | Description |
|------|------|----------|-------------|
| `id` | `string` | No | Entity UUID |
| `hardDelete` | `boolean` | Yes | Permanently delete (default: false) |

**Returns:** `Promise<boolean>`

**Example:**

```typescript
const deleted = await invoke('delete_entity', {
  id: '550e8400-e29b-41d4-a716-446655440000'
});

// Hard delete
await invoke('delete_entity', {
  id: '550e8400-e29b-41d4-a716-446655440000',
  hardDelete: true
});
```

---

### search_entities

Search entities using full-text search.

**Parameters:**

| Name | Type | Optional | Description |
|------|------|----------|-------------|
| `query` | `string` | No | Search query string |
| `entityTypes` | `string[]` | Yes | Filter by entity types |
| `limit` | `number` | Yes | Maximum results (default: 20) |
| `minScore` | `number` | Yes | Minimum relevance score |

**Returns:** `Promise<SearchResult[]>`

**Example:**

```typescript
const results = await invoke('search_entities', {
  query: 'software engineer',
  entityTypes: ['Person', 'Organization'],
  limit: 10,
  minScore: 0.7
});
```

---

### find_duplicates

Find potential duplicate entities based on similarity.

**Parameters:**

| Name | Type | Optional | Description |
|------|------|----------|-------------|
| `entityId` | `string` | Yes | Check specific entity, or all if omitted |
| `threshold` | `number` | Yes | Similarity threshold (0.0 - 1.0, default: 0.85) |
| `entityType` | `string` | Yes | Filter by entity type |

**Returns:** `Promise<DuplicateCandidate[]>`

**Example:**

```typescript
const duplicates = await invoke('find_duplicates', {
  threshold: 0.85,
  entityType: 'Person'
});

// Check specific entity
const entityDuplicates = await invoke('find_duplicates', {
  entityId: '550e8400-e29b-41d4-a716-446655440000',
  threshold: 0.9
});
```

---

## Relationship Commands

### get_relationships

Get relationships for a specific entity.

**Parameters:**

| Name | Type | Optional | Description |
|------|------|----------|-------------|
| `entityId` | `string` | No | Entity UUID |
| `direction` | `string` | Yes | 'incoming', 'outgoing', or 'both' (default: 'both') |
| `relationType` | `string` | Yes | Filter by relationship type |
| `minConfidence` | `number` | Yes | Minimum confidence threshold |

**Returns:** `Promise<Relationship[]>`

**Example:**

```typescript
const relationships = await invoke('get_relationships', {
  entityId: '550e8400-e29b-41d4-a716-446655440000',
  direction: 'outgoing',
  relationType: 'works_at',
  minConfidence: 0.8
});
```

---

### create_relationship

Create a new relationship between entities.

**Parameters:**

| Name | Type | Optional | Description |
|------|------|----------|-------------|
| `input` | `CreateRelationshipInput` | No | Relationship data |

**Returns:** `Promise<Relationship>`

**Example:**

```typescript
const relationship = await invoke('create_relationship', {
  input: {
    sourceId: '550e8400-e29b-41d4-a716-446655440000',
    targetId: '660e8400-e29b-41d4-a716-446655440001',
    relationType: 'works_at',
    confidence: 0.95,
    validFrom: '2023-01-01T00:00:00Z'
  }
});
```

---

### update_relationship

Update an existing relationship.

**Parameters:**

| Name | Type | Optional | Description |
|------|------|----------|-------------|
| `id` | `string` | No | Relationship UUID |
| `input` | `UpdateRelationshipInput` | No | Update data |

**Returns:** `Promise<Relationship>`

---

### delete_relationship

Delete a relationship.

**Parameters:**

| Name | Type | Optional | Description |
|------|------|----------|-------------|
| `id` | `string` | No | Relationship UUID |

**Returns:** `Promise<boolean>`

---

## Graph Commands

### get_entity_graph

Get a graph view starting from a specific entity.

**Parameters:**

| Name | Type | Optional | Description |
|------|------|----------|-------------|
| `entityId` | `string` | No | Starting entity UUID |
| `depth` | `number` | Yes | Traversal depth (default: 2, max: 5) |
| `nodeLimit` | `number` | Yes | Maximum nodes to return |
| `relationTypes` | `string[]` | Yes | Filter by relationship types |

**Returns:** `Promise<GraphResult>`

**Example:**

```typescript
const graph = await invoke('get_entity_graph', {
  entityId: '550e8400-e29b-41d4-a716-446655440000',
  depth: 2,
  nodeLimit: 50
});

// Returns: { nodes: Entity[], edges: Relationship[], paths: Path[] }
```

---

### find_path

Find paths between two entities.

**Parameters:**

| Name | Type | Optional | Description |
|------|------|----------|-------------|
| `fromId` | `string` | No | Starting entity UUID |
| `toId` | `string` | No | Target entity UUID |
| `maxDepth` | `number` | Yes | Maximum path length (default: 5) |
| `relationTypes` | `string[]` | Yes | Allowed relationship types |

**Returns:** `Promise<PathResult[]>`

**Example:**

```typescript
const paths = await invoke('find_path', {
  fromId: '550e8400-e29b-41d4-a716-446655440000',
  toId: '660e8400-e29b-41d4-a716-446655440001',
  maxDepth: 4
});
```

---

### get_connected_components

Find connected components in the graph.

**Parameters:**

| Name | Type | Optional | Description |
|------|------|----------|-------------|
| `minSize` | `number` | Yes | Minimum component size |
| `maxSize` | `number` | Yes | Maximum component size |

**Returns:** `Promise<Component[]>`

---

## Validation Commands

### validate_entity

Run validation checks on a specific entity.

**Parameters:**

| Name | Type | Optional | Description |
|------|------|----------|-------------|
| `entityId` | `string` | No | Entity UUID |
| `validationTypes` | `string[]` | Yes | Specific validations to run |

**Returns:** `Promise<ValidationResult>`

**Example:**

```typescript
const result = await invoke('validate_entity', {
  entityId: '550e8400-e29b-41d4-a716-446655440000',
  validationTypes: ['temporal', 'factual', 'source_conflict']
});

// Returns: { isValid: boolean, contradictions: Contradiction[], warnings: string[] }
```

---

### get_contradictions

Get all validation contradictions.

**Parameters:**

| Name | Type | Optional | Description |
|------|------|----------|-------------|
| `filter` | `ContradictionFilter` | Yes | Filter by severity, type, status |
| `limit` | `number` | Yes | Maximum results |
| `offset` | `number` | Yes | Pagination offset |

**Returns:** `Promise<Contradiction[]>`

**Example:**

```typescript
const contradictions = await invoke('get_contradictions', {
  filter: {
    severity: ['high', 'critical'],
    isResolved: false
  },
  limit: 50
});
```

---

### get_confidence_score

Get confidence metrics for an entity.

**Parameters:**

| Name | Type | Optional | Description |
|------|------|----------|-------------|
| `entityId` | `string` | No | Entity UUID |
| `includeFactors` | `boolean` | Yes | Include breakdown of factors |

**Returns:** `Promise<ConfidenceScore>`

**Example:**

```typescript
const confidence = await invoke('get_confidence_score', {
  entityId: '550e8400-e29b-41d4-a716-446655440000',
  includeFactors: true
});

// Returns: { score: number, factors: { extraction: number, validation: number, ... } }
```

---

### resolve_contradiction

Mark a contradiction as resolved.

**Parameters:**

| Name | Type | Optional | Description |
|------|------|----------|-------------|
| `contradictionId` | `string` | No | Contradiction UUID |
| `resolution` | `string` | Yes | Resolution notes |

**Returns:** `Promise<boolean>`

---

### merge_entities

Merge two duplicate entities.

**Parameters:**

| Name | Type | Optional | Description |
|------|------|----------|-------------|
| `primaryId` | `string` | No | Target entity (kept) |
| `secondaryId` | `string` | No | Source entity (merged and deleted) |
| `config` | `MergeConfig` | Yes | Merge configuration |

**Returns:** `Promise<Entity>`

**Example:**

```typescript
const merged = await invoke('merge_entities', {
  primaryId: '550e8400-e29b-41d4-a716-446655440000',
  secondaryId: '660e8400-e29b-41d4-a716-446655440001',
  config: {
    keepSourceMetadata: true,
    mergeRelationships: true,
    preferredName: 'primary'
  }
});
```

---

## Error Handling

All commands return `Result<T, String>` where errors are string messages. Handle errors using try-catch:

```typescript
try {
  const entity = await invoke('get_entity', { id });
  console.log('Entity:', entity);
} catch (error) {
  console.error('Failed to get entity:', error);
  
  // Common error patterns
  if (error.includes('not found')) {
    // Handle missing entity
  } else if (error.includes('unauthorized')) {
    // Handle auth error
  } else {
    // Handle general error
  }
}
```

### Common Error Messages

| Error Pattern | Description | Handling |
|---------------|-------------|----------|
| `Entity not found` | Invalid or deleted ID | Verify ID, check if deleted |
| `Invalid input` | Validation failed | Check input parameters |
| `Circular reference` | Would create cycle | Restructure relationship |
| `Duplicate entity` | Name/type conflict | Use different name or merge |
| `Database error` | Internal DB issue | Retry or report |

### Type-Safe Invokes

For better TypeScript support, wrap invocations:

```typescript
async function getEntity(id: string): Promise<Entity> {
  return invoke<Entity>('get_entity', { id });
}

async function safeGetEntity(id: string): Promise<Entity | null> {
  try {
    return await getEntity(id);
  } catch (error) {
    console.error(error);
    return null;
  }
}
```
