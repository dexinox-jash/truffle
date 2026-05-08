import React, { useState } from 'react';
import { EntityType } from '../../types/knowledge-graph';

export interface GraphControlsProps {
  onZoomIn: () => void;
  onZoomOut: () => void;
  onFit: () => void;
  onReset: () => void;
  filters?: {
    entityTypes: EntityType[];
    relationshipTypes: string[];
  };
  onFilterChange?: (filters: { entityTypes: EntityType[]; relationshipTypes: string[] }) => void;
}

export const GraphControls: React.FC<GraphControlsProps> = ({
  onZoomIn,
  onZoomOut,
  onFit,
  onReset,
  filters,
  onFilterChange,
}) => {
  const [showFilters, setShowFilters] = useState(false);
  const [selectedEntityTypes, setSelectedEntityTypes] = useState<EntityType[]>([]);
  const [selectedRelationshipTypes, setSelectedRelationshipTypes] = useState<string[]>([]);

  const handleEntityTypeToggle = (type: EntityType) => {
    const newTypes = selectedEntityTypes.includes(type)
      ? selectedEntityTypes.filter(t => t !== type)
      : [...selectedEntityTypes, type];
    setSelectedEntityTypes(newTypes);
    onFilterChange?.({
      entityTypes: newTypes,
      relationshipTypes: selectedRelationshipTypes,
    });
  };

  const handleRelationshipTypeToggle = (type: string) => {
    const newTypes = selectedRelationshipTypes.includes(type)
      ? selectedRelationshipTypes.filter(t => t !== type)
      : [...selectedRelationshipTypes, type];
    setSelectedRelationshipTypes(newTypes);
    onFilterChange?.({
      entityTypes: selectedEntityTypes,
      relationshipTypes: newTypes,
    });
  };

  const clearFilters = () => {
    setSelectedEntityTypes([]);
    setSelectedRelationshipTypes([]);
    onFilterChange?.({
      entityTypes: [],
      relationshipTypes: [],
    });
  };

  const hasActiveFilters = selectedEntityTypes.length > 0 || selectedRelationshipTypes.length > 0;

  return (
    <div className="absolute top-4 right-4 flex flex-col gap-2 z-10">
      {/* Main Controls */}
      <div className="flex flex-col gap-1 p-2 bg-gray-800/90 backdrop-blur-sm rounded-lg shadow-lg border border-gray-700">
        <button
          onClick={onZoomIn}
          className="p-2 rounded hover:bg-gray-700 transition-colors"
          title="Zoom In"
          aria-label="Zoom In"
        >
          <svg className="w-5 h-5 text-gray-300" fill="none" stroke="currentColor" viewBox="0 0 24 24">
            <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M12 6v6m0 0v6m0-6h6m-6 0H6" />
          </svg>
        </button>
        <button
          onClick={onZoomOut}
          className="p-2 rounded hover:bg-gray-700 transition-colors"
          title="Zoom Out"
          aria-label="Zoom Out"
        >
          <svg className="w-5 h-5 text-gray-300" fill="none" stroke="currentColor" viewBox="0 0 24 24">
            <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M20 12H4" />
          </svg>
        </button>
        <div className="w-full h-px bg-gray-700 my-1" />
        <button
          onClick={onFit}
          className="p-2 rounded hover:bg-gray-700 transition-colors"
          title="Fit to Screen"
          aria-label="Fit to Screen"
        >
          <svg className="w-5 h-5 text-gray-300" fill="none" stroke="currentColor" viewBox="0 0 24 24">
            <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M4 8V4m0 0h4M4 4l5 5m11-1V4m0 0h-4m4 0l-5 5M4 16v4m0 0h4m-4 0l5-5m11 5l-5-5m5 5v-4m0 4h-4" />
          </svg>
        </button>
        <button
          onClick={onReset}
          className="p-2 rounded hover:bg-gray-700 transition-colors"
          title="Reset View"
          aria-label="Reset View"
        >
          <svg className="w-5 h-5 text-gray-300" fill="none" stroke="currentColor" viewBox="0 0 24 24">
            <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M3 12l2-2m0 0l7-7 7 7M5 10v10a1 1 0 001 1h3m10-11l2 2m-2-2v10a1 1 0 01-1 1h-3m-6 0a1 1 0 001-1v-4a1 1 0 011-1h2a1 1 0 011 1v4a1 1 0 001 1m-6 0h6" />
          </svg>
        </button>
        {filters && (
          <>
            <div className="w-full h-px bg-gray-700 my-1" />
            <button
              onClick={() => setShowFilters(!showFilters)}
              className={`p-2 rounded transition-colors ${showFilters ? 'bg-blue-600' : 'hover:bg-gray-700'}`}
              title="Toggle Filters"
              aria-label="Toggle Filters"
            >
              <svg className="w-5 h-5 text-gray-300" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M3 4a1 1 0 011-1h16a1 1 0 011 1v2.586a1 1 0 01-.293.707l-6.414 6.414a1 1 0 00-.293.707V17l-4 4v-6.586a1 1 0 00-.293-.707L3.293 7.293A1 1 0 013 6.586V4z" />
              </svg>
              {hasActiveFilters && (
                <span className="absolute top-1 right-1 w-2 h-2 bg-blue-500 rounded-full" />
              )}
            </button>
          </>
        )}
      </div>

      {/* Filter Panel */}
      {showFilters && filters && (
        <div className="w-64 p-4 bg-gray-800/95 backdrop-blur-sm rounded-lg shadow-lg border border-gray-700">
          <div className="flex items-center justify-between mb-3">
            <h3 className="text-sm font-semibold text-gray-200">Filters</h3>
            {hasActiveFilters && (
              <button
                onClick={clearFilters}
                className="text-xs text-blue-400 hover:text-blue-300"
              >
                Clear all
              </button>
            )}
          </div>

          {/* Entity Type Filters */}
          {filters.entityTypes.length > 0 && (
            <div className="mb-4">
              <h4 className="text-xs font-medium text-gray-400 mb-2">Entity Types</h4>
              <div className="space-y-1">
                {filters.entityTypes.map(type => (
                  <label
                    key={type}
                    className="flex items-center gap-2 px-2 py-1 rounded hover:bg-gray-700/50 cursor-pointer"
                  >
                    <input
                      type="checkbox"
                      checked={selectedEntityTypes.includes(type)}
                      onChange={() => handleEntityTypeToggle(type)}
                      className="w-4 h-4 rounded border-gray-600 bg-gray-700 text-blue-600 focus:ring-blue-500"
                    />
                    <span className="text-sm text-gray-300 capitalize">{type}</span>
                  </label>
                ))}
              </div>
            </div>
          )}

          {/* Relationship Type Filters */}
          {filters.relationshipTypes.length > 0 && (
            <div>
              <h4 className="text-xs font-medium text-gray-400 mb-2">Relationship Types</h4>
              <div className="space-y-1 max-h-32 overflow-y-auto">
                {filters.relationshipTypes.map(type => (
                  <label
                    key={type}
                    className="flex items-center gap-2 px-2 py-1 rounded hover:bg-gray-700/50 cursor-pointer"
                  >
                    <input
                      type="checkbox"
                      checked={selectedRelationshipTypes.includes(type)}
                      onChange={() => handleRelationshipTypeToggle(type)}
                      className="w-4 h-4 rounded border-gray-600 bg-gray-700 text-blue-600 focus:ring-blue-500"
                    />
                    <span className="text-sm text-gray-300">
                      {type.replace(/_/g, ' ')}
                    </span>
                  </label>
                ))}
              </div>
            </div>
          )}
        </div>
      )}
    </div>
  );
};

export default GraphControls;
