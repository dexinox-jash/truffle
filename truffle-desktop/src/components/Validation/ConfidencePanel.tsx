// SPEC v2.0: Confidence Panel Component
// Confidence visualization with factors breakdown

import React from 'react';
import {
  Gauge,
  RefreshCw,
  CheckCircle2,
  AlertTriangle,
  FileText,
  Shield,
  Clock,
  TrendingUp,
} from 'lucide-react';
import { useConfidenceScore, useRecalculateConfidence } from '../../hooks/useValidation';
import type { ConfidenceFactor } from '../../types/knowledge-graph';

export interface ConfidencePanelProps {
  entityId: string;
  showFactors?: boolean;
  className?: string;
}

const factorIcons: Record<string, typeof FileText> = {
  extraction: FileText,
  source: Shield,
  verification: CheckCircle2,
  recency: Clock,
  default: TrendingUp,
};

const factorLabels: Record<string, string> = {
  extraction: 'Extraction Confidence',
  source: 'Source Reliability',
  verification: 'Verification Status',
  recency: 'Data Recency',
};

function FactorRow({ factor }: { factor: ConfidenceFactor }) {
  const Icon = factorIcons[factor.category] || factorIcons.default;

  return (
    <div className="flex items-center gap-3 p-3 rounded-lg bg-surface">
      <div className="p-2 rounded-md bg-surface-elevated">
        <Icon className="w-4 h-4 text-text-secondary" />
      </div>
      <div className="flex-1 min-w-0">
        <div className="flex items-center justify-between">
          <span className="text-sm text-text-primary">
            {factorLabels[factor.category] || factor.name}
          </span>
          <span className="text-sm font-medium text-text-primary">
            {Math.round(factor.value * 100)}%
          </span>
        </div>
        <div className="mt-1.5 h-1.5 bg-surface-elevated rounded-full overflow-hidden">
          <div
            className={`h-full rounded-full transition-all duration-500 ${
              factor.value >= 0.8
                ? 'bg-success'
                : factor.value >= 0.5
                ? 'bg-warning'
                : 'bg-error'
            }`}
            style={{ width: `${factor.value * 100}%` }}
          />
        </div>
        <p className="text-xs text-text-muted mt-1">
          Weight: {Math.round(factor.weight * 100)}%
        </p>
      </div>
    </div>
  );
}

function ConfidenceTier({ score }: { score: number }) {
  let tier: { label: string; color: string; icon: typeof Gauge } = {
    label: 'Unknown',
    color: 'text-text-muted',
    icon: Gauge,
  };

  if (score >= 0.85) {
    tier = { label: 'High Confidence', color: 'text-success', icon: CheckCircle2 };
  } else if (score >= 0.6) {
    tier = { label: 'Medium Confidence', color: 'text-warning', icon: Gauge };
  } else {
    tier = { label: 'Low Confidence', color: 'text-error', icon: AlertTriangle };
  }

  const Icon = tier.icon;

  return (
    <div className={`flex items-center gap-2 ${tier.color}`}>
      <Icon className="w-4 h-4" />
      <span className="text-sm font-medium">{tier.label}</span>
    </div>
  );
}

function RecommendationItem({ text, priority }: { text: string; priority: 'high' | 'medium' | 'low' }) {
  const colors = {
    high: 'border-error/30 bg-error/5',
    medium: 'border-warning/30 bg-warning/5',
    low: 'border-primary/30 bg-primary/5',
  };

  return (
    <li className={`p-3 rounded-lg border ${colors[priority]} flex items-start gap-2`}>
      <div
        className={`w-1.5 h-1.5 rounded-full mt-2 flex-shrink-0 ${
          priority === 'high' ? 'bg-error' : priority === 'medium' ? 'bg-warning' : 'bg-primary'
        }`}
      />
      <span className="text-sm text-text-secondary">{text}</span>
    </li>
  );
}

