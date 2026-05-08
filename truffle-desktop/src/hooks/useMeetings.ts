import { useQuery, useMutation, useQueryClient } from '@tanstack/react-query';
import { meetingsApi } from '../api/meetings';
import type { MeetingFilter, CreateMeetingInput, UpdateMeetingInput, ImportMeetingInput } from '../types/meeting';

const MEETINGS_KEY = 'meetings';
const MEETING_KEY = 'meeting';
const EXTRACTION_KEY = 'extraction';
const DECISIONS_KEY = 'decisions';
const ACTION_ITEMS_KEY = 'actionItems';

// Meetings
export function useMeetings(filter?: MeetingFilter) {
  return useQuery({
    queryKey: [MEETINGS_KEY, filter],
    queryFn: () => meetingsApi.getAllMeetings(filter),
  });
}

export function useMeeting(id: string) {
  return useQuery({
    queryKey: [MEETING_KEY, id],
    queryFn: () => meetingsApi.getMeeting(id),
    enabled: !!id,
  });
}

export function useCreateMeeting() {
  const queryClient = useQueryClient();
  return useMutation({
    mutationFn: meetingsApi.createMeeting,
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: [MEETINGS_KEY] });
    },
  });
}

export function useUpdateMeeting() {
  const queryClient = useQueryClient();
  return useMutation({
    mutationFn: ({ id, input }: { id: string; input: UpdateMeetingInput }) =>
      meetingsApi.updateMeeting(id, input),
    onSuccess: (_, { id }) => {
      queryClient.invalidateQueries({ queryKey: [MEETING_KEY, id] });
      queryClient.invalidateQueries({ queryKey: [MEETINGS_KEY] });
    },
  });
}

export function useDeleteMeeting() {
  const queryClient = useQueryClient();
  return useMutation({
    mutationFn: meetingsApi.deleteMeeting,
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: [MEETINGS_KEY] });
    },
  });
}

export function useImportMeeting() {
  const queryClient = useQueryClient();
  return useMutation({
    mutationFn: meetingsApi.importMeeting,
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: [MEETINGS_KEY] });
    },
  });
}

// Extraction
export function useExtractFromMeeting() {
  const queryClient = useQueryClient();
  return useMutation({
    mutationFn: meetingsApi.extractFromMeeting,
    onSuccess: (_, meetingId) => {
      queryClient.invalidateQueries({ queryKey: [MEETING_KEY, meetingId] });
      queryClient.invalidateQueries({ queryKey: [EXTRACTION_KEY, meetingId] });
      queryClient.invalidateQueries({ queryKey: [MEETINGS_KEY] });
    },
  });
}

export function useExtractionProgress(meetingId: string) {
  return useQuery({
    queryKey: [EXTRACTION_KEY, 'progress', meetingId],
    queryFn: () => meetingsApi.getExtractionProgress(meetingId),
    enabled: !!meetingId,
    refetchInterval: (query) => {
      const data = query.state.data;
      if (data && data.percent >= 100) return false;
      return 1000; // Poll every second while extracting
    },
  });
}

export function useExtractionResult(meetingId: string) {
  return useQuery({
    queryKey: [EXTRACTION_KEY, 'result', meetingId],
    queryFn: () => meetingsApi.getExtractionResult(meetingId),
    enabled: !!meetingId,
  });
}

export function useCancelExtraction() {
  const queryClient = useQueryClient();
  return useMutation({
    mutationFn: meetingsApi.cancelExtraction,
    onSuccess: (_, meetingId) => {
      queryClient.invalidateQueries({ queryKey: [EXTRACTION_KEY, 'progress', meetingId] });
    },
  });
}

// Decisions
export function useDecisions(meetingId?: string) {
  return useQuery({
    queryKey: [DECISIONS_KEY, meetingId],
    queryFn: () => meetingsApi.getDecisions(meetingId),
    enabled: !meetingId || !!meetingId,
  });
}

export function useDecision(id: string) {
  return useQuery({
    queryKey: [DECISIONS_KEY, id],
    queryFn: () => meetingsApi.getDecision(id),
    enabled: !!id,
  });
}

// Action Items
export function useActionItems(meetingId?: string, status?: string) {
  return useQuery({
    queryKey: [ACTION_ITEMS_KEY, meetingId, status],
    queryFn: () => meetingsApi.getActionItems(meetingId, status),
    enabled: (!meetingId || !!meetingId) && (!status || !!status),
  });
}

export function useActionItem(id: string) {
  return useQuery({
    queryKey: [ACTION_ITEMS_KEY, id],
    queryFn: () => meetingsApi.getActionItem(id),
    enabled: !!id,
  });
}

export function useUpdateActionItemStatus() {
  const queryClient = useQueryClient();
  return useMutation({
    mutationFn: ({ id, status }: { id: string; status: string }) =>
      meetingsApi.updateActionItemStatus(id, status),
    onSuccess: (_, { id }) => {
      queryClient.invalidateQueries({ queryKey: [ACTION_ITEMS_KEY] });
      queryClient.invalidateQueries({ queryKey: [ACTION_ITEMS_KEY, id] });
    },
  });
}
