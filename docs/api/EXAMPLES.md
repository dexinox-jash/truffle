# API Usage Examples

## Overview

This document provides practical code examples for common operations using the Truffle API. Examples are provided in TypeScript for both GraphQL and Tauri command usage.

---

## Entity CRUD Examples

### Create a Person

```typescript
import { invoke } from '@tauri-apps/api/tauri';

// Create a person entity
const person = await invoke('create_entity', {
  input: {
    entityType: 'Person',
    name: 'John Doe',
    description: 'Senior Software Engineer with expertise in distributed systems',
    metadata: {
      email: 'john.doe@example.com',
      role: 'Senior Engineer',
      department: 'Engineering',
      location: 'San Francisco, CA',
      skills: ['Rust', 'TypeScript', 'GraphQL', 'Distributed Systems']
    },
    privacyLevel: 'Private',
    source: 'manual_entry'
  }
});

console.log('Created person:', person.id);
```

### Create an Organization

```typescript
// Create an organization entity
const org = await invoke('create_entity', {
  input: {
    entityType: 'Organization',
    name: 'Acme Corporation',
    description: 'Leading technology company specializing in cloud solutions',
    metadata: {
      industry: 'Technology',
      foundedYear: 2020,
      website: 'https://acme.example.com',
      size: '100-500',
      headquarters: 'San Francisco, CA'
    },
    privacyLevel: 'Public'
  }
});
```

### Create an Event

```typescript
// Create an event entity
const event = await invoke('create_entity', {
  input: {
    entityType: 'Event',
    name: 'Q4 Engineering All-Hands',
    description: 'Quarterly engineering team meeting',
    metadata: {
      date: '2024-12-15',
      time: '14:00',
      location: 'Conference Room A',
      attendees: ['john.doe@example.com', 'jane.smith@example.com']
    }
  }
});
```

---

## Relationship Examples

### Create Employment Relationship

```typescript
// Create a "works_at" relationship between person and organization
await invoke('create_relationship', {
  input: {
    sourceId: person.id,
    targetId: org.id,
    relationType: 'works_at',
    confidence: 0.95,
    validFrom: '2020-03-15T00:00:00Z',
    metadata: {
      role: 'Senior Engineer',
      department: 'Engineering'
    }
  }
});
```

### Create Reporting Structure

```typescript
// Create a "reports_to" relationship
const manager = await invoke('create_entity', {
  input: {
    entityType: 'Person',
    name: 'Jane Smith',
    metadata: { role: 'Engineering Manager' }
  }
});

await invoke('create_relationship', {
  input: {
    sourceId: person.id,
    targetId: manager.id,
    relationType: 'reports_to',
    confidence: 1.0,
    validFrom: '2020-03-15T00:00:00Z'
  }
});
```

### Create Event Participation

```typescript
// Mark person as attending event
await invoke('create_relationship', {
  input: {
    sourceId: person.id,
    targetId: event.id,
    relationType: 'attended',
    confidence: 0.9,
    metadata: {
      rsvpStatus: 'confirmed',
      notes: 'Will present on distributed systems'
    }
  }
});
```

---

## Graph Query Examples

### Get Entity Graph

```typescript
// Get 2-hop neighborhood around an entity
const graph = await invoke('get_entity_graph', {
  entityId: person.id,
  depth: 2,
  nodeLimit: 50
});

console.log(`Found ${graph.nodes.length} nodes and ${graph.edges.length} edges`);

// Process the graph
graph.nodes.forEach(node => {
  console.log(`${node.entityType}: ${node.name}`);
});

// Find specific relationship types
const worksAtEdges = graph.edges.filter(
  e => e.relationType === 'works_at'
);
```

### Find Path Between Entities

