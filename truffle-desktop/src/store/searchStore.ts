// SPEC v2.0 SECTION 5.2: Search State Management
// FlexSearch integration for Truffle Desktop

import { create } from 'zustand';
import { devtools } from 'zustand/middleware';
import { immer } from 'zustand/middleware/immer';
import FlexSearch from 'flexsearch';

// ==================== TYPES ====================

export interface SearchResult {
  id: string;
  type: 'artifact' | 'wiki-node' | 'compilation';
  title: string;
  excerpt: string;
  score: number;
  metadata?: Record<string, unknown>;
}

export interface SearchFilters {
  types: ('artifact' | 'wiki-node' | 'compilation')[];
  dateRange?: { start: Date; end: Date };
  nodeTypes?: string[];
  status?: string[];
}

export interface SearchState {
  // Search state
  query: string;
  results: SearchResult[];
  isSearching: boolean;
  searchTime: number;
  
  // Filters
  filters: SearchFilters;
  showFilters: boolean;
  
  // History
  recentSearches: string[];
  
  // Indexes (FlexSearch instances)
  artifactIndex: FlexSearch.Index | null;
  wikiIndex: FlexSearch.Document<any, any> | null;
  
  // Actions
  setQuery: (query: string) => void;
  setResults: (results: SearchResult[]) => void;
  setIsSearching: (isSearching: boolean) => void;
  setSearchTime: (time: number) => void;
  
  setFilters: (filters: Partial<SearchFilters>) => void;
  toggleFilter: (type: SearchFilters['types'][number]) => void;
  toggleShowFilters: () => void;
  resetFilters: () => void;
  
  addRecentSearch: (query: string) => void;
  clearRecentSearches: () => void;
  
  initializeIndexes: () => void;
  indexArtifact: (artifact: { id: string; filename: string; ocrText?: string }) => void;
  indexWikiNode: (node: { id: string; title: string; content: string }) => void;
  removeFromIndex: (id: string, type: 'artifact' | 'wiki-node') => void;
  
  performSearch: (query: string) => Promise<SearchResult[]>;
}

// ==================== STORE ====================

export const useSearchStore = create<SearchState>()(
  devtools(
    immer((set, get) => ({
      // Initial state
      query: '',
      results: [],
      isSearching: false,
      searchTime: 0,
      
      filters: {
        types: ['artifact', 'wiki-node', 'compilation'],
      },
      showFilters: false,
      
      recentSearches: [],
      
      artifactIndex: null,
      wikiIndex: null,
      
      // Actions
      setQuery: (query) => set({ query }),
      setResults: (results) => set({ results }),
      setIsSearching: (isSearching) => set({ isSearching }),
      setSearchTime: (searchTime) => set({ searchTime }),
      
      setFilters: (filters) => set((state) => {
        Object.assign(state.filters, filters);
      }),
      
      toggleFilter: (type) => set((state) => {
        const index = state.filters.types.indexOf(type);
        if (index > -1) {
          state.filters.types.splice(index, 1);
        } else {
          state.filters.types.push(type);
        }
      }),
      
      toggleShowFilters: () => set((state) => ({ 
        showFilters: !state.showFilters 
      })),
      
      resetFilters: () => set({
        filters: {
          types: ['artifact', 'wiki-node', 'compilation'],
        },
      }),
      
      addRecentSearch: (query) => set((state) => {
        if (!query.trim()) return;
        
        // Remove if exists
        const index = state.recentSearches.indexOf(query);
        if (index > -1) {
          state.recentSearches.splice(index, 1);
        }
        
        // Add to front
        state.recentSearches.unshift(query);
        
        // Keep only last 10
        if (state.recentSearches.length > 10) {
          state.recentSearches.pop();
        }
      }),
      
      clearRecentSearches: () => set({ recentSearches: [] }),
      
      initializeIndexes: () => set((state) => {
        // Initialize FlexSearch indexes
        state.artifactIndex = new FlexSearch.Index({
          preset: 'match',
          tokenize: 'forward',
          cache: true,
        });
        
        state.wikiIndex = new FlexSearch.Document({
          document: {
            id: 'id',
            index: ['title', 'content'],
          },
          tokenize: 'forward',
          cache: true,
        });
      }),
      
      indexArtifact: (artifact) => set((state) => {
        if (!state.artifactIndex) return;
        
        const text = `${artifact.filename} ${artifact.ocrText || ''}`;
        state.artifactIndex.add(artifact.id, text);
      }),
      
      indexWikiNode: (node) => set((state) => {
        if (!state.wikiIndex) return;
        
        state.wikiIndex.add({
          id: node.id,
          title: node.title,
          content: node.content,
        });
      }),
      
      removeFromIndex: (id, type) => set((state) => {
        if (type === 'artifact' && state.artifactIndex) {
          state.artifactIndex.remove(id);
        } else if (type === 'wiki-node' && state.wikiIndex) {
          state.wikiIndex.remove(id);
        }
      }),
      
      performSearch: async (query) => {
        const state = get();
        const startTime = performance.now();
        
        if (!query.trim()) {
          set({ results: [], searchTime: 0 });
          return [];
        }
        
        set({ isSearching: true });
        
        const results: SearchResult[] = [];
        
        // Search artifacts
        if (state.filters.types.includes('artifact') && state.artifactIndex) {
          const artifactIds = await state.artifactIndex.searchAsync(query, { limit: 20 });
          // Convert to SearchResult format
          artifactIds.forEach((id: string | number) => {
            results.push({
              id: String(id),
              type: 'artifact',
              title: `Artifact ${id}`,
              excerpt: '',
              score: 1,
            });
          });
        }
        
        // Search wiki nodes
        if (state.filters.types.includes('wiki-node') && state.wikiIndex) {
          const wikiResults = await state.wikiIndex.searchAsync(query, { limit: 20 });
          // Process wiki results
          wikiResults.forEach((result: any) => {
            if (result.result) {
              result.result.forEach((id: string | number) => {
                results.push({
                  id: String(id),
                  type: 'wiki-node',
                  title: `Wiki Node ${id}`,
                  excerpt: '',
                  score: 1,
                });
              });
            }
          });
        }
        
        const searchTime = performance.now() - startTime;
        
        set({ 
          results, 
          isSearching: false, 
          searchTime 
        });
        
        return results;
      },
    })),
    { name: 'truffle-search-store' }
  )
);

// ==================== HOOKS ====================

export function useSearch() {
  const store = useSearchStore();
  
  return {
    query: store.query,
    results: store.results,
    isSearching: store.isSearching,
    searchTime: store.searchTime,
    filters: store.filters,
    showFilters: store.showFilters,
    recentSearches: store.recentSearches,
    
    setQuery: store.setQuery,
    performSearch: store.performSearch,
    toggleFilter: store.toggleFilter,
    toggleShowFilters: store.toggleShowFilters,
    resetFilters: store.resetFilters,
    addRecentSearch: store.addRecentSearch,
    clearRecentSearches: store.clearRecentSearches,
  };
}
