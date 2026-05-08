import React, { ReactElement } from 'react';
import { render, RenderOptions } from '@testing-library/react';
import { QueryClient, QueryClientProvider } from '@tanstack/react-query';
import { vi } from 'vitest';

/**
 * Create a test query client with default options
 */
export function createTestQueryClient(): QueryClient {
  return new QueryClient({
    defaultOptions: {
      queries: {
        retry: false,
        staleTime: 0,
        gcTime: 0,
      },
      mutations: {
        retry: false,
      },
    },
  });
}

interface ProvidersProps {
  children: React.ReactNode;
}

/**
 * All providers wrapper for tests
 */
function AllTheProviders({ children }: ProvidersProps) {
  const queryClient = createTestQueryClient();

  return (
    <QueryClientProvider client={queryClient}>
      {children}
    </QueryClientProvider>
  );
}

/**
 * Custom render function that wraps components with providers
 */
export function renderWithProviders(
  ui: ReactElement,
  options?: Omit<RenderOptions, 'wrapper'>
) {
  return render(ui, { wrapper: AllTheProviders, ...options });
}

/**
 * Mock data factories
 */
export const mockFactories = {
  entity: (overrides = {}) => ({
    id: 'entity-1',
    name: 'Test Entity',
    entityType: 'person' as const,
    slug: 'test-entity',
    description: 'A test entity',
    metadata: {},
    extractionConfidence: 0.9,
    privacyLevel: 'shared' as const,
    createdAt: new Date().toISOString(),
    updatedAt: new Date().toISOString(),
    ...overrides,
  }),

  entitySummary: (overrides = {}) => ({
    id: 'entity-1',
    name: 'Test Entity',
    entityType: 'person' as const,
    description: 'A test entity',
    extractionConfidence: 0.9,
    ...overrides,
  }),

  meeting: (overrides = {}) => ({
    id: 'meeting-1',
    title: 'Test Meeting',
    meetingType: 'team' as const,
    startedAt: new Date().toISOString(),
    endedAt: new Date(Date.now() + 3600000).toISOString(),
    duration: 60,
    participants: [
      { id: 'p1', name: 'Alice', email: 'alice@example.com' },
      { id: 'p2', name: 'Bob', email: 'bob@example.com' },
    ],
    transcript: 'This is a test transcript',
    summary: 'Meeting summary',
    extractionStatus: 'completed' as const,
    extractedAt: new Date().toISOString(),
    createdAt: new Date().toISOString(),
    updatedAt: new Date().toISOString(),
    ...overrides,
  }),

  relationship: (overrides = {}) => ({
    id: 'rel-1',
    sourceId: 'entity-1',
    targetId: 'entity-2',
    relationType: 'works_for',
    confidence: 0.85,
    ...overrides,
  }),

  contradiction: (overrides = {}) => ({
    id: 'contradiction-1',
    entityId: 'entity-1',
    factAId: 'fact-1',
    factBId: 'fact-2',
    contradictionType: 'factual' as const,
    severity: 'high' as const,
    description: 'Conflicting information detected',
    detectedAt: new Date().toISOString(),
    isResolved: false,
    ...overrides,
  }),

  duplicateCandidate: (overrides = {}) => ({
    entityAId: 'entity-1',
    entityBId: 'entity-2',
    similarityScore: 0.92,
    nameSimilarity: 0.95,
    embeddingSimilarity: 0.88,
    relationshipOverlap: 0.75,
    detectedAt: new Date().toISOString(),
    ...overrides,
  }),

  validationReport: (overrides = {}) => ({
    averageConfidence: 0.82,
    totalEntities: 100,
    contradictionsCount: 5,
    duplicatesCount: 3,
    lowConfidenceEntities: ['entity-1', 'entity-2'],
    ...overrides,
  }),
};

/**
 * Mock hook responses
 */
export function mockHookResponse<T>(data: T, isLoading = false, error: Error | null = null) {
  return { data, isLoading, error };
}

/**
 * Wait for a specified amount of time
 */
export function wait(ms: number): Promise<void> {
  return new Promise((resolve) => setTimeout(resolve, ms));
}

/**
 * Create a mock mutation result
 */
export function createMockMutation(options: {
  isPending?: boolean;
  isSuccess?: boolean;
  isError?: boolean;
  error?: Error | null;
  mutateAsync?: vi.Mock;
} = {}) {
  return {
    mutate: vi.fn(),
    mutateAsync: options.mutateAsync || vi.fn().mockResolvedValue(undefined),
    isPending: options.isPending || false,
    isSuccess: options.isSuccess || false,
    isError: options.isError || false,
    error: options.error || null,
    reset: vi.fn(),
  };
}
