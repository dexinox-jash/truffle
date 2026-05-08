// SPEC v2.0 SECTION 5.2.2: Compilation Preview Panel
// Center panel showing compilation progress and results

import React, { useState } from 'react';
import { 
  Play, 
  Pause, 
  RotateCcw, 
  ChevronDown, 
  ChevronUp,
  Terminal,
  Sparkles,
  FileText,
  Link,
  Clock
} from 'lucide-react';
import { useAppStore } from '../../store/appStore';
import { BeforeAfterDiff } from './BeforeAfterDiff';
import { GemmaLogs } from './GemmaLogs';

interface CompilationPanelProps {
  className?: string;
}

export const CompilationPanel: React.FC<CompilationPanelProps> = ({ className = '' }) => {
  const {
    selectedArtifactId,
    isCompiling,
    currentCompilationJob,
    compilationQueue,
    artifacts,
  } = useAppStore();

  const [showLogs, setShowLogs] = useState(true);
  const [activeTab, setActiveTab] = useState<'preview' | 'diff' | 'logs'>('preview');

  // Get selected artifact
  const selectedArtifact = artifacts.find(a => a.id === selectedArtifactId);

  // Get compilation progress
  const progress = currentCompilationJob?.progress || 0;

  // Handle compile action
  const handleCompile = () => {
    if (!selectedArtifactId) return;
    // TODO: Trigger compilation via Tauri API
    console.log('Compile:', selectedArtifactId);
  };

  // Handle compile all
  const handleCompileAll = () => {
    // TODO: Trigger batch compilation
    console.log('Compile all pending');
  };

  // Handle cancel
  const handleCancel = () => {
    // TODO: Cancel compilation
    console.log('Cancel compilation');
  };

  return (
    <div className={`flex flex-col h-full bg-background ${className}`}>
      {/* Panel Header */}
      <div className="flex items-center justify-between px-3 py-2 border-b border-border">
        <div className="flex items-center gap-2">
          <Sparkles className="w-4 h-4 text-primary" />
          <h2 className="text-sm font-semibold text-text-primary">Compilation Preview</h2>
        </div>
        
        <div className="flex items-center gap-2">
          {/* Compile Actions */}
          {isCompiling ? (
            <button
              onClick={handleCancel}
              className="flex items-center gap-1.5 px-2 py-1.5 bg-error/10 text-error rounded-md text-xs font-medium hover:bg-error/20 transition-colors"
            >
              <Pause className="w-3.5 h-3.5" />
              <span>Cancel</span>
            </button>
          ) : (
            <>
              {selectedArtifactId && selectedArtifact?.status === 'pending' && (
                <button
                  onClick={handleCompile}
                  className="flex items-center gap-1.5 px-2 py-1.5 bg-primary text-background rounded-md text-xs font-medium hover:bg-primary-hover transition-colors"
                >
                  <Play className="w-3.5 h-3.5" />
                  <span>Compile</span>
                </button>
              )}
              
              {compilationQueue.length > 0 && (
                <button
                  onClick={handleCompileAll}
                  className="flex items-center gap-1.5 px-2 py-1.5 bg-surface border border-border text-text-primary rounded-md text-xs font-medium hover:bg-surface-hover transition-colors"
                >
                  <Sparkles className="w-3.5 h-3.5" />
                  <span>Compile All ({compilationQueue.length})</span>
                </button>
              )}
            </>
          )}
        </div>
      </div>

      {/* Compilation Progress */}
      {isCompiling && (
        <div className="px-3 py-3 border-b border-border bg-surface/50">
          {/* Progress Bar with Chemical Animation */}
          <div className="compile-progress mb-2">
            <div 
              className="compile-progress-bar"
              style={{ width: `${progress}%` }}
            >
              {/* Chemical bubbles */}
              <div className="chemical-bubbles">
                <div className="chemical-bubble" />
                <div className="chemical-bubble" />
                <div className="chemical-bubble" />
                <div className="chemical-bubble" />
                <div className="chemical-bubble" />
              </div>
            </div>
          </div>
          
          <div className="flex items-center justify-between text-xs">
            <span className="text-text-secondary">
              Compiling {currentCompilationJob?.artifactId}...
            </span>
            <span className="text-primary font-medium">{Math.round(progress)}%</span>
          </div>
        </div>
      )}

      {/* Tabs */}
      <div className="flex items-center gap-1 px-3 py-2 border-b border-border">
        {(['preview', 'diff', 'logs'] as const).map((tab) => (
          <button
            key={tab}
            onClick={() => setActiveTab(tab)}
            className={`
              px-3 py-1.5 rounded-md text-xs font-medium capitalize
              transition-colors
              ${activeTab === tab 
                ? 'bg-surface text-text-primary' 
                : 'text-text-secondary hover:text-text-primary hover:bg-surface-hover'
              }
            `}
          >
            {tab === 'diff' ? 'Before/After' : tab}
          </button>
        ))}
      </div>

      {/* Content */}
      <div className="flex-1 overflow-hidden">
        {!selectedArtifact ? (
          <div className="flex flex-col items-center justify-center h-full text-text-muted">
            <div className="w-16 h-16 rounded-full bg-surface flex items-center justify-center mb-4">
              <Sparkles className="w-8 h-8" />
            </div>
            <p className="text-sm">Select an artifact to compile</p>
            <p className="text-xs mt-1">AI will extract entities and create wiki nodes</p>
          </div>
        ) : (
          <>
            {activeTab === 'preview' && (
              <div className="h-full overflow-auto p-4">
                {/* Preview Content */}
                <div className="space-y-4">
                  {/* Artifact Preview */}
                  <div className="rounded-lg overflow-hidden bg-surface border border-border">
                    <div className="aspect-video bg-surface-elevated flex items-center justify-center">
                      {selectedArtifact.thumbnailUrl ? (
                        <img
                          src={selectedArtifact.thumbnailUrl}
                          alt={selectedArtifact.filename}
                          className="w-full h-full object-contain"
                        />
                      ) : (
                        <div className="text-text-muted">
                          <FileText className="w-12 h-12 mx-auto mb-2" />
                          <p className="text-sm">No preview available</p>
                        </div>
                      )}
                    </div>
                    <div className="p-3">
                      <p className="text-sm font-medium text-text-primary">
                        {selectedArtifact.filename}
                      </p>
                      <p className="text-xs text-text-muted mt-1">
                        Captured {new Date(selectedArtifact.capturedAt).toLocaleString()}
                      </p>
                      {selectedArtifact.appContext && (
                        <p className="text-xs text-text-muted">
                          From: {selectedArtifact.appContext.appName}
                        </p>
                      )}
                    </div>
                  </div>

                  {/* Compilation Status */}
                  <div className="rounded-lg bg-surface border border-border p-3">
                    <h4 className="text-xs font-medium text-text-secondary uppercase tracking-wide mb-2">
                      Compilation Status
                    </h4>
                    <div className="flex items-center gap-2">
                      <div className={`
                        w-2 h-2 rounded-full
                        ${selectedArtifact.status === 'compiled' ? 'bg-secondary' : ''}
                        ${selectedArtifact.status === 'pending' ? 'bg-text-muted' : ''}
                        ${selectedArtifact.status === 'compiling' ? 'bg-primary animate-pulse' : ''}
                        ${selectedArtifact.status === 'failed' ? 'bg-error' : ''}
                      `} />
                      <span className="text-sm capitalize text-text-primary">
                        {selectedArtifact.status}
                      </span>
                    </div>
                  </div>

                  {/* Extracted Entities Preview */}
                  {selectedArtifact.status === 'compiled' && (
                    <div className="rounded-lg bg-surface border border-border p-3">
                      <h4 className="text-xs font-medium text-text-secondary uppercase tracking-wide mb-2">
                        Extracted Entities
                      </h4>
                      <div className="space-y-2">
                        <div className="flex items-center gap-2 text-sm">
                          <Link className="w-4 h-4 text-primary" />
                          <span className="text-text-primary">3 wiki nodes created</span>
                        </div>
                        <div className="flex items-center gap-2 text-sm">
                          <Clock className="w-4 h-4 text-secondary" />
                          <span className="text-text-primary">2 temporal references</span>
                        </div>
                      </div>
                    </div>
                  )}
                </div>
              </div>
            )}

            {activeTab === 'diff' && (
              <BeforeAfterDiff artifactId={selectedArtifactId} />
            )}

            {activeTab === 'logs' && (
              <GemmaLogs artifactId={selectedArtifactId} />
            )}
          </>
        )}
      </div>
    </div>
  );
};

export default CompilationPanel;
