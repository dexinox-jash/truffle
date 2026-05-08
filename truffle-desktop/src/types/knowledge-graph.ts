// Enums
export enum EntityType {
  Person = 'person',
  Organization = 'organization',
  Location = 'location',
  Event = 'event',
  Product = 'product',
  Concept = 'concept',
  Decision = 'decision',
  ActionItem = 'action_item',
}

export enum DecisionStatus {
  Proposed = 'proposed',
  Approved = 'approved',
  Rejected = 'rejected',
  Superseded = 'superseded',
  Implemented = 'implemented',
  Abandoned = 'abandoned',
}

export enum ActionItemStatus {
  Open = 'open',
  InProgress = 'in_progress',
  Blocked = 'blocked',
  Completed = 'completed',
  Cancelled = 'cancelled',
  Overdue = 'overdue',
}

export enum Priority {
  Low = 'low',
  Medium = 'medium',
  High = 'high',
  Urgent = 'urgent',
}

export enum PrivacyLevel {
  Private = 'private',
  Shared = 'shared',
  Public = 'public',
}

// Entity types
export interface Entity {
  id: string;
  entityType: EntityType;
  name: string;
  slug: string;
  description?: string;
  metadata: EntityMetadata;
  extractionConfidence: number;
  privacyLevel: PrivacyLevel;
  createdAt: string;
  updatedAt: string;
}

export interface EntitySummary {
  id: string;
  entityType: EntityType;
  name: string;
  description?: string;
  extractionConfidence: number;
}

export interface EntityMetadata {
  // Person
  role?: string;
  email?: string;
  // Organization
  industry?: string;
  foundedYear?: number;
  // Location
  coordinates?: [number, number];
  // All types
  aliases?: string[];
  tags?: string[];
}

// Relationship types
export interface Relationship {
  id: string;
  sourceId: string;
  targetId: string;
  relationType: string;
  confidence: number;
  validFrom?: string;
  validUntil?: string;
  metadata?: Record<string, unknown>;
}

// Validation types
export interface Contradiction {
  id: string;
  entityId: string;
  factAId: string;
  factBId: string;
  contradictionType: 'temporal' | 'factual' | 'source_conflict';
  severity: 'low' | 'medium' | 'high' | 'critical';
  description: string;
  detectedAt: string;
  isResolved: boolean;
}

export interface ConfidenceScore {
  overall: number;
  byAttribute: Record<string, number>;
  uncertainty: number;
  factors: ConfidenceFactor[];
}

export interface ConfidenceFactor {
  name: string;
  category: string;
  value: number;
  weight: number;
}

export interface DuplicateCandidate {
  entityAId: string;
  entityBId: string;
  similarityScore: number;
  nameSimilarity: number;
  embeddingSimilarity: number;
  relationshipOverlap: number;
  detectedAt: string;
}

// Meeting types
export interface Meeting {
  id: string;
  title: string;
  meetingType: string;
  startedAt: string;
  endedAt?: string;
  participants: string[];
  transcript?: string;
  summary?: string;
}

export interface Decision {
  id: string;
  decisionText: string;
  status: DecisionStatus;
  proposedById?: string;
  decidedByIds: string[];
  impactScore?: number;
}

export interface ActionItem {
  id: string;
  description: string;
  assigneeId?: string;
  status: ActionItemStatus;
  deadline?: string;
  priority: Priority;
}

// Input types
export interface CreateEntityInput {
  entityType: EntityType;
  name: string;
  description?: string;
  metadata?: Partial<EntityMetadata>;
}

export interface UpdateEntityInput {
  name?: string;
  description?: string;
  metadata?: Partial<EntityMetadata>;
}

export interface CreateRelationshipInput {
  sourceId: string;
  targetId: string;
  relationType: string;
  confidence?: number;
}

export interface UpdateRelationshipInput {
  relationType?: string;
  confidence?: number;
  validFrom?: string;
  validUntil?: string;
  metadata?: Record<string, unknown>;
}

export interface EntityFilter {
  entityType?: EntityType;
  nameContains?: string;
  createdAfter?: string;
  createdBefore?: string;
  minConfidence?: number;
}

export interface RelationshipFilter {
  sourceId?: string;
  targetId?: string;
  relationType?: string;
  minConfidence?: number;
}

// Graph types
export interface GraphNode {
  id: string;
  entityType: EntityType;
  name: string;
  x?: number;
  y?: number;
}

export interface GraphEdge {
  id: string;
  source: string;
  target: string;
  relationType: string;
  confidence: number;
}

export interface GraphData {
  nodes: GraphNode[];
  edges: GraphEdge[];
}

export interface EntityWithRelationships extends Entity {
  relationships: Relationship[];
  relatedEntities: EntitySummary[];
}
