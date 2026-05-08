// SPEC v2.0: Duplicate List Component
// List of duplicate candidates with merge actions

import React, { useState, useMemo } from 'react';
import {
  Copy,
  GitMerge,
  X,
  ChevronRight,
  User,
  Building2,
  MapPin,
  Calendar,
  Package,
  Lightbulb,
  Scale,
  CheckSquare,
  Gauge,
} from 'lucide-react';
import {
  useDuplicateCandidates,
  useEntityDuplicates,
  useMarkNotDuplicate,
} from '../../hooks/useValidation';
import { useEntity } from '../../hooks/useEntities';
import type { DuplicateCandidate, EntityType } from '../../types/knowledge-graph';

export interface DuplicateListProps {
  threshold?: number;
  onSelect?: (candidate: DuplicateCandidate) => void;
  onMerge?: (primaryId: string, secondaryId: string) => void;
  onDismiss?: (entityAId: string, entityBId: string) => void;
  className?: string;
}

const typeIcons: Record<EntityType, typeof User> = {
  person: User,
  organization: Building2,
  location: MapPin,
  event: Calendar,
  product: Package,
  concept: Lightbulb,
  decision: Scale,
  action_item: CheckSquare,
};

function SimilarityBar({
  label,
  value,
  color,
}: {
  label: string;
  value: number;
  color: string;
}) {
  return (
    <div className="flex items-center gap-2 text-xs">
      <span className="w-20 text-text-muted">{label}</span>
      <div className="flex-1 h-1.5 bg-surface-elevated rounded-full overflow-hidden">
        <div
          className={`h-full rounded-full ${color}`}
          style={{ width: `${value * 100}%` }}
        />
      </div>
      <span className="w-10 text-right text-text-secondary">
        {Math.round(value * 100)}%
      </span>
    </div>
  );
}

function EntityCard({
  entityId,
  isPrimary,
}: {
  entityId: string;
  isPrimary?: boolean;
}) {
  const { data: entity } = useEntity(entityId);
  const Icon = entity ? typeIcons[entity.entityType] : User;

  if (!entity) {
    return (
      <div className="p-3 rounded-lg bg-surface border border-border">
        <div className="animate-pulse h-4 bg-surface-elevated rounded w-24" />
      </div>
    );
  }

  return (
    <div
      className={`
        p-3 rounded-lg border transition-all
        ${isPrimary ? 'border-primary bg-primary/5' : 'border-border bg-surface'}
      `}
    >
      <div className="flex items-center gap-2">
        <Icon className="w-4 h-4 text-text-muted" />
        <span className="font-medium text-text-primary truncate">{entity.name}</span>
      </div>
      {entity.description && (
        <p className="text-xs text-text-secondary mt-1 line-clamp-2">
          {entity.description}
        </p>
      )}
      <div className="flex items-center gap-2 mt-2">
        <span className="text-xs px-1.5 py-0.5 rounded bg-surface-elevated text-text-secondary">
          {entity.entityType.replace('_', ' ')}
        </span>
        {isPrimary && (
          <span className="text-xs px-1.5 py-0.5 rounded bg-primary/20 text-primary">
            Primary
          </span>
        )}
      </div>
    </div>
  );
}

