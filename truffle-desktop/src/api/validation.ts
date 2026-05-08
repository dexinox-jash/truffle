import { invokeCommand } from './client';
import type { 
  Contradiction, 
  ConfidenceScore, 
  DuplicateCandidate,
  EntitySummary
} from '../types/knowledge-graph';

export const validationApi = {
  // Contradiction detection
  getContradictions: (resolved?: boolean) =>
    invokeCommand<Contradiction[]>('get_contradictions', { resolved }),

  getEntityContradictions: (entityId: string) =>
    invokeCommand<Contradiction[]>('get_entity_contradictions', { entityId }),

  resolveContradiction: (contradictionId: string, resolution: string) =>
    invokeCommand<void>('resolve_contradiction', { contradictionId, resolution }),

  detectContradictions: (entityId?: string) =>
    invokeCommand<Contradiction[]>('detect_contradictions', { entityId }),

  // Confidence scoring
  getConfidenceScore: (entityId: string) =>
    invokeCommand<ConfidenceScore>('get_confidence_score', { entityId }),

  recalculateConfidence: (entityId: string) =>
    invokeCommand<ConfidenceScore>('recalculate_confidence', { entityId }),

  // Duplicate detection
  getDuplicateCandidates: (minSimilarity?: number) =>
    invokeCommand<DuplicateCandidate[]>('get_duplicate_candidates', { minSimilarity }),

  getEntityDuplicates: (entityId: string) =>
    invokeCommand<DuplicateCandidate[]>('get_entity_duplicates', { entityId }),

  markNotDuplicate: (entityAId: string, entityBId: string) =>
    invokeCommand<void>('mark_not_duplicate', { entityAId, entityBId }),

  runDuplicateDetection: (threshold?: number) =>
    invokeCommand<DuplicateCandidate[]>('run_duplicate_detection', { threshold }),

  // Validation reports
  getValidationReport: () =>
    invokeCommand<{
      totalContradictions: number;
      unresolvedContradictions: number;
      duplicateCandidates: number;
      averageConfidence: number;
      lowConfidenceEntities: EntitySummary[];
    }>('get_validation_report', {}),

  // Batch validation
  validateAll: () =>
    invokeCommand<{
      contradictionsFound: number;
      duplicatesFound: number;
      entitiesProcessed: number;
    }>('validate_all', {}),
};
