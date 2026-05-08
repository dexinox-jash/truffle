// SPEC v2.0 SECTION 5.2.2: Search Interface Component
// Global search with FlexSearch integration

import React, { useState, useEffect, useCallback, useRef } from 'react';
import { 
  Search, 
  X, 
  Filter, 
  Clock, 
  FileText, 
  Image,
  Sparkles,
  ArrowRight,
  CornerDownLeft
} from 'lucide-react';
import { useSearch } from '../../store/searchStore';
import { useAppStore } from '../../store/appStore';

interface SearchBarProps {
  onClose: () => void;
  className?: string;
}

export const SearchBar: React.FC<SearchBarProps> = ({ 
  onClose,
  className = '' 
}) => {
  const {
    query,
    results,
    isSearching,
    searchTime,
    filters,
    recentSearches,
    setQuery,
    performSearch,
    toggleFilter,
    addRecentSearch,
    clearRecentSearches,
  } = useSearch();

  const { selectArtifact, selectNode, openCommandPalette } = useAppStore();
  
  const [showFilters, setShowFilters] = useState(false);
  const [selectedIndex, setSelectedIndex] = useState(0);
  const inputRef = useRef<HTMLInputElement>(null);

  // Focus input on mount
  useEffect(() => {
    inputRef.current?.focus();
  }, []);

  // Perform search when query changes
  useEffect(() => {
    const timeout = setTimeout(() => {
      if (query.trim()) {
        performSearch(query);
      }
    }, 150);

    return () => clearTimeout(timeout);
  }, [query, performSearch]);

  // Handle keyboard navigation
  const handleKeyDown = useCallback((e: React.KeyboardEvent) => {
    switch (e.key) {
      case 'ArrowDown':
        e.preventDefault();
        setSelectedIndex(prev => 
          Math.min(prev + 1, results.length - 1)
        );
        break;
      case 'ArrowUp':
        e.preventDefault();
        setSelectedIndex(prev => Math.max(prev - 1, 0));
        break;
      case 'Enter':
        e.preventDefault();
        if (results[selectedIndex]) {
          handleSelectResult(results[selectedIndex]);
        }
        break;
      case 'Escape':
        e.preventDefault();
        onClose();
        break;
    }
  }, [results, selectedIndex, onClose]);

  // Handle result selection
  const handleSelectResult = useCallback((result: typeof results[0]) => {
    addRecentSearch(query);
    
    switch (result.type) {
      case 'artifact':
        selectArtifact(result.id);
        break;
      case 'wiki-node':
        selectNode(result.id);
        break;
      case 'compilation':
        // Navigate to compilation
        break;
    }
    
    onClose();
  }, [query, selectArtifact, selectNode, addRecentSearch, onClose]);

  // Get result icon
  const getResultIcon = (type: string) => {
    switch (type) {
      case 'artifact':
        return <Image className="w-4 h-4" />;
      case 'wiki-node':
        return <FileText className="w-4 h-4" />;
      case 'compilation':
        return <Sparkles className="w-4 h-4" />;
      default:
        return <FileText className="w-4 h-4" />;
    }
  };

  // Get result color
  const getResultColor = (type: string) => {
    switch (type) {
      case 'artifact':
        return 'text-primary';
      case 'wiki-node':
        return 'text-secondary';
      case 'compilation':
        return 'text-warning';
      default:
        return 'text-text-muted';
    }
  };

  return (
    <div 
      className={`
        fixed inset-0 z-command-palette
        bg-black/60 backdrop-blur-sm
        flex items-start justify-center pt-[15vh]
        ${className}
      `}
      onClick={(e) => {
        if (e.target === e.currentTarget) onClose();
      }}
    >
      <div 
        className="w-full max-w-2xl mx-4 bg-surface-elevated rounded-xl shadow-2xl border border-border overflow-hidden command-palette-enter"
        onClick={(e) => e.stopPropagation()}
      >
        {/* Search Input */}
        <div className="flex items-center gap-3 px-4 py-3 border-b border-border">
          <Search className="w-5 h-5 text-text-muted" />
          
          <input
            ref={inputRef}
            type="text"
            value={query}
            onChange={(e) => {
              setQuery(e.target.value);
              setSelectedIndex(0);
            }}
            onKeyDown={handleKeyDown}
            placeholder="Search artifacts, wiki nodes, and more..."
            className="flex-1 bg-transparent text-lg text-text-primary placeholder:text-text-muted focus:outline-none"
          />
          
          {query && (
            <button
              onClick={() => {
                setQuery('');
                inputRef.current?.focus();
              }}
              className="p-1 rounded text-text-muted hover:text-text-primary hover:bg-surface-hover transition-colors"
            >
              <X className="w-4 h-4" />
            </button>
          )}
          
          <button
            onClick={() => setShowFilters(!showFilters)}
            className={`
              p-2 rounded transition-colors
              ${showFilters 
                ? 'bg-primary/10 text-primary' 
                : 'text-text-muted hover:text-text-primary hover:bg-surface-hover'
              }
            `}
          >
            <Filter className="w-4 h-4" />
          </button>
          
          <kbd className="hidden sm:flex items-center gap-1 px-2 py-1 bg-background rounded text-xs text-text-muted">
            <span>ESC</span>
          </kbd>
        </div>

        {/* Filters */}
        {showFilters && (
          <div className="px-4 py-2 border-b border-border bg-surface/50">
            <div className="flex items-center gap-2">
              <span className="text-xs text-text-muted">Filter:</span>
              {(['artifact', 'wiki-node', 'compilation'] as const).map((type) => (
                <button
                  key={type}
                  onClick={() => toggleFilter(type)}
                  className={`
                    px-2 py-1 rounded text-xs capitalize transition-colors
                    ${filters.types.includes(type)
                      ? 'bg-primary/10 text-primary'
                      : 'bg-background text-text-muted hover:text-text-primary'
                    }
                  `}
                >
                  {type.replace('-', ' ')}
                </button>
              ))}
            </div>
          </div>
        )}

        {/* Results */}
        <div className="max-h-[50vh] overflow-auto">
          {isSearching ? (
            <div className="flex items-center justify-center py-8">
              <div className="w-6 h-6 border-2 border-primary border-t-transparent rounded-full animate-spin" />
            </div>
          ) : query.trim() && results.length > 0 ? (
            <>
              {/* Results Header */}
              <div className="px-4 py-2 text-xs text-text-muted border-b border-border">
                Found {results.length} results in {searchTime.toFixed(0)}ms
              </div>
              
              {/* Results List */}
              <div className="py-2">
                {results.map((result, index) => (
                  <button
                    key={result.id}
                    onClick={() => handleSelectResult(result)}
                    onMouseEnter={() => setSelectedIndex(index)}
                    className={`
                      w-full flex items-start gap-3 px-4 py-3 text-left
                      transition-colors
                      ${selectedIndex === index 
                        ? 'bg-primary/10' 
                        : 'hover:bg-surface-hover'
                      }
                    `}
                  >
                    <span className={getResultColor(result.type)}>
                      {getResultIcon(result.type)}
                    </span>
                    
                    <div className="flex-1 min-w-0">
                      <p className={`
                        text-sm font-medium truncate
                        ${selectedIndex === index ? 'text-primary' : 'text-text-primary'}
                      `}>
                        {result.title}
                      </p>
                      <p className="text-xs text-text-muted line-clamp-2 mt-0.5">
                        {result.excerpt}
                      </p>
                    </div>
                    
                    <span className="text-xs text-text-muted capitalize">
                      {result.type.replace('-', ' ')}
                    </span>
                  </button>
                ))}
              </div>
            </>
          ) : query.trim() ? (
            <div className="flex flex-col items-center justify-center py-8 text-text-muted">
              <Search className="w-10 h-10 mb-3 opacity-50" />
              <p className="text-sm">No results found</p>
              <p className="text-xs mt-1">Try a different search term</p>
            </div>
          ) : (
            /* Recent Searches */
            recentSearches.length > 0 && (
              <div className="py-2">
                <div className="px-4 py-2 text-xs text-text-muted uppercase tracking-wide">
                  Recent Searches
                </div>
                {recentSearches.map((search, index) => (
                  <button
                    key={search}
                    onClick={() => {
                      setQuery(search);
                      performSearch(search);
                    }}
                    className="w-full flex items-center gap-3 px-4 py-2 text-left hover:bg-surface-hover transition-colors"
                  >
                    <Clock className="w-4 h-4 text-text-muted" />
                    <span className="text-sm text-text-primary">{search}</span>
                  </button>
                ))}
                <button
                  onClick={clearRecentSearches}
                  className="w-full px-4 py-2 text-left text-xs text-text-muted hover:text-text-primary transition-colors"
                >
                  Clear recent searches
                </button>
              </div>
            )
          )}
        </div>

        {/* Footer */}
        <div className="flex items-center justify-between px-4 py-2 border-t border-border bg-surface/50 text-xs text-text-muted">
          <div className="flex items-center gap-3">
            <span className="flex items-center gap-1">
              <ArrowRight className="w-3 h-3" /> to select
            </span>
            <span className="flex items-center gap-1">
              <CornerDownLeft className="w-3 h-3" /> to open
            </span>
          </div>
          
          <button
            onClick={() => {
              onClose();
              openCommandPalette();
            }}
            className="text-text-muted hover:text-text-primary transition-colors"
          >
            Open command palette
          </button>
        </div>
      </div>
    </div>
  );
};

export default SearchBar;
