import React, { useRef, useCallback, useMemo, useState, useEffect } from 'react';
import ForceGraph2D from 'react-force-graph-2d';
import * as d3 from 'd3';
import { Entity, Relationship, EntityType, GraphNode as APIGraphNode, GraphEdge as APIGraphEdge } from '../../types/knowledge-graph';
import { GraphData } from './types';
import { useEntityGraph } from '../../hooks/useEntities';
import { GraphControls } from './GraphControls';
import { MiniMap } from './MiniMap';
import { GraphSearch } from './GraphSearch';

export interface GraphViewerProps {
  rootEntityId?: string;
  depth?: number;
  width: number;
  height: number;
  onNodeClick?: (entity: Entity) => void;
  onNodeHover?: (entity: Entity | null) => void;
  highlightEntityIds?: string[];
  className?: string;
}

const NODE_COLORS: Record<EntityType, string> = {
  [EntityType.Person]: '#3b82f6',
  [EntityType.Organization]: '#10b981',
  [EntityType.Location]: '#f59e0b',
  [EntityType.Event]: '#ef4444',
  [EntityType.Product]: '#8b5cf6',
  [EntityType.Concept]: '#ec4899',
  [EntityType.Decision]: '#06b6d4',
  [EntityType.ActionItem]: '#f97316',
};

const EDGE_COLORS: Record<string, string> = {
  works_for: '#3b82f6',
  located_in: '#f59e0b',
  participated_in: '#ef4444',
  mentions: '#8b5cf6',
  related_to: '#6b7280',
  knows: '#10b981',
  created: '#ec4899',
  works_at: '#3b82f6',
  part_of: '#10b981',
  leads: '#f59e0b',
  reports_to: '#8b5cf6',
  default: '#6b7280',
};

const MAX_NODES = 500;

