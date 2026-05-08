import { useState } from 'react';
import { 
  Edit2, 
  Trash2, 
  X, 
  ChevronDown, 
  ChevronUp,
  Link2,
  Calendar,
  Shield,
  AlertTriangle,
  CheckCircle2
} from 'lucide-react';
import { format } from 'date-fns';
import { EntityTypeBadge } from './EntityTypeBadge';
import { useEntityWithRelationships, useDeleteEntity } from '../../hooks/useEntities';
import type { EntityMetadata, PrivacyLevel } from '../../types/knowledge-graph';

export interface EntityDetailProps {
  entityId: string;
  onEdit?: () => void;
  onDelete?: () => void;
  onClose?: () => void;
}

function Skeleton() {
  return (
    <div className="space-y-4 animate-pulse">
      <div className="h-8 bg-gray-800 rounded w-3/4" />
      <div className="h-4 bg-gray-800 rounded w-1/2" />
      <div className="h-32 bg-gray-800 rounded" />
      <div className="h-20 bg-gray-800 rounded" />
    </div>
  );
}

function ConfidenceScore({ score }: { score: number }) {
  let colorClass = 'text-gray-400';
  let bgClass = 'bg-gray-500/20';
  let Icon = AlertTriangle;
  
  if (score >= 0.9) {
    colorClass = 'text-green-400';
    bgClass = 'bg-green-500/20';
    Icon = CheckCircle2;
  } else if (score >= 0.7) {
    colorClass = 'text-yellow-400';
    bgClass = 'bg-yellow-500/20';
  } else if (score >= 0.5) {
    colorClass = 'text-orange-400';
    bgClass = 'bg-orange-500/20';
  } else {
    colorClass = 'text-red-400';
    bgClass = 'bg-red-500/20';
  }

  return (
    <div className={`flex items-center gap-2 px-3 py-2 rounded-lg ${bgClass}`}>
      <Icon size={18} className={colorClass} />
      <div>
        <span className={`text-sm font-medium ${colorClass}`}>
          {Math.round(score * 100)}% Confidence
        </span>
        <div className="h-1.5 w-24 rounded-full bg-gray-700 mt-1">
          <div
            className={`h-full rounded-full ${colorClass.replace('text-', 'bg-')}`}
            style={{ width: `${score * 100}%` }}
          />
        </div>
      </div>
    </div>
  );
}

function PrivacyBadge({ level }: { level: PrivacyLevel }) {
  const configs = {
    [PrivacyLevel.Private]: { color: 'text-red-400 bg-red-500/20', label: 'Private' },
    [PrivacyLevel.Shared]: { color: 'text-yellow-400 bg-yellow-500/20', label: 'Shared' },
    [PrivacyLevel.Public]: { color: 'text-green-400 bg-green-500/20', label: 'Public' },
  };
  const config = configs[level];

  return (
    <div className={`flex items-center gap-1.5 px-2 py-1 rounded text-xs font-medium ${config.color}`}>
      <Shield size={14} />
      {config.label}
    </div>
  );
}

