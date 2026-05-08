# GraphQL API Reference

## Overview

The Truffle GraphQL API provides access to the knowledge graph with support for querying entities, relationships, and performing graph traversal operations. The API is designed for high performance with caching, pagination, and efficient query execution.

### Features

- **Entity Management**: Create, read, update, and delete knowledge entities
- **Relationship Traversal**: Navigate complex relationship graphs
- **Full-Text Search**: Search entities with relevance scoring
- **Validation**: Access contradiction detection and confidence scoring
- **Real-time Updates**: Subscribe to entity changes (WebSocket)

## Endpoint

```
POST /graphql
Content-Type: application/json
Authorization: Bearer <token>
```

### Base URL

- Development: `http://localhost:3000/graphql`
- Production: `https://api.truffle.dev/graphql`

### Headers

| Header | Required | Description |
|--------|----------|-------------|
| `Content-Type` | Yes | Must be `application/json` |
| `Authorization` | Yes | Bearer token for authentication |
| `X-Request-ID` | No | Unique request identifier for tracing |

---

## Queries

### entities

Retrieve a list of entities with optional filtering, pagination, and sorting.

**Arguments:**

| Name | Type | Required | Description |
|------|------|----------|-------------|
| filter | `EntityFilterInput` | No | Optional filter criteria |
| limit | `Int` | No | Maximum results (default: 50, max: 1000) |
| offset | `Int` | No | Pagination offset (default: 0) |
| sort | `EntitySortInput` | No | Sort configuration |
| includeDeleted | `Boolean` | No | Include soft-deleted entities (default: false) |

**Returns:** `EntityConnection`

**Example:**

```graphql
query GetEntities($filter: EntityFilterInput, $limit: Int, $offset: Int) {
  entities(filter: $filter, limit: $limit, offset: $offset) {
    nodes {
      id
      name
      entityType
      extractionConfidence
      createdAt
      updatedAt
    }
    pageInfo {
      hasNextPage
      hasPreviousPage
      totalCount
    }
  }
}
```

**Variables:**

```json
{
  "filter": {
    "entityType": "PERSON",
    "nameContains": "Alice",
    "privacyLevel": "PUBLIC",
    "minConfidence": 0.8
  },
  "limit": 10,
  "offset": 0
}
```

**Response:**

```json
{
  "data": {
    "entities": {
      "nodes": [
        {
          "id": "550e8400-e29b-41d4-a716-446655440000",
          "name": "Alice Smith",
          "entityType": "PERSON",
          "extractionConfidence": 0.92,
          "createdAt": "2024-01-15T10:30:00Z",
          "updatedAt": "2024-01-20T14:22:00Z"
        }
      ],
      "pageInfo": {
        "hasNextPage": true,
        "hasPreviousPage": false,
        "totalCount": 156
      }
    }
  }
}
```

---

### entity

Retrieve a single entity by ID with optional relationship depth.

**Arguments:**

| Name | Type | Required | Description |
|------|------|----------|-------------|
| id | `ID!` | Yes | Unique entity identifier |
| includeRelationships | `Boolean` | No | Include related entities (default: false) |
| relationshipDepth | `Int` | No | Depth of relationship traversal (default: 1, max: 3) |

**Returns:** `Entity`

**Example:**

```graphql
query GetEntity($id: ID!) {
  entity(id: $id, includeRelationships: true, relationshipDepth: 2) {
    id
    name
    entityType
    description
    metadata {
      key
      value
    }
    relationships {
      id
      relationType
      confidence
      target {
        id
        name
        entityType
      }
    }
    extractionConfidence
    privacyLevel
    createdAt
    updatedAt
  }
}
```

**Variables:**

```json
{
  "id": "550e8400-e29b-41d4-a716-446655440000"
}
```

---

### searchEntities

Search entities by text query using full-text search with relevance scoring.

**Arguments:**

