import React, { useEffect, useCallback, useState } from 'react';
import { useParams, useNavigate } from 'react-router-dom';
import {
  ZoomIn,
  ZoomOut,
  Maximize,
  Layers,
  ChevronRight,
  ChevronLeft,
  Minus,
  Plus,
} from 'lucide-react';
import { useEntityStore } from '../store/entityStore';
import { Button } from '../components/ui/Button';
import { GraphViewer } from '../components/Graph/GraphViewer';
import { EntityCard } from '../components/Entity/EntityCard';
import { Slider } from '../components/ui/Slider';

export const GraphPage: React.FC = () => {
  const { rootId } = useParams<{ rootId?: string }>();
  const navigate = useNavigate();

  const {
    graphRootId,
    graphDepth,
    setGraphRootId,
    setGraphDepth,
    selectedEntityId,
    setSelectedEntityId,
    openEntityDetail,
  } = useEntityStore();

  const [showSidebar, setShowSidebar] = useState(true);
  const [zoom, setZoom] = useState(100);

  // Sync URL params with store
  useEffect(() => {
    if (rootId) {
      setGraphRootId(rootId);
    }
  }, [rootId, setGraphRootId]);

  const handleNodeSelect = useCallback(
    (nodeId: string) => {
      setSelectedEntityId(nodeId);
    },
    [setSelectedEntityId]
  );

  const handleNodeDoubleClick = useCallback(
    (nodeId: string) => {
      setGraphRootId(nodeId);
      navigate(`/graph/${nodeId}`);
    },
    [setGraphRootId, navigate]
  );

  const handleZoomIn = () => setZoom((z) => Math.min(z + 10, 200));
  const handleZoomOut = () => setZoom((z) => Math.max(z - 10, 25));
  const handleResetZoom = () => setZoom(100);

  const handleDepthChange = useCallback(
    (value: number) => {
      setGraphDepth(value);
    },
    [setGraphDepth]
  );

  return (
    <div className="h-[calc(100vh-4rem)] flex bg-darkroom-bg">
      {/* Main graph area */}
      <div className="flex-1 relative">
        {/* Graph Viewer */}
        <GraphViewer
          rootId={graphRootId}
          depth={graphDepth}
          zoom={zoom}
          selectedNodeId={selectedEntityId}
          onNodeSelect={handleNodeSelect}
          onNodeDoubleClick={handleNodeDoubleClick}
        />

        {/* Floating controls */}
        <div className="absolute bottom-6 left-6 flex flex-col gap-2">
          {/* Zoom controls */}
          <div className="bg-darkroom-card border border-darkroom-gray-700 rounded-lg shadow-lg p-1">
            <Button
              variant="ghost"
              size="sm"
              onClick={handleZoomIn}
              className="w-8 h-8 p-0"
            >
              <Plus className="w-4 h-4" />
            </Button>
            <div className="text-center text-xs text-darkroom-text-secondary py-1">
              {zoom}%
            </div>
            <Button
              variant="ghost"
              size="sm"
              onClick={handleZoomOut}
              className="w-8 h-8 p-0"
            >
              <Minus className="w-4 h-4" />
            </Button>
          </div>

          <Button
            variant="secondary"
            size="sm"
            onClick={handleResetZoom}
            leftIcon={<Maximize className="w-4 h-4" />}
          >
            Fit
          </Button>
        </div>

        {/* Depth controls */}
        <div className="absolute bottom-6 left-1/2 -translate-x-1/2 bg-darkroom-card border border-darkroom-gray-700 rounded-lg shadow-lg px-4 py-3">
          <div className="flex items-center gap-4">
            <span className="text-sm text-darkroom-text-secondary">Depth:</span>
            <Slider
              min={1}
              max={5}
              value={graphDepth}
              onChange={handleDepthChange}
              className="w-32"
            />
            <span className="text-sm font-medium text-darkroom-text-primary w-6">
              {graphDepth}
            </span>
          </div>
        </div>

        {/* MiniMap toggle */}
        <div className="absolute top-6 right-6">
          <Button
            variant="secondary"
            size="sm"
            leftIcon={<Layers className="w-4 h-4" />}
          >
            MiniMap
          </Button>
        </div>

        {/* Sidebar toggle */}
        <button
          onClick={() => setShowSidebar(!showSidebar)}
          className="absolute top-1/2 -translate-y-1/2 right-0 bg-darkroom-card border border-darkroom-gray-700 border-r-0 rounded-l-lg p-2 shadow-lg hover:bg-darkroom-gray-700 transition-colors"
          style={{ right: showSidebar ? '320px' : '0' }}
        >
          {showSidebar ? (
            <ChevronRight className="w-4 h-4 text-darkroom-text-secondary" />
          ) : (
            <ChevronLeft className="w-4 h-4 text-darkroom-text-secondary" />
          )}
        </button>
      </div>

      {/* Side panel */}
      {showSidebar && (
        <div className="w-80 border-l border-darkroom-gray-700 bg-darkroom-card flex flex-col">
          <div className="p-4 border-b border-darkroom-gray-700">
            <h3 className="font-semibold text-darkroom-text-primary">
              Selected Entity
            </h3>
          </div>

          <div className="flex-1 overflow-auto p-4">
            {selectedEntityId ? (
              <EntityCard
                entityId={selectedEntityId}
                onViewDetail={() => navigate(`/entities/${selectedEntityId}`)}
                onSetAsRoot={() => {
                  setGraphRootId(selectedEntityId);
                  navigate(`/graph/${selectedEntityId}`);
                }}
              />
            ) : (
              <div className="text-center py-8 text-darkroom-text-tertiary">
                <Layers className="w-12 h-12 mx-auto mb-3 opacity-50" />
                <p className="text-sm">
                  Select a node in the graph to view details
                </p>
              </div>
            )}
          </div>

          {/* Graph legend */}
          <div className="p-4 border-t border-darkroom-gray-700">
            <h4 className="text-xs font-medium text-darkroom-text-secondary mb-3 uppercase tracking-wide">
              Legend
            </h4>
            <div className="space-y-2">
              <div className="flex items-center gap-2">
                <div className="w-3 h-3 rounded-full bg-accent" />
                <span className="text-xs text-darkroom-text-secondary">
                  Person
                </span>
              </div>
              <div className="flex items-center gap-2">
                <div className="w-3 h-3 rounded-full bg-purple-500" />
                <span className="text-xs text-darkroom-text-secondary">
                  Organization
                </span>
              </div>
              <div className="flex items-center gap-2">
                <div className="w-3 h-3 rounded-full bg-green-500" />
                <span className="text-xs text-darkroom-text-secondary">
                  Project
                </span>
              </div>
              <div className="flex items-center gap-2">
                <div className="w-3 h-3 rounded-full bg-orange-500" />
                <span className="text-xs text-darkroom-text-secondary">
                  Location
                </span>
              </div>
            </div>
          </div>
        </div>
      )}
    </div>
  );
};

export default GraphPage;
