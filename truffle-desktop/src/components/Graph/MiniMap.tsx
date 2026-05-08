import React, { useMemo, useRef, useCallback } from 'react';
import { GraphData } from './types';

export interface MiniMapProps {
  graphData: GraphData;
  viewport: { x: number; y: number; width: number; height: number };
  onViewportChange?: (viewport: { x: number; y: number; width: number; height: number }) => void;
}

const MINIMAP_WIDTH = 180;
const MINIMAP_HEIGHT = 120;
const PADDING = 10;

export const MiniMap: React.FC<MiniMapProps> = ({
  graphData,
  viewport,
  onViewportChange,
}) => {
  const containerRef = useRef<HTMLDivElement>(null);

  // Calculate bounds of the graph
  const bounds = useMemo(() => {
    if (graphData.nodes.length === 0) {
      return { minX: -100, maxX: 100, minY: -100, maxY: 100 };
    }

    let minX = Infinity, maxX = -Infinity;
    let minY = Infinity, maxY = -Infinity;

    graphData.nodes.forEach(node => {
      minX = Math.min(minX, node.x || 0);
      maxX = Math.max(maxX, node.x || 0);
      minY = Math.min(minY, node.y || 0);
      maxY = Math.max(maxY, node.y || 0);
    });

    // Add padding
    const padding = 50;
    return {
      minX: minX - padding,
      maxX: maxX + padding,
      minY: minY - padding,
      maxY: maxY + padding,
    };
  }, [graphData.nodes]);

  // Calculate scale to fit graph in minimap
  const scale = useMemo(() => {
    const graphWidth = bounds.maxX - bounds.minX;
    const graphHeight = bounds.maxY - bounds.minY;
    
    if (graphWidth === 0 || graphHeight === 0) return 1;
    
    const scaleX = (MINIMAP_WIDTH - PADDING * 2) / graphWidth;
    const scaleY = (MINIMAP_HEIGHT - PADDING * 2) / graphHeight;
    
    return Math.min(scaleX, scaleY);
  }, [bounds]);

  // Transform coordinates
  const transform = useCallback((x: number, y: number) => {
    return {
      x: (x - bounds.minX) * scale + PADDING,
      y: (y - bounds.minY) * scale + PADDING,
    };
  }, [bounds, scale]);

  // Viewport rectangle in minimap coordinates
  const viewportRect = useMemo(() => {
    const topLeft = transform(viewport.x, viewport.y);
    const bottomRight = transform(viewport.x + viewport.width, viewport.y + viewport.height);
    
    return {
      x: topLeft.x,
      y: topLeft.y,
      width: bottomRight.x - topLeft.x,
      height: bottomRight.y - topLeft.y,
    };
  }, [viewport, transform]);

  // Handle click on minimap to navigate
  const handleClick = useCallback((e: React.MouseEvent) => {
    if (!containerRef.current || !onViewportChange) return;

    const rect = containerRef.current.getBoundingClientRect();
    const clickX = e.clientX - rect.left;
    const clickY = e.clientY - rect.top;

    // Convert minimap coordinates back to graph coordinates
    const graphX = (clickX - PADDING) / scale + bounds.minX - viewport.width / 2;
    const graphY = (clickY - PADDING) / scale + bounds.minY - viewport.height / 2;

    onViewportChange({
      x: graphX,
      y: graphY,
      width: viewport.width,
      height: viewport.height,
    });
  }, [bounds, scale, viewport, onViewportChange]);

  // Limit the number of nodes shown in minimap for performance
  const visibleNodes = useMemo(() => {
    if (graphData.nodes.length <= 100) return graphData.nodes;
    // Sample nodes evenly
    const step = Math.ceil(graphData.nodes.length / 100);
    return graphData.nodes.filter((_, i) => i % step === 0);
  }, [graphData.nodes]);

  if (graphData.nodes.length === 0) {
    return null;
  }

  return (
    <div
      ref={containerRef}
      className="absolute bottom-4 right-4 bg-gray-800/90 backdrop-blur-sm rounded-lg shadow-lg border border-gray-700 overflow-hidden z-10"
      style={{ width: MINIMAP_WIDTH, height: MINIMAP_HEIGHT }}
      onClick={handleClick}
    >
      <svg
        width={MINIMAP_WIDTH}
        height={MINIMAP_HEIGHT}
        className="cursor-pointer"
      >
        {/* Background */}
        <rect
          width={MINIMAP_WIDTH}
          height={MINIMAP_HEIGHT}
          fill="#1f2937"
          opacity={0.5}
        />

        {/* Edges (simplified) */}
        {graphData.links.slice(0, 50).map((link, i) => {
          const sourceNode = graphData.nodes.find(n => n.id === link.source);
          const targetNode = graphData.nodes.find(n => n.id === link.target);
          if (!sourceNode || !targetNode) return null;

          const s = transform(sourceNode.x || 0, sourceNode.y || 0);
          const t = transform(targetNode.x || 0, targetNode.y || 0);

          return (
            <line
              key={i}
              x1={s.x}
              y1={s.y}
              x2={t.x}
              y2={t.y}
              stroke="#4b5563"
              strokeWidth={0.5}
              opacity={0.3}
            />
          );
        })}

        {/* Nodes */}
        {visibleNodes.map(node => {
          const pos = transform(node.x || 0, node.y || 0);
          return (
            <circle
              key={node.id}
              cx={pos.x}
              cy={pos.y}
              r={2}
              fill={node.color}
              opacity={0.8}
            />
          );
        })}

        {/* Viewport rectangle */}
        <rect
          x={viewportRect.x}
          y={viewportRect.y}
          width={viewportRect.width}
          height={viewportRect.height}
          fill="rgba(59, 130, 246, 0.15)"
          stroke="#3b82f6"
          strokeWidth={1}
          rx={2}
        />
      </svg>

      {/* Label */}
      <div className="absolute top-1 left-2 text-[10px] text-gray-500 font-medium">
        Overview
      </div>
    </div>
  );
};

export default MiniMap;
