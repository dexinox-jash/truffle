import { invokeCommand } from './client';
import type { 
  Meeting, 
  MeetingFilter,
  CreateMeetingInput,
  UpdateMeetingInput,
  ImportMeetingInput,
  ExtractionResult,
  ExtractionProgress,
  Decision,
  ActionItem,
} from '../types/meeting';
import type { PaginationParams, PaginatedResponse } from './client';

export const meetingsApi = {
  getMeetings: (filter?: MeetingFilter, pagination?: PaginationParams) =>
    invokeCommand<PaginatedResponse<Meeting>>('get_meetings', { filter, ...pagination }),

  getAllMeetings: (filter?: MeetingFilter) =>
    invokeCommand<Meeting[]>('get_all_meetings', { filter }),

  getMeeting: (id: string) =>
    invokeCommand<Meeting>('get_meeting', { id }),

  createMeeting: (input: CreateMeetingInput) =>
    invokeCommand<Meeting>('create_meeting', { input }),

  updateMeeting: (id: string, input: UpdateMeetingInput) =>
    invokeCommand<Meeting>('update_meeting', { id, input }),

  deleteMeeting: (id: string) =>
    invokeCommand<void>('delete_meeting', { id }),

  importMeeting: (input: ImportMeetingInput) =>
    invokeCommand<Meeting>('import_meeting', { input }),

  // Extraction
  extractFromMeeting: (meetingId: string) =>
    invokeCommand<ExtractionResult>('extract_from_meeting', { meetingId }),

  getExtractionProgress: (meetingId: string) =>
    invokeCommand<ExtractionProgress>('get_extraction_progress', { meetingId }),

  getExtractionResult: (meetingId: string) =>
    invokeCommand<ExtractionResult>('get_extraction_result', { meetingId }),

  cancelExtraction: (meetingId: string) =>
    invokeCommand<void>('cancel_extraction', { meetingId }),

  // Decisions
  getDecisions: (meetingId?: string) =>
    invokeCommand<Decision[]>('get_decisions', { meetingId }),

  getDecision: (id: string) =>
    invokeCommand<Decision>('get_decision', { id }),

  // Action Items
  getActionItems: (meetingId?: string, status?: string) =>
    invokeCommand<ActionItem[]>('get_action_items', { meetingId, status }),

  getActionItem: (id: string) =>
    invokeCommand<ActionItem>('get_action_item', { id }),

  updateActionItemStatus: (id: string, status: string) =>
    invokeCommand<ActionItem>('update_action_item_status', { id, status }),
};
