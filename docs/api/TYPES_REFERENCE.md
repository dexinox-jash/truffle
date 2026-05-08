# TypeScript Types Reference

## Overview

This document provides complete TypeScript type definitions for the Truffle API, including entities, relationships, validation results, and input types.

---

## Entity Types

### Entity

Core entity interface representing a node in the knowledge graph.

```typescript
interface Entity {
  /** Unique identifier (UUID v4) */
  id: string;
  
  /** Entity classification type */
  entityType: EntityType;
  
  /** Display name */
  name: string;
  
  /** URL-friendly identifier */
  slug: string;
  
  /** Optional description */
  description?: string;
  
  /** Key-value metadata pairs */
  metadata: EntityMetadata;
  
  /** Extraction confidence score (0.0 - 1.0) */
  extractionConfidence: number;
  
  /** Data privacy classification */
  privacyLevel: PrivacyLevel;
  
  /** ISO 8601 creation timestamp */
  createdAt: string;
  
  /** ISO 8601 last update timestamp */
  updatedAt: string;
  
  /** ISO 8601 soft-delete timestamp (if deleted) */
  deletedAt?: string;
  
  /** Data source identifier */
  source?: string;
  
  /** Associated relationships (when requested) */
  relationships?: Relationship[];
  
  /** Extracted facts */
  facts?: Fact[];
}
```

### EntityType

Enum defining valid entity classifications.

```typescript
enum EntityType {
  Person = 'person',
  Organization = 'organization',
  Location = 'location',
  Event = 'event',
  Product = 'product',
  Concept = 'concept',
  Decision = 'decision',
  ActionItem = 'action_item',
}
```

### EntityMetadata

Flexible metadata container for entity attributes.

```typescript
interface EntityMetadata {
  [key: string]: string | number | boolean | string[] | undefined;
}

// Common metadata fields by type
interface PersonMetadata extends EntityMetadata {
  email?: string;
  phone?: string;
  role?: string;
  department?: string;
  location?: string;
}

interface OrganizationMetadata extends EntityMetadata {
  industry?: string;
  foundedYear?: number;
  website?: string;
  size?: string;
  headquarters?: string;
}
```

### EntitySummary

Lightweight entity representation for list views.

```typescript
interface EntitySummary {
  id: string;
  entityType: EntityType;
  name: string;
  slug: string;
  extractionConfidence: number;
  updatedAt: string;
}
```

### PrivacyLevel

Data privacy classification levels.

```typescript
enum PrivacyLevel {
  Public = 'public',
  Internal = 'internal',
  Private = 'private',
  Restricted = 'restricted',
}
```

---

## Relationship Types

### Relationship

Connection between two entities with optional temporal validity.

```typescript
interface Relationship {
  /** Unique identifier */
  id: string;
  
  /** Source entity ID */
  sourceId: string;
  
  /** Source entity (when expanded) */
  source?: EntitySummary;
  
  /** Target entity ID */
  targetId: string;
  
  /** Target entity (when expanded) */
  target?: EntitySummary;
  
  /** Relationship classification */
  relationType: string;
  
  /** Confidence score (0.0 - 1.0) */
  confidence: number;
  
  /** ISO 8601 relationship start date */
  validFrom?: string;
  
  /** ISO 8601 relationship end date */
  validUntil?: string;
  
  /** Additional properties */
  metadata?: Record<string, unknown>;
  
  /** ISO 8601 creation timestamp */
  createdAt: string;
}
```

### RelationType

Common relationship types.

```typescript
enum RelationType {
  // Person relationships
  WorksAt = 'works_at',
  ReportsTo = 'reports_to',
  Knows = 'knows',
  RelatedTo = 'related_to',
  LocatedAt = 'located_at',
  
  // Organization relationships
  Owns = 'owns',
  PartnersWith = 'partners_with',
  CompetesWith = 'competes_with',
  Acquired = 'acquired',
  
  // Event relationships
  Attended = 'attended',
  Organized = 'organized',
  SpokeAt = 'spoke_at',
  
  // General
  PartOf = 'part_of',
  DependsOn = 'depends_on',
  Caused = 'caused',
  References = 'references',
}
```

### RelationshipDirection

Direction filter for relationship queries.

```typescript
enum RelationshipDirection {
  Incoming = 'incoming',
  Outgoing = 'outgoing',
  Both = 'both',
}
```

---

## Validation Types

### Contradiction

Detected inconsistency between facts.

```typescript
interface Contradiction {
  /** Unique identifier */
  id: string;
  
  /** Affected entity ID */
  entityId: string;
  
  /** First conflicting fact ID */
  factAId: string;
  
  /** Second conflicting fact ID */
  factBId: string;
  
  /** Type of contradiction */
  contradictionType: ContradictionType;
  
  /** Impact severity */
  severity: Severity;
  
  /** Human-readable explanation */
  description: string;
  
  /** ISO 8601 detection timestamp */
  detectedAt: string;
  
  /** Resolution status */
  isResolved: boolean;
  
  /** ISO 8601 resolution timestamp */
  resolvedAt?: string;
  
  /** Resolution notes */
  resolution?: string;
  
  /** Involved facts (when expanded) */
  involvedFacts?: Fact[];
}
```

### ContradictionType

Types of contradictions that can be detected.

```typescript
enum ContradictionType {
  Temporal = 'temporal',
  Factual = 'factual',
  SourceConflict = 'source_conflict',
  Logical = 'logical',
  ValueMismatch = 'value_mismatch',
}
```

### Severity

Contradiction severity levels.

```typescript
enum Severity {
  Low = 'low',
  Medium = 'medium',
  High = 'high',
  Critical = 'critical',
}
```

### ValidationResult

Entity validation outcome.

