// SPEC v2.0 SECTION 5.2.2: Thumbnail Grid Component
// Grid view for raw artifacts

import React, { useRef, useCallback } from 'react';
import { Play, CheckCircle2, AlertCircle, Clock, MoreHorizontal } from 'lucide-react';
import { useAppStore } from '../../store/appStore';

interface Artifact {
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

interface ThumbnailGridProps {
  artifacts: Artifact[];
  selectedId: string | null;
  onSelect: (id: string | null) => void;
  className?: string;
}

export const ThumbnailGrid: React.FC<ThumbnailGridProps> = ({
  artifacts,
  selectedId,
  onSelect,
  className = '',
}) => {
  const gridRef = useRef<HTMLDivElement>(null);
  const { isCompiling } = useAppStore();

  // Virtual scrolling for performance
  const rowHeight = 160;
  const columnCount = 3;
  const gap = 12;

  // Get status badge
  const getStatusBadge = (status: string) => {
    switch (status) {
      case 'pending':
        return (
          <div className="absolute top-2 right-2 flex items-center gap-1 px-1.5 py-0.5 bg-background/80 backdrop-blur rounded text-xs text-text-muted">
            <Clock className="w-3 h-3" />
            <span>Pending</span>
          </div>
        );
      case 'compiling':
        return (
          <div className="absolute top-2 right-2 flex items-center gap-1 px-1.5 py-0.5 bg-primary/20 backdrop-blur rounded text-xs text-primary">
            <div className="w-3 h-3 border-2 border-primary border-t-transparent rounded-full animate-spin" />
            <span>Compiling</span>
          </div>
        );
      case 'compiled':
        return (
          <div className="absolute top-2 right-2 flex items-center gap-1 px-1.5 py-0.5 bg-secondary/20 backdrop-blur rounded text-xs text-secondary">
            <CheckCircle2 className="w-3 h-3" />
            <span>Done</span>
          </div>
        );
      case 'failed':
        return (
          <div className="absolute top-2 right-2 flex items-center gap-1 px-1.5 py-0.5 bg-error/20 backdrop-blur rounded text-xs text-error">
            <AlertCircle className="w-3 h-3" />
            <span>Failed</span>
          </div>
        );
      default:
        return null;
    }
  };

  // Handle compile click
  const handleCompile = useCallback((e: React.MouseEvent, artifactId: string) => {
    e.stopPropagation();
    // TODO: Trigger compilation via Tauri API
    console.log('Compile artifact:', artifactId);
  }, []);

  // Handle context menu
  const handleContextMenu = useCallback((e: React.MouseEvent, artifact: Artifact) => {
    e.preventDefault();
    e.stopPropagation();
    // TODO: Show context menu
    console.log('Context menu for:', artifact.id);
  }, []);

  return (
    <div 
      ref={gridRef}
      className={`overflow-auto h-full p-3 ${className}`}
    >
      <div 
        className="grid gap-3"
        style={{
          gridTemplateColumns: 'repeat(auto-fill, minmax(140px, 1fr))',
        }}
      >
        {artifacts.map((artifact) => (
          <div
            key={artifact.id}
            onClick={() => onSelect(artifact.id === selectedId ? null : artifact.id)}
            onContextMenu={(e) => handleContextMenu(e, artifact)}
            className={`
              group relative rounded-lg overflow-hidden cursor-pointer
              transition-all duration-200
              ${selectedId === artifact.id 
                ? 'ring-2 ring-primary ring-offset-2 ring-offset-background' 
                : 'hover:ring-1 hover:ring-border'
              }
            `}
          >
            {/* Thumbnail */}
            <div className="aspect-square bg-surface relative">
              {artifact.thumbnailUrl ? (
                <img
                  src={artifact.thumbnailUrl}
                  alt={artifact.filename}
                  className="w-full h-full object-cover"
                  loading="lazy"
                />
              ) : (
                <div className="w-full h-full flex items-center justify-center bg-surface-elevated">
                  <div className="w-12 h-12 rounded-full bg-background flex items-center justify-center">
                    <span className="text-2xl text-text-muted">📷</span>
                  </div>
                </div>
              )}
              
              {/* Status Badge */}
              {getStatusBadge(artifact.status)}
              
              {/* Hover Overlay */}
              <div className="absolute inset-0 bg-black/50 opacity-0 group-hover:opacity-100 transition-opacity flex items-center justify-center gap-2">
                {artifact.status === 'pending' && (
                  <button
                    onClick={(e) => handleCompile(e, artifact.id)}
                    disabled={isCompiling}
                    className="p-2 bg-primary text-background rounded-full hover:bg-primary-hover transition-colors disabled:opacity-50"
                  >
                    <Play className="w-4 h-4" />
                  </button>
                )}
                <button className="p-2 bg-surface text-text-primary rounded-full hover:bg-surface-hover transition-colors">
                  <MoreHorizontal className="w-4 h-4" />
                </button>
              </div>
            </div>
            
            {/* Info */}
            <div className="p-2 bg-surface">
              <p className="text-xs font-medium truncate text-text-primary">
                {artifact.filename}
              </p>
              <p className="text-[10px] text-text-muted mt-0.5">
                {new Date(artifact.capturedAt).toLocaleDateString()}
              </p>
              {artifact.appContext && (
                <p className="text-[10px] text-text-muted truncate">
                  {artifact.appContext.appName}
                </p>
              )}
            </div>
          </div>
        ))}
      </div>
    </div>
  );
};

export default ThumbnailGrid;
