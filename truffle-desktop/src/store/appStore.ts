// SPEC v2.0 SECTION 5.2: Application State Management
// Zustand store for Truffle Desktop

import { create } from 'zustand';
import { devtools, persist } from 'zustand/middleware';
import { immer } from 'zustand/middleware/immer';

// ==================== TYPES ====================

export type ViewMode = 'raw' | 'wiki' | 'split';
export type PanelFocus = 'left' | 'center' | 'right';

export interface RawArtifact {
  id: string;
  filename: string;
  thumbnailUrl?: string;
  capturedAt: string;
  status: 'pending' | 'compiling' | 'compiled' | 'failed';
  appContext?: {
    appName: string;
    windowTitle?: string;
  };
}

export interface WikiNode {
  id: string;
  title: string;
  content: string;
  nodeType: 'entity' | 'concept' | 'chronology' | 'index';
  updatedAt: string;
  backlinkCount: number;
  forwardLinkCount: number;
}

export interface CompilationJob {
  id: string;
  artifactId: string;
  status: 'queued' | 'processing' | 'completed' | 'failed';
  progress: number;
  startedAt?: string;
  completedAt?: string;
  error?: string;
}

export interface AppState {
  // View state
  viewMode: ViewMode;
  panelFocus: PanelFocus;
  sidebarCollapsed: boolean;
  
  // Selection state
  selectedArtifactId: string | null;
  selectedNodeId: string | null;
  selectedCompilationId: string | null;
  
  // Data
  artifacts: RawArtifact[];
  wikiNodes: WikiNode[];
  compilationJobs: CompilationJob[];
  
  // UI state
  isCommandPaletteOpen: boolean;
  isSearchOpen: boolean;
  isSettingsOpen: boolean;
  isExporting: boolean;
  
  // Compilation state
  isCompiling: boolean;
  compilationQueue: string[];
  currentCompilationJob: CompilationJob | null;
  
  // Sync state
  syncStatus: 'idle' | 'syncing' | 'error' | 'offline';
  lastSyncAt: string | null;
  pendingChanges: number;
  
  // Actions
  setViewMode: (mode: ViewMode) => void;
  setPanelFocus: (focus: PanelFocus) => void;
  toggleSidebar: () => void;
  
  selectArtifact: (id: string | null) => void;
  selectNode: (id: string | null) => void;
  selectCompilation: (id: string | null) => void;
  
  setArtifacts: (artifacts: RawArtifact[]) => void;
  addArtifact: (artifact: RawArtifact) => void;
  updateArtifact: (id: string, updates: Partial<RawArtifact>) => void;
  removeArtifact: (id: string) => void;
  
  setWikiNodes: (nodes: WikiNode[]) => void;
  addWikiNode: (node: WikiNode) => void;
  updateWikiNode: (id: string, updates: Partial<WikiNode>) => void;
  removeWikiNode: (id: string) => void;
  
  setCompilationJobs: (jobs: CompilationJob[]) => void;
  addCompilationJob: (job: CompilationJob) => void;
  updateCompilationJob: (id: string, updates: Partial<CompilationJob>) => void;
  
  openCommandPalette: () => void;
  closeCommandPalette: () => void;
  toggleCommandPalette: () => void;
  
  openSearch: () => void;
  closeSearch: () => void;
  
  setIsCompiling: (isCompiling: boolean) => void;
  setCurrentCompilationJob: (job: CompilationJob | null) => void;
  
  setSyncStatus: (status: AppState['syncStatus']) => void;
  setLastSyncAt: (date: string | null) => void;
  setPendingChanges: (count: number) => void;
}

// ==================== STORE ====================

