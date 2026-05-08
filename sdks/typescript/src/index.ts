/**
 * Ruflo SDK - Official TypeScript SDK for Ruflo AI Meeting Notes Platform
 * @module @ruflo/sdk
 */

import axios, { AxiosInstance, AxiosRequestConfig } from 'axios';
import WebSocket from 'ws';

// Types

export interface RufloConfig {
  apiKey: string;
  apiSecret?: string;
  baseURL?: string;
  timeout?: number;
}

export interface Meeting {
  id: string;
  title: string;
  status: 'scheduled' | 'live' | 'completed' | 'cancelled';
  started_at?: string;
  ended_at?: string;
  duration_seconds?: number;
  participants: Participant[];
  recording_url?: string;
  transcription_status?: 'pending' | 'processing' | 'completed' | 'error';
}

export interface Participant {
  id: string;
  name: string;
  email: string;
  role?: string;
  joined_at?: string;
  left_at?: string;
}

export interface TranscriptionSegment {
  id: string;
  speaker: string;
  text: string;
  start_time: number;
  end_time: number;
  confidence: number;
}

export interface Transcription {
  meeting_id: string;
  language: string;
  segments: TranscriptionSegment[];
  full_text: string;
}

export interface ActionItem {
  id: string;
  text: string;
  assignee?: string;
  due_date?: string;
  priority: 'low' | 'medium' | 'high' | 'critical';
  completed: boolean;
}

export interface MeetingAnalysis {
  meeting_id: string;
  sentiment: {
    overall: number;
    timeline: Array<{ timestamp: number; sentiment: number }>;
  };
  action_items: ActionItem[];
  decisions: Decision[];
  topics: Topic[];
  speakers: SpeakerAnalysis[];
}

export interface Decision {
  id: string;
  text: string;
  category: string;
  timestamp: number;
}

export interface Topic {
  id: string;
  name: string;
  keywords: string[];
  relevance: number;
}

export interface SpeakerAnalysis {
  speaker_id: string;
  name: string;
  talk_time_seconds: number;
  talk_time_percentage: number;
  word_count: number;
}

export interface CreateMeetingRequest {
  title: string;
  scheduled_at?: string;
  duration_minutes?: number;
  participants: Array<{ email: string; name?: string }>;
  settings?: {
    record?: boolean;
    transcribe?: boolean;
    language?: string;
  };
}

export interface Workflow {
  id: string;
  name: string;
  enabled: boolean;
  trigger: {
    type: string;
    config: Record<string, any>;
  };
  actions: Array<{
    type: string;
    config: Record<string, any>;
  }>;
}

export interface CreateWorkflowRequest {
  name: string;
  trigger: {
    type: string;
    config: Record<string, any>;
  };
  actions: Array<{
    type: string;
    config: Record<string, any>;
  }>;
}

export interface TranslationResult {
  translated_text: string;
  source_language: string;
  target_language: string;
  confidence: number;
}

export interface PredictionResult {
  success_probability: number;
  confidence: number;
  factors: Array<{
    name: string;
    impact: number;
    direction: 'positive' | 'negative';
  }>;
  recommendations: string[];
}

// Main SDK Class

export class RufloClient {
  private http: AxiosInstance;
  private config: RufloConfig;

  constructor(config: RufloConfig) {
    this.config = {
      baseURL: 'https://api.ruflo.io/v1',
      timeout: 30000,
      ...config,
    };

    this.http = axios.create({
      baseURL: this.config.baseURL,
      timeout: this.config.timeout,
      headers: {
        'Authorization': `Bearer ${this.config.apiKey}`,
        'Content-Type': 'application/json',
      },
    });

    // Response interceptor for error handling
    this.http.interceptors.response.use(
      (response) => response,
      (error) => {
        if (error.response) {
          const { status, data } = error.response;
          throw new RufloError(data.message || 'API Error', status, data.code);
        }
        throw error;
      }
    );
  }

  // Meetings API

