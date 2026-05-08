import React from 'react';
import { Relationship } from '../../types/entity';
import { Entity } from '../../types/entity';

export interface GraphEdgeProps {
  relationship: Relationship;
  source: Entity;
  target: Entity;
  highlighted?: boolean;
}

const RELATIONSHIP_COLORS: Record<string, string> = {
  works_for: '#3b82f6',
  located_in: '#f59e0b',
  participated_in: '#ef4444',
  mentions: '#8b5cf6',
  related_to: '#6b7280',
  knows: '#10b981',
  created: '#ec4899',
  authored: '#06b6d4',
  belongs_to: '#f97316',
  works_at: '#3b82f6',
  part_of: '#10b981',
  leads: '#f59e0b',
  reports_to: '#8b5cf6',
  default: '#6b7280',
};

export const GraphEdge: React.FC<GraphEdgeProps> = ({
  relationship,
  source,
  target,
  highlighted = false,
}) => {
  const color = RELATIONSHIP_COLORS[relationship.relationType] || RELATIONSHIP_COLORS.default;
  const isLowConfidence = relationship.confidence < 0.5;
  const strokeDasharray = isLowConfidence ? '5,5' : 'none';
  const strokeWidth = highlighted ? 2 : 1;
  const opacity = highlighted ? 1 : 0.5;

  // Calculate arrow position
  const dx = (target.x || 0) - (source.x || 0);
  const dy = (target.y || 0) - (source.y || 0);
  const angle = Math.atan2(dy, dx);
  const arrowLength = 10;
  const targetRadius = 20;

  const arrowX = (target.x || 0) - Math.cos(angle) * targetRadius;
  const arrowY = (target.y || 0) - Math.sin(angle) * targetRadius;

  const arrowPoint1X = arrowX - arrowLength * Math.cos(angle - Math.PI / 6);
  const arrowPoint1Y = arrowY - arrowLength * Math.sin(angle - Math.PI / 6);
  const arrowPoint2X = arrowX - arrowLength * Math.cos(angle + Math.PI / 6);
  const arrowPoint2Y = arrowY - arrowLength * Math.sin(angle + Math.PI / 6);

  // Label position (middle of the line)
  const labelX = ((source.x || 0) + (target.x || 0)) / 2;
  const labelY = ((source.y || 0) + (target.y || 0)) / 2 - 10;

  const formattedType = relationship.relationType.replace(/_/g, ' ');

  return (
    <g className="graph-edge" opacity={opacity}>
      {/* Main line */}
      <line
        x1={source.x || 0}
        y1={source.y || 0}
        x2={target.x || 0}
        y2={target.y || 0}
        stroke={color}
        strokeWidth={strokeWidth}
        strokeDasharray={strokeDasharray}
      />

      {/* Arrowhead */}
      <polygon
        points={`${arrowX},${arrowY} ${arrowPoint1X},${arrowPoint1Y} ${arrowPoint2X},${arrowPoint2Y}`}
        fill={color}
      />

      {/* Relationship label background */}
      <rect
        x={labelX - 40}
        y={labelY - 8}
        width={80}
        height={16}
        rx={4}
        fill="#1f2937"
        opacity={0.8}
      />

      {/* Relationship label */}
      <text
        x={labelX}
        y={labelY}
        textAnchor="middle"
        dominantBaseline="central"
        fontSize={9}
        fill={color}
        fontWeight={highlighted ? 'bold' : 'normal'}
        style={{ pointerEvents: 'none' }}
      >
        {formattedType}
        {relationship.confidence && (
          <tspan fontSize={7} fill="#9ca3af">
            {' '}({Math.round(relationship.confidence * 100)}%)
          </tspan>
        )}
      </text>

      {/* Highlight glow effect */}
      {highlighted && (
        <line
          x1={source.x || 0}
          y1={source.y || 0}
          x2={target.x || 0}
          y2={target.y || 0}
          stroke={color}
          strokeWidth={strokeWidth + 4}
          opacity={0.3}
          strokeLinecap="round"
        />
      )}
    </g>
  );
};

export default GraphEdge;
