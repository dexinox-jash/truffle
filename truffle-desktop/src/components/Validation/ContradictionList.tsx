// SPEC v2.0: Contradiction List Component
// List of contradictions with filtering and sorting

import React, { useState, useMemo } from 'react';
import {
  AlertTriangle,
  AlertCircle,
  CheckCircle2,
  Filter,
  ArrowUpDown,
  Calendar,
  MoreHorizontal,
  Check,
  X,
} from 'lucide-react';
import {
  useContradictions,
  useEntityContradictions,
  useResolveContradiction,
} from '../../hooks/useValidation';
import type { Contradiction, ContradictionSeverity } from '../../types/knowledge-graph';

export interface ContradictionListProps {
  entityId?: string;
  severity?: 'low' | 'medium' | 'high' | 'critical';
  onSelect?: (contradiction: Contradiction) => void;
  onResolve?: (id: string) => void;
  className?: string;
}

type SortField = 'detectedAt' | 'severity';
type SortOrder = 'asc' | 'desc';

const severityConfig: Record<
  ContradictionSeverity,
  { icon: typeof AlertTriangle; color: string; label: string; priority: number }
> = {
  critical: {
    icon: AlertTriangle,
    color: 'text-error bg-error/10 border-error/30',
    label: 'Critical',
    priority: 4,
  },
  high: {
    icon: AlertTriangle,
    color: 'text-orange-400 bg-orange-500/10 border-orange-500/30',
    label: 'High',
    priority: 3,
  },
  medium: {
    icon: AlertCircle,
    color: 'text-warning bg-warning/10 border-warning/30',
    label: 'Medium',
    priority: 2,
  },
  low: {
    icon: AlertCircle,
    color: 'text-text-muted bg-text-muted/10 border-text-muted/30',
    label: 'Low',
    priority: 1,
  },
};

function ContradictionItem({
  contradiction,
  onSelect,
  onResolve,
  isResolving,
}: {
  contradiction: Contradiction;
  onSelect?: (c: Contradiction) => void;
  onResolve?: (id: string) => void;
  isResolving: boolean;
}) {
  const config = severityConfig[contradiction.severity];
  const Icon = config.icon;

  const formattedDate = new Date(contradiction.detectedAt).toLocaleDateString('en-US', {
    month: 'short',
    day: 'numeric',
    hour: '2-digit',
    minute: '2-digit',
  });

  return (
    <div
      onClick={() => onSelect?.(contradiction)}
      className={`
        group p-4 rounded-lg border transition-all duration-200
        ${config.color}
        ${contradiction.isResolved ? 'opacity-60' : 'bg-surface'}
        ${onSelect ? 'cursor-pointer hover:border-border-hover hover:bg-surface-hover' : ''}
      `}
    >
      <div className="flex items-start gap-3">
        {/* Severity Icon */}
        <div className="flex-shrink-0 mt-0.5">
          <Icon className="w-5 h-5" />
        </div>

        {/* Content */}
        <div className="flex-1 min-w-0">
          <div className="flex items-start justify-between gap-2">
            <div>
              <p className="font-medium text-text-primary line-clamp-2">
                {contradiction.description}
              </p>
              <div className="flex items-center gap-3 mt-2 text-xs text-text-secondary">
                <span className="flex items-center gap-1">
                  <Calendar className="w-3 h-3" />
                  {formattedDate}
                </span>
                <span className="px-1.5 py-0.5 rounded bg-background/50">
                  {contradiction.contradictionType.replace('_', ' ')}
                </span>
                {contradiction.isResolved && (
                  <span className="flex items-center gap-1 text-success">
                    <CheckCircle2 className="w-3 h-3" />
                    Resolved
                  </span>
                )}
              </div>
            </div>

            {/* Actions */}
            {!contradiction.isResolved && onResolve && (
              <button
                onClick={(e) => {
                  e.stopPropagation();
                  onResolve(contradiction.id);
                }}
                disabled={isResolving}
                className="
                  flex-shrink-0 p-2 rounded-md
                  bg-success/10 text-success
                  hover:bg-success/20
                  transition-colors
                  disabled:opacity-50 disabled:cursor-not-allowed
                "
                title="Mark as resolved"
              >
                <Check className="w-4 h-4" />
              </button>
            )}
          </div>
        </div>
      </div>
    </div>
  );
}

