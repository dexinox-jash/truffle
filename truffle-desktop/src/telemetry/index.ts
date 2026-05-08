// Telemetry System for Truffle Desktop
// Anonymous, opt-in usage metrics

import { invoke } from '@tauri-apps/api/tauri';

export interface TelemetryEvent {
  event: string;
  properties?: Record<string, unknown>;
  timestamp: string;
  sessionId: string;
}

export interface TelemetryConfig {
  enabled: boolean;
  endpoint?: string;
  sampleRate: number;
}

class TelemetryClient {
  private config: TelemetryConfig = { enabled: false, sampleRate: 1.0 };
  private sessionId: string;
  private queue: TelemetryEvent[] = [];
  private flushInterval: number | null = null;

  constructor() {
    this.sessionId = this.generateSessionId();
    this.loadConfig();
  }

  private generateSessionId(): string {
    return `${Date.now()}-${Math.random().toString(36).substr(2, 9)}`;
  }

  private async loadConfig(): Promise<void> {
    try {
      const config = await invoke<TelemetryConfig>('get_telemetry_config');
      this.config = config;
      if (config.enabled) {
        this.startFlushTimer();
      }
    } catch (error) {
      console.error('Failed to load telemetry config:', error);
    }
  }

  private startFlushTimer(): void {
    if (this.flushInterval) return;
    this.flushInterval = window.setInterval(() => {
      this.flush();
    }, 30000);
  }

  track(event: string, properties?: Record<string, unknown>): void {
    if (!this.config.enabled) return;
    if (Math.random() > this.config.sampleRate) return;

    const telemetryEvent: TelemetryEvent = {
      event,
      properties,
      timestamp: new Date().toISOString(),
      sessionId: this.sessionId,
    };

    this.queue.push(telemetryEvent);

    if (event === 'app_crash' || event === 'app_error') {
      this.flush();
    }
  }

  async flush(): Promise<void> {
    if (this.queue.length === 0) return;

    const events = [...this.queue];
    this.queue = [];

    try {
      await invoke('send_telemetry', { events });
    } catch (error) {
      this.queue.unshift(...events);
      console.error('Failed to send telemetry:', error);
    }
  }

  async setEnabled(enabled: boolean): Promise<void> {
    this.config.enabled = enabled;
    await invoke('set_telemetry_enabled', { enabled });
    
    if (enabled) {
      this.startFlushTimer();
    } else if (this.flushInterval) {
      clearInterval(this.flushInterval);
      this.flushInterval = null;
    }
  }

  isEnabled(): boolean {
    return this.config.enabled;
  }
}

export const telemetry = new TelemetryClient();

export function useTelemetry() {
  return {
    track: (event: string, properties?: Record<string, unknown>) => {
      telemetry.track(event, properties);
    },
    setEnabled: (enabled: boolean) => telemetry.setEnabled(enabled),
    isEnabled: () => telemetry.isEnabled(),
  };
}

export const TelemetryEvents = {
  APP_STARTED: 'app_started',
  APP_CLOSED: 'app_closed',
  APP_CRASH: 'app_crash',
  ENTITY_CREATED: 'entity_created',
  ENTITY_UPDATED: 'entity_updated',
  ENTITY_DELETED: 'entity_deleted',
  GRAPH_VIEWED: 'graph_viewed',
  VALIDATION_RUN: 'validation_run',
  MEETING_IMPORTED: 'meeting_imported',
  EXTRACTION_COMPLETED: 'extraction_completed',
} as const;
