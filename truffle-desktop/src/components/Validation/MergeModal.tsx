// SPEC v2.0: Merge Modal Component
// Entity merge UI with conflict resolution

import React, { useState, useMemo } from 'react';
import {
  X,
  GitMerge,
  AlertTriangle,
  Check,
  ChevronRight,
  User,
  Building2,
  MapPin,
  Calendar,
  Package,
  Lightbulb,
  Scale,
  CheckSquare,
  Eye,
} from 'lucide-react';
import { useEntity, useMergeEntities } from '../../hooks/useEntities';
import type { DuplicateCandidate, Entity, EntityType } from '../../types/knowledge-graph';

export interface MergeConflict {
  field: string;
  valueA: string;
  valueB: string;
  selected?: 'A' | 'B' | 'custom';
  customValue?: string;
}

export interface MergeModalProps {
  isOpen: boolean;
  candidate: DuplicateCandidate | null;
  onClose: () => void;
  onConfirm: (primaryId: string, conflicts: MergeConflict[]) => void;
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

function EntityPreviewCard({
  entity,
  isPrimary,
  onSelectAsPrimary,
}: {
  entity: Entity;
  isPrimary?: boolean;
  onSelectAsPrimary?: () => void;
}) {
  const Icon = typeIcons[entity.entityType];

  return (
    <div
      onClick={onSelectAsPrimary}
      className={`
        p-4 rounded-lg border-2 transition-all
        ${
          isPrimary
            ? 'border-primary bg-primary/5'
            : 'border-border bg-surface hover:border-border-hover'
        }
        ${onSelectAsPrimary ? 'cursor-pointer' : ''}
      `}
    >
      <div className="flex items-start gap-3">
        <div
          className={`
            p-2 rounded-lg
            ${isPrimary ? 'bg-primary/20' : 'bg-surface-elevated'}
          `}
        >
          <Icon
            className={`w-5 h-5 ${isPrimary ? 'text-primary' : 'text-text-secondary'}`}
          />
        </div>
        <div className="flex-1 min-w-0">
          <div className="flex items-center gap-2">
            <h4 className="font-semibold text-text-primary truncate">{entity.name}</h4>
            {isPrimary && (
              <span className="px-2 py-0.5 rounded-full bg-primary/20 text-primary text-xs font-medium">
                Primary
              </span>
            )}
          </div>
          {entity.description && (
            <p className="text-sm text-text-secondary mt-1 line-clamp-2">
              {entity.description}
            </p>
          )}
          <div className="flex items-center gap-2 mt-2">
            <span className="text-xs px-2 py-0.5 rounded-full bg-surface-elevated text-text-secondary">
              {entity.entityType.replace('_', ' ')}
            </span>
            <span className="text-xs text-text-muted">
              Confidence: {Math.round(entity.extractionConfidence * 100)}%
            </span>
          </div>
        </div>
      </div>
    </div>
  );
}

function ConflictRow({
  conflict,
  onSelect,
}: {
  conflict: MergeConflict;
  onSelect: (selected: 'A' | 'B' | 'custom', customValue?: string) => void;
}) {
  const [customValue, setCustomValue] = useState(conflict.customValue || '');

  return (
    <div className="p-3 rounded-lg bg-surface border border-border">
      <p className="text-sm font-medium text-text-primary mb-3">{conflict.field}</p>

      <div className="space-y-2">
        {/* Option A */}
        <button
          onClick={() => onSelect('A')}
          className={`
            w-full p-2 rounded-md border text-left transition-all
            ${
              conflict.selected === 'A'
                ? 'border-primary bg-primary/5'
                : 'border-border hover:border-border-hover'
            }
          `}
        >
          <div className="flex items-center gap-2">
            <div
              className={`
                w-4 h-4 rounded-full border-2 flex items-center justify-center
                ${conflict.selected === 'A' ? 'border-primary' : 'border-text-muted'}
              `}
            >
              {conflict.selected === 'A' && (
                <div className="w-2 h-2 rounded-full bg-primary" />
              )}
            </div>
            <span className="text-sm text-text-secondary">Entity A:</span>
            <span className="text-sm text-text-primary">{conflict.valueA}</span>
          </div>
        </button>

        {/* Option B */}
        <button
          onClick={() => onSelect('B')}
          className={`
            w-full p-2 rounded-md border text-left transition-all
            ${
              conflict.selected === 'B'
                ? 'border-primary bg-primary/5'
                : 'border-border hover:border-border-hover'
            }
          `}
        >
          <div className="flex items-center gap-2">
            <div
              className={`
                w-4 h-4 rounded-full border-2 flex items-center justify-center
                ${conflict.selected === 'B' ? 'border-primary' : 'border-text-muted'}
              `}
            >
              {conflict.selected === 'B' && (
                <div className="w-2 h-2 rounded-full bg-primary" />
              )}
            </div>
            <span className="text-sm text-text-secondary">Entity B:</span>
            <span className="text-sm text-text-primary">{conflict.valueB}</span>
          </div>
        </button>

        {/* Custom Option */}
        <button
          onClick={() => onSelect('custom', customValue)}
          className={`
            w-full p-2 rounded-md border text-left transition-all
            ${
              conflict.selected === 'custom'
                ? 'border-primary bg-primary/5'
                : 'border-border hover:border-border-hover'
            }
          `}
        >
          <div className="flex items-center gap-2">
            <div
              className={`
                w-4 h-4 rounded-full border-2 flex items-center justify-center
                ${conflict.selected === 'custom' ? 'border-primary' : 'border-text-muted'}
              `}
            >
              {conflict.selected === 'custom' && (
                <div className="w-2 h-2 rounded-full bg-primary" />
              )}
            </div>
            <span className="text-sm text-text-secondary">Custom:</span>
          </div>
          {conflict.selected === 'custom' && (
            <input
              type="text"
              value={customValue}
              onChange={(e) => {
                setCustomValue(e.target.value);
                onSelect('custom', e.target.value);
              }}
              placeholder="Enter custom value..."
              className="mt-2 w-full bg-background border border-border rounded-md px-2 py-1 text-sm"
              onClick={(e) => e.stopPropagation()}
            />
          )}
        </button>
      </div>
    </div>
  );
}

export function MergeModal({
  isOpen,
  candidate,
  onClose,
  onConfirm,
  className = '',
}: MergeModalProps) {
  const [primaryId, setPrimaryId] = useState<string | null>(null);
  const [conflicts, setConflicts] = useState<MergeConflict[]>([]);
  const [showPreview, setShowPreview] = useState(false);
  const [isMerging, setIsMerging] = useState(false);

  const { data: entityA } = useEntity(candidate?.entityAId || '');
  const { data: entityB } = useEntity(candidate?.entityBId || '');
  const { mutate: mergeEntities, isPending } = useMergeEntities();

  // Reset state when candidate changes
  React.useEffect(() => {
    if (candidate) {
      setPrimaryId(candidate.entityAId);
      // Mock conflicts - in real implementation, these would come from API
      setConflicts([
        {
          field: 'Name',
          valueA: entityA?.name || 'Entity A Name',
          valueB: entityB?.name || 'Entity B Name',
        },
        {
          field: 'Description',
          valueA: entityA?.description || 'Description A',
          valueB: entityB?.description || 'Description B',
        },
      ]);
      setShowPreview(false);
    }
  }, [candidate, entityA, entityB]);

  // Update conflicts when entities load
  React.useEffect(() => {
    if (entityA && entityB) {
      const newConflicts: MergeConflict[] = [];

      if (entityA.name !== entityB.name) {
        newConflicts.push({
          field: 'Name',
          valueA: entityA.name,
          valueB: entityB.name,
        });
      }

      if (entityA.description !== entityB.description) {
        newConflicts.push({
          field: 'Description',
          valueA: entityA.description || '(empty)',
          valueB: entityB.description || '(empty)',
        });
      }

      setConflicts(newConflicts);
    }
  }, [entityA, entityB]);

  if (!isOpen || !candidate) return null;

  const handleConflictSelect = (
    index: number,
    selected: 'A' | 'B' | 'custom',
    customValue?: string
  ) => {
    setConflicts((prev) =>
      prev.map((c, i) => (i === index ? { ...c, selected, customValue } : c))
    );
  };

  const resolvedConflicts = conflicts.filter((c) => c.selected);
  const allConflictsResolved = resolvedConflicts.length === conflicts.length;

  const handleConfirm = () => {
    if (!primaryId || !allConflictsResolved) return;

    setIsMerging(true);

    mergeEntities(
      { sourceId: primaryId, targetId: primaryId === candidate.entityAId ? candidate.entityBId : candidate.entityAId },
      {
        onSuccess: () => {
          onConfirm(primaryId, conflicts);
          setIsMerging(false);
          onClose();
        },
        onError: () => {
          setIsMerging(false);
        },
      }
    );
  };

  // Preview merged entity
  const previewEntity = useMemo(() => {
    if (!entityA || !entityB) return null;

    const primary = primaryId === entityA.id ? entityA : entityB;
    const merged: Partial<Entity> = {
      ...primary,
      name:
        conflicts.find((c) => c.field === 'Name')?.selected === 'B'
          ? entityB.name
          : conflicts.find((c) => c.field === 'Name')?.selected === 'custom'
          ? conflicts.find((c) => c.field === 'Name')?.customValue
          : entityA.name,
    };

    return merged;
  }, [entityA, entityB, primaryId, conflicts]);

  return (
    <div className="fixed inset-0 z-modal flex items-center justify-center">
      {/* Backdrop */}
      <div
        className="absolute inset-0 bg-background/80 backdrop-blur-sm"
        onClick={onClose}
      />

      {/* Modal */}
      <div
        className={`
          relative w-full max-w-2xl max-h-[90vh] overflow-auto
          bg-surface-elevated rounded-xl border border-border shadow-glow-lg
          ${className}
        `}
      >
        {/* Header */}
        <div className="flex items-center justify-between p-4 border-b border-border">
          <div className="flex items-center gap-3">
            <div className="p-2 rounded-lg bg-primary/20">
              <GitMerge className="w-5 h-5 text-primary" />
            </div>
            <div>
              <h2 className="text-lg font-semibold text-text-primary">
                Merge Entities
              </h2>
              <p className="text-sm text-text-secondary">
                Combine duplicate entities into one
              </p>
            </div>
          </div>
          <button
            onClick={onClose}
            className="p-2 rounded-md hover:bg-surface transition-colors"
          >
            <X className="w-5 h-5 text-text-secondary" />
          </button>
        </div>

        {/* Content */}
        <div className="p-4 space-y-4">
          {/* Entity Selection */}
          <div>
            <h3 className="text-sm font-medium text-text-secondary mb-3">
              Select Primary Entity
            </h3>
            <div className="grid grid-cols-2 gap-3">
              {entityA && (
                <EntityPreviewCard
                  entity={entityA}
                  isPrimary={primaryId === entityA.id}
                  onSelectAsPrimary={() => setPrimaryId(entityA.id)}
                />
              )}
              {entityB && (
                <EntityPreviewCard
                  entity={entityB}
                  isPrimary={primaryId === entityB.id}
                  onSelectAsPrimary={() => setPrimaryId(entityB.id)}
                />
              )}
            </div>
          </div>

          {/* Conflicts */}
          {conflicts.length > 0 && (
            <div>
              <div className="flex items-center justify-between mb-3">
                <h3 className="text-sm font-medium text-text-secondary">
                  Resolve Conflicts
                </h3>
                <span className="text-xs text-text-muted">
                  {resolvedConflicts.length} of {conflicts.length} resolved
                </span>
              </div>
              <div className="space-y-2">
                {conflicts.map((conflict, index) => (
                  <ConflictRow
                    key={conflict.field}
                    conflict={conflict}
                    onSelect={(selected, customValue) =>
                      handleConflictSelect(index, selected, customValue)
                    }
                  />
                ))}
              </div>
            </div>
          )}

          {/* Preview Toggle */}
          {allConflictsResolved && (
            <button
              onClick={() => setShowPreview(!showPreview)}
              className="
                flex items-center gap-2 text-sm text-primary hover:text-primary-hover
                transition-colors
              "
            >
              <Eye className="w-4 h-4" />
              {showPreview ? 'Hide Preview' : 'Show Preview'}
              <ChevronRight
                className={`w-4 h-4 transition-transform ${
                  showPreview ? 'rotate-90' : ''
                }`}
              />
            </button>
          )}

          {/* Preview */}
          {showPreview && previewEntity && (
            <div className="p-4 rounded-lg bg-success/5 border border-success/30">
              <div className="flex items-center gap-2 mb-3">
                <Eye className="w-4 h-4 text-success" />
                <span className="text-sm font-medium text-success">
                  Merged Result Preview
                </span>
              </div>
              <div className="space-y-2 text-sm">
                <p>
                  <span className="text-text-secondary">Name:</span>{' '}
                  <span className="text-text-primary">{previewEntity.name}</span>
                </p>
                {previewEntity.description && (
                  <p>
                    <span className="text-text-secondary">Description:</span>{' '}
                    <span className="text-text-primary">
                      {previewEntity.description}
                    </span>
                  </p>
                )}
              </div>
            </div>
          )}
        </div>

        {/* Footer */}
        <div className="flex items-center justify-between p-4 border-t border-border">
          <div className="flex items-center gap-2 text-sm text-text-muted">
            <AlertTriangle className="w-4 h-4" />
            <span>This action cannot be undone</span>
          </div>
          <div className="flex items-center gap-3">
            <button
              onClick={onClose}
              className="px-4 py-2 text-sm text-text-secondary hover:text-text-primary transition-colors"
            >
              Cancel
            </button>
            <button
              onClick={handleConfirm}
              disabled={!primaryId || !allConflictsResolved || isMerging || isPending}
              className="
                flex items-center gap-2 px-4 py-2
                bg-primary text-background font-medium rounded-md
                hover:bg-primary-hover
                disabled:opacity-50 disabled:cursor-not-allowed
                transition-colors
              "
            >
              {isMerging || isPending ? (
                <>
                  <div className="w-4 h-4 border-2 border-background/30 border-t-background rounded-full animate-spin" />
                  Merging...
                </>
              ) : (
                <>
                  <Check className="w-4 h-4" />
                  Confirm Merge
                </>
              )}
            </button>
          </div>
        </div>
      </div>
    </div>
  );
}

export default MergeModal;
