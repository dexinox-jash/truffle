// SPEC v2.0: Contradiction Detail Component
// Side-by-side comparison of conflicting facts

import React, { useState } from 'react';
import {
  X,
  AlertTriangle,
  Check,
  GitMerge,
  Edit3,
  Source,
  Gauge,
  ArrowRight,
} from 'lucide-react';
import { useResolveContradiction } from '../../hooks/useValidation';
import type { Contradiction } from '../../types/knowledge-graph';

export interface ResolutionInput {
  selectedFact: 'A' | 'B' | 'merged' | 'manual';
  manualValue?: string;
  mergedValue?: string;
  notes?: string;
}

export interface ContradictionDetailProps {
  contradiction: Contradiction;
  onResolve?: (resolution: ResolutionInput) => void;
  onClose?: () => void;
  className?: string;
}

interface FactCardProps {
  label: string;
  factId: string;
  value: string;
  source: string;
  confidence: number;
  isSelected: boolean;
  onSelect: () => void;
}

function FactCard({
  label,
  factId,
  value,
  source,
  confidence,
  isSelected,
  onSelect,
}: FactCardProps) {
  return (
    <div
      onClick={onSelect}
      className={`
        p-4 rounded-lg border-2 cursor-pointer transition-all duration-200
        ${
          isSelected
            ? 'border-primary bg-primary/5'
            : 'border-border bg-surface hover:border-border-hover'
        }
      `}
    >
      <div className="flex items-center justify-between mb-3">
        <span className="text-sm font-medium text-text-secondary">{label}</span>
        {isSelected && (
          <div className="w-5 h-5 rounded-full bg-primary flex items-center justify-center">
            <Check className="w-3 h-3 text-background" />
          </div>
        )}
      </div>

      <p className="text-text-primary font-medium mb-3">{value}</p>

      <div className="space-y-2 text-xs">
        <div className="flex items-center gap-2 text-text-secondary">
          <Source className="w-3 h-3" />
          <span className="truncate">{source}</span>
        </div>
        <div className="flex items-center gap-2 text-text-secondary">
          <Gauge className="w-3 h-3" />
          <div className="flex items-center gap-2">
            <div className="w-16 h-1.5 bg-surface-elevated rounded-full overflow-hidden">
              <div
                className={`h-full rounded-full ${
                  confidence > 0.7 ? 'bg-success' : confidence > 0.4 ? 'bg-warning' : 'bg-error'
                }`}
                style={{ width: `${confidence * 100}%` }}
              />
            </div>
            <span>{Math.round(confidence * 100)}%</span>
          </div>
        </div>
      </div>
    </div>
  );
}

