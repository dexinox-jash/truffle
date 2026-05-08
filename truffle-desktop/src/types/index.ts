// TypeScript type definitions for Truffle Desktop

// ==================== RAW ARTIFACT TYPES ====================

export interface RawArtifact {
  id: string;
  filename: string;
  path?: string;
  thumbnailUrl?: string;
  capturedAt: string;
  deviceId?: string;
  status: 'pending' | 'compiling' | 'compiled' | 'failed' | 'quarantined';
  appContext?: {
    bundleId?: string;
    appName: string;
    windowTitle?: string;
  };
  geohash?: string;
  metadata?: {
    sizeBytes: number;
    width: number;
    height: number;
    ocrText?: string;
    embeddingVector?: number[];
  };
}

// ==================== WIKI NODE TYPES ====================

export type NodeType = 'entity' | 'concept' | 'chronology' | 'index';
export type PrivacyClassification = 'public' | 'personal' | 'sensitive' | 'financial';
export type EncryptionStatus = 'plaintext' | 'aes256-gcm';

export interface WikiNode {
  id: string;
  title: string;
  content: string;
  nodeType: NodeType;
  backlinks: WikiLink[];
  forwardLinks: WikiLink[];
  provenance?: {
    sourceArtifacts: string[];
    compiledAt: string;
    modelVersion: string;
    confidenceScore: number;
  };
  temporalVectors?: {
    mentionedDates: string[];
    fiscalQuarter?: string;
  };
  privacyClassification: PrivacyClassification;
  encryptionStatus: EncryptionStatus;
  version: number;
  createdAt: string;
  updatedAt: string;
  excerpt?: string;
  backlinkCount: number;
  forwardLinkCount: number;
}

export interface WikiLink {
  sourceId: string;
  targetId: string;
  targetTitle: string;
  context: string;
}

// ==================== COMPILATION TYPES ====================

export interface CompilationJob {
  id: string;
  artifactId: string;
  status: 'queued' | 'processing' | 'completed' | 'failed' | 'cancelled';
  progress: number;
  startedAt?: string;
  completedAt?: string;
  error?: string;
  logs?: CompilationLog[];
}

export interface CompilationLog {
  id: string;
  jobId: string;
  timestamp: string;
  level: 'debug' | 'info' | 'warning' | 'error';
  message: string;
  metadata?: Record<string, unknown>;
}

export interface CompilationResult {
  artifactId: string;
  success: boolean;
  nodesCreated: string[];
  nodesUpdated: string[];
  error?: string;
  durationMs: number;
}

// ==================== SEARCH TYPES ====================

export interface SearchResult {
  id: string;
  type: 'artifact' | 'wiki-node' | 'compilation';
  title: string;
  excerpt: string;
  score: number;
  metadata?: Record<string, unknown>;
}

export interface SemanticSearchResult {
  node: WikiNode;
  similarityScore: number;
}

// ==================== SYNC TYPES ====================

export interface SyncDevice {
  id: string;
  name: string;
  type: 'desktop' | 'mobile' | 'web';
  lastSeen: string;
  isOnline: boolean;
  isPrimary: boolean;
}

export interface SyncChange {
  id: string;
  type: 'create' | 'update' | 'delete';
  entityType: 'artifact' | 'wiki-node' | 'schema';
  entityId: string;
  timestamp: string;
  deviceId: string;
  isLocal: boolean;
}

export interface SyncStatus {
  isSyncing: boolean;
  lastSyncAt: string | null;
  pendingChanges: number;
  connectedDevices: SyncDevice[];
  syncHealth: 'healthy' | 'degraded' | 'offline' | 'error';
}

// ==================== SCHEMA TYPES ====================

export interface SchemaDefinition {
  version: string;
  content: string;
  updatedAt: string;
}

export interface SchemaRule {
  trigger: string;
  action: string;
  extract: string[];
  linkTo: string[];
  privacy: PrivacyClassification;
}

// ==================== SYSTEM TYPES ====================

export interface StorageInfo {
  rawStorageBytes: number;
  wikiStorageBytes: number;
  thumbnailCacheBytes: number;
  totalBytes: number;
  artifactCount: number;
  nodeCount: number;
}

export interface AppInfo {
  version: string;
  name: string;
  buildDate: string;
}

// ==================== UI TYPES ====================

export type ViewMode = 'raw' | 'wiki' | 'split';
export type PanelFocus = 'left' | 'center' | 'right';

export interface Toast {
  id: string;
  type: 'success' | 'error' | 'warning' | 'info';
  title: string;
  message?: string;
  duration?: number;
}

// ==================== KNOWLEDGE GRAPH TYPES ====================

export * from './knowledge-graph';

// ==================== MEETING TYPES ====================

export * from './meeting';