export const useAppStore = create<AppState>()(
  devtools(
    immer(
      (set) => ({
        // Initial state
        viewMode: 'split',
        panelFocus: 'center',
        sidebarCollapsed: false,
        
        selectedArtifactId: null,
        selectedNodeId: null,
        selectedCompilationId: null,
        
        artifacts: [],
        wikiNodes: [],
        compilationJobs: [],
        
        isCommandPaletteOpen: false,
        isSearchOpen: false,
        isSettingsOpen: false,
        isExporting: false,
        
        isCompiling: false,
        compilationQueue: [],
        currentCompilationJob: null,
        
        syncStatus: 'idle',
        lastSyncAt: null,
        pendingChanges: 0,
        
        // Actions
        setViewMode: (mode) => set({ viewMode: mode }),
        setPanelFocus: (focus) => set({ panelFocus: focus }),
        toggleSidebar: () => set((state) => ({ sidebarCollapsed: !state.sidebarCollapsed })),
        
        selectArtifact: (id) => set({ selectedArtifactId: id }),
        selectNode: (id) => set({ selectedNodeId: id }),
        selectCompilation: (id) => set({ selectedCompilationId: id }),
        
        setArtifacts: (artifacts) => set({ artifacts }),
        addArtifact: (artifact) => set((state) => {
          state.artifacts.unshift(artifact);
        }),
        updateArtifact: (id, updates) => set((state) => {
          const artifact = state.artifacts.find(a => a.id === id);
          if (artifact) {
            Object.assign(artifact, updates);
          }
        }),
        removeArtifact: (id) => set((state) => {
          state.artifacts = state.artifacts.filter(a => a.id !== id);
        }),
        
        setWikiNodes: (nodes) => set({ wikiNodes: nodes }),
        addWikiNode: (node) => set((state) => {
          state.wikiNodes.unshift(node);
        }),
        updateWikiNode: (id, updates) => set((state) => {
          const node = state.wikiNodes.find(n => n.id === id);
          if (node) {
            Object.assign(node, updates);
          }
        }),
        removeWikiNode: (id) => set((state) => {
          state.wikiNodes = state.wikiNodes.filter(n => n.id !== id);
        }),
        
        setCompilationJobs: (jobs) => set({ compilationJobs: jobs }),
        addCompilationJob: (job) => set((state) => {
          state.compilationJobs.unshift(job);
        }),
        updateCompilationJob: (id, updates) => set((state) => {
          const job = state.compilationJobs.find(j => j.id === id);
          if (job) {
            Object.assign(job, updates);
          }
        }),
        
        openCommandPalette: () => set({ isCommandPaletteOpen: true }),
        closeCommandPalette: () => set({ isCommandPaletteOpen: false }),
        toggleCommandPalette: () => set((state) => ({ 
          isCommandPaletteOpen: !state.isCommandPaletteOpen 
        })),
        
        openSearch: () => set({ isSearchOpen: true }),
        closeSearch: () => set({ isSearchOpen: false }),
        
        setIsCompiling: (isCompiling) => set({ isCompiling }),
        setCurrentCompilationJob: (job) => set({ currentCompilationJob: job }),
        
        setSyncStatus: (status) => set({ syncStatus: status }),
        setLastSyncAt: (date) => set({ lastSyncAt: date }),
        setPendingChanges: (count) => set({ pendingChanges: count }),
      })
    ),
    { name: 'truffle-app-store' }
  )
);

// ==================== SELECTORS ====================

export const selectSelectedArtifact = (state: AppState) => 
  state.artifacts.find(a => a.id === state.selectedArtifactId);

export const selectSelectedNode = (state: AppState) =>
  state.wikiNodes.find(n => n.id === state.selectedNodeId);

export const selectPendingArtifacts = (state: AppState) =>
  state.artifacts.filter(a => a.status === 'pending');

export const selectCompiledArtifacts = (state: AppState) =>
  state.artifacts.filter(a => a.status === 'compiled');

export const selectRecentNodes = (state: AppState) =>
  [...state.wikiNodes]
    .sort((a, b) => new Date(b.updatedAt).getTime() - new Date(a.updatedAt).getTime())
    .slice(0, 10);

export const selectNodeByTitle = (state: AppState, title: string) =>
  state.wikiNodes.find(n => n.title.toLowerCase() === title.toLowerCase());
