// Feature Flags System for Truffle Desktop

import { invoke } from '@tauri-apps/api/tauri';

export interface FeatureFlags {
  'knowledge-graph-v2': boolean;
  'advanced-validation': boolean;
  'beta-meetings': boolean;
  'telemetry-opt-in': boolean;
  [key: string]: boolean;
}

const DEFAULT_FLAGS: FeatureFlags = {
  'knowledge-graph-v2': true,
  'advanced-validation': false,
  'beta-meetings': true,
  'telemetry-opt-in': true,
};

class FeatureFlagClient {
  private flags: FeatureFlags = { ...DEFAULT_FLAGS };
  private listeners: Set<(flags: FeatureFlags) => void> = new Set();

  constructor() {
    this.loadFlags();
  }

  private async loadFlags(): Promise<void> {
    try {
      const flags = await invoke<Partial<FeatureFlags>>('get_feature_flags');
      this.flags = { ...DEFAULT_FLAGS, ...flags };
      this.notifyListeners();
    } catch (error) {
      console.warn('Failed to load feature flags:', error);
    }
  }

  isEnabled(flagName: keyof FeatureFlags): boolean {
    return this.flags[flagName] ?? false;
  }

  getAllFlags(): FeatureFlags {
    return { ...this.flags };
  }

  subscribe(listener: (flags: FeatureFlags) => void): () => void {
    this.listeners.add(listener);
    return () => this.listeners.delete(listener);
  }

  private notifyListeners(): void {
    this.listeners.forEach(listener => listener(this.flags));
  }

  async refresh(): Promise<void> {
    await this.loadFlags();
  }
}

export const featureFlags = new FeatureFlagClient();

export function useFeatureFlag(flagName: keyof FeatureFlags): boolean {
  return featureFlags.isEnabled(flagName);
}

export function useFeatureFlags(): FeatureFlags {
  return featureFlags.getAllFlags();
}

import React from 'react';

interface FeatureGateProps {
  flag: keyof FeatureFlags;
  children: React.ReactNode;
  fallback?: React.ReactNode;
}

export const FeatureGate: React.FC<FeatureGateProps> = ({
  flag,
  children,
  fallback = null,
}) => {
  const isEnabled = useFeatureFlag(flag);
  return isEnabled ? <>{children}</> : <>{fallback}</>;
};
