// SPEC v2.0 SECTION 5.2.2: Status Bar Component
// Bottom status bar for Truffle Desktop

import React from 'react';
import { 
  Clock, 
  HardDrive, 
  Image as ImageIcon, 
  FileText,
  CheckCircle2,
  AlertCircle,
  Loader2
} from 'lucide-react';
import { useAppStore } from '../../store/appStore';
import { useSync } from '../../store/syncStore';
import { formatDistanceToNow } from 'date-fns';

interface StatusBarProps {
  className?: string;
}

export const StatusBar: React.FC<StatusBarProps> = ({ className = '' }) => {
  const { 
    artifacts, 
    wikiNodes, 
    isCompiling, 
    compilationQueue,
    currentCompilationJob,
    lastSyncAt,
  } = useAppStore();
  
  const { syncStatus, pendingChanges } = useSync();

  // Calculate stats
  const pendingCount = artifacts.filter(a => a.status === 'pending').length;
  const compiledCount = artifacts.filter(a => a.status === 'compiled').length;
  const failedCount = artifacts.filter(a => a.status === 'failed').length;
  
  // Format last compile time
  const getLastCompileText = () => {
    if (currentCompilationJob?.completedAt) {
      return `Last compile: ${formatDistanceToNow(new Date(currentCompilationJob.completedAt), { addSuffix: true })}`;
    }
    return 'No recent compiles';
  };

  // Get compilation status indicator
  const getCompilationIndicator = () => {
    if (isCompiling) {
      return (
        <div className="flex items-center gap-2 text-primary">
          <Loader2 className="w-3 h-3 animate-spin" />
          <span className="text-xs">Compiling... ({compilationQueue.length} pending)</span>
        </div>
      );
    }
    
    if (failedCount > 0) {
      return (
        <div className="flex items-center gap-2 text-error">
          <AlertCircle className="w-3 h-3" />
          <span className="text-xs">{failedCount} failed</span>
        </div>
      );
    }
    
    return (
      <div className="flex items-center gap-2 text-secondary">
        <CheckCircle2 className="w-3 h-3" />
        <span className="text-xs">{getLastCompileText()}</span>
      </div>
    );
  };

  return (
    <footer 
      className={`
        h-8 flex items-center justify-between px-4 
        bg-surface border-t border-border text-xs
        ${className}
      `}
    >
      {/* Left: Compilation Status */}
      <div className="flex items-center gap-4">
        {getCompilationIndicator()}
        
        {/* Progress bar when compiling */}
        {isCompiling && currentCompilationJob && (
          <div className="w-32 h-1.5 bg-background rounded-full overflow-hidden">
            <div 
              className="h-full bg-primary rounded-full transition-all duration-300"
              style={{ width: `${currentCompilationJob.progress}%` }}
            />
          </div>
        )}
      </div>
      
      {/* Center: Item Counts */}
      <div className="flex items-center gap-4 text-text-secondary">
        <div className="flex items-center gap-1.5">
          <ImageIcon className="w-3 h-3" />
          <span>{artifacts.length} artifacts</span>
          {pendingCount > 0 && (
            <span className="text-warning">({pendingCount} pending)</span>
          )}
        </div>
        
        <div className="flex items-center gap-1.5">
          <FileText className="w-3 h-3" />
          <span>{wikiNodes.length} wiki nodes</span>
        </div>
      </div>
      
      {/* Right: Storage & Sync */}
      <div className="flex items-center gap-4 text-text-secondary">
        {/* Sync Status */}
        {syncStatus !== 'offline' && (
          <div className="flex items-center gap-1.5">
            <Clock className="w-3 h-3" />
            <span>
              {lastSyncAt 
                ? `Synced ${formatDistanceToNow(new Date(lastSyncAt), { addSuffix: true })}`
                : 'Never synced'
              }
            </span>
            {pendingChanges > 0 && (
              <span className="text-warning">({pendingChanges} pending)</span>
            )}
          </div>
        )}
        
        {/* Storage */}
        <div className="flex items-center gap-1.5">
          <HardDrive className="w-3 h-3" />
          <span>12GB used</span>
        </div>
      </div>
    </footer>
  );
};

export default StatusBar;
