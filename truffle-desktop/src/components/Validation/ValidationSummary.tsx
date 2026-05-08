// SPEC v2.0: Validation Summary Component
// Overview stats for contradictions, duplicates, and confidence

import React from 'react';
import {
  AlertTriangle,
  AlertCircle,
  Copy,
  Gauge,
  ChevronRight,
} from 'lucide-react';
import {
  useValidationReport,
  useContradictions,
  useDuplicateCandidates,
} from '../../hooks/useValidation';
import type { ContradictionSeverity } from '../../types/knowledge-graph';

export interface ValidationSummaryProps {
  entityId?: string; // undefined = global summary
  onContradictionClick?: (count: number) => void;
  onDuplicateClick?: (count: number) => void;
  onConfidenceClick?: () => void;
  className?: string;
}

interface StatCardProps {
  icon: React.ReactNode;
  label: string;
  value: number | string;
  subtext?: string;
  severity?: 'neutral' | 'warning' | 'error' | 'success';
  onClick?: () => void;
  clickable?: boolean;
}

const severityColors: Record<string, string> = {
  neutral: 'bg-surface border-border text-text-primary',
  warning: 'bg-warning/10 border-warning/30 text-warning',
  error: 'bg-error/10 border-error/30 text-error',
  success: 'bg-success/10 border-success/30 text-success',
};

function StatCard({
  icon,
  label,
  value,
  subtext,
  severity = 'neutral',
  onClick,
  clickable = false,
}: StatCardProps) {
  return (
    <div
      onClick={onClick}
      className={`
        relative p-4 rounded-lg border transition-all duration-200
        ${severityColors[severity]}
        ${clickable ? 'cursor-pointer hover:border-border-hover hover:bg-surface-hover' : ''}
      `}
    >
      <div className="flex items-start justify-between">
        <div className="flex items-center gap-3">
          <div className="p-2 rounded-md bg-background/50">{icon}</div>
          <div>
            <p className="text-sm text-text-secondary">{label}</p>
            <p className="text-2xl font-semibold mt-1">{value}</p>
            {subtext && <p className="text-xs text-text-muted mt-1">{subtext}</p>}
          </div>
        </div>
        {clickable && (
          <ChevronRight className="w-4 h-4 text-text-muted" />
        )}
      </div>
    </div>
  );
}

function SeverityBadge({
  severity,
  count,
}: {
  severity: ContradictionSeverity;
  count: number;
}) {
  const colors: Record<ContradictionSeverity, string> = {
    low: 'bg-text-muted/20 text-text-muted',
    medium: 'bg-warning/20 text-warning',
    high: 'bg-orange-500/20 text-orange-400',
    critical: 'bg-error/20 text-error',
  };

  return (
    <span
      className={`
        inline-flex items-center px-2 py-0.5 rounded-full text-xs font-medium
        ${colors[severity]}
      `}
    >
      {severity.charAt(0).toUpperCase() + severity.slice(1)}: {count}
    </span>
  );
}

export function ValidationSummary({
  entityId,
  onContradictionClick,
  onDuplicateClick,
  onConfidenceClick,
  className = '',
}: ValidationSummaryProps) {
  // Fetch data
  const { data: report, isLoading: reportLoading } = useValidationReport();
  const { data: contradictions, isLoading: contradictionsLoading } = useContradictions(false);
  const { data: duplicates, isLoading: duplicatesLoading } = useDuplicateCandidates();

  // Calculate severity breakdown
  const severityCounts = React.useMemo(() => {
    if (!contradictions) return { low: 0, medium: 0, high: 0, critical: 0 };
    return contradictions.reduce(
      (acc, c) => {
        acc[c.severity] = (acc[c.severity] || 0) + 1;
        return acc;
      },
      { low: 0, medium: 0, high: 0, critical: 0 } as Record<ContradictionSeverity, number>
    );
  }, [contradictions]);

  // Determine overall severity
  const getOverallSeverity = (): 'neutral' | 'warning' | 'error' => {
    const total = contradictions?.length || 0;
    const critical = severityCounts.critical || 0;
    const high = severityCounts.high || 0;

    if (critical > 0 || high > 3) return 'error';
    if (high > 0 || total > 5) return 'warning';
    return 'neutral';
  };

  if (reportLoading || contradictionsLoading || duplicatesLoading) {
    return (
      <div className={`grid grid-cols-2 gap-4 ${className}`}>
        {[...Array(4)].map((_, i) => (
          <div
            key={i}
            className="h-24 rounded-lg bg-surface border border-border animate-pulse"
          />
        ))}
      </div>
    );
  }

  const totalContradictions = contradictions?.length || 0;
  const duplicateCount = duplicates?.length || 0;
  const averageConfidence = report?.averageConfidence || 0;
  const entitiesWithIssues = report?.lowConfidenceEntities?.length || 0;

  return (
    <div className={`space-y-4 ${className}`}>
      {/* Summary Cards */}
      <div className="grid grid-cols-2 gap-4">
        <StatCard
          icon={<AlertTriangle className="w-5 h-5" />}
          label="Contradictions"
          value={totalContradictions}
          severity={getOverallSeverity()}
          clickable={!!onContradictionClick}
          onClick={() => onContradictionClick?.(totalContradictions)}
        />

        <StatCard
          icon={<Copy className="w-5 h-5" />}
          label="Duplicate Candidates"
          value={duplicateCount}
          severity={duplicateCount > 0 ? 'warning' : 'success'}
          clickable={!!onDuplicateClick}
          onClick={() => onDuplicateClick?.(duplicateCount)}
        />

        <StatCard
          icon={<Gauge className="w-5 h-5" />}
          label="Avg. Confidence"
          value={`${Math.round(averageConfidence * 100)}%`}
          subtext={averageConfidence < 0.7 ? 'Below threshold' : 'Good'}
          severity={averageConfidence < 0.5 ? 'error' : averageConfidence < 0.7 ? 'warning' : 'success'}
          clickable={!!onConfidenceClick}
          onClick={onConfidenceClick}
        />

        <StatCard
          icon={<AlertCircle className="w-5 h-5" />}
          label="Entities with Issues"
          value={entitiesWithIssues}
          severity={entitiesWithIssues > 0 ? 'warning' : 'success'}
        />
      </div>

      {/* Severity Breakdown */}
      {totalContradictions > 0 && (
        <div className="p-4 rounded-lg bg-surface border border-border">
          <h4 className="text-sm font-medium text-text-secondary mb-3">
            Contradictions by Severity
          </h4>
          <div className="flex flex-wrap gap-2">
            {(['critical', 'high', 'medium', 'low'] as ContradictionSeverity[]).map(
              (severity) =>
                severityCounts[severity] > 0 && (
                  <SeverityBadge
                    key={severity}
                    severity={severity}
                    count={severityCounts[severity]}
                  />
                )
            )}
          </div>
        </div>
      )}

      {/* Empty State */}
      {totalContradictions === 0 &&
        duplicateCount === 0 &&
        entitiesWithIssues === 0 && (
          <div className="p-6 text-center rounded-lg bg-surface border border-border">
            <div className="inline-flex items-center justify-center w-12 h-12 rounded-full bg-success/10 mb-3">
              <Gauge className="w-6 h-6 text-success" />
            </div>
            <h4 className="text-sm font-medium text-text-primary">All Clear</h4>
            <p className="text-sm text-text-secondary mt-1">
              No validation issues detected
            </p>
          </div>
        )}
    </div>
  );
}

export default ValidationSummary;