function MetadataSection({ metadata }: { metadata: EntityMetadata }) {
  const [isExpanded, setIsExpanded] = useState(false);
  const hasMetadata = Object.keys(metadata).length > 0;

  if (!hasMetadata) {
    return (
      <div className="text-sm text-gray-500 italic">
        No additional metadata
      </div>
    );
  }

  return (
    <div className="border border-gray-700 rounded-lg overflow-hidden">
      <button
        onClick={() => setIsExpanded(!isExpanded)}
        className="w-full flex items-center justify-between px-4 py-3 bg-gray-800/50 hover:bg-gray-800 transition-colors"
      >
        <span className="font-medium text-gray-300">Metadata</span>
        {isExpanded ? <ChevronUp size={18} className="text-gray-400" /> : <ChevronDown size={18} className="text-gray-400" />}
      </button>
      
      {isExpanded && (
        <div className="p-4 space-y-2 bg-gray-900/50">
          {metadata.role && (
            <div className="flex justify-between text-sm">
              <span className="text-gray-500">Role</span>
              <span className="text-gray-300">{metadata.role}</span>
            </div>
          )}
          {metadata.email && (
            <div className="flex justify-between text-sm">
              <span className="text-gray-500">Email</span>
              <a href={`mailto:${metadata.email}`} className="text-blue-400 hover:underline">{metadata.email}</a>
            </div>
          )}
          {metadata.industry && (
            <div className="flex justify-between text-sm">
              <span className="text-gray-500">Industry</span>
              <span className="text-gray-300">{metadata.industry}</span>
            </div>
          )}
          {metadata.foundedYear && (
            <div className="flex justify-between text-sm">
              <span className="text-gray-500">Founded</span>
              <span className="text-gray-300">{metadata.foundedYear}</span>
            </div>
          )}
          {metadata.coordinates && (
            <div className="flex justify-between text-sm">
              <span className="text-gray-500">Coordinates</span>
              <span className="text-gray-300 font-mono">
                {metadata.coordinates[0].toFixed(4)}, {metadata.coordinates[1].toFixed(4)}
              </span>
            </div>
          )}
          {metadata.aliases && metadata.aliases.length > 0 && (
            <div className="flex justify-between text-sm">
              <span className="text-gray-500">Aliases</span>
              <span className="text-gray-300">{metadata.aliases.join(', ')}</span>
            </div>
          )}
          {metadata.tags && metadata.tags.length > 0 && (
            <div className="pt-2">
              <span className="text-gray-500 text-sm block mb-2">Tags</span>
              <div className="flex flex-wrap gap-1">
                {metadata.tags.map((tag) => (
                  <span key={tag} className="px-2 py-0.5 bg-gray-800 text-gray-400 text-xs rounded">
                    {tag}
                  </span>
                ))}
              </div>
            </div>
          )}
        </div>
      )}
    </div>
  );
}

function RelationshipsSection({ 
  relationships, 
  relatedEntities 
}: { 
  relationships: any[]; 
  relatedEntities: any[];
}) {
  if (relationships.length === 0) {
    return (
      <div className="text-sm text-gray-500 italic">
        No relationships found
      </div>
    );
  }

  const entityMap = new Map(relatedEntities.map(e => [e.id, e]));

  return (
    <div className="space-y-2">
      {relationships.map((rel) => {
        const relatedEntity = entityMap.get(rel.targetId) || entityMap.get(rel.sourceId);
        if (!relatedEntity) return null;

        return (
          <div
            key={rel.id}
            className="flex items-center gap-3 p-3 rounded-lg border border-gray-700 bg-gray-800/30 hover:bg-gray-800 transition-colors"
          >
            <Link2 size={16} className="text-gray-500 flex-shrink-0" />
            <div className="flex-1 min-w-0">
              <div className="flex items-center gap-2">
                <span className="text-sm font-medium text-gray-300 truncate">
                  {relatedEntity.name}
                </span>
                <EntityTypeBadge type={relatedEntity.entityType} size="sm" showLabel={false} />
              </div>
              <div className="flex items-center gap-2 text-xs text-gray-500">
                <span className="text-blue-400">{rel.relationType}</span>
                <span>•</span>
                <span>{Math.round(rel.confidence * 100)}% confidence</span>
              </div>
            </div>
          </div>
        );
      })}
    </div>
  );
}

