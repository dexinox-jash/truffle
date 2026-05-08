// Validation Dashboard Components
// Project Truffle - Phase 5.3 Frontend UI

export { ValidationSummary } from './ValidationSummary';
export { ContradictionList } from './ContradictionList';
export { ContradictionDetail } from './ContradictionDetail';
export { DuplicateList } from './DuplicateList';
export { ConfidencePanel } from './ConfidencePanel';
export { MergeModal } from './MergeModal';

// Type re-exports for convenience
export type {
  Contradiction,
  DuplicateCandidate,
  ConfidenceScore,
} from '../../types/knowledge-graph';

// Local type exports
export type { ResolutionInput } from './ContradictionDetail';
export type { MergeConflict } from './MergeModal';