export function ContradictionList({
  entityId,
  severity: initialSeverity,
  onSelect,
  onResolve,
  className = '',
}: ContradictionListProps) {
  // State
  const [filterSeverity, setFilterSeverity] = useState<ContradictionSeverity | 'all'>(
    initialSeverity || 'all'
  );
  const [showResolved, setShowResolved] = useState(false);
  const [sortField, setSortField] = useState<SortField>('detectedAt');
  const [sortOrder, setSortOrder] = useState<SortOrder>('desc');

  // Data fetching
  const {
    data: entityContradictions,
    isLoading: entityLoading,
  } = useEntityContradictions(entityId || '');

  const { data: allContradictions, isLoading: allLoading } = useContradictions(showResolved);

  const { mutate: resolveContradiction, isPending: isResolving } = useResolveContradiction();

  // Use entity-specific or all contradictions
  const rawContradictions = entityId ? entityContradictions : allContradictions;

  // Filter and sort
  const filteredContradictions = useMemo(() => {
    if (!rawContradictions) return [];

    let result = [...rawContradictions];

    // Filter by severity
    if (filterSeverity !== 'all') {
      result = result.filter((c) => c.severity === filterSeverity);
    }

    // Filter by resolved status
    if (!showResolved) {
      result = result.filter((c) => !c.isResolved);
    }

    // Sort
    result.sort((a, b) => {
      let comparison = 0;

      if (sortField === 'detectedAt') {
        comparison = new Date(a.detectedAt).getTime() - new Date(b.detectedAt).getTime();
      } else if (sortField === 'severity') {
        comparison = severityConfig[a.severity].priority - severityConfig[b.severity].priority;
      }

      return sortOrder === 'asc' ? comparison : -comparison;
    });

    return result;
  }, [rawContradictions, filterSeverity, showResolved, sortField, sortOrder]);

  // Handle resolve
  const handleResolve = (id: string) => {
    resolveContradiction(
      { contradictionId: id, resolution: 'resolved' },
      {
        onSuccess: () => {
          onResolve?.(id);
        },
      }
    );
  };

  // Toggle sort
  const toggleSort = (field: SortField) => {
    if (sortField === field) {
      setSortOrder(sortOrder === 'asc' ? 'desc' : 'asc');
    } else {
      setSortField(field);
      setSortOrder('desc');
    }
  };

  if (entityLoading || allLoading) {
    return (
      <div className={`space-y-3 ${className}`}>
        {[...Array(3)].map((_, i) => (
          <div
            key={i}
            className="h-20 rounded-lg bg-surface border border-border animate-pulse"
          />
        ))}
      </div>
    );
  }

  return (
    <div className={`space-y-4 ${className}`}>
      {/* Filters */}
      <div className="flex flex-wrap items-center gap-3">
        {/* Severity Filter */}
        <div className="flex items-center gap-2">
          <Filter className="w-4 h-4 text-text-muted" />
          <select
            value={filterSeverity}
            onChange={(e) => setFilterSeverity(e.target.value as ContradictionSeverity | 'all')}
            className="bg-surface border border-border rounded-md px-2 py-1 text-sm"
          >
            <option value="all">All Severities</option>
            <option value="critical">Critical</option>
            <option value="high">High</option>
            <option value="medium">Medium</option>
            <option value="low">Low</option>
          </select>
        </div>

        {/* Show Resolved Toggle */}
        <label className="flex items-center gap-2 text-sm text-text-secondary cursor-pointer">
          <input
            type="checkbox"
            checked={showResolved}
            onChange={(e) => setShowResolved(e.target.checked)}
            className="rounded border-border bg-surface"
          />
          Show Resolved
        </label>

        {/* Sort */}
        <div className="flex items-center gap-2 ml-auto">
          <ArrowUpDown className="w-4 h-4 text-text-muted" />
          <button
            onClick={() => toggleSort('detectedAt')}
            className={`
              px-2 py-1 text-sm rounded-md transition-colors
              ${sortField === 'detectedAt' ? 'bg-primary/20 text-primary' : 'text-text-secondary hover:bg-surface'}
            `}
          >
            Date {sortField === 'detectedAt' && (sortOrder === 'asc' ? '↑' : '↓')}
          </button>
          <button
            onClick={() => toggleSort('severity')}
            className={`
              px-2 py-1 text-sm rounded-md transition-colors
              ${sortField === 'severity' ? 'bg-primary/20 text-primary' : 'text-text-secondary hover:bg-surface'}
            `}
          >
            Severity {sortField === 'severity' && (sortOrder === 'asc' ? '↑' : '↓')}
          </button>
        </div>
      </div>

      {/* Contradiction Count */}
      <div className="text-sm text-text-secondary">
        Showing {filteredContradictions.length} contradiction
        {filteredContradictions.length !== 1 ? 's' : ''}
      </div>

      {/* List */}
      {filteredContradictions.length > 0 ? (
        <div className="space-y-2">
          {filteredContradictions.map((contradiction) => (
            <ContradictionItem
              key={contradiction.id}
              contradiction={contradiction}
              onSelect={onSelect}
              onResolve={handleResolve}
              isResolving={isResolving}
            />
          ))}
        </div>
      ) : (
        /* Empty State */
        <div className="p-8 text-center rounded-lg bg-surface border border-border">
          <div className="inline-flex items-center justify-center w-12 h-12 rounded-full bg-success/10 mb-3">
            <CheckCircle2 className="w-6 h-6 text-success" />
          </div>
          <h4 className="text-sm font-medium text-text-primary">
            No Contradictions Found
          </h4>
          <p className="text-sm text-text-secondary mt-1">
            {filterSeverity !== 'all'
              ? `No ${filterSeverity} severity contradictions${showResolved ? '' : ' (unresolved)'}`
              : showResolved
              ? 'No contradictions in the system'
              : 'All contradictions have been resolved'}
          </p>
        </div>
      )}
    </div>
  );
}

export default ContradictionList;
