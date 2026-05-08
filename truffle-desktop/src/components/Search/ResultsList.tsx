// SPEC v2.0 SECTION 5.2.2: Search Results List Component
// Displays search results with grouping and filtering

import React from 'react';
import { 
  FileText, 
  Image, 
  Sparkles, 
  ChevronRight,
  Folder
} from 'lucide-react';

interface SearchResult {
  id: string;
  type: 'artifact' | 'wiki-node' | 'compilation';
  title: string;
  excerpt: string;
  score: number;
  metadata?: Record<string, unknown>;
}

interface ResultsListProps {
  results: SearchResult[];
  selectedId: string | null;
  onSelect: (result: SearchResult) => void;
  groupByType?: boolean;
  className?: string;
}

export const ResultsList: React.FC<ResultsListProps> = ({
  results,
  selectedId,
  onSelect,
  groupByType = true,
  className = '',
}) => {
  // Group results by type
  const groupedResults = React.useMemo(() => {
    if (!groupByType) return { all: results };
    
    const groups: Record<string, SearchResult[]> = {
      artifact: [],
      'wiki-node': [],
      compilation: [],
    };
    
    results.forEach(result => {
      groups[result.type]?.push(result);
    });
    
    return groups;
  }, [results, groupByType]);

  // Get icon for result type
  const getTypeIcon = (type: string) => {
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

  // Get color for result type
  const getTypeColor = (type: string) => {
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

  // Get label for result type
  const getTypeLabel = (type: string) => {
    switch (type) {
      case 'artifact':
        return 'Artifacts';
      case 'wiki-node':
        return 'Wiki Nodes';
      case 'compilation':
        return 'Compilations';
      default:
        return type;
    }
  };

  // Render a single result item
  const renderResult = (result: SearchResult) => (
    <button
      key={result.id}
      onClick={() => onSelect(result)}
      className={`
        w-full flex items-start gap-3 px-3 py-2.5 rounded-lg text-left
        transition-all duration-150
        ${selectedId === result.id 
          ? 'bg-primary/10 border border-primary/30' 
          : 'hover:bg-surface-hover border border-transparent'
        }
      `}
    >
      <span className={getTypeColor(result.type)}>
        {getTypeIcon(result.type)}
      </span>
      
      <div className="flex-1 min-w-0">
        <div className="flex items-center gap-2">
          <p className={`
            text-sm font-medium truncate
            ${selectedId === result.id ? 'text-primary' : 'text-text-primary'}
          `}>
            {result.title}
          </p>
          
          {/* Score indicator (for debugging) */}
          {process.env.NODE_ENV === 'development' && (
            <span className="text-[10px] text-text-muted">
              {result.score.toFixed(2)}
            </span>
          )}
        </div>
        
        <p className="text-xs text-text-muted line-clamp-2 mt-0.5">
          {result.excerpt}
        </p>
        
        {/* Metadata tags */}
        {result.metadata && Object.keys(result.metadata).length > 0 && (
          <div className="flex items-center gap-1.5 mt-1.5">
            {Object.entries(result.metadata).slice(0, 3).map(([key, value]) => (
              <span 
                key={key}
                className="px-1.5 py-0.5 bg-surface rounded text-[10px] text-text-muted"
              >
                {key}: {String(value).slice(0, 20)}
              </span>
            ))}
          </div>
        )}
      </div>
      
      <ChevronRight className={`
        w-4 h-4 flex-shrink-0 transition-colors
        ${selectedId === result.id ? 'text-primary' : 'text-text-muted'}
      `} />
    </button>
  );

  // Render grouped results
  const renderGroupedResults = () => {
    const groups = Object.entries(groupedResults).filter(([, items]) => items.length > 0);
    
    return (
      <div className="space-y-4">
        {groups.map(([type, items]) => (
          <div key={type}>
            {/* Group Header */}
            <div className="flex items-center gap-2 px-3 py-2 mb-1">
              <Folder className="w-4 h-4 text-text-muted" />
              <span className="text-xs font-medium text-text-secondary uppercase tracking-wide">
                {getTypeLabel(type)}
              </span>
              <span className="text-xs text-text-muted">
                ({items.length})
              </span>
            </div>
            
            {/* Group Items */}
            <div className="space-y-1">
              {items.map(renderResult)}
            </div>
          </div>
        ))}
      </div>
    );
  };

  // Render flat results
  const renderFlatResults = () => (
    <div className="space-y-1">
      {results.map(renderResult)}
    </div>
  );

  if (results.length === 0) {
    return (
      <div className={`flex flex-col items-center justify-center py-8 text-text-muted ${className}`}>
        <FileText className="w-10 h-10 mb-3 opacity-50" />
        <p className="text-sm">No results found</p>
      </div>
    );
  }

  return (
    <div className={`overflow-auto ${className}`}>
      {groupByType ? renderGroupedResults() : renderFlatResults()}
    </div>
  );
};

export default ResultsList;
