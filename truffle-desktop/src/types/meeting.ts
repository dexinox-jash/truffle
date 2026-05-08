// Meeting and Extraction types for Truffle Desktop

import type { Entity, Relationship, Decision, ActionItem } from './knowledge-graph';

export interface Meeting {
  id: string;
  title: string;
  meetingType: MeetingType;
  startedAt: string;
  endedAt?: string;
  duration?: number; // in minutes
  participants: Participant[];
  transcript?: string;
  summary?: string;
  extractionStatus: ExtractionStatus;
  extractedAt?: string;
  createdAt: string;
  updatedAt: string;
}

export enum MeetingType {
  Standup = 'standup',
  Planning = 'planning',
  Review = 'review',
  Retrospective = 'retrospective',
  OneOnOne = 'one_on_one',
  Client = 'client',
  Team = 'team',
  Workshop = 'workshop',
  Other = 'other',
}

export enum ExtractionStatus {
  Pending = 'pending',
  Extracting = 'extracting',
  Completed = 'completed',
  Failed = 'failed',
}

export interface Participant {
  id: string;
  name: string;
  email?: string;
  avatarUrl?: string;
  isExternal?: boolean;
}

export interface MeetingFilter {
  search?: string;
  from?: Date;
  to?: Date;
  hasExtraction?: boolean;
  meetingType?: MeetingType;
  participantId?: string;
}

export interface ExtractionResult {
  meetingId: string;
  status: 'success' | 'partial' | 'failed';
  entities: Entity[];
  relationships: Relationship[];
  decisions: Decision[];
  actionItems: ActionItem[];
  extractedAt: string;
  error?: string;
}

export interface ExtractionProgress {
  stage: ExtractionStage;
  percent: number;
  message?: string;
}

export enum ExtractionStage {
  Preprocessing = 'preprocessing',
  Extracting = 'extracting',
  Resolving = 'resolving',
  Building = 'building',
  Completed = 'completed',
}

export interface EntityMention {
  entityId: string;
  startIndex: number;
  endIndex: number;
  text: string;
  confidence: number;
}

export interface MeetingSummary {
  keyPoints: string[];
  decisions: string[];
  actionItems: string[];
  topics: string[];
}

// Input types
export interface CreateMeetingInput {
  title: string;
  meetingType: MeetingType;
  startedAt: string;
  endedAt?: string;
  participants: string[];
  transcript?: string;
}

export interface UpdateMeetingInput {
  title?: string;
  summary?: string;
  transcript?: string;
}

export interface ImportMeetingInput {
  source: 'file' | 'recording' | 'calendar';
  filePath?: string;
  calendarEventId?: string;
  metadata?: Record<string, unknown>;
}