export function EntityDetail({ entityId, onEdit, onDelete, onClose }: EntityDetailProps) {
  const { data: entity, isLoading, error } = useEntityWithRelationships(entityId);
  const deleteMutation = useDeleteEntity();
  const [showDeleteConfirm, setShowDeleteConfirm] = useState(false);

  const handleDelete = async () => {
    if (!showDeleteConfirm) {
      setShowDeleteConfirm(true);
      return;
    }
    
    try {
      await deleteMutation.mutateAsync(entityId);
      onDelete?.();
    } catch (err) {
      console.error('Failed to delete entity:', err);
    }
  };

  if (isLoading) {
    return (
      <div className="p-6">
        <Skeleton />
      </div>
    );
  }

  if (error || !entity) {
    return (
      <div className="p-6 text-center">
        <AlertTriangle size={48} className="mx-auto text-red-400 mb-4" />
        <h3 className="text-lg font-medium text-gray-200 mb-2">Failed to load entity</h3>
        <p className="text-sm text-gray-500">
          {error instanceof Error ? error.message : 'Unknown error'}
        </p>
      </div>
    );
  }

  return (
    <div className="h-full flex flex-col">
      {/* Header */}
      <div className="flex items-start justify-between p-6 border-b border-gray-700">
        <div className="flex-1 min-w-0">
          <div className="flex items-center gap-3 mb-2">
            <EntityTypeBadge type={entity.entityType} size="md" />
            <PrivacyBadge level={entity.privacyLevel} />
          </div>
          <h2 className="text-2xl font-bold text-gray-100 truncate">{entity.name}</h2>
          {entity.slug && (
            <p className="text-sm text-gray-500 font-mono mt-1">{entity.slug}</p>
          )}
        </div>
        
        <div className="flex items-center gap-2">
          {onEdit && (
            <button
              onClick={onEdit}
              className="p-2 text-gray-400 hover:text-blue-400 hover:bg-blue-500/10 rounded-lg transition-colors"
              title="Edit entity"
            >
              <Edit2 size={20} />
            </button>
          )}
          {onDelete && (
            <button
              onClick={handleDelete}
              className={`p-2 rounded-lg transition-colors ${
                showDeleteConfirm 
                  ? 'text-red-400 bg-red-500/20 hover:bg-red-500/30' 
                  : 'text-gray-400 hover:text-red-400 hover:bg-red-500/10'
              }`}
              title={showDeleteConfirm ? 'Click again to confirm' : 'Delete entity'}
            >
              <Trash2 size={20} />
            </button>
          )}
          {onClose && (
            <button
              onClick={onClose}
              className="p-2 text-gray-400 hover:text-gray-200 hover:bg-gray-800 rounded-lg transition-colors"
              title="Close"
            >
              <X size={20} />
            </button>
          )}
        </div>
      </div>

      {/* Content */}
      <div className="flex-1 overflow-y-auto p-6 space-y-6">
        {/* Confidence Score */}
        <ConfidenceScore score={entity.extractionConfidence} />

        {/* Description */}
        {entity.description && (
          <div>
            <h3 className="text-sm font-medium text-gray-400 uppercase tracking-wide mb-2">
              Description
            </h3>
            <p className="text-gray-300 leading-relaxed">{entity.description}</p>
          </div>
        )}

        {/* Metadata */}
        <div>
          <h3 className="text-sm font-medium text-gray-400 uppercase tracking-wide mb-3">
            Metadata
          </h3>
          <MetadataSection metadata={entity.metadata} />
        </div>

        {/* Relationships */}
        <div>
          <h3 className="text-sm font-medium text-gray-400 uppercase tracking-wide mb-3">
            Relationships
            <span className="ml-2 text-xs text-gray-500 normal-case">
              ({entity.relationships?.length || 0})
            </span>
          </h3>
          <RelationshipsSection 
            relationships={entity.relationships || []}
            relatedEntities={entity.relatedEntities || []}
          />
        </div>

        {/* Timestamps */}
        <div className="pt-4 border-t border-gray-700">
          <div className="flex items-center gap-6 text-sm text-gray-500">
            <div className="flex items-center gap-2">
              <Calendar size={14} />
              <span>Created {format(new Date(entity.createdAt), 'MMM d, yyyy HH:mm')}</span>
            </div>
            <div className="flex items-center gap-2">
              <Calendar size={14} />
              <span>Updated {format(new Date(entity.updatedAt), 'MMM d, yyyy HH:mm')}</span>
            </div>
          </div>
        </div>
      </div>

      {/* Delete Confirmation */}
      {showDeleteConfirm && (
        <div className="p-4 bg-red-500/10 border-t border-red-500/30">
          <div className="flex items-center justify-between">
            <div className="flex items-center gap-2 text-red-400">
              <AlertTriangle size={18} />
              <span className="text-sm font-medium">
                Are you sure? This action cannot be undone.
              </span>
            </div>
            <button
              onClick={() => setShowDeleteConfirm(false)}
              className="text-sm text-gray-400 hover:text-gray-300"
            >
              Cancel
            </button>
          </div>
        </div>
      )}
    </div>
  );
}

export default EntityDetail;
