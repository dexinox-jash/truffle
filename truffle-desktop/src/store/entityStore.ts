import { create } from 'zustand';
import { devtools } from 'zustand/middleware';

export interface EntityFilter {
  search: string;
  types: string[];
  status: ('active' | 'archived' | 'draft')[];
  tags: string[];
  dateRange: {
    from: Date | null;
    to: Date | null;
  } | null;
}

const defaultFilter: EntityFilter = {
  search: '',
  types: [],
  status: ['active'],
  tags: [],
  dateRange: null,
};

interface EntityState {
  // Selected entity
  selectedEntityId: string | null;
  setSelectedEntityId: (id: string | null) => void;

  // View mode
  viewMode: 'list' | 'detail' | 'edit' | 'create';
  setViewMode: (mode: EntityState['viewMode']) => void;

  // Filters
  listFilter: EntityFilter;
  setListFilter: (filter: EntityFilter) => void;
  resetListFilter: () => void;

  // Graph view settings
  graphRootId: string | null;
  graphDepth: number;
  setGraphRootId: (id: string | null) => void;
  setGraphDepth: (depth: number) => void;

  // Validation view
  validationTab: 'summary' | 'contradictions' | 'duplicates';
  setValidationTab: (tab: EntityState['validationTab']) => void;
  selectedContradictionId: string | null;
  setSelectedContradictionId: (id: string | null) => void;
  selectedDuplicateIds: [string, string] | null;
  setSelectedDuplicateIds: (ids: [string, string] | null) => void;

  // Meeting view
  selectedMeetingId: string | null;
  setSelectedMeetingId: (id: string | null) => void;
  meetingSearchQuery: string;
  setMeetingSearchQuery: (query: string) => void;

  // Actions
  openEntityDetail: (id: string) => void;
  openEntityEdit: (id: string) => void;
  openEntityCreate: () => void;
  closeDetail: () => void;
}

export const useEntityStore = create<EntityState>()(
  devtools(
    (set) => ({
      // Selected entity
      selectedEntityId: null,
      setSelectedEntityId: (id) => set({ selectedEntityId: id }),

      // View mode
      viewMode: 'list',
      setViewMode: (mode) => set({ viewMode: mode }),

      // Filters
      listFilter: { ...defaultFilter },
      setListFilter: (filter) => set({ listFilter: filter }),
      resetListFilter: () => set({ listFilter: { ...defaultFilter } }),

      // Graph view settings
      graphRootId: null,
      graphDepth: 2,
      setGraphRootId: (id) => set({ graphRootId: id }),
      setGraphDepth: (depth) => set({ graphDepth: Math.max(1, Math.min(5, depth)) }),

      // Validation view
      validationTab: 'summary',
      setValidationTab: (tab) => set({ validationTab: tab }),
      selectedContradictionId: null,
      setSelectedContradictionId: (id) => set({ selectedContradictionId: id }),
      selectedDuplicateIds: null,
      setSelectedDuplicateIds: (ids) => set({ selectedDuplicateIds: ids }),

      // Meeting view
      selectedMeetingId: null,
      setSelectedMeetingId: (id) => set({ selectedMeetingId: id }),
      meetingSearchQuery: '',
      setMeetingSearchQuery: (query) => set({ meetingSearchQuery: query }),

      // Actions
      openEntityDetail: (id) =>
        set({
          selectedEntityId: id,
          viewMode: 'detail',
        }),
      openEntityEdit: (id) =>
        set({
          selectedEntityId: id,
          viewMode: 'edit',
        }),
      openEntityCreate: () =>
        set({
          selectedEntityId: null,
          viewMode: 'create',
        }),
      closeDetail: () =>
        set({
          viewMode: 'list',
          selectedEntityId: null,
        }),
    }),
    { name: 'EntityStore' }
  )
);

// Selector hooks for better performance
export const useSelectedEntityId = () =>
  useEntityStore((state) => state.selectedEntityId);

export const useViewMode = () =>
  useEntityStore((state) => state.viewMode);

export const useListFilter = () =>
  useEntityStore((state) => state.listFilter);

export const useGraphSettings = () =>
  useEntityStore((state) => ({
    rootId: state.graphRootId,
    depth: state.graphDepth,
  }));

export const useValidationState = () =>
  useEntityStore((state) => ({
    tab: state.validationTab,
    selectedContradictionId: state.selectedContradictionId,
    selectedDuplicateIds: state.selectedDuplicateIds,
  }));

export const useMeetingState = () =>
  useEntityStore((state) => ({
    selectedMeetingId: state.selectedMeetingId,
    searchQuery: state.meetingSearchQuery,
  }));