```typescript
// Find connection path between two people
const paths = await invoke('find_path', {
  fromId: person1.id,
  toId: person2.id,
  maxDepth: 5,
  relationTypes: ['knows', 'works_at', 'reports_to']
});

if (paths.length > 0) {
  const shortestPath = paths[0];
  console.log('Connection found:');
  shortestPath.nodes.forEach((node, i) => {
    console.log(`${i + 1}. ${node.name} (${node.entityType})`);
  });
}
```

### Get Organizational Hierarchy

```typescript
// Get all reports under a manager
async function getReports(managerId: string, depth = 3): Promise<Entity[]> {
  const graph = await invoke('get_entity_graph', {
    entityId: managerId,
    depth,
    relationTypes: ['reports_to']
  });
  
  // Filter for incoming "reports_to" relationships
  return graph.nodes.filter(node => node.id !== managerId);
}

const reports = await getReports(manager.id);
console.log(`Manager has ${reports.length} reports in hierarchy`);
```

---

## Search Examples

### Full-Text Search

```typescript
// Search for entities by text
const results = await invoke('search_entities', {
  query: 'software engineer San Francisco',
  entityTypes: ['Person'],
  limit: 10,
  minScore: 0.7
});

results.forEach(result => {
  console.log(`${result.entity.name} (score: ${result.score.toFixed(2)})`);
  console.log(`  Highlights: ${result.highlights.join(', ')}`);
});
```

### Filtered Entity List

```typescript
// Get all people in engineering
const engineers = await invoke('get_entities', {
  filter: {
    entityType: 'Person',
    metadataEquals: { key: 'department', value: 'Engineering' }
  },
  sort: { field: 'name', direction: 'asc' }
});
```

---

## Validation Examples

### Run Entity Validation

```typescript
// Validate an entity for contradictions
const validation = await invoke('validate_entity', {
  entityId: person.id,
  validationTypes: ['temporal', 'factual', 'source_conflict']
});

if (!validation.isValid) {
  console.warn(`Found ${validation.contradictions.length} contradictions:`);
  validation.contradictions.forEach(c => {
    console.warn(`- [${c.severity}] ${c.description}`);
  });
}

// Log warnings
validation.warnings.forEach(w => {
  console.log(`Warning: ${w.message}`);
  if (w.suggestion) {
    console.log(`  Suggestion: ${w.suggestion}`);
  }
});
```

### Get All Contradictions

```typescript
// Get high and critical contradictions
const contradictions = await invoke('get_contradictions', {
  filter: {
    severity: ['high', 'critical'],
    isResolved: false
  },
  limit: 50
});

// Group by entity
const byEntity = contradictions.reduce((acc, c) => {
  acc[c.entityId] = acc[c.entityId] || [];
  acc[c.entityId].push(c);
  return acc;
}, {} as Record<string, Contradiction[]>);
```

### Resolve Contradiction

```typescript
// Mark a contradiction as resolved
await invoke('resolve_contradiction', {
  contradictionId: 'contradiction-uuid',
  resolution: 'Verified with primary source - fact B is correct'
});
```

---

## Duplicate Detection Examples

### Find Duplicate Candidates

```typescript
// Find potential duplicates across all entities
const duplicates = await invoke('find_duplicates', {
  threshold: 0.85,
  entityType: 'Person'
});

// Review candidates
for (const candidate of duplicates) {
  console.log(`Potential duplicate:`);
  console.log(`  A: ${candidate.entityA.name}`);
  console.log(`  B: ${candidate.entityB.name}`);
  console.log(`  Similarity: ${(candidate.similarity * 100).toFixed(1)}%`);
  console.log(`  Matching fields: ${candidate.matchingFields.join(', ')}`);
  console.log(`  Recommendation: ${candidate.recommendation}`);
}
```

### Check Specific Entity

```typescript
// Check if a specific entity has duplicates
const candidates = await invoke('find_duplicates', {
  entityId: person.id,
  threshold: 0.9
});

if (candidates.length > 0) {
  console.log(`Found ${candidates.length} potential duplicates`);
}
```

