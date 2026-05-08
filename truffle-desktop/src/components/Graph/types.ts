import { Entity, Relationship } from '../../types/knowledge-graph';

export interface GraphNode {
  id: string;
  entity: Entity;
  val: number;
  color: string;
  x?: number;
  y?: number;
  fx?: number;
  fy?: number;
}

export interface GraphLink {
  source: string | GraphNode;
  target: string | GraphNode;
  relationship: Relationship;
  color: string;
  dashed?: boolean;
}

export interface GraphData {
  nodes: GraphNode[];
  links: GraphLink[];
}

export interface ViewportState {
  x: number;
  y: number;
  width: number;
  height: number;
  zoom: number;
}

export interface GraphFilterState {
  entityTypes: string[];
  relationshipTypes: string[];
  confidenceThreshold: number;
  dateRange?: {
    start?: Date;
    end?: Date;
  };
}

export interface GraphLayoutConfig {
  width: number;
  height: number;
  nodeCharge: number;
  linkDistance: number;
  collisionRadius: number;
  centerForce: number;
  warmupTicks: number;
  cooldownTicks: number;
  alphaDecay: number;
  velocityDecay: number;
}

export const DEFAULT_LAYOUT_CONFIG: GraphLayoutConfig = {
  width: 800,
  height: 600,
  nodeCharge: -100,
  linkDistance: 100,
  collisionRadius: 20,
  centerForce: 0.05,
  warmupTicks: 100,
  cooldownTicks: 50,
  alphaDecay: 0.02,
  velocityDecay: 0.3,
};
