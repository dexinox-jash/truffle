import { useEffect, useState } from 'react';
import { 
  X, 
  Loader2, 
  CheckCircle, 
  AlertCircle, 
  RotateCcw,
  FileText,
  Brain,
  Network,
  GitMerge,
  Sparkles,
} from 'lucide-react';
import { useExtractionProgress, useCancelExtraction } from '../../hooks/useMeetings';
import { ExtractionStage, type ExtractionResult } from '../../types/meeting';

export interface ExtractionPanelProps {
  meetingId: string;
  isExtracting: boolean;
  progress?: {
    stage: string;
    percent: number;
  };
  results?: ExtractionResult;
  onCancel?: () => void;
}

interface StageInfo {
  icon: typeof FileText;
  label: string;
  description: string;
}

const stageConfig: Record<ExtractionStage, StageInfo> = {
  [ExtractionStage.Preprocessing]: {
    icon: FileText,
    label: 'Preprocessing',
    description: 'Analyzing transcript structure...',
  },
  [ExtractionStage.Extracting]: {
    icon: Brain,
    label: 'Extracting Entities',
    description: 'Identifying people, organizations, and concepts...',
  },
  [ExtractionStage.Resolving]: {
    icon: GitMerge,
    label: 'Resolving Entities',
    description: 'Matching against existing knowledge base...',
  },
  [ExtractionStage.Building]: {
    icon: Network,
    label: 'Building Relationships',
    description: 'Connecting entities and extracting context...',
  },
  [ExtractionStage.Completed]: {
    icon: CheckCircle,
    label: 'Completed',
    description: 'Extraction finished successfully',
  },
};

function StageIndicator({ 
  stage, 
  currentStage, 
  isCompleted 
}: { 
  stage: ExtractionStage; 
  currentStage: ExtractionStage;
  isCompleted: boolean;
}) {
  const config = stageConfig[stage];
  const Icon = config.icon;
  
  const stageOrder = [
    ExtractionStage.Preprocessing,
    ExtractionStage.Extracting,
    ExtractionStage.Resolving,
    ExtractionStage.Building,
    ExtractionStage.Completed,
  ];
  
  const currentIndex = stageOrder.indexOf(currentStage);
  const stageIndex = stageOrder.indexOf(stage);
  
  let status: 'pending' | 'active' | 'completed' = 'pending';
  if (isCompleted || stageIndex < currentIndex) {
    status = 'completed';
  } else if (stage === currentStage) {
    status = 'active';
  }

  return (
    <div className={`flex items-center gap-3 ${status === 'pending' ? 'opacity-40' : ''}`}>
      <div className={`
        w-8 h-8 rounded-full flex items-center justify-center
        ${status === 'completed' ? 'bg-green-500/20 text-green-400' : ''}
        ${status === 'active' ? 'bg-blue-500/20 text-blue-400 ring-2 ring-blue-500/50' : ''}
        ${status === 'pending' ? 'bg-gray-700 text-gray-500' : ''}
      `}>
        {status === 'completed' ? (
          <CheckCircle size={16} />
        ) : status === 'active' ? (
          <Icon size={16} />
        ) : (
          <div className="w-2 h-2 rounded-full bg-gray-500" />
        )}
      </div>
      <div>
        <p className={`text-sm font-medium ${
          status === 'active' ? 'text-blue-400' : 'text-gray-300'
        }`}>
          {config.label}
        </p>
        {status === 'active' && (
          <p className="text-xs text-gray-500">{config.description}</p>
        )}
      </div>
    </div>
  );
}

function ProgressBar({ percent }: { percent: number }) {
  return (
    <div className="relative h-2 bg-gray-700 rounded-full overflow-hidden">
      <div
        className="absolute inset-y-0 left-0 bg-gradient-to-r from-blue-500 to-purple-500 
                   rounded-full transition-all duration-300 ease-out"
        style={{ width: `${percent}%` }}
      >
        <div className="absolute inset-0 bg-white/20 animate-pulse" />
      </div>
    </div>
  );
}

function ResultsPreview({ results }: { results: ExtractionResult }) {
  const entityCount = results.entities.length;
  const relationshipCount = results.relationships.length;
  const decisionCount = results.decisions.length;
  const actionItemCount = results.actionItems.length;

  return (
    <div className="bg-green-500/5 border border-green-500/20 rounded-lg p-4">
      <div className="flex items-center gap-2 mb-3">
        <Sparkles size={18} className="text-green-400" />
        <h4 className="font-medium text-green-400">Extraction Complete</h4>
      </div>
      
      <div className="grid grid-cols-2 gap-3">
        <div className="bg-gray-800/50 rounded-lg p-3">
          <p className="text-2xl font-semibold text-gray-200">{entityCount}</p>
          <p className="text-xs text-gray-500">Entities Found</p>
        </div>
        <div className="bg-gray-800/50 rounded-lg p-3">
          <p className="text-2xl font-semibold text-gray-200">{relationshipCount}</p>
          <p className="text-xs text-gray-500">Relationships</p>
        </div>
        <div className="bg-gray-800/50 rounded-lg p-3">
          <p className="text-2xl font-semibold text-gray-200">{decisionCount}</p>
          <p className="text-xs text-gray-500">Decisions</p>
        </div>
        <div className="bg-gray-800/50 rounded-lg p-3">
          <p className="text-2xl font-semibold text-gray-200">{actionItemCount}</p>
          <p className="text-xs text-gray-500">Action Items</p>
        </div>
      </div>
    </div>
  );
}

