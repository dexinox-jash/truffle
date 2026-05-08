// Export all stores

export { 
  useAppStore, 
  selectSelectedArtifact,
  selectSelectedNode,
  selectPendingArtifacts,
  selectCompiledArtifacts,
  selectRecentNodes,
  selectNodeByTitle,
  type ViewMode,
  type PanelFocus,
  type RawArtifact,
  type WikiNode,
  type CompilationJob,
} from './appStore';

export { 
  useSearchStore, 
  useSearch,
  type SearchResult,
  type SearchFilters,
} from './searchStore';

export { 
  useSyncStore, 
  useSync,
  createYDoc,
  encodeYDoc,
  decodeYDoc,
  getYDocStateVector,
  getYDocDiff,
  type SyncDevice,
  type SyncChange,
  type ConflictResolution,
} from './syncStore';