export function ConfidencePanel({
  entityId,
  showFactors = true,
  className = '',
}: ConfidencePanelProps) {
  const { data: score, isLoading, isError } = useConfidenceScore(entityId);
  const { mutate: recalculate, isPending: isRecalculating } = useRecalculateConfidence();

  const handleRecalculate = () => {
    recalculate(entityId);
  };

  // Generate recommendations based on factors
  const recommendations = React.useMemo(() => {
    if (!score?.factors) return [];

    const recs: { text: string; priority: 'high' | 'medium' | 'low' }[] = [];

    score.factors.forEach((factor) => {
      if (factor.category === 'extraction' && factor.value < 0.6) {
        recs.push({
          text: 'Review extracted data for accuracy and completeness',
          priority: 'high',
        });
      }
      if (factor.category === 'source' && factor.value < 0.5) {
        recs.push({
          text: 'Add more reliable sources to improve confidence',
          priority: 'medium',
        });
      }
      if (factor.category === 'verification' && factor.value < 0.7) {
        recs.push({
          text: 'Verify information through cross-referencing',
          priority: 'medium',
        });
      }
      if (factor.category === 'recency' && factor.value < 0.5) {
        recs.push({
          text: 'Update outdated information with recent data',
          priority: 'low',
        });
      }
    });

    return recs;
  }, [score]);

  if (isLoading) {
    return (
      <div className={`p-4 rounded-lg bg-surface border border-border ${className}`}>
        <div className="animate-pulse space-y-4">
          <div className="h-24 bg-surface-elevated rounded-lg" />
          {showFactors && (
            <>
              <div className="h-16 bg-surface-elevated rounded-lg" />
              <div className="h-16 bg-surface-elevated rounded-lg" />
              <div className="h-16 bg-surface-elevated rounded-lg" />
            </>
          )}
        </div>
      </div>
    );
  }

  if (isError || !score) {
    return (
      <div className={`p-4 rounded-lg bg-surface border border-error/30 ${className}`}>
        <div className="flex items-center gap-2 text-error">
          <AlertTriangle className="w-5 h-5" />
          <span>Failed to load confidence score</span>
        </div>
      </div>
    );
  }

  const overallPercentage = Math.round(score.overall * 100);

  return (
    <div className={`space-y-4 ${className}`}>
      {/* Overall Score Card */}
      <div className="p-4 rounded-lg bg-surface border border-border">
        <div className="flex items-start justify-between">
          <div>
            <h3 className="text-sm font-medium text-text-secondary mb-1">
              Confidence Score
            </h3>
            <div className="flex items-baseline gap-2">
              <span
                className={`text-4xl font-bold ${
                  score.overall >= 0.8
                    ? 'text-success'
                    : score.overall >= 0.5
                    ? 'text-warning'
                    : 'text-error'
                }`}
              >
                {overallPercentage}%
              </span>
            </div>
            <div className="mt-2">
              <ConfidenceTier score={score.overall} />
            </div>
          </div>
          <button
            onClick={handleRecalculate}
            disabled={isRecalculating}
            className="
              p-2 rounded-md text-text-secondary
              hover:bg-surface-hover hover:text-text-primary
              disabled:opacity-50 disabled:cursor-not-allowed
              transition-colors
            "
            title="Recalculate confidence"
          >
            <RefreshCw className={`w-4 h-4 ${isRecalculating ? 'animate-spin' : ''}`} />
          </button>
        </div>

        {/* Progress Bar */}
        <div className="mt-4 h-2 bg-surface-elevated rounded-full overflow-hidden">
          <div
            className={`h-full rounded-full transition-all duration-500 ${
              score.overall >= 0.8
                ? 'bg-success'
                : score.overall >= 0.5
                ? 'bg-warning'
                : 'bg-error'
            }`}
            style={{ width: `${overallPercentage}%` }}
          />
        </div>

        {/* Uncertainty */}
        <div className="mt-2 flex items-center justify-between text-xs text-text-muted">
          <span>Uncertainty: {Math.round(score.uncertainty * 100)}%</span>
          <span>{score.factors?.length || 0} factors</span>
        </div>
      </div>

      {/* Factors Breakdown */}
      {showFactors && score.factors && score.factors.length > 0 && (
        <div className="space-y-2">
          <h4 className="text-sm font-medium text-text-secondary px-1">
            Confidence Factors
          </h4>
          {score.factors.map((factor, index) => (
            <FactorRow key={`${factor.name}-${index}`} factor={factor} />
          ))}
        </div>
      )}

      {/* Recommendations */}
      {recommendations.length > 0 && (
        <div className="space-y-2">
          <h4 className="text-sm font-medium text-text-secondary px-1">
            Recommendations
          </h4>
          <ul className="space-y-2">
            {recommendations.map((rec, index) => (
              <RecommendationItem key={index} text={rec.text} priority={rec.priority} />
            ))}
          </ul>
        </div>
      )}

      {/* All Good State */}
      {recommendations.length === 0 && score.overall >= 0.8 && (
        <div className="p-4 rounded-lg bg-success/5 border border-success/30 text-center">
          <CheckCircle2 className="w-8 h-8 text-success mx-auto mb-2" />
          <p className="text-sm text-text-primary font-medium">Confidence is High</p>
          <p className="text-xs text-text-secondary mt-1">
            No improvements needed at this time
          </p>
        </div>
      )}
    </div>
  );
}

export default ConfidencePanel;
