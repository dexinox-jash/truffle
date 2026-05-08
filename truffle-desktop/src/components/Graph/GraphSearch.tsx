import React, { useState, useMemo, useRef, useEffect } from 'react';
import { Entity } from '../../types/entity';

export interface GraphSearchProps {
  entities: Entity[];
  onSelect: (entityId: string) => void;
  highlightedIds: string[];
  onHighlightChange: (ids: string[]) => void;
}

export const GraphSearch: React.FC<GraphSearchProps> = ({
  entities,
  onSelect,
  highlightedIds,
  onHighlightChange,
}) => {
  const [query, setQuery] = useState('');
  const [isOpen, setIsOpen] = useState(false);
  const [selectedIndex, setSelectedIndex] = useState(0);
  const inputRef = useRef<HTMLInputElement>(null);
  const containerRef = useRef<HTMLDivElement>(null);

  // Filter entities based on query
  const filteredEntities = useMemo(() => {
    if (!query.trim()) return [];
    
    const lowerQuery = query.toLowerCase();
    return entities
      .filter(entity => 
        entity.name?.toLowerCase().includes(lowerQuery) ||
        entity.id.toLowerCase().includes(lowerQuery) ||
        entity.type.toLowerCase().includes(lowerQuery)
      )
      .slice(0, 10);
  }, [entities, query]);

  // Update highlighted nodes as user types
  useEffect(() => {
    if (query.trim()) {
      const matchingIds = entities
        .filter(e => 
          e.name?.toLowerCase().includes(query.toLowerCase()) ||
          e.id.toLowerCase().includes(query.toLowerCase())
        )
        .map(e => e.id);
      onHighlightChange(matchingIds);
    } else {
      onHighlightChange([]);
    }
  }, [query, entities, onHighlightChange]);

  // Handle click outside to close dropdown
  useEffect(() => {
    const handleClickOutside = (e: MouseEvent) => {
      if (containerRef.current && !containerRef.current.contains(e.target as Node)) {
        setIsOpen(false);
      }
    };

    document.addEventListener('mousedown', handleClickOutside);
    return () => document.removeEventListener('mousedown', handleClickOutside);
  }, []);

  const handleSelect = (entity: Entity) => {
    setQuery(entity.name || entity.id);
    setIsOpen(false);
    onSelect(entity.id);
    onHighlightChange([entity.id]);
  };

  const handleKeyDown = (e: React.KeyboardEvent) => {
    if (!isOpen || filteredEntities.length === 0) return;

    switch (e.key) {
      case 'ArrowDown':
        e.preventDefault();
        setSelectedIndex(prev => (prev + 1) % filteredEntities.length);
        break;
      case 'ArrowUp':
        e.preventDefault();
        setSelectedIndex(prev => (prev - 1 + filteredEntities.length) % filteredEntities.length);
        break;
      case 'Enter':
        e.preventDefault();
        handleSelect(filteredEntities[selectedIndex]);
        break;
      case 'Escape':
        setIsOpen(false);
        inputRef.current?.blur();
        break;
    }
  };

  const clearSearch = () => {
    setQuery('');
    onHighlightChange([]);
    inputRef.current?.focus();
  };

  const ENTITY_ICONS: Record<string, string> = {
    person: '👤',
    organization: '🏢',
    location: '📍',
    event: '📅',
    document: '📄',
    topic: '🏷️',
    unknown: '❓',
  };

  return (
    <div ref={containerRef} className="relative">
      <div className="flex items-center gap-2">
        <div className="relative">
          <input
            ref={inputRef}
            type="text"
            value={query}
            onChange={(e) => {
              setQuery(e.target.value);
              setIsOpen(true);
              setSelectedIndex(0);
            }}
            onFocus={() => setIsOpen(true)}
            onKeyDown={handleKeyDown}
            placeholder="Search entities..."
            className="w-64 px-4 py-2 pl-10 bg-gray-800/90 backdrop-blur-sm border border-gray-700 rounded-lg text-sm text-gray-200 placeholder-gray-500 focus:outline-none focus:border-blue-500 focus:ring-1 focus:ring-blue-500"
          />
          <svg
            className="absolute left-3 top-1/2 -translate-y-1/2 w-4 h-4 text-gray-500"
            fill="none"
            stroke="currentColor"
            viewBox="0 0 24 24"
          >
            <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M21 21l-6-6m2-5a7 7 0 11-14 0 7 7 0 0114 0z" />
          </svg>
          {query && (
            <button
              onClick={clearSearch}
              className="absolute right-3 top-1/2 -translate-y-1/2 text-gray-500 hover:text-gray-300"
            >
              <svg className="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M6 18L18 6M6 6l12 12" />
              </svg>
            </button>
          )}
        </div>

        {/* Highlight indicator */}
        {highlightedIds.length > 0 && (
          <div className="flex items-center gap-2 px-3 py-1.5 bg-blue-900/50 border border-blue-700/50 rounded-lg">
            <span className="text-xs text-blue-300">
              {highlightedIds.length} match{highlightedIds.length !== 1 ? 'es' : ''}
            </span>
            <button
              onClick={() => onHighlightChange([])}
              className="text-blue-400 hover:text-blue-200"
              title="Clear highlights"
            >
              <svg className="w-3 h-3" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M6 18L18 6M6 6l12 12" />
              </svg>
            </button>
          </div>
        )}
      </div>

      {/* Autocomplete dropdown */}
      {isOpen && filteredEntities.length > 0 && (
        <div className="absolute top-full left-0 mt-1 w-64 bg-gray-800 border border-gray-700 rounded-lg shadow-lg overflow-hidden z-20">
          <ul className="max-h-60 overflow-y-auto py-1">
            {filteredEntities.map((entity, index) => (
              <li
                key={entity.id}
                onClick={() => handleSelect(entity)}
                className={`flex items-center gap-3 px-3 py-2 cursor-pointer transition-colors ${
                  index === selectedIndex ? 'bg-gray-700' : 'hover:bg-gray-700/50'
                }`}
              >
                <span className="text-lg">{ENTITY_ICONS[entity.entityType] || '❓'}</span>
                <div className="flex-1 min-w-0">
                  <p className="text-sm text-gray-200 truncate">
                    {entity.name || entity.id}
                  </p>
                  <p className="text-xs text-gray-500">{entity.entityType}</p>
                </div>
              </li>
            ))}
          </ul>
          {filteredEntities.length === 10 && (
            <div className="px-3 py-1.5 bg-gray-900/50 text-xs text-gray-500 border-t border-gray-700">
              Showing top 10 results
            </div>
          )}
        </div>
      )}

      {/* No results */}
      {isOpen && query && filteredEntities.length === 0 && (
        <div className="absolute top-full left-0 mt-1 w-64 bg-gray-800 border border-gray-700 rounded-lg shadow-lg z-20">
          <div className="px-3 py-4 text-center">
            <svg className="w-8 h-8 mx-auto text-gray-600 mb-2" fill="none" stroke="currentColor" viewBox="0 0 24 24">
              <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M9.172 16.172a4 4 0 015.656 0M9 10h.01M15 10h.01M21 12a9 9 0 11-18 0 9 9 0 0118 0z" />
            </svg>
            <p className="text-sm text-gray-400">No entities found</p>
          </div>
        </div>
      )}
    </div>
  );
};

export default GraphSearch;