export const GraphViewer: React.FC<GraphViewerProps> = ({
  rootEntityId,
  depth = 2,
  width,
  height,
  onNodeClick,
  onNodeHover,
  highlightEntityIds = [],
  className = '',
}) => {
  const fgRef = useRef<any>(null);
  const [zoom, setZoom] = useState(1);
  const [viewport, setViewport] = useState({ x: 0, y: 0, width, height });
  const [entityFilters, setEntityFilters] = useState<EntityType[]>([]);
  const [relationshipFilters, setRelationshipFilters] = useState<string[]>([]);
  const [searchHighlightedIds, setSearchHighlightedIds] = useState<string[]>([]);
  const [isLoading, setIsLoading] = useState(true);

  const { data: graphResponse, isLoading: graphLoading } = useEntityGraph(rootEntityId || '', depth);

  const rawGraphData: GraphData = useMemo(() => {
    if (!graphResponse || !rootEntityId) return { nodes: [], links: [] };
    
    // API returns GraphData with nodes and edges
    const apiNodes: APIGraphNode[] = (graphResponse as any).nodes || [];
    const apiEdges: APIGraphEdge[] = (graphResponse as any).edges || [];
    
    // Convert API nodes to our GraphNode format
    // Note: API nodes are simplified (id, entityType, name), we need to fetch full entities
    // For now, create minimal Entity objects
    const nodes: GraphNode[] = apiNodes.map((apiNode) => {
      const minimalEntity: Entity = {
        id: apiNode.id,
        entityType: apiNode.entityType,
        name: apiNode.name,
        slug: apiNode.name.toLowerCase().replace(/\s+/g, '-'),
        extractionConfidence: 0.8,
        privacyLevel: 'shared' as any,
        createdAt: new Date().toISOString(),
        updatedAt: new Date().toISOString(),
        metadata: {},
      };
      
      return {
        id: apiNode.id,
        entity: minimalEntity,
        val: Math.max(5, Math.min(20, 10)),
        color: NODE_COLORS[apiNode.entityType] || '#6b7280',
        x: apiNode.x,
        y: apiNode.y,
      };
    });

    // Convert API edges to our GraphLink format
    const links: GraphLink[] = apiEdges.map((apiEdge) => {
      const minimalRelationship: Relationship = {
        id: apiEdge.id,
        sourceId: apiEdge.source,
        targetId: apiEdge.target,
        relationType: apiEdge.relationType,
        confidence: apiEdge.confidence,
      };
      
      return {
        source: apiEdge.source,
        target: apiEdge.target,
        relationship: minimalRelationship,
        color: EDGE_COLORS[apiEdge.relationType] || EDGE_COLORS.default,
        dashed: apiEdge.confidence < 0.5,
      };
    });

    return { nodes, links };
  }, [graphResponse, rootEntityId]);

  const graphData = useMemo(() => {
    let filteredNodes = rawGraphData.nodes;
    let filteredLinks = rawGraphData.links;

    if (entityFilters.length > 0) {
      filteredNodes = filteredNodes.filter(n => entityFilters.includes(n.entity.entityType));
      const nodeIds = new Set(filteredNodes.map(n => n.id));
      filteredLinks = filteredLinks.filter(l => 
        nodeIds.has(l.source as string) && nodeIds.has(l.target as string)
      );
    }

    if (relationshipFilters.length > 0) {
      filteredLinks = filteredLinks.filter(l => relationshipFilters.includes(l.relationship.relationType));
    }

    // Limit nodes for performance
    if (filteredNodes.length > MAX_NODES) {
      filteredNodes = filteredNodes.slice(0, MAX_NODES);
      const nodeIds = new Set(filteredNodes.map(n => n.id));
      filteredLinks = filteredLinks.filter(l => 
        nodeIds.has(l.source as string) && nodeIds.has(l.target as string)
      );
    }

    return { nodes: filteredNodes, links: filteredLinks };
  }, [rawGraphData, entityFilters, relationshipFilters]);

  useEffect(() => {
    setIsLoading(graphLoading);
  }, [graphLoading]);

  // Fit to screen on initial load
  useEffect(() => {
    if (!isLoading && fgRef.current && graphData.nodes.length > 0) {
      setTimeout(() => {
        fgRef.current?.zoomToFit(400, 50);
      }, 100);
    }
  }, [isLoading, graphData.nodes.length]);

  const handleZoomIn = useCallback(() => {
    if (fgRef.current) {
      const newZoom = zoom * 1.3;
      fgRef.current.zoom(newZoom);
      setZoom(newZoom);
    }
  }, [zoom]);

  const handleZoomOut = useCallback(() => {
    if (fgRef.current) {
      const newZoom = zoom / 1.3;
      fgRef.current.zoom(newZoom);
      setZoom(newZoom);
    }
  }, [zoom]);

  const handleFit = useCallback(() => {
    fgRef.current?.zoomToFit(400, 50);
  }, []);

  const handleReset = useCallback(() => {
    if (fgRef.current) {
      fgRef.current.centerAt(0, 0);
      fgRef.current.zoom(1);
      setZoom(1);
    }
  }, []);

  const handleNodeClick = useCallback((node: any) => {
    if (onNodeClick && node.entity) {
      onNodeClick(node.entity);
    }
  }, [onNodeClick]);

  const handleNodeHover = useCallback((node: any) => {
    if (onNodeHover) {
      onNodeHover(node?.entity || null);
    }
  }, [onNodeHover]);

  const handleFilterChange = useCallback((filters: { entityTypes: EntityType[]; relationshipTypes: string[] }) => {
    setEntityFilters(filters.entityTypes);
    setRelationshipFilters(filters.relationshipTypes);
  }, []);

  const allHighlightedIds = useMemo(() => {
    const combined = new Set([...highlightEntityIds, ...searchHighlightedIds]);
    return Array.from(combined);
  }, [highlightEntityIds, searchHighlightedIds]);

  const nodeCanvasObject = useCallback((node: any, ctx: CanvasRenderingContext2D, globalScale: number) => {
    const size = node.val * 0.5;
    const isHighlighted = allHighlightedIds.includes(node.id);
    const isSelected = highlightEntityIds.includes(node.id);
    
    // Draw glow effect for highlighted/selected nodes
    if (isHighlighted || isSelected) {
      ctx.beginPath();
      ctx.arc(node.x, node.y, size + 4, 0, 2 * Math.PI);
      ctx.fillStyle = isSelected ? 'rgba(59, 130, 246, 0.3)' : 'rgba(255, 255, 255, 0.2)';
      ctx.fill();
    }

    // Draw node circle
    ctx.beginPath();
    ctx.arc(node.x, node.y, size, 0, 2 * Math.PI);
    ctx.fillStyle = node.color;
    ctx.fill();

    // Draw selection ring
    if (isSelected) {
      ctx.beginPath();
      ctx.arc(node.x, node.y, size + 2, 0, 2 * Math.PI);
      ctx.strokeStyle = '#3b82f6';
      ctx.lineWidth = 2;
      ctx.stroke();
    }

    // Draw label
    const label = node.entity.name || node.entity.id;
    const fontSize = Math.max(8, 12 / globalScale);
    ctx.font = `${isHighlighted ? 'bold' : 'normal'} ${fontSize}px Inter, sans-serif`;
    ctx.textAlign = 'center';
    ctx.textBaseline = 'middle';
    
    // Text shadow for readability
    ctx.fillStyle = 'rgba(0, 0, 0, 0.8)';
    ctx.fillText(label, node.x + 1, node.y + size + fontSize + 1);
    
    ctx.fillStyle = isHighlighted ? '#ffffff' : '#d1d5db';
    ctx.fillText(label, node.x, node.y + size + fontSize);
  }, [allHighlightedIds, highlightEntityIds]);

  const entities = useMemo(() => {
    return graphData.nodes.map(n => n.entity);
  }, [graphData.nodes]);

  const availableEntityTypes = useMemo(() => {
    const types = new Set<EntityType>();
    graphData.nodes.forEach(n => types.add(n.entity.entityType));
    return Array.from(types);
  }, [graphData.nodes]);

  const availableRelationshipTypes = useMemo(() => {
    const types = new Set<string>();
    graphData.links.forEach(l => types.add(l.relationship.relationType));
    return Array.from(types);
  }, [graphData.links]);

  if (isLoading) {
    return (
      <div className={`flex items-center justify-center bg-gray-900 ${className}`} style={{ width, height }}>
        <div className="flex flex-col items-center gap-3">
          <div className="w-8 h-8 border-2 border-blue-500 border-t-transparent rounded-full animate-spin" />
          <span className="text-sm text-gray-400">Loading graph...</span>
        </div>
      </div>
    );
  }

  return (
    <div className={`relative bg-gray-900 overflow-hidden ${className}`} style={{ width, height }}>
      {/* Search */}
      <div className="absolute top-4 left-4 z-10">
        <GraphSearch
          entities={entities}
          onSelect={(entityId) => {
            const node = graphData.nodes.find(n => n.id === entityId);
            if (node && fgRef.current) {
              fgRef.current.centerAt(node.x, node.y, 400);
              fgRef.current.zoom(2);
            }
          }}
          highlightedIds={searchHighlightedIds}
          onHighlightChange={setSearchHighlightedIds}
        />
      </div>

      {/* Main Graph */}
      <ForceGraph2D
        ref={fgRef}
        graphData={graphData}
        width={width}
        height={height}
        backgroundColor="#111827"
        nodeRelSize={6}
        nodeVal="val"
        nodeLabel={(n: any) => n.entity.name || n.entity.id}
        nodeColor="color"
        linkWidth={1}
        linkColor={(l: any) => l.color}
        linkDirectionalArrowLength={6}
        linkDirectionalArrowRelPos={1}
        linkLineDash={(l: any) => l.dashed ? [5, 5] : null}
        linkDirectionalParticles={2}
        linkDirectionalParticleSpeed={0.01}
        linkDirectionalParticleWidth={2}
        onNodeClick={handleNodeClick}
        onNodeHover={handleNodeHover}
        nodeCanvasObject={nodeCanvasObject}
        nodeCanvasObjectMode={() => 'replace'}
        warmupTicks={100}
        cooldownTicks={50}
        d3AlphaDecay={0.02}
        d3VelocityDecay={0.3}
        onZoom={({ k }: { k: number }) => setZoom(k)}
        onZoomEnd={({ transform }: { transform: { x: number; y: number; k: number } }) => {
          // Debounced viewport update
          const timer = setTimeout(() => {
            setViewport({
              x: -transform.x / transform.k,
              y: -transform.y / transform.k,
              width: width / transform.k,
              height: height / transform.k,
            });
          }, 100);
          return () => clearTimeout(timer);
        }}
      />

      {/* Controls */}
      <GraphControls
        onZoomIn={handleZoomIn}
        onZoomOut={handleZoomOut}
        onFit={handleFit}
        onReset={handleReset}
        filters={{
          entityTypes: availableEntityTypes,
          relationshipTypes: availableRelationshipTypes,
        }}
        onFilterChange={handleFilterChange}
      />

      {/* MiniMap */}
      <MiniMap
        graphData={graphData}
        viewport={viewport}
        onViewportChange={(newViewport) => {
          if (fgRef.current) {
            fgRef.current.centerAt(
              newViewport.x + newViewport.width / 2,
              newViewport.y + newViewport.height / 2
            );
          }
        }}
      />

      {/* Node Count Indicator */}
      <div className="absolute bottom-4 left-4 px-3 py-1.5 bg-gray-800/90 rounded-lg text-xs text-gray-400">
        {graphData.nodes.length} nodes · {graphData.links.length} relationships
        {graphData.nodes.length >= MAX_NODES && (
          <span className="ml-2 text-amber-400">(limited)</span>
        )}
      </div>
    </div>
  );
};

export default GraphViewer;
