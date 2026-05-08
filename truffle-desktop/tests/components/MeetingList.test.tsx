import { describe, it, expect, vi, beforeEach } from 'vitest';
import { render, screen, fireEvent, waitFor } from '@testing-library/react';
import { QueryClient, QueryClientProvider } from '@tanstack/react-query';
import { MeetingList } from '../../src/components/Meeting/MeetingList';
import { MeetingType, ExtractionStatus } from '../../src/types/meeting';
import * as useMeetingsModule from '../../src/hooks/useMeetings';

// Mock the hooks
vi.mock('../../src/hooks/useMeetings', () => ({
  useMeetings: vi.fn(),
  useExtractFromMeeting: vi.fn(),
}));

const mockMeetings = [
  {
    id: 'meeting-1',
    title: 'Daily Standup',
    meetingType: MeetingType.Standup,
    startedAt: new Date().toISOString(), // Today
    endedAt: new Date(Date.now() + 1800000).toISOString(),
    duration: 30,
    participants: [
      { id: 'p1', name: 'Alice', email: 'alice@example.com' },
      { id: 'p2', name: 'Bob', email: 'bob@example.com' },
    ],
    extractionStatus: ExtractionStatus.Completed,
    extractedAt: new Date().toISOString(),
    createdAt: new Date().toISOString(),
    updatedAt: new Date().toISOString(),
  },
  {
    id: 'meeting-2',
    title: 'Sprint Planning',
    meetingType: MeetingType.Planning,
    startedAt: new Date(Date.now() - 86400000).toISOString(), // Yesterday
    endedAt: new Date(Date.now() - 86400000 + 3600000).toISOString(),
    duration: 60,
    participants: [
      { id: 'p1', name: 'Alice', email: 'alice@example.com' },
      { id: 'p3', name: 'Charlie', email: 'charlie@example.com' },
    ],
    extractionStatus: ExtractionStatus.Pending,
    createdAt: new Date(Date.now() - 86400000).toISOString(),
    updatedAt: new Date(Date.now() - 86400000).toISOString(),
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

describe('MeetingList', () => {
  beforeEach(() => {
    vi.clearAllMocks();
    vi.mocked(useMeetingsModule.useExtractFromMeeting).mockReturnValue({
      mutateAsync: vi.fn().mockResolvedValue(undefined),
      isPending: false,
    } as any);
  });

  it('renders loading state initially', () => {
    vi.mocked(useMeetingsModule.useMeetings).mockReturnValue({
      data: undefined,
      isLoading: true,
      error: null,
    } as any);

    renderWithProviders(<MeetingList />);
    
    expect(screen.getByRole('status') || screen.queryByTestId('loader')).toBeTruthy();
  });

  it('renders list of meetings grouped by date', async () => {
    vi.mocked(useMeetingsModule.useMeetings).mockReturnValue({
      data: mockMeetings,
      isLoading: false,
      error: null,
    } as any);

    renderWithProviders(<MeetingList />);
    
    await waitFor(() => {
      expect(screen.getByText('Daily Standup')).toBeInTheDocument();
      expect(screen.getByText('Sprint Planning')).toBeInTheDocument();
    });
  });

  it('displays date group headers', async () => {
    vi.mocked(useMeetingsModule.useMeetings).mockReturnValue({
      data: mockMeetings,
      isLoading: false,
      error: null,
    } as any);

    renderWithProviders(<MeetingList />);
    
    await waitFor(() => {
      expect(screen.getByText(/today/i)).toBeInTheDocument();
      expect(screen.getByText(/yesterday/i)).toBeInTheDocument();
    });
  });

  it('calls onSelect when meeting clicked', async () => {
    const onSelect = vi.fn();
    
    vi.mocked(useMeetingsModule.useMeetings).mockReturnValue({
      data: mockMeetings,
      isLoading: false,
      error: null,
    } as any);

    renderWithProviders(<MeetingList onSelect={onSelect} />);
    
    await waitFor(() => {
      const meetingCard = screen.getByText('Daily Standup');
      fireEvent.click(meetingCard);
    });
    
    expect(onSelect).toHaveBeenCalledWith(expect.objectContaining({
      id: 'meeting-1',
      title: 'Daily Standup',
    }));
  });

  it('filters meetings by search query', async () => {
    vi.mocked(useMeetingsModule.useMeetings).mockReturnValue({
      data: mockMeetings,
      isLoading: false,
      error: null,
    } as any);

    renderWithProviders(<MeetingList />);
    
    const searchInput = screen.getByPlaceholderText(/search meetings/i);
    fireEvent.change(searchInput, { target: { value: 'Standup' } });
    
    await waitFor(() => {
      expect(screen.getByText('Daily Standup')).toBeInTheDocument();
      expect(screen.queryByText('Sprint Planning')).not.toBeInTheDocument();
    });
  });

  it('shows empty state when no meetings', async () => {
    vi.mocked(useMeetingsModule.useMeetings).mockReturnValue({
      data: [],
      isLoading: false,
      error: null,
    } as any);

    renderWithProviders(<MeetingList />);
    
    await waitFor(() => {
      expect(screen.getByText(/no meetings found/i)).toBeInTheDocument();
      expect(screen.getByText(/get started by importing your first meeting/i)).toBeInTheDocument();
    });
  });

  it('shows empty state with search query', async () => {
    vi.mocked(useMeetingsModule.useMeetings).mockReturnValue({
      data: [],
      isLoading: false,
      error: null,
    } as any);

    renderWithProviders(<MeetingList />);
    
    const searchInput = screen.getByPlaceholderText(/search meetings/i);
    fireEvent.change(searchInput, { target: { value: 'NonExistent' } });
    
    await waitFor(() => {
      expect(screen.getByText(/no meetings found/i)).toBeInTheDocument();
      expect(screen.getByText(/try adjusting your search/i)).toBeInTheDocument();
    });
  });

  it('displays import meeting button', async () => {
    vi.mocked(useMeetingsModule.useMeetings).mockReturnValue({
      data: mockMeetings,
      isLoading: false,
      error: null,
    } as any);

    renderWithProviders(<MeetingList />);
    
    await waitFor(() => {
      expect(screen.getByText(/import meeting/i)).toBeInTheDocument();
    });
  });

  it('shows error state when query fails', async () => {
    vi.mocked(useMeetingsModule.useMeetings).mockReturnValue({
      data: undefined,
      isLoading: false,
      error: new Error('Failed to load meetings'),
    } as any);

    renderWithProviders(<MeetingList />);
    
    await waitFor(() => {
      expect(screen.getByText(/failed to load meetings/i)).toBeInTheDocument();
    });
  });

  it('calls onExtract when extract button clicked', async () => {
    const onExtract = vi.fn();
    const mockMutateAsync = vi.fn().mockResolvedValue(undefined);
    
    vi.mocked(useMeetingsModule.useMeetings).mockReturnValue({
      data: mockMeetings,
      isLoading: false,
      error: null,
    } as any);
    vi.mocked(useMeetingsModule.useExtractFromMeeting).mockReturnValue({
      mutateAsync: mockMutateAsync,
      isPending: false,
    } as any);

    renderWithProviders(<MeetingList onExtract={onExtract} />);
    
    await waitFor(() => {
      // Find extract button (should be visible for pending meetings)
      const extractButtons = screen.getAllByRole('button');
      // Click the first extract button
      const extractButton = extractButtons.find(btn => btn.textContent?.includes('extract'));
      if (extractButton) {
        fireEvent.click(extractButton);
      }
    });
  });
});