---

## Merge Examples

### Merge Duplicate Entities

```typescript
// Merge two duplicate entities
const merged = await invoke('merge_entities', {
  primaryId: entity1.id,
  secondaryId: entity2.id,
  config: {
    keepSourceMetadata: true,
    mergeRelationships: true,
    preferredName: 'primary' // or 'secondary', 'longest'
  }
});

console.log(`Merged into entity: ${merged.id}`);
console.log(`Merged from: ${merged.mergedFrom?.map(e => e.name).join(', ')}`);
```

### Bulk Merge Reviewed Duplicates

```typescript
// Automatically merge high-confidence duplicates
async function bulkMerge(duplicates: DuplicateCandidate[]): Promise<void> {
  for (const dup of duplicates) {
    if (dup.recommendation === 'merge' && dup.similarity > 0.95) {
      try {
        await invoke('merge_entities', {
          primaryId: dup.entityA.id,
          secondaryId: dup.entityB.id,
          config: { mergeRelationships: true }
        });
        console.log(`Merged: ${dup.entityA.name}`);
      } catch (error) {
        console.error(`Failed to merge: ${error}`);
      }
    }
  }
}
```

---

## Advanced Query Patterns

### Complex Filter Query

```typescript
// Find recently updated high-confidence persons
const recentPersons = await invoke('get_entities', {
  filter: {
    entityType: 'Person',
    minConfidence: 0.9,
    createdAfter: '2024-01-01T00:00:00Z'
  },
  limit: 100,
  sort: { field: 'updatedAt', direction: 'desc' }
});
```

### Batch Operations

```typescript
// Batch create multiple entities
async function batchCreateEntities(
  inputs: CreateEntityInput[]
): Promise<Entity[]> {
  const results: Entity[] = [];
  
  for (const input of inputs) {
    try {
      const entity = await invoke('create_entity', { input });
      results.push(entity);
    } catch (error) {
      console.error(`Failed to create ${input.name}:`, error);
    }
  }
  
  return results;
}

// Usage
const people = [
  { entityType: 'Person', name: 'Alice', metadata: { role: 'Developer' } },
  { entityType: 'Person', name: 'Bob', metadata: { role: 'Designer' } },
];

const created = await batchCreateEntities(people);
```

### Error Handling Pattern

```typescript
// Robust error handling wrapper
async function safeInvoke<T>(
  command: string,
  args: Record<string, unknown>
): Promise<{ success: true; data: T } | { success: false; error: string }> {
  try {
    const result = await invoke<T>(command, args);
    return { success: true, data: result };
  } catch (error) {
    const errorMessage = error instanceof Error ? error.message : String(error);
    
    // Log for debugging
    console.error(`Command ${command} failed:`, errorMessage);
    
    return { success: false, error: errorMessage };
  }
}

// Usage
const result = await safeInvoke<Entity>('get_entity', { id });

if (result.success) {
  console.log('Entity:', result.data);
} else {
  if (result.error.includes('not found')) {
    // Handle 404
  } else {
    // Handle other errors
  }
}
```

---

## GraphQL Integration Examples

### Using GraphQL Client

```typescript
import { request, gql } from 'graphql-request';

const endpoint = 'http://localhost:3000/graphql';

// Query with variables
const query = gql`
  query GetEntity($id: ID!) {
    entity(id: $id) {
      id
      name
      entityType
      relationships {
        relationType
        target {
          name
        }
      }
    }
  }
`;

const data = await request(endpoint, query, { id: 'entity-uuid' });
```

### GraphQL Mutation

```typescript
const mutation = gql`
  mutation CreateEntity($input: CreateEntityInput!) {
    createEntity(input: $input) {
      id
      name
      createdAt
    }
  }
`;

const result = await request(endpoint, mutation, {
  input: {
    entityType: 'PERSON',
    name: 'New Person',
    description: 'Created via GraphQL'
  }
});
```
