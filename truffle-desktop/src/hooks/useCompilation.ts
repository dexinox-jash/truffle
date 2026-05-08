// Custom hook for compilation operations

import { useState, useCallback } from 'react';
import { useAppStore } from '../store/appStore';

interface CompilationOptions {
  priority?: 'high' | 'normal' | 'low';
  onProgress?: (progress: number) => void;
  onComplete?: (result: CompilationResult) => void;
  onError?: (error: Error) => void;
}

interface CompilationResult {
  artifactId: string;
  success: boolean;
  nodesCreated: string[];
  nodesUpdated: string[];
  error?: string;
}

export function useCompilation() {
  const {
    isCompiling,
    setIsCompiling,
    addCompilationJob,
    updateCompilationJob,
    setCurrentCompilationJob,
  } = useAppStore();

  const [progress, setProgress] = useState(0);
  const [error, setError] = useState<Error | null>(null);

  // Compile a single artifact
  const compileArtifact = useCallback(async (
    artifactId: string,
    options: CompilationOptions = {}
  ): Promise<CompilationResult> => {
    const { priority = 'normal', onProgress, onComplete, onError } = options;

    setIsCompiling(true);
    setProgress(0);
    setError(null);

    // Create job
    const jobId = `job-${Date.now()}`;
    addCompilationJob({
      id: jobId,
      artifactId,
      status: 'processing',
      progress: 0,
      startedAt: new Date().toISOString(),
    });

    setCurrentCompilationJob({
      id: jobId,
      artifactId,
      status: 'processing',
      progress: 0,
    });

    try {
      // Simulate compilation (replace with actual Tauri API call)
      const result = await simulateCompilation(artifactId, (p) => {
        setProgress(p);
        onProgress?.(p);
        updateCompilationJob(jobId, { progress: p });
      });

      // Update job status
      updateCompilationJob(jobId, {
        status: 'completed',
        progress: 100,
        completedAt: new Date().toISOString(),
      });

      setCurrentCompilationJob(null);
      setIsCompiling(false);
      onComplete?.(result);

      return result;
    } catch (err) {
      const error = err instanceof Error ? err : new Error('Compilation failed');
      
      setError(error);
      setIsCompiling(false);
      
      updateCompilationJob(jobId, {
        status: 'failed',
        error: error.message,
      });

      setCurrentCompilationJob(null);
      onError?.(error);

      throw error;
    }
  }, [
    setIsCompiling,
    addCompilationJob,
    updateCompilationJob,
    setCurrentCompilationJob,
  ]);

  // Compile all pending artifacts
  const compileAllPending = useCallback(async (
    options: CompilationOptions = {}
  ): Promise<CompilationResult[]> => {
    const { onProgress } = options;
    const { artifacts } = useAppStore.getState();
    
    const pendingArtifacts = artifacts.filter(a => a.status === 'pending');
    const results: CompilationResult[] = [];

    for (let i = 0; i < pendingArtifacts.length; i++) {
      const artifact = pendingArtifacts[i];
      const overallProgress = (i / pendingArtifacts.length) * 100;
      
      try {
        const result = await compileArtifact(artifact.id, {
          ...options,
          onProgress: (p) => {
            onProgress?.(overallProgress + (p / pendingArtifacts.length));
          },
        });
        results.push(result);
      } catch (error) {
        results.push({
          artifactId: artifact.id,
          success: false,
          nodesCreated: [],
          nodesUpdated: [],
          error: error instanceof Error ? error.message : 'Unknown error',
        });
      }
    }

    return results;
  }, [compileArtifact]);

  // Cancel current compilation
  const cancelCompilation = useCallback(() => {
    setIsCompiling(false);
    setCurrentCompilationJob(null);
    // TODO: Cancel via Tauri API
  }, [setIsCompiling, setCurrentCompilationJob]);

  return {
    isCompiling,
    progress,
    error,
    compileArtifact,
    compileAllPending,
    cancelCompilation,
  };
}

// Simulate compilation (replace with actual Tauri API)
async function simulateCompilation(
  artifactId: string,
  onProgress: (progress: number) => void
): Promise<CompilationResult> {
  return new Promise((resolve, reject) => {
    let progress = 0;
    
    const interval = setInterval(() => {
      progress += 10;
      onProgress(progress);
      
      if (progress >= 100) {
        clearInterval(interval);
        
        // Simulate occasional failure
        if (Math.random() > 0.9) {
          reject(new Error('Compilation failed: Model error'));
        } else {
          resolve({
            artifactId,
            success: true,
            nodesCreated: [`node-${Date.now()}`],
            nodesUpdated: [],
          });
        }
      }
    }, 200);
  });
}