| Name | Type | Required | Description |
|------|------|----------|-------------|
| query | `String!` | Yes | Search query string |
| entityTypes | `[EntityType!]` | No | Filter by entity types |
| limit | `Int` | No | Maximum results (default: 20) |
| minScore | `Float` | No | Minimum relevance score (default: 0.5) |

**Returns:** `[SearchResult!]!`

**Example:**

```graphql
query SearchEntities($query: String!) {
  searchEntities(
    query: $query,
    entityTypes: [PERSON, ORGANIZATION],
    limit: 10,
    minScore: 0.7
  ) {
    entity {
      id
      name
      entityType
    }
    score
    highlights
  }
}
```

**Variables:**

```json
{
  "query": "software engineer San Francisco"
}
```

**Response:**

```json
{
  "data": {
    "searchEntities": [
      {
        "entity": {
          "id": "550e8400-e29b-41d4-a716-446655440000",
          "name": "Alice Smith",
          "entityType": "PERSON"
        },
        "score": 0.95,
        "highlights": ["Software Engineer at <em>Google</em>"]
      }
    ]
  }
}
```

---

### relationships

Get relationships for a specific entity with filtering options.

**Arguments:**

| Name | Type | Required | Description |
|------|------|----------|-------------|
| entityId | `ID!` | Yes | Entity ID to query |
| relationType | `String` | No | Filter by relationship type |
| direction | `RelationshipDirection` | No | INCOMING, OUTGOING, or BOTH (default: BOTH) |
| minConfidence | `Float` | No | Minimum confidence threshold |

**Returns:** `RelationshipConnection`

**Example:**

```graphql
query GetRelationships($entityId: ID!) {
  relationships(
    entityId: $entityId,
    direction: OUTGOING,
    minConfidence: 0.8
  ) {
    nodes {
      id
      relationType
      confidence
      source {
        id
        name
      }
      target {
        id
        name
      }
      validFrom
      validUntil
    }
  }
}
```

---

### graph

Traverse the knowledge graph starting from a specific entity with configurable depth and filters.

**Arguments:**

| Name | Type | Required | Description |
|------|------|----------|-------------|
| startEntityId | `ID!` | Yes | Starting entity ID |
| depth | `Int` | No | Maximum traversal depth (default: 2, max: 5) |
| nodeLimit | `Int` | No | Maximum nodes to return (default: 100) |
| relationTypes | `[String!]` | No | Filter by relationship types |

**Returns:** `GraphTraversalResult`

**Example:**

```graphql
query GetGraph($startEntityId: ID!) {
  graph(
    startEntityId: $startEntityId,
    depth: 2,
    nodeLimit: 50
  ) {
    nodes {
      id
      name
      entityType
    }
    edges {
      sourceId
      targetId
      relationType
      confidence
    }
    paths {
      nodeIds
      edgeIds
      totalConfidence
    }
  }
}
```

---

### contradictions

Get validation contradictions with filtering and pagination.

**Arguments:**

| Name | Type | Required | Description |
|------|------|----------|-------------|
| filter | `ContradictionFilterInput` | No | Filter criteria |
| limit | `Int` | No | Maximum results (default: 50) |
| offset | `Int` | No | Pagination offset |

**Returns:** `ContradictionConnection`

**Example:**

```graphql
query GetContradictions {
  contradictions(
    filter: {
      severity: [HIGH, CRITICAL],
      isResolved: false
    },
    limit: 20
  ) {
    nodes {
      id
      entityId
      contradictionType
      severity
      description
      detectedAt
      involvedFacts {
        id
        value
        source
      }
    }
  }
}
```

---

## Mutations

### createEntity

Create a new entity in the knowledge graph.

**Arguments:**

| Name | Type | Required | Description |
|------|------|----------|-------------|
| input | `CreateEntityInput!` | Yes | Entity creation data |

**Returns:** `Entity`

**Example:**

```graphql
mutation CreateEntity($input: CreateEntityInput!) {
  createEntity(input: $input) {
    id
    name
    entityType
    extractionConfidence
    createdAt
  }
}
```

