// SPEC v2.0 SECTION 5.2.2: Raw Waterfall Panel
// Left panel showing raw screenshot artifacts

import React, { useState, useCallback } from 'react';
import { 
  Upload, 
  Filter, 
  Grid3X3, 
  List, 
  MoreVertical,
  Trash2,
  Eye,
  Play,
  Clock,
  CheckCircle2,
  AlertCircle
} from 'lucide-react';
import { useAppStore } from '../../store/appStore';
import { ThumbnailGrid } from './ThumbnailGrid';
import { DragDropZone } from './DragDropZone';

interface RawPanelProps {
  className?: string;
}

type ViewType = 'grid' | 'list';
type FilterType = 'all' | 'pending' | 'compiled' | 'failed';

export const RawPanel: React.FC<RawPanelProps> = ({ className = '' }) => {
  const { 
    artifacts, 
    selectedArtifactId,
    selectArtifact,
    isCompiling,
  } = useAppStore();
  
  const [viewType, setViewType] = useState<ViewType>('grid');
  const [filterType, setFilterType] = useState<FilterType>('all');
  const [isDragging, setIsDragging] = useState(false);
  const [showDropZone, setShowDropZone] = useState(false);

  // Filter artifacts
  const filteredArtifacts = React.useMemo(() => {
    switch (filterType) {
      case 'pending':
        return artifacts.filter(a => a.status === 'pending');
      case 'compiled':
        return artifacts.filter(a => a.status === 'compiled');
      case 'failed':
        return artifacts.filter(a => a.status === 'failed');
      default:
        return artifacts;
    }
  }, [artifacts, filterType]);

  // Handle drag events
  const handleDragEnter = useCallback((e: React.DragEvent) => {
    e.preventDefault();
    setIsDragging(true);
    setShowDropZone(true);
  }, []);

  const handleDragLeave = useCallback((e: React.DragEvent) => {
    e.preventDefault();
    setIsDragging(false);
    setShowDropZone(false);
  }, []);

  const handleDrop = useCallback((e: React.DragEvent) => {
    e.preventDefault();
    setIsDragging(false);
    setShowDropZone(false);
    
    // Handle dropped files
    const files = Array.from(e.dataTransfer.files).filter(
      file => file.type.startsWith('image/')
    );
    
    if (files.length > 0) {
      // TODO: Import files via Tauri API
      console.log('Dropped files:', files);
    }
  }, []);

  // Get status icon
  const getStatusIcon = (status: string) => {
    switch (status) {
      case 'pending':
        return <Clock className="w-3 h-3 text-text-muted" />;
      case 'compiling':
        return <div className="w-3 h-3 border-2 border-primary border-t-transparent rounded-full animate-spin" />;
      case 'compiled':
        return <CheckCircle2 className="w-3 h-3 text-secondary" />;
      case 'failed':
        return <AlertCircle className="w-3 h-3 text-error" />;
      default:
        return null;
    }
  };

  return (
    <div 
      className={`flex flex-col h-full bg-background ${className}`}
      onDragEnter={handleDragEnter}
      onDragOver={(e) => e.preventDefault()}
      onDrop={handleDrop}
    >
      {/* Panel Header */}
      <div className="flex items-center justify-between px-3 py-2 border-b border-border">
        <div className="flex items-center gap-2">
          <h2 className="text-sm font-semibold text-text-primary">Raw Waterfall</h2>
          <span className="text-xs text-text-muted">({filteredArtifacts.length})</span>
        </div>
        
        <div className="flex items-center gap-1">
          {/* Filter Dropdown */}
          <div className="relative group">
            <button 
              className="p-1.5 rounded-md text-text-secondary hover:text-text-primary hover:bg-surface-hover transition-colors"
              title="Filter"
            >
              <Filter className="w-4 h-4" />
            </button>
            
            {/* Filter Menu */}
            <div className="absolute top-full right-0 mt-1 w-32 bg-surface-elevated border border-border rounded-lg shadow-lg opacity-0 invisible group-hover:opacity-100 group-hover:visible transition-all z-50">
              {(['all', 'pending', 'compiled', 'failed'] as FilterType[]).map((type) => (
                <button
                  key={type}
                  onClick={() => setFilterType(type)}
                  className={`
                    w-full px-3 py-2 text-left text-sm capitalize
                    ${filterType === type 
                      ? 'text-primary bg-primary/10' 
                      : 'text-text-secondary hover:text-text-primary hover:bg-surface-hover'
                    }
                  `}
                >
                  {type}
                </button>
              ))}
            </div>
          </div>
          
          {/* View Toggle */}
          <div className="flex items-center bg-surface rounded-md p-0.5">
            <button
              onClick={() => setViewType('grid')}
              className={`
                p-1 rounded transition-colors
                ${viewType === 'grid' 
                  ? 'bg-surface-elevated text-text-primary' 
                  : 'text-text-secondary hover:text-text-primary'
                }
              `}
            >
              <Grid3X3 className="w-4 h-4" />
            </button>
            <button
              onClick={() => setViewType('list')}
              className={`
                p-1 rounded transition-colors
                ${viewType === 'list' 
                  ? 'bg-surface-elevated text-text-primary' 
                  : 'text-text-secondary hover:text-text-primary'
                }
              `}
            >
              <List className="w-4 h-4" />
            </button>
          </div>
          
          {/* Import Button */}
          <button
            className="flex items-center gap-1.5 px-2 py-1.5 bg-primary text-background rounded-md text-xs font-medium hover:bg-primary-hover transition-colors"
            onClick={() => setShowDropZone(true)}
          >
            <Upload className="w-3.5 h-3.5" />
            <span className="hidden sm:inline">Import</span>
          </button>
        </div>
      </div>
      
      {/* Content */}
      <div className="flex-1 overflow-hidden relative">
        {filteredArtifacts.length === 0 ? (
          <div className="flex flex-col items-center justify-center h-full text-text-muted">
            <div className="w-16 h-16 rounded-full bg-surface flex items-center justify-center mb-4">
              <Upload className="w-8 h-8" />
            </div>
            <p className="text-sm">No screenshots yet</p>
            <p className="text-xs mt-1">Drag and drop images or click Import</p>
          </div>
        ) : viewType === 'grid' ? (
          <ThumbnailGrid 
            artifacts={filteredArtifacts}
            selectedId={selectedArtifactId}
            onSelect={selectArtifact}
          />
        ) : (
          <div className="overflow-auto h-full">
            <table className="w-full">
              <thead className="sticky top-0 bg-surface z-10">
                <tr className="text-left text-xs text-text-muted border-b border-border">
                  <th className="px-3 py-2 font-medium">Name</th>
                  <th className="px-3 py-2 font-medium">Status</th>
                  <th className="px-3 py-2 font-medium">Date</th>
                  <th className="px-3 py-2 font-medium w-10"></th>
                </tr>
              </thead>
              <tbody>
                {filteredArtifacts.map((artifact) => (
                  <tr
                    key={artifact.id}
                    onClick={() => selectArtifact(artifact.id)}
                    className={`
                      text-sm border-b border-border cursor-pointer
                      ${selectedArtifactId === artifact.id 
                        ? 'bg-primary/10' 
                        : 'hover:bg-surface-hover'
                      }
                    `}
                  >
                    <td className="px-3 py-2">
                      <div className="flex items-center gap-2">
                        <div className="w-8 h-8 rounded bg-surface flex items-center justify-center">
                          {artifact.thumbnailUrl ? (
                            <img 
                              src={artifact.thumbnailUrl} 
                              alt="" 
                              className="w-full h-full object-cover rounded"
                            />
                          ) : (
                            <div className="w-full h-full bg-surface-elevated rounded" />
                          )}
                        </div>
                        <span className="truncate max-w-[150px]">{artifact.filename}</span>
                      </div>
                    </td>
                    <td className="px-3 py-2">
                      <div className="flex items-center gap-1.5">
                        {getStatusIcon(artifact.status)}
                        <span className="capitalize text-xs">{artifact.status}</span>
                      </div>
                    </td>
                    <td className="px-3 py-2 text-text-muted text-xs">
                      {new Date(artifact.capturedAt).toLocaleDateString()}
                    </td>
                    <td className="px-3 py-2">
                      <button 
                        className="p-1 rounded hover:bg-surface-hover text-text-muted hover:text-text-primary"
                        onClick={(e) => {
                          e.stopPropagation();
                          // Show context menu
                        }}
                      >
                        <MoreVertical className="w-4 h-4" />
                      </button>
                    </td>
                  </tr>
                ))}
              </tbody>
            </table>
          </div>
        )}
        
        {/* Drag Drop Zone Overlay */}
        {(isDragging || showDropZone) && (
          <DragDropZone 
            isActive={isDragging}
            onClose={() => setShowDropZone(false)}
          />
        )}
      </div>
    </div>
  );
};

export default RawPanel;
