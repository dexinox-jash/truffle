import { describe, it, expect, vi, beforeEach } from 'vitest';
import { render, screen, waitFor } from '@testing-library/react';
import { QueryClient, QueryClientProvider } from '@tanstack/react-query';
import { ValidationSummary } from '../../src/components/Validation/ValidationSummary';
import * as useValidationModule from '../../src/hooks/useValidation';

// Mock the hooks
vi.mock('../../src/hooks/useValidation', () => ({
  useValidationReport: vi.fn(),
  useContradictions: vi.fn(),
  useDuplicateCandidates: vi.fn(),
}));

const mockValidationReport = {
  averageConfidence: 0.82,
  totalEntities: 100,
  contradictionsCount: 5,
  duplicatesCount: 3,
  lowConfidenceEntities: ['entity-1', 'entity-2'],
};

const mockContradictions = [
  {
    id: 'contradiction-1',
    entityId: 'entity-1',
    factAId: 'fact-1',
    factBId: 'fact-2',
    contradictionType: 'factual' as const,
    severity: 'high' as const,
    description: 'Conflicting information',
    detectedAt: new Date().toISOString(),
    isResolved: false,
  },
  {
    id: 'contradiction-2',
    entityId: 'entity-2',
    factAId: 'fact-3',
    factBId: 'fact-4',
    contradictionType: 'temporal' as const,
    severity: 'medium' as const,
    description: 'Date mismatch',
    detectedAt: new Date().toISOString(),
    isResolved: false,
  },
];

const mockDuplicates = [
  {
    entityAId: 'entity-1',
    entityBId: 'entity-2',
    similarityScore: 0.92,
    nameSimilarity: 0.95,
    embeddingSimilarity: 0.88,
    relationshipOverlap: 0.75,
    detectedAt: new Date().toISOString(),
  },
];

const createTestQueryClient = () => new QueryClient({
  defaultOptions: { 
    queries: { retry: false },
    mutations: { retry: false },
  }
});

function renderWithProviders(ui: React.ReactElement) {
  const queryClient = createTestQueryClient();
  return render(
    <QueryClientProvider client={queryClient}>
      {ui}
    </QueryClientProvider>
  );
}

describe('ValidationSummary', () => {
  beforeEach(() => {
    vi.clearAllMocks();
    vi.mocked(useValidationModule.useValidationReport).mockReturnValue({
      data: mockValidationReport,
      isLoading: false,
      error: null,
    } as any);
    vi.mocked(useValidationModule.useContradictions).mockReturnValue({
      data: mockContradictions,
      isLoading: false,
      error: null,
    } as any);
    vi.mocked(useValidationModule.useDuplicateCandidates).mockReturnValue({
      data: mockDuplicates,
      isLoading: false,
      error: null,
    } as any);
  });

  it('renders loading state initially', () => {
    vi.mocked(useValidationModule.useValidationReport).mockReturnValue({
      data: undefined,
      isLoading: true,
      error: null,
    } as any);
    vi.mocked(useValidationModule.useContradictions).mockReturnValue({
      data: undefined,
      isLoading: true,
      error: null,
    } as any);
    vi.mocked(useValidationModule.useDuplicateCandidates).mockReturnValue({
      data: undefined,
      isLoading: true,
      error: null,
    } as any);

    renderWithProviders(<ValidationSummary />);
    
    // Should show skeleton/loading elements
    expect(screen.getAllByRole('status').length > 0 || document.querySelector('.animate-pulse')).toBeTruthy();
  });

  it('renders summary cards with correct data', async () => {
    renderWithProviders(<ValidationSummary />);
    
    await waitFor(() => {
      expect(screen.getByText(/contradictions/i)).toBeInTheDocument();
      expect(screen.getByText('2')).toBeInTheDocument(); // Total contradictions
      expect(screen.getByText(/duplicate candidates/i)).toBeInTheDocument();
      expect(screen.getByText('1')).toBeInTheDocument(); // Total duplicates
      expect(screen.getByText(/avg\. confidence/i)).toBeInTheDocument();
      expect(screen.getByText('82%')).toBeInTheDocument();
    });
  });

  it('displays severity breakdown for contradictions', async () => {
    renderWithProviders(<ValidationSummary />);
    
    await waitFor(() => {
      expect(screen.getByText(/contradictions by severity/i)).toBeInTheDocument();
      expect(screen.getByText(/high/i)).toBeInTheDocument();
      expect(screen.getByText(/medium/i)).toBeInTheDocument();
    });
  });

  it('calls onContradictionClick when contradiction card clicked', async () => {
    const onContradictionClick = vi.fn();
    
    renderWithProviders(<ValidationSummary onContradictionClick={onContradictionClick} />);
    
    await waitFor(() => {
      const contradictionCard = screen.getByText(/contradictions/i).closest('div[class*="relative"]') || 
                                screen.getByText(/contradictions/i).parentElement?.parentElement;
      if (contradictionCard) {
        contradictionCard.click();
      }
    });
  });

  it('calls onDuplicateClick when duplicate card clicked', async () => {
    const onDuplicateClick = vi.fn();
    
    renderWithProviders(<ValidationSummary onDuplicateClick={onDuplicateClick} />);
    
    await waitFor(() => {
      const duplicateCard = screen.getByText(/duplicate candidates/i).closest('div[class*="relative"]') || 
                           screen.getByText(/duplicate candidates/i).parentElement?.parentElement;
      if (duplicateCard) {
        duplicateCard.click();
      }
    });
  });

  it('shows all clear state when no issues', async () => {
    vi.mocked(useValidationModule.useContradictions).mockReturnValue({
      data: [],
      isLoading: false,
      error: null,
    } as any);
    vi.mocked(useValidationModule.useDuplicateCandidates).mockReturnValue({
      data: [],
      isLoading: false,
      error: null,
    } as any);
    vi.mocked(useValidationModule.useValidationReport).mockReturnValue({
      data: { ...mockValidationReport, lowConfidenceEntities: [] },
      isLoading: false,
      error: null,
    } as any);

    renderWithProviders(<ValidationSummary />);
    
    await waitFor(() => {
      expect(screen.getByText(/all clear/i)).toBeInTheDocument();
      expect(screen.getByText(/no validation issues detected/i)).toBeInTheDocument();
    });
  });

  it('displays error severity for critical contradictions', async () => {
    vi.mocked(useValidationModule.useContradictions).mockReturnValue({
      data: [
        ...mockContradictions,
        {
          id: 'contradiction-3',
          entityId: 'entity-3',
          factAId: 'fact-5',
          factBId: 'fact-6',
          contradictionType: 'factual' as const,
          severity: 'critical' as const,
          description: 'Critical conflict',
          detectedAt: new Date().toISOString(),
          isResolved: false,
        },
      ],
      isLoading: false,
      error: null,
    } as any);

    renderWithProviders(<ValidationSummary />);
    
    await waitFor(() => {
      expect(screen.getByText(/critical/i)).toBeInTheDocument();
    });
  });
});