**Variables:**

```json
{
  "input": {
    "entityType": "PERSON",
    "name": "John Doe",
    "description": "Senior Software Engineer",
    "metadata": [
      { "key": "email", "value": "john@example.com" },
      { "key": "department", "value": "Engineering" }
    ],
    "privacyLevel": "PRIVATE",
    "source": "manual_entry"
  }
}
```

---

### updateEntity

Update an existing entity's properties.

**Arguments:**

| Name | Type | Required | Description |
|------|------|----------|-------------|
| id | `ID!` | Yes | Entity ID to update |
| input | `UpdateEntityInput!` | Yes | Entity update data |

**Returns:** `Entity`

**Example:**

```graphql
mutation UpdateEntity($id: ID!, $input: UpdateEntityInput!) {
  updateEntity(id: $id, input: $input) {
    id
    name
    description
    updatedAt
  }
}
```

---

### deleteEntity

Soft-delete an entity (can be restored later).

**Arguments:**

| Name | Type | Required | Description |
|------|------|----------|-------------|
| id | `ID!` | Yes | Entity ID to delete |
| hardDelete | `Boolean` | No | Permanently delete (default: false) |

**Returns:** `Boolean`

**Example:**

```graphql
mutation DeleteEntity($id: ID!) {
  deleteEntity(id: $id)
}
```

---

### createRelationship

Create a relationship between two entities.

**Arguments:**

| Name | Type | Required | Description |
|------|------|----------|-------------|
| input | `CreateRelationshipInput!` | Yes | Relationship data |

**Returns:** `Relationship`

**Example:**

```graphql
mutation CreateRelationship($input: CreateRelationshipInput!) {
  createRelationship(input: $input) {
    id
    source {
      id
      name
    }
    target {
      id
      name
    }
    relationType
    confidence
  }
}
```

**Variables:**

```json
{
  "input": {
    "sourceId": "550e8400-e29b-41d4-a716-446655440000",
    "targetId": "660e8400-e29b-41d4-a716-446655440001",
    "relationType": "works_at",
    "confidence": 0.95,
    "validFrom": "2023-01-01T00:00:00Z"
  }
}
```

---

### mergeEntities

Merge duplicate entities into a single canonical entity.

**Arguments:**

| Name | Type | Required | Description |
|------|------|----------|-------------|
| primaryId | `ID!` | Yes | Target entity ID (kept) |
| secondaryId | `ID!` | Yes | Source entity ID (merged and deleted) |
| mergeConfig | `MergeConfigInput` | No | Merge configuration options |

**Returns:** `Entity`

**Example:**

```graphql
mutation MergeEntities($primaryId: ID!, $secondaryId: ID!) {
  mergeEntities(
    primaryId: $primaryId,
    secondaryId: $secondaryId,
    mergeConfig: {
      keepSourceMetadata: true,
      mergeRelationships: true
    }
  ) {
    id
    name
    mergedFrom {
      id
      name
    }
  }
}
```

---

## Types

### Entity

Core entity type representing a node in the knowledge graph.

| Field | Type | Description |
|-------|------|-------------|
| `id` | `ID!` | Unique identifier (UUID) |
| `name` | `String!` | Display name |
| `slug` | `String!` | URL-friendly identifier |
| `entityType` | `EntityType!` | Entity classification |
| `description` | `String` | Optional description |
| `metadata` | `[EntityMetadata!]!` | Key-value metadata pairs |
| `extractionConfidence` | `Float!` | Confidence score (0.0 - 1.0) |
| `privacyLevel` | `PrivacyLevel!` | Data privacy classification |
| `createdAt` | `DateTime!` | Creation timestamp |
| `updatedAt` | `DateTime!` | Last update timestamp |
| `deletedAt` | `DateTime` | Soft-delete timestamp |
| `source` | `String` | Data source identifier |
| `relationships` | `RelationshipConnection` | Associated relationships |
| `facts` | `FactConnection` | Extracted facts |

