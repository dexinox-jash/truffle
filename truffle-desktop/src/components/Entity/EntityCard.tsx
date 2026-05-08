import { formatDistanceToNow } from 'date-fns';
import { EntityTypeBadge } from './EntityTypeBadge';
import type { EntitySummary } from '../../types/knowledge-graph';

export interface EntityCardProps {
  entity: EntitySummary;
  onClick?: () => void;
  selected?: boolean;
  compact?: boolean;
}

function ConfidenceIndicator({ confidence }: { confidence: number }) {
  let colorClass = 'bg-gray-500';
  if (confidence >= 0.9) {
    colorClass = 'bg-green-500';
  } else if (confidence >= 0.7) {
    colorClass = 'bg-yellow-500';
  } else if (confidence >= 0.5) {
    colorClass = 'bg-orange-500';
  } else {
    colorClass = 'bg-red-500';
  }

  return (
    <div className="flex items-center gap-1.5" title={`Confidence: ${Math.round(confidence * 100)}%`}>
      <div className="h-1.5 w-16 rounded-full bg-gray-700 overflow-hidden">
        <div
          className={`h-full rounded-full ${colorClass} transition-all duration-300`}
          style={{ width: `${confidence * 100}%` }}
        />
      </div>
      <span className="text-xs text-gray-500">{Math.round(confidence * 100)}%</span>
    </div>
  );
}

export function EntityCard({ entity, onClick, selected = false, compact = false }: EntityCardProps) {
  const descriptionPreview = entity.description
    ? entity.description.length > 100
      ? `${entity.description.slice(0, 100)}...`
      : entity.description
    : null;

  if (compact) {
    return (
      <div
        onClick={onClick}
        className={`
          p-3 rounded-lg border cursor-pointer transition-all duration-200
          ${selected 
            ? 'border-blue-500 bg-blue-500/10 shadow-lg shadow-blue-500/10' 
            : 'border-gray-700 bg-gray-800/50 hover:border-gray-600 hover:bg-gray-800 hover:shadow-lg'
          }
        `}
      >
        <div className="flex items-start justify-between gap-2">
          <div className="min-w-0 flex-1">
            <div className="flex items-center gap-2 mb-1">
              <EntityTypeBadge type={entity.entityType} size="sm" showLabel={false} />
              <h4 className="font-medium text-gray-200 truncate text-sm">{entity.name}</h4>
            </div>
            {descriptionPreview && (
              <p className="text-xs text-gray-500 line-clamp-1">{descriptionPreview}</p>
            )}
          </div>
          <ConfidenceIndicator confidence={entity.extractionConfidence} />
        </div>
      </div>
    );
  }

  return (
    <div
      onClick={onClick}
      className={`
        p-4 rounded-lg border cursor-pointer transition-all duration-200
        ${selected 
          ? 'border-blue-500 bg-blue-500/10 shadow-lg shadow-blue-500/10' 
          : 'border-gray-700 bg-gray-800/50 hover:border-gray-600 hover:bg-gray-800 hover:shadow-lg'
        }
      `}
    >
      <div className="flex items-start justify-between gap-3 mb-2">
        <div className="flex items-center gap-2 min-w-0">
          <EntityTypeBadge type={entity.entityType} size="sm" />
        </div>
        <ConfidenceIndicator confidence={entity.extractionConfidence} />
      </div>
      
      <h4 className="font-semibold text-gray-100 mb-1 truncate">{entity.name}</h4>
      
      {descriptionPreview && (
        <p className="text-sm text-gray-400 line-clamp-2 mb-3">{descriptionPreview}</p>
      )}
    </div>
  );
}

export default EntityCard;