  async listMeetings(params?: {
    org_id?: string;
    status?: string;
    from?: string;
    to?: string;
    limit?: number;
    offset?: number;
  }): Promise<{ meetings: Meeting[]; total: number }> {
    const response = await this.http.get('/meetings', { params });
    return response.data;
  }

  async getMeeting(meetingId: string): Promise<Meeting> {
    const response = await this.http.get(`/meetings/${meetingId}`);
    return response.data;
  }

  async createMeeting(request: CreateMeetingRequest): Promise<Meeting> {
    const response = await this.http.post('/meetings', request);
    return response.data;
  }

  async updateMeeting(meetingId: string, updates: Partial<Meeting>): Promise<Meeting> {
    const response = await this.http.put(`/meetings/${meetingId}`, updates);
    return response.data;
  }

  async deleteMeeting(meetingId: string): Promise<void> {
    await this.http.delete(`/meetings/${meetingId}`);
  }

  // Transcriptions API

  async getTranscription(meetingId: string): Promise<Transcription> {
    const response = await this.http.get(`/meetings/${meetingId}/transcription`);
    return response.data;
  }

  async searchTranscriptions(query: string, filters?: {
    org_id?: string;
    from_date?: string;
    to_date?: string;
  }): Promise<{ results: TranscriptionSegment[]; total: number }> {
    const response = await this.http.post('/transcriptions/search', {
      query,
      ...filters,
    });
    return response.data;
  }

  // Analysis API

  async getAnalysis(meetingId: string): Promise<MeetingAnalysis> {
    const response = await this.http.get(`/meetings/${meetingId}/analysis`);
    return response.data;
  }

  async generateSummary(meetingId: string, options?: {
    style?: 'brief' | 'detailed' | 'bullet_points';
    max_length?: number;
  }): Promise<{ summary: string }> {
    const response = await this.http.post(`/meetings/${meetingId}/summarize`, options);
    return response.data;
  }

  // Workflows API

  async listWorkflows(): Promise<Workflow[]> {
    const response = await this.http.get('/workflows');
    return response.data.workflows;
  }

  async createWorkflow(request: CreateWorkflowRequest): Promise<Workflow> {
    const response = await this.http.post('/workflows', request);
    return response.data;
  }

  async executeWorkflow(workflowId: string, data?: Record<string, any>): Promise<void> {
    await this.http.post(`/workflows/${workflowId}/execute`, data);
  }

  // Predictions API

  async predictMeetingOutcome(params: {
    meeting_type: string;
    scheduled_duration: number;
    participants: string[];
    agenda_items: string[];
  }): Promise<PredictionResult> {
    const response = await this.http.post('/predict/outcome', params);
    return response.data;
  }

  // Translation API

  async translate(text: string, targetLanguage: string, sourceLanguage?: string): Promise<TranslationResult> {
    const response = await this.http.post('/translate', {
      text,
      target_language: targetLanguage,
      source_language: sourceLanguage,
    });
    return response.data;
  }

  async detectLanguage(text: string): Promise<{ language: string; confidence: number }> {
    const response = await this.http.post('/detect-language', { text });
    return {
      language: response.data.detected_language,
      confidence: response.data.confidence,
    };
  }

  // Real-time Transcription WebSocket

  connectToRealtimeTranscription(meetingId: string, onMessage: (segment: TranscriptionSegment) => void): WebSocket {
    const wsUrl = `${this.config.baseURL?.replace('https', 'wss')}/meetings/${meetingId}/transcription/stream`;
    const ws = new WebSocket(wsUrl, {
      headers: {
        'Authorization': `Bearer ${this.config.apiKey}`,
      },
    });

    ws.on('message', (data: WebSocket.Data) => {
      try {
        const segment = JSON.parse(data.toString());
        onMessage(segment);
      } catch (error) {
        console.error('Failed to parse WebSocket message:', error);
      }
    });

    return ws;
  }
}

// Error Class

export class RufloError extends Error {
  constructor(
    message: string,
    public statusCode: number,
    public code: string
  ) {
    super(message);
    this.name = 'RufloError';
  }
}

// Export default
export default RufloClient;