```typescript
interface ValidationResult {
  /** Overall validation status */
  isValid: boolean;
  
  /** Detected contradictions */
  contradictions: Contradiction[];
  
  /** Non-critical warnings */
  warnings: ValidationWarning[];
  
  /** Validation timestamp */
  validatedAt: string;
}

interface ValidationWarning {
  /** Warning code */
  code: string;
  
  /** Human-readable message */
  message: string;
  
  /** Affected field/attribute */
  field?: string;
  
  /** Suggested action */
  suggestion?: string;
}
```

### ConfidenceScore

Confidence metrics breakdown.

```typescript
interface ConfidenceScore {
  /** Overall confidence (0.0 - 1.0) */
  score: number;
  
  /** Factor breakdown (when requested) */
  factors?: {
    /** Extraction confidence */
    extraction: number;
    
    /** Validation confidence */
    validation: number;
    
    /** Source reliability */
    sourceReliability: number;
    
    /** Data completeness */
    completeness: number;
    
    /** Recency factor */
    recency: number;
  };
  
  /** Calculation timestamp */
  calculatedAt: string;
}
```

---

## Input Types

### CreateEntityInput

Input for creating a new entity.

```typescript
interface CreateEntityInput {
  /** Entity classification */
  entityType: EntityType;
  
  /** Display name (2-200 characters) */
  name: string;
  
  /** Optional description */
  description?: string;
  
  /** Key-value metadata */
  metadata?: Partial<EntityMetadata>;
  
  /** Privacy level (default: Private) */
  privacyLevel?: PrivacyLevel;
  
  /** Data source identifier */
  source?: string;
}
```

### UpdateEntityInput

Input for updating an existing entity (all fields optional).

```typescript
interface UpdateEntityInput {
  /** New display name */
  name?: string;
  
  /** New description */
  description?: string;
  
  /** Metadata updates (merged with existing) */
  metadata?: Partial<EntityMetadata>;
  
  /** New privacy level */
  privacyLevel?: PrivacyLevel;
}
```

### EntityFilter

Filter criteria for entity queries.

```typescript
interface EntityFilter {
  /** Filter by entity type */
  entityType?: EntityType;
  
  /** Name contains substring (case-insensitive) */
  nameContains?: string;
  
  /** Exact name match */
  nameEquals?: string;
  
  /** Filter by privacy level */
  privacyLevel?: PrivacyLevel;
  
  /** Minimum confidence threshold */
  minConfidence?: number;
  
  /** Has specific metadata key */
  hasMetadataKey?: string;
  
  /** Metadata key-value match */
  metadataEquals?: { key: string; value: string };
  
  /** Created after date */
  createdAfter?: string;
  
  /** Created before date */
  createdBefore?: string;
  
  /** Include deleted entities */
  includeDeleted?: boolean;
}
```

### CreateRelationshipInput

Input for creating relationships.

```typescript
interface CreateRelationshipInput {
  /** Source entity ID */
  sourceId: string;
  
  /** Target entity ID */
  targetId: string;
  
  /** Relationship type */
  relationType: string;
  
  /** Confidence score (0.0 - 1.0, default: 1.0) */
  confidence?: number;
  
  /** Relationship start date */
  validFrom?: string;
  
  /** Relationship end date */
  validUntil?: string;
  
  /** Additional properties */
  metadata?: Record<string, unknown>;
}
```

---

## Graph Types

### GraphResult

Result of graph traversal query.

```typescript
interface GraphResult {
  /** Nodes in the graph */
  nodes: EntitySummary[];
  
  /** Edges connecting nodes */
  edges: GraphEdge[];
  
  /** Discovered paths */
  paths: GraphPath[];
}

interface GraphEdge {
  sourceId: string;
  targetId: string;
  relationType: string;
  confidence: number;
}

interface GraphPath {
  nodeIds: string[];
  edgeIds: string[];
  totalConfidence: number;
}
```

### PathResult

Path between two entities.

```typescript
interface PathResult {
  /** Path nodes in order */
  nodes: EntitySummary[];
  
  /** Path edges in order */
  edges: Relationship[];
  
  /** Path length (number of edges) */
  length: number;
  
  /** Aggregated confidence score */
  confidence: number;
}
```

---

## Search Types

### SearchResult

Full-text search result with relevance scoring.

```typescript
interface SearchResult {
  /** Matched entity */
  entity: EntitySummary;
  
  /** Relevance score (0.0 - 1.0) */
  score: number;
  
  /** Matching text highlights */
  highlights: string[];
  
  /** Matched fields */
  matchedFields: string[];
}
```

### DuplicateCandidate

Potential duplicate entity pair.

```typescript
interface DuplicateCandidate {
  /** First entity */
  entityA: EntitySummary;
  
  /** Second entity */
  entityB: EntitySummary;
  
  /** Similarity score (0.0 - 1.0) */
  similarity: number;
  
  /** Matching attributes */
  matchingFields: string[];
  
  /** Recommended action */
  recommendation: 'merge' | 'review' | 'ignore';
}
```

---

## Utility Types

### Pagination

Pagination metadata for list queries.

```typescript
interface PageInfo {
  /** Total number of items */
  totalCount: number;
  
  /** Has more pages */
  hasNextPage: boolean;
  
  /** Has previous pages */
  hasPreviousPage: boolean;
  
  /** Current page start cursor */
  startCursor?: string;
  
  /** Current page end cursor */
  endCursor?: string;
}

interface Connection<T> {
  nodes: T[];
  pageInfo: PageInfo;
}
```

### Sort Configuration

Sorting options for queries.

```typescript
interface EntitySort {
  /** Field to sort by */
  field: 'name' | 'createdAt' | 'updatedAt' | 'extractionConfidence';
  
  /** Sort direction */
  direction: 'asc' | 'desc';
}
```