function DuplicateCandidateItem({
  candidate,
  onSelect,
  onMerge,
  onDismiss,
}: {
  candidate: DuplicateCandidate;
  onSelect?: (c: DuplicateCandidate) => void;
  onMerge?: (primaryId: string, secondaryId: string) => void;
  onDismiss?: (entityAId: string, entityBId: string) => void;
}) {
  const [isDismissing, setIsDismissing] = useState(false);
  const { mutate: markNotDuplicate, isPending } = useMarkNotDuplicate();

  const handleDismiss = (e: React.MouseEvent) => {
    e.stopPropagation();
    markNotDuplicate(
      { entityAId: candidate.entityAId, entityBId: candidate.entityBId },
      {
        onSuccess: () => {
          onDismiss?.(candidate.entityAId, candidate.entityBId);
        },
      }
    );
  };

  const handleMerge = (e: React.MouseEvent) => {
    e.stopPropagation();
    // Primary is typically the one with higher confidence or older
    onMerge?.(candidate.entityAId, candidate.entityBId);
  };

  // Determine similarity color
  const getSimilarityColor = (score: number) => {
    if (score >= 0.9) return 'bg-error';
    if (score >= 0.8) return 'bg-warning';
    if (score >= 0.7) return 'bg-primary';
    return 'bg-text-muted';
  };

  return (
    <div
      onClick={() => onSelect?.(candidate)}
      className={`
        p-4 rounded-lg border border-border bg-surface
        ${onSelect ? 'cursor-pointer hover:border-border-hover hover:bg-surface-hover' : ''}
        transition-all duration-200
      `}
    >
      {/* Header with similarity score */}
      <div className="flex items-center justify-between mb-4">
        <div className="flex items-center gap-3">
          <div
            className={`
              px-2 py-1 rounded-md text-sm font-semibold
              ${
                candidate.similarityScore >= 0.9
                  ? 'bg-error/20 text-error'
                  : candidate.similarityScore >= 0.8
                  ? 'bg-warning/20 text-warning'
                  : 'bg-primary/20 text-primary'
              }
            `}
          >
            {Math.round(candidate.similarityScore * 100)}% Match
          </div>
          <span className="text-xs text-text-muted">
            {new Date(candidate.detectedAt).toLocaleDateString()}
          </span>
        </div>
        <div className="flex items-center gap-2">
          <button
            onClick={handleDismiss}
            disabled={isPending}
            className="
              p-2 rounded-md text-text-secondary
              hover:bg-error/10 hover:text-error
              transition-colors
              disabled:opacity-50
            "
            title="Not a duplicate"
          >
            <X className="w-4 h-4" />
          </button>
          <button
            onClick={handleMerge}
            className="
              flex items-center gap-1 px-3 py-1.5 rounded-md
              bg-primary text-background text-sm font-medium
              hover:bg-primary-hover
              transition-colors
            "
          >
            <GitMerge className="w-3.5 h-3.5" />
            Merge
          </button>
        </div>
      </div>

      {/* Entity Cards */}
      <div className="grid grid-cols-2 gap-3 mb-4">
        <EntityCard entityId={candidate.entityAId} isPrimary />
        <EntityCard entityId={candidate.entityBId} />
      </div>

      {/* Similarity Breakdown */}
      <div className="space-y-1.5 p-3 rounded-md bg-surface-elevated">
        <SimilarityBar
          label="Name"
          value={candidate.nameSimilarity}
          color={getSimilarityColor(candidate.nameSimilarity)}
        />
        <SimilarityBar
          label="Embedding"
          value={candidate.embeddingSimilarity}
          color={getSimilarityColor(candidate.embeddingSimilarity)}
        />
        <SimilarityBar
          label="Relations"
          value={candidate.relationshipOverlap}
          color={getSimilarityColor(candidate.relationshipOverlap)}
        />
      </div>
    </div>
  );
}

export function DuplicateList({
  threshold = 0.7,
  onSelect,
  onMerge,
  onDismiss,
  className = '',
}: DuplicateListProps) {
  const [minThreshold, setMinThreshold] = useState(threshold);

  // Fetch all duplicate candidates
  const { data: allCandidates, isLoading } = useDuplicateCandidates(minThreshold);

  // Filter by threshold
  const filteredCandidates = useMemo(() => {
    if (!allCandidates) return [];
    return allCandidates.filter((c) => c.similarityScore >= minThreshold);
  }, [allCandidates, minThreshold]);

  if (isLoading) {
    return (
      <div className={`space-y-3 ${className}`}>
        {[...Array(3)].map((_, i) => (
          <div
            key={i}
            className="h-48 rounded-lg bg-surface border border-border animate-pulse"
          />
        ))}
      </div>
    );
  }

  return (
    <div className={`space-y-4 ${className}`}>
      {/* Threshold Filter */}
      <div className="flex items-center gap-4 p-3 rounded-lg bg-surface border border-border">
        <div className="flex items-center gap-2">
          <Gauge className="w-4 h-4 text-text-muted" />
          <span className="text-sm text-text-secondary">Similarity Threshold</span>
        </div>
        <input
          type="range"
          min={0.5}
          max={0.95}
          step={0.05}
          value={minThreshold}
          onChange={(e) => setMinThreshold(parseFloat(e.target.value))}
          className="flex-1"
        />
        <span className="text-sm font-medium text-text-primary w-12 text-right">
          {Math.round(minThreshold * 100)}%
        </span>
      </div>

      {/* Count */}
      <div className="text-sm text-text-secondary">
        Showing {filteredCandidates.length} duplicate candidate
        {filteredCandidates.length !== 1 ? 's' : ''}
      </div>

      {/* List */}
      {filteredCandidates.length > 0 ? (
        <div className="space-y-3">
          {filteredCandidates.map((candidate) => (
            <DuplicateCandidateItem
              key={`${candidate.entityAId}-${candidate.entityBId}`}
              candidate={candidate}
              onSelect={onSelect}
              onMerge={onMerge}
              onDismiss={onDismiss}
            />
          ))}
        </div>
      ) : (
        /* Empty State */
        <div className="p-8 text-center rounded-lg bg-surface border border-border">
          <div className="inline-flex items-center justify-center w-12 h-12 rounded-full bg-success/10 mb-3">
            <Copy className="w-6 h-6 text-success" />
          </div>
          <h4 className="text-sm font-medium text-text-primary">
            No Duplicates Found
          </h4>
          <p className="text-sm text-text-secondary mt-1">
            No duplicate candidates above {Math.round(minThreshold * 100)}% similarity
          </p>
        </div>
      )}
    </div>
  );
}

export default DuplicateList;
