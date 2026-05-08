import { useState, useMemo, useCallback } from 'react';
import { 
  Search, 
  Filter, 
  ChevronLeft, 
  ChevronRight, 
  Plus,
  Loader2,
  AlertCircle,
  ArrowUpDown,
  ArrowUp,
  ArrowDown
} from 'lucide-react';
import { FixedSizeList as List } from 'react-window';
import AutoSizer from 'react-virtualized-auto-sizer';
import { useDebounce } from '../../hooks/useDebounce';
import { useEntities } from '../../hooks/useEntities';
import { EntityCard } from './EntityCard';
import { EntityTypeBadge } from './EntityTypeBadge';
import type { Entity, EntityType, EntityFilter, EntitySummary } from '../../types/knowledge-graph';

export interface EntityListProps {
  filter?: EntityFilter;
  onSelect?: (entity: EntitySummary) => void;
  selectable?: boolean;
  showActions?: boolean;
  className?: string;
}

type SortField = 'name' | 'createdAt' | 'extractionConfidence';
type SortDirection = 'asc' | 'desc';

const PAGE_SIZE = 20;

export function EntityList({ 
  filter: initialFilter, 
  onSelect, 
  selectable = false,
  showActions = false,
  className = '' 
}: EntityListProps) {
  // State
  const [searchQuery, setSearchQuery] = useState('');
  const [selectedType, setSelectedType] = useState<EntityType | undefined>(initialFilter?.entityType);
  const [sortField, setSortField] = useState<SortField>('createdAt');
  const [sortDirection, setSortDirection] = useState<SortDirection>('desc');
  const [currentPage, setCurrentPage] = useState(0);
  const [selectedEntityId, setSelectedEntityId] = useState<string | null>(null);

  // Debounce search query
  const debouncedSearch = useDebounce(searchQuery, 300);

  // Build filter
  const filter: EntityFilter = useMemo(() => ({
    ...initialFilter,
    entityType: selectedType,
    nameContains: debouncedSearch || undefined,
  }), [initialFilter, selectedType, debouncedSearch]);

  // Fetch data
  const { 
    data, 
    isLoading, 
    error,
    isFetching 
  } = useEntities(filter, { 
    limit: PAGE_SIZE, 
    offset: currentPage * PAGE_SIZE 
  });

  const entities = data?.items || [];
  const totalCount = data?.total || 0;
  const hasMore = data?.hasMore || false;
  const totalPages = Math.ceil(totalCount / PAGE_SIZE);

  // Handlers
  const handleSort = (field: SortField) => {
    if (sortField === field) {
      setSortDirection(prev => prev === 'asc' ? 'desc' : 'asc');
    } else {
      setSortField(field);
      setSortDirection('desc');
    }
  };

  const handleSelect = useCallback((entity: EntitySummary) => {
    setSelectedEntityId(entity.id);
    onSelect?.(entity);
  }, [onSelect]);

  const handlePageChange = (newPage: number) => {
    if (newPage >= 0 && newPage < totalPages) {
      setCurrentPage(newPage);
    }
  };

  // Sort entities client-side
  const sortedEntities = useMemo(() => {
    return [...entities].sort((a, b) => {
      let comparison = 0;
      switch (sortField) {
        case 'name':
          comparison = a.name.localeCompare(b.name);
          break;
        case 'createdAt':
          comparison = new Date(a.id).getTime() - new Date(b.id).getTime();
          break;
        case 'extractionConfidence':
          comparison = a.extractionConfidence - b.extractionConfidence;
          break;
      }
      return sortDirection === 'asc' ? comparison : -comparison;
    });
  }, [entities, sortField, sortDirection]);

  // Virtual list row renderer
  const Row = useCallback(({ index, style }: { index: number; style: React.CSSProperties }) => {
    const entity = sortedEntities[index];
    if (!entity) return null;

    return (
      <div style={style} className="px-4 py-2">
        <EntityCard
          entity={entity}
          onClick={() => handleSelect(entity)}
          selected={selectable && selectedEntityId === entity.id}
          compact
        />
      </div>
    );
  }, [sortedEntities, handleSelect, selectable, selectedEntityId]);

  // Sort icon helper
  const SortIcon = ({ field }: { field: SortField }) => {
    if (sortField !== field) {
      return <ArrowUpDown size={14} className="text-gray-600" />;
    }
    return sortDirection === 'asc' 
      ? <ArrowUp size={14} className="text-blue-400" />
      : <ArrowDown size={14} className="text-blue-400" />;
  };

  // Empty state
  if (!isLoading && !error && entities.length === 0 && !isFetching) {
    return (
      <div className={`flex flex-col items-center justify-center py-16 px-4 ${className}`}>
        <div className="w-16 h-16 rounded-full bg-gray-800 flex items-center justify-center mb-4">
          <Search size={24} className="text-gray-500" />
        </div>
        <h3 className="text-lg font-medium text-gray-300 mb-2">
          {debouncedSearch ? 'No entities found' : 'No entities yet'}
        </h3>
        <p className="text-sm text-gray-500 text-center max-w-sm mb-6">
          {debouncedSearch 
            ? `No entities match "${debouncedSearch}". Try a different search term.`
            : 'Get started by creating your first entity to track people, organizations, and more.'
          }
        </p>
        {showActions && (
          <button className="flex items-center gap-2 px-4 py-2 bg-blue-600 hover:bg-blue-500 text-white rounded-lg transition-colors">
            <Plus size={18} />
            Create Entity
          </button>
        )}
      </div>
    );
  }

  return (
    <div className={`flex flex-col h-full ${className}`}>
      {/* Header with Search and Filters */}
      <div className="p-4 border-b border-gray-700 space-y-4">
        {/* Search */}
        <div className="relative">
          <Search size={18} className="absolute left-3 top-1/2 -translate-y-1/2 text-gray-500" />
          <input
            type="text"
            value={searchQuery}
            onChange={(e) => setSearchQuery(e.target.value)}
            placeholder="Search entities..."
            className="w-full pl-10 pr-4 py-2 bg-gray-800 border border-gray-700 rounded-lg text-gray-200 placeholder-gray-500 focus:border-blue-500 focus:ring-1 focus:ring-blue-500 outline-none transition-colors"
          />
          {isFetching && (
            <Loader2 size={16} className="absolute right-3 top-1/2 -translate-y-1/2 text-blue-400 animate-spin" />
          )}
        </div>

        {/* Filters Row */}
        <div className="flex items-center gap-3 flex-wrap">
          {/* Type Filter */}
          <div className="relative">
            <Filter size={14} className="absolute left-2.5 top-1/2 -translate-y-1/2 text-gray-500" />
            <select
              value={selectedType || ''}
              onChange={(e) => {
                setSelectedType(e.target.value as EntityType || undefined);
                setCurrentPage(0);
              }}
              className="pl-9 pr-8 py-1.5 bg-gray-800 border border-gray-700 rounded-lg text-sm text-gray-300 focus:border-blue-500 outline-none appearance-none cursor-pointer hover:bg-gray-750 transition-colors"
            >
              <option value="">All Types</option>
              {Object.values(EntityType).map((type) => (
                <option key={type} value={type}>
                  {type.charAt(0).toUpperCase() + type.slice(1).replace('_', ' ')}
                </option>
              ))}
            </select>
            <ChevronDown size={14} className="absolute right-2.5 top-1/2 -translate-y-1/2 text-gray-500 pointer-events-none" />
          </div>

          {/* Sort Controls */}
          <div className="flex items-center gap-1 border-l border-gray-700 pl-3">
            <button
              onClick={() => handleSort('name')}
              className={`flex items-center gap-1 px-2 py-1.5 rounded text-sm transition-colors ${
                sortField === 'name' ? 'text-blue-400 bg-blue-500/10' : 'text-gray-400 hover:text-gray-300 hover:bg-gray-800'
              }`}
            >
              Name
              <SortIcon field="name" />
            </button>
            <button
              onClick={() => handleSort('createdAt')}
              className={`flex items-center gap-1 px-2 py-1.5 rounded text-sm transition-colors ${
                sortField === 'createdAt' ? 'text-blue-400 bg-blue-500/10' : 'text-gray-400 hover:text-gray-300 hover:bg-gray-800'
              }`}
            >
              Created
              <SortIcon field="createdAt" />
            </button>
            <button
              onClick={() => handleSort('extractionConfidence')}
              className={`flex items-center gap-1 px-2 py-1.5 rounded text-sm transition-colors ${
                sortField === 'extractionConfidence' ? 'text-blue-400 bg-blue-500/10' : 'text-gray-400 hover:text-gray-300 hover:bg-gray-800'
              }`}
            >
              Confidence
              <SortIcon field="extractionConfidence" />
            </button>
          </div>

          {/* Results count */}
          <div className="ml-auto text-sm text-gray-500">
            {totalCount.toLocaleString()} result{totalCount !== 1 ? 's' : ''}
          </div>
        </div>
      </div>

      {/* Error State */}
      {error && (
        <div className="p-4 m-4 rounded-lg bg-red-500/10 border border-red-500/30">
          <div className="flex items-start gap-3">
            <AlertCircle size={20} className="text-red-400 flex-shrink-0 mt-0.5" />
            <div>
              <h4 className="font-medium text-red-400">Failed to load entities</h4>
              <p className="text-sm text-red-300/80 mt-1">
                {error instanceof Error ? error.message : 'Unknown error occurred'}
              </p>
            </div>
          </div>
        </div>
      )}

      {/* Entity List - Virtualized */}
      <div className="flex-1 min-h-0">
        {isLoading ? (
          <div className="flex items-center justify-center h-full">
            <Loader2 size={32} className="text-blue-400 animate-spin" />
          </div>
        ) : (
          <AutoSizer>
            {({ height, width }) => (
              <List
                height={height}
                itemCount={sortedEntities.length}
                itemSize={80}
                width={width}
                overscanCount={5}
              >
                {Row}
              </List>
            )}
          </AutoSizer>
        )}
      </div>

      {/* Pagination */}
      {totalPages > 1 && (
        <div className="p-4 border-t border-gray-700 flex items-center justify-between">
          <button
            onClick={() => handlePageChange(currentPage - 1)}
            disabled={currentPage === 0}
            className="flex items-center gap-1 px-3 py-1.5 text-sm text-gray-400 hover:text-gray-200 hover:bg-gray-800 rounded-lg transition-colors disabled:opacity-50 disabled:cursor-not-allowed"
          >
            <ChevronLeft size={16} />
            Previous
          </button>
          
          <div className="flex items-center gap-1">
            {Array.from({ length: Math.min(5, totalPages) }, (_, i) => {
              let pageNum: number;
              if (totalPages <= 5) {
                pageNum = i;
              } else if (currentPage < 2) {
                pageNum = i;
              } else if (currentPage > totalPages - 3) {
                pageNum = totalPages - 5 + i;
              } else {
                pageNum = currentPage - 2 + i;
              }

              return (
                <button
                  key={pageNum}
                  onClick={() => handlePageChange(pageNum)}
                  className={`w-8 h-8 rounded-lg text-sm font-medium transition-colors ${
                    currentPage === pageNum
                      ? 'bg-blue-600 text-white'
                      : 'text-gray-400 hover:text-gray-200 hover:bg-gray-800'
                  }`}
                >
                  {pageNum + 1}
                </button>
              );
            })}
          </div>

          <button
            onClick={() => handlePageChange(currentPage + 1)}
            disabled={!hasMore}
            className="flex items-center gap-1 px-3 py-1.5 text-sm text-gray-400 hover:text-gray-200 hover:bg-gray-800 rounded-lg transition-colors disabled:opacity-50 disabled:cursor-not-allowed"
          >
            Next
            <ChevronRight size={16} />
          </button>
        </div>
      )}
    </div>
  );
}

export default EntityList;