export function ContradictionDetail({
  contradiction,
  onResolve,
  onClose,
  className = '',
}: ContradictionDetailProps) {
  const [selectedOption, setSelectedOption] = useState<'A' | 'B' | 'merged' | 'manual' | null>(
    null
  );
  const [manualValue, setManualValue] = useState('');
  const [mergedValue, setMergedValue] = useState('');
  const [notes, setNotes] = useState('');

  const { mutate: resolve, isPending: isResolving } = useResolveContradiction();

  // Mock fact data (in real implementation, these would come from the API)
  const factA = {
    id: contradiction.factAId,
    value: 'Fact A Value',
    source: 'Source Document A',
    confidence: 0.85,
  };

  const factB = {
    id: contradiction.factBId,
    value: 'Fact B Value',
    source: 'Source Document B',
    confidence: 0.72,
  };

  const severityColors: Record<string, string> = {
    critical: 'text-error',
    high: 'text-orange-400',
    medium: 'text-warning',
    low: 'text-text-muted',
  };

  const handleResolve = () => {
    if (!selectedOption) return;

    const resolution: ResolutionInput = {
      selectedFact: selectedOption,
      manualValue: selectedOption === 'manual' ? manualValue : undefined,
      mergedValue: selectedOption === 'merged' ? mergedValue : undefined,
      notes,
    };

    resolve(
      {
        contradictionId: contradiction.id,
        resolution: JSON.stringify(resolution),
      },
      {
        onSuccess: () => {
          onResolve?.(resolution);
        },
      }
    );
  };

  return (
    <div className={`bg-surface-elevated rounded-xl border border-border ${className}`}>
      {/* Header */}
      <div className="flex items-start justify-between p-4 border-b border-border">
        <div>
          <div className="flex items-center gap-2 mb-1">
            <AlertTriangle className={`w-5 h-5 ${severityColors[contradiction.severity]}`} />
            <h3 className="text-lg font-semibold text-text-primary">
              Contradiction Details
            </h3>
          </div>
          <p className="text-sm text-text-secondary">{contradiction.description}</p>
          <div className="flex items-center gap-2 mt-2">
            <span
              className={`
                px-2 py-0.5 rounded-full text-xs font-medium
                ${
                  contradiction.severity === 'critical'
                    ? 'bg-error/20 text-error'
                    : contradiction.severity === 'high'
                    ? 'bg-orange-500/20 text-orange-400'
                    : contradiction.severity === 'medium'
                    ? 'bg-warning/20 text-warning'
                    : 'bg-text-muted/20 text-text-muted'
                }
              `}
            >
              {contradiction.severity}
            </span>
            <span className="text-xs text-text-muted">
              {contradiction.contradictionType.replace('_', ' ')}
            </span>
          </div>
        </div>
        {onClose && (
          <button
            onClick={onClose}
            className="p-2 rounded-md hover:bg-surface transition-colors"
          >
            <X className="w-5 h-5 text-text-secondary" />
          </button>
        )}
      </div>

      {/* Comparison */}
      <div className="p-4 space-y-4">
        <h4 className="text-sm font-medium text-text-secondary">Compare Facts</h4>

        <div className="grid grid-cols-2 gap-4">
          <FactCard
            label="Fact A"
            factId={factA.id}
            value={factA.value}
            source={factA.source}
            confidence={factA.confidence}
            isSelected={selectedOption === 'A'}
            onSelect={() => setSelectedOption('A')}
          />

          <FactCard
            label="Fact B"
            factId={factB.id}
            value={factB.value}
            source={factB.source}
            confidence={factB.confidence}
            isSelected={selectedOption === 'B'}
            onSelect={() => setSelectedOption('B')}
          />
        </div>

        {/* Resolution Options */}
        <div className="space-y-3">
          <h4 className="text-sm font-medium text-text-secondary">Resolution Options</h4>

          {/* Merge Option */}
          <button
            onClick={() => setSelectedOption('merged')}
            className={`
              w-full p-3 rounded-lg border text-left transition-all
              ${
                selectedOption === 'merged'
                  ? 'border-primary bg-primary/5'
                  : 'border-border bg-surface hover:border-border-hover'
              }
            `}
          >
            <div className="flex items-center gap-3">
              <GitMerge className="w-4 h-4 text-primary" />
              <div className="flex-1">
                <p className="font-medium text-text-primary">Merge Values</p>
                <p className="text-xs text-text-secondary">Combine both facts</p>
              </div>
              {selectedOption === 'merged' && (
                <Check className="w-4 h-4 text-primary" />
              )}
            </div>
            {selectedOption === 'merged' && (
              <input
                type="text"
                value={mergedValue}
                onChange={(e) => setMergedValue(e.target.value)}
                placeholder="Enter merged value..."
                className="mt-3 w-full bg-background border border-border rounded-md px-3 py-2 text-sm"
                onClick={(e) => e.stopPropagation()}
              />
            )}
          </button>

          {/* Manual Edit Option */}
          <button
            onClick={() => setSelectedOption('manual')}
            className={`
              w-full p-3 rounded-lg border text-left transition-all
              ${
                selectedOption === 'manual'
                  ? 'border-primary bg-primary/5'
                  : 'border-border bg-surface hover:border-border-hover'
              }
            `}
          >
            <div className="flex items-center gap-3">
              <Edit3 className="w-4 h-4 text-secondary" />
              <div className="flex-1">
                <p className="font-medium text-text-primary">Manual Edit</p>
                <p className="text-xs text-text-secondary">Enter custom value</p>
              </div>
              {selectedOption === 'manual' && (
                <Check className="w-4 h-4 text-primary" />
              )}
            </div>
            {selectedOption === 'manual' && (
              <input
                type="text"
                value={manualValue}
                onChange={(e) => setManualValue(e.target.value)}
                placeholder="Enter custom value..."
                className="mt-3 w-full bg-background border border-border rounded-md px-3 py-2 text-sm"
                onClick={(e) => e.stopPropagation()}
              />
            )}
          </button>
        </div>

        {/* Notes */}
        <div>
          <label className="text-sm text-text-secondary block mb-2">
            Resolution Notes (optional)
          </label>
          <textarea
            value={notes}
            onChange={(e) => setNotes(e.target.value)}
            placeholder="Add notes about this resolution..."
            rows={2}
            className="w-full bg-background border border-border rounded-md px-3 py-2 text-sm resize-none"
          />
        </div>
      </div>

      {/* Footer */}
      <div className="flex items-center justify-end gap-3 p-4 border-t border-border">
        {onClose && (
          <button
            onClick={onClose}
            className="px-4 py-2 text-sm text-text-secondary hover:text-text-primary transition-colors"
          >
            Cancel
          </button>
        )}
        <button
          onClick={handleResolve}
          disabled={!selectedOption || isResolving}
          className="
            flex items-center gap-2 px-4 py-2
            bg-primary text-background font-medium rounded-md
            hover:bg-primary-hover
            disabled:opacity-50 disabled:cursor-not-allowed
            transition-colors
          "
        >
          {isResolving ? (
            <>
              <div className="w-4 h-4 border-2 border-background/30 border-t-background rounded-full animate-spin" />
              Resolving...
            </>
          ) : (
            <>
              <Check className="w-4 h-4" />
              Mark as Resolved
            </>
          )}
        </button>
      </div>
    </div>
  );
}

export default ContradictionDetail;
