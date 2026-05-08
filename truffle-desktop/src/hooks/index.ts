// Export all custom hooks

export { useKeyboard, useShortcut, useEscape, useEnter } from './useKeyboard';
export { useDebounce, useDebouncedCallback } from './useDebounce';
export { useLocalStorage } from './useLocalStorage';
export { useCompilation } from './useCompilation';

// Knowledge Graph hooks
export {
  useEntities,
  useAllEntities,
  useEntity,
  useEntityWithRelationships,
  useSearchEntities,
  useSemanticSearch,
  useEntityGraph,
  useCreateEntity,
  useUpdateEntity,
  useDeleteEntity,
  useMergeEntities,
} from './useEntities';

export {
  useRelationships,
  useAllRelationships,
  useRelationship,
  useEntityRelationships,
  useRelationshipTypes,
  useInferRelationships,
  useCreateRelationship,
  useUpdateRelationship,
  useDeleteRelationship,
  useDeleteRelationshipsByEntity,
} from './useRelationships';

export {
  useContradictions,
  useEntityContradictions,
  useConfidenceScore,
  useDuplicateCandidates,
  useEntityDuplicates,
  useValidationReport,
  useResolveContradiction,
  useDetectContradictions,
  useRecalculateConfidence,
  useMarkNotDuplicate,
  useRunDuplicateDetection,
  useValidateAll,
} from './useValidation';

// Meeting hooks
export {
  useMeetings,
  useMeeting,
  useCreateMeeting,
  useUpdateMeeting,
  useDeleteMeeting,
  useImportMeeting,
  useExtractFromMeeting,
  useExtractionProgress,
  useExtractionResult,
  useCancelExtraction,
  useDecisions,
  useDecision,
  useActionItems,
  useActionItem,
  useUpdateActionItemStatus,
} from './useMeetings';
