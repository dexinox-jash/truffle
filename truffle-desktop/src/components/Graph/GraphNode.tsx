import React from 'react';
import { Entity, EntityType } from '../../types/entity';

export interface GraphNodeProps {
  entity: Entity;
  selected?: boolean;
  highlighted?: boolean;
  size?: number;
}

const ENTITY_ICONS: Partial<Record<EntityType, string>> = {
  [EntityType.Person]: '👤',
  [EntityType.Organization]: '🏢',
  [EntityType.Location]: '📍',
  [EntityType.Event]: '📅',
  [EntityType.Product]: '📦',
  [EntityType.Concept]: '💡',
  [EntityType.Decision]: '✓',
  [EntityType.ActionItem]: '⚡',
};

const ENTITY_COLORS: Record<EntityType, string> = {
  [EntityType.Person]: '#3b82f6',
  [EntityType.Organization]: '#10b981',
  [EntityType.Location]: '#f59e0b',
  [EntityType.Event]: '#ef4444',
  [EntityType.Product]: '#8b5cf6',
  [EntityType.Concept]: '#ec4899',
  [EntityType.Decision]: '#06b6d4',
  [EntityType.ActionItem]: '#f97316',
};

export const GraphNode: React.FC<GraphNodeProps> = ({
  entity,
  selected = false,
  highlighted = false,
  size = 20,
}) => {
  const color = ENTITY_COLORS[entity.entityType] || '#6b7280';
  const icon = ENTITY_ICONS[entity.entityType] || '❓';
  const displayName = entity.name || entity.id;

  return (
    <g className="graph-node cursor-pointer">
      {/* Glow effect for highlighted/selected state */}
      {(highlighted || selected) && (
        <circle
          cx={0}
          cy={0}
          r={size + 6}
          fill={selected ? 'rgba(59, 130, 246, 0.2)' : 'rgba(255, 255, 255, 0.1)'}
          className="animate-pulse"
        />
      )}
      
      {/* Main node circle */}
      <circle
        cx={0}
        cy={0}
        r={size}
        fill={color}
        stroke={selected ? '#3b82f6' : 'transparent'}
        strokeWidth={selected ? 3 : 0}
        className="transition-all duration-200"
        style={{
          filter: highlighted ? 'drop-shadow(0 0 8px rgba(255, 255, 255, 0.5))' : 'none',
        }}
      />

      {/* Icon */}
      <text
        x={0}
        y={0}
        textAnchor="middle"
        dominantBaseline="central"
        fontSize={size * 0.6}
        fill="white"
        style={{ pointerEvents: 'none' }}
      >
        {icon}
      </text>

      {/* Entity name label */}
      <text
        x={0}
        y={size + 14}
        textAnchor="middle"
        fontSize={11}
        fontWeight={highlighted ? 'bold' : 'normal'}
        fill={highlighted ? '#ffffff' : '#d1d5db'}
        style={{ pointerEvents: 'none' }}
      >
        {displayName.length > 20 ? `${displayName.slice(0, 18)}...` : displayName}
      </text>

      {/* Type label */}
      <text
        x={0}
        y={size + 26}
        textAnchor="middle"
        fontSize={9}
        fill="#9ca3af"
        style={{ pointerEvents: 'none' }}
      >
        {entity.entityType}
      </text>
    </g>
  );
};

export default GraphNode;
