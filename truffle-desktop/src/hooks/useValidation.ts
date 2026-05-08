import { useQuery, useMutation, useQueryClient } from '@tanstack/react-query';
import { validationApi } from '../api/validation';

const CONTRADICTIONS_KEY = 'contradictions';
const DUPLICATE_CANDIDATES_KEY = 'duplicateCandidates';
const CONFIDENCE_KEY = 'confidence';
const VALIDATION_REPORT_KEY = 'validationReport';

export function useContradictions(resolved?: boolean) {
  return useQuery({
    queryKey: [CONTRADICTIONS_KEY, resolved],
    queryFn: () => validationApi.getContradictions(resolved),
  });
}

export function useEntityContradictions(entityId: string) {
  return useQuery({
    queryKey: [CONTRADICTIONS_KEY, 'entity', entityId],
    queryFn: () => validationApi.getEntityContradictions(entityId),
    enabled: !!entityId,
  });
}

export function useConfidenceScore(entityId: string) {
  return useQuery({
    queryKey: [CONFIDENCE_KEY, entityId],
    queryFn: () => validationApi.getConfidenceScore(entityId),
    enabled: !!entityId,
  });
}

export function useDuplicateCandidates(minSimilarity?: number) {
  return useQuery({
    queryKey: [DUPLICATE_CANDIDATES_KEY, minSimilarity],
    queryFn: () => validationApi.getDuplicateCandidates(minSimilarity),
  });
}

export function useEntityDuplicates(entityId: string) {
  return useQuery({
    queryKey: [DUPLICATE_CANDIDATES_KEY, 'entity', entityId],
    queryFn: () => validationApi.getEntityDuplicates(entityId),
    enabled: !!entityId,
  });
}

export function useValidationReport() {
  return useQuery({
    queryKey: [VALIDATION_REPORT_KEY],
    queryFn: () => validationApi.getValidationReport(),
  });
}

export function useResolveContradiction() {
  const queryClient = useQueryClient();
  return useMutation({
    mutationFn: ({ contradictionId, resolution }: { contradictionId: string; resolution: string }) =>
      validationApi.resolveContradiction(contradictionId, resolution),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: [CONTRADICTIONS_KEY] });
      queryClient.invalidateQueries({ queryKey: [VALIDATION_REPORT_KEY] });
    },
  });
}

export function useDetectContradictions() {
  const queryClient = useQueryClient();
  return useMutation({
    mutationFn: (entityId?: string) => validationApi.detectContradictions(entityId),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: [CONTRADICTIONS_KEY] });
      queryClient.invalidateQueries({ queryKey: [VALIDATION_REPORT_KEY] });
    },
  });
}

export function useRecalculateConfidence() {
  const queryClient = useQueryClient();
  return useMutation({
    mutationFn: (entityId: string) => validationApi.recalculateConfidence(entityId),
    onSuccess: (_, entityId) => {
      queryClient.invalidateQueries({ queryKey: [CONFIDENCE_KEY, entityId] });
    },
  });
}

export function useMarkNotDuplicate() {
  const queryClient = useQueryClient();
  return useMutation({
    mutationFn: ({ entityAId, entityBId }: { entityAId: string; entityBId: string }) =>
      validationApi.markNotDuplicate(entityAId, entityBId),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: [DUPLICATE_CANDIDATES_KEY] });
      queryClient.invalidateQueries({ queryKey: [VALIDATION_REPORT_KEY] });
    },
  });
}

export function useRunDuplicateDetection() {
  const queryClient = useQueryClient();
  return useMutation({
    mutationFn: (threshold?: number) => validationApi.runDuplicateDetection(threshold),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: [DUPLICATE_CANDIDATES_KEY] });
      queryClient.invalidateQueries({ queryKey: [VALIDATION_REPORT_KEY] });
    },
  });
}

export function useValidateAll() {
  const queryClient = useQueryClient();
  return useMutation({
    mutationFn: () => validationApi.validateAll(),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: [CONTRADICTIONS_KEY] });
      queryClient.invalidateQueries({ queryKey: [DUPLICATE_CANDIDATES_KEY] });
      queryClient.invalidateQueries({ queryKey: [VALIDATION_REPORT_KEY] });
    },
  });
}