### EntityType

Enum defining valid entity types.

| Value | Description |
|-------|-------------|
| `PERSON` | Individual person |
| `ORGANIZATION` | Company, group, or institution |
| `LOCATION` | Geographic location |
| `EVENT` | Temporal occurrence |
| `PRODUCT` | Goods or services |
| `CONCEPT` | Abstract idea or topic |
| `DECISION` | Recorded decision |
| `ACTION_ITEM` | Task or action item |

### Relationship

Connection between two entities.

| Field | Type | Description |
|-------|------|-------------|
| `id` | `ID!` | Unique identifier |
| `source` | `Entity!` | Source entity |
| `target` | `Entity!` | Target entity |
| `relationType` | `String!` | Relationship classification |
| `confidence` | `Float!` | Confidence score |
| `validFrom` | `DateTime` | Relationship start |
| `validUntil` | `DateTime` | Relationship end |
| `metadata` | `JSON` | Additional properties |

### Contradiction

Detected inconsistency between facts.

| Field | Type | Description |
|-------|------|-------------|
| `id` | `ID!` | Unique identifier |
| `entityId` | `ID!` | Affected entity |
| `factAId` | `ID!` | First conflicting fact |
| `factBId` | `ID!` | Second conflicting fact |
| `contradictionType` | `ContradictionType!` | Type of conflict |
| `severity` | `Severity!` | Impact level |
| `description` | `String!` | Human-readable explanation |
| `detectedAt` | `DateTime!` | Detection timestamp |
| `isResolved` | `Boolean!` | Resolution status |

### ContradictionType

Types of contradictions.

- `TEMPORAL` - Time-based conflicts
- `FACTUAL` - Direct factual disagreement
- `SOURCE_CONFLICT` - Different sources disagree
- `LOGICAL` - Logical inconsistency
- `VALUE_MISMATCH` - Attribute value conflict

### Severity

Contradiction severity levels.

- `LOW` - Minor inconsistency
- `MEDIUM` - Moderate impact
- `HIGH` - Significant issue
- `CRITICAL` - Requires immediate attention

---

## Error Handling

### Error Format

```json
{
  "errors": [
    {
      "message": "Entity not found",
      "extensions": {
        "code": "ENTITY_NOT_FOUND",
        "entityId": "550e8400-e29b-41d4-a716-446655440000"
      }
    }
  ]
}
```

### Error Codes

| Code | HTTP Status | Description |
|------|-------------|-------------|
| `ENTITY_NOT_FOUND` | 404 | Requested entity does not exist |
| `INVALID_INPUT` | 400 | Input validation failed |
| `DUPLICATE_ENTITY` | 409 | Entity with same name already exists |
| `UNAUTHORIZED` | 401 | Authentication required |
| `FORBIDDEN` | 403 | Insufficient permissions |
| `RELATIONSHIP_NOT_FOUND` | 404 | Relationship does not exist |
| `CIRCULAR_REFERENCE` | 400 | Would create circular dependency |
| `RATE_LIMITED` | 429 | Too many requests |
| `INTERNAL_ERROR` | 500 | Server error |

### Validation Errors

Input validation failures return detailed field-level errors:

```json
{
  "errors": [
    {
      "message": "Validation failed",
      "extensions": {
        "code": "INVALID_INPUT",
        "validationErrors": [
          {
            "field": "name",
            "message": "Name must be at least 2 characters"
          },
          {
            "field": "entityType",
            "message": "Invalid entity type"
          }
        ]
      }
    }
  ]
}
```

---

## Rate Limits

| Operation | Limit | Window |
|-----------|-------|--------|
| Queries | 1000 | 1 minute |
| Mutations | 100 | 1 minute |
| Complex queries | 50 | 1 minute |
| Subscriptions | 10 concurrent | - |

Rate limit headers are included in all responses:

```
X-RateLimit-Limit: 1000
X-RateLimit-Remaining: 999
X-RateLimit-Reset: 1704067200
```