function ErrorState({ error, onRetry }: { error: string; onRetry: () => void }) {
  return (
    <div className="bg-red-500/5 border border-red-500/20 rounded-lg p-4">
      <div className="flex items-center gap-2 mb-2">
        <AlertCircle size={18} className="text-red-400" />
        <h4 className="font-medium text-red-400">Extraction Failed</h4>
      </div>
      <p className="text-sm text-gray-400 mb-3">{error}</p>
      <button
        onClick={onRetry}
        className="flex items-center gap-2 px-3 py-1.5 text-sm bg-red-500/10 text-red-400 
                   rounded-lg hover:bg-red-500/20 transition-colors"
      >
        <RotateCcw size={14} />
        Retry Extraction
      </button>
    </div>
  );
}

export function ExtractionPanel({ 
  meetingId, 
  isExtracting, 
  progress, 
  results, 
  onCancel 
}: ExtractionPanelProps) {
  const { data: liveProgress } = useExtractionProgress(meetingId);
  const cancelMutation = useCancelExtraction();
  
  const currentProgress = liveProgress || progress;
  const currentStage = (liveProgress?.stage || progress?.stage || ExtractionStage.Preprocessing) as ExtractionStage;
  const percent = currentProgress?.percent || 0;

  const handleCancel = async () => {
    await cancelMutation.mutateAsync(meetingId);
    onCancel?.();
  };

  // Show results if completed
  if (results && results.status === 'success' && !isExtracting) {
    return (
      <div className="p-4 bg-gray-900 rounded-lg border border-gray-800">
        <ResultsPreview results={results} />
      </div>
    );
  }

  // Show error if failed
  if (results?.status === 'failed' && !isExtracting) {
    return (
      <div className="p-4 bg-gray-900 rounded-lg border border-gray-800">
        <ErrorState error={results.error || 'Unknown error occurred'} onRetry={() => {}} />
      </div>
    );
  }

  // Show progress if extracting
  if (isExtracting) {
    return (
      <div className="p-4 bg-gray-900 rounded-lg border border-gray-800">
        {/* Header */}
        <div className="flex items-center justify-between mb-4">
          <div className="flex items-center gap-2">
            <Loader2 size={18} className="animate-spin text-blue-500" />
            <h3 className="font-medium text-gray-200">Extracting Entities</h3>
          </div>
          <button
            onClick={handleCancel}
            disabled={cancelMutation.isPending}
            className="p-1.5 text-gray-400 hover:text-gray-200 hover:bg-gray-800 rounded-lg transition-colors"
            title="Cancel extraction"
          >
            <X size={16} />
          </button>
        </div>

        {/* Progress Bar */}
        <div className="mb-6">
          <div className="flex items-center justify-between mb-2">
            <span className="text-sm text-gray-400">
              {stageConfig[currentStage]?.label || 'Processing...'}
            </span>
            <span className="text-sm font-medium text-gray-300">{Math.round(percent)}%</span>
          </div>
          <ProgressBar percent={percent} />
        </div>

        {/* Stage Indicators */}
        <div className="space-y-3">
          <StageIndicator 
            stage={ExtractionStage.Preprocessing} 
            currentStage={currentStage}
            isCompleted={false}
          />
          <StageIndicator 
            stage={ExtractionStage.Extracting} 
            currentStage={currentStage}
            isCompleted={false}
          />
          <StageIndicator 
            stage={ExtractionStage.Resolving} 
            currentStage={currentStage}
            isCompleted={false}
          />
          <StageIndicator 
            stage={ExtractionStage.Building} 
            currentStage={currentStage}
            isCompleted={false}
          />
        </div>

        {/* Cancel button */}
        <div className="mt-4 pt-4 border-t border-gray-800">
          <button
            onClick={handleCancel}
            disabled={cancelMutation.isPending}
            className="w-full py-2 text-sm text-gray-400 hover:text-gray-200 
                       hover:bg-gray-800 rounded-lg transition-colors"
          >
            Cancel Extraction
          </button>
        </div>
      </div>
    );
  }

  return null;
}

export default ExtractionPanel;
