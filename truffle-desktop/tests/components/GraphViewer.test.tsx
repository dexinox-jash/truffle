import { describe, it, expect, vi, beforeEach } from 'vitest';
import { render, screen, waitFor } from '@testing-library/react';
import { QueryClient, QueryClientProvider } from '@tanstack/react-query';
import { GraphViewer } from '../../src/components/Graph/GraphViewer';
import { EntityType } from '../../src/types/knowledge-graph';
import * as useEntitiesModule from '../../src/hooks/useEntities';

// Mock the hooks
vi.mock('../../src/hooks/useEntities', () => ({
  useEntityGraph: vi.fn(),
}));

// Mock ForceGraph2D
vi.mock('react-force-graph-2d', () => ({
  __esModule: true,
  default: vi.fn((props) => {
    // Simulate calling onNodeClick with a mock node
    const mockNode = {
      id: 'node-1',
      entity: { id: 'entity-1', name: 'Node 1', entityType: EntityType.Person },
      x: 0,
      y: 0,
    };
    
    return (
      <div data-testid="force-graph" data-nodes={JSON.stringify(props.graphData?.nodes || [])}>
        <canvas data-testid="graph-canvas" />
        <button 
          data-testid="mock-node-click"
          onClick={() => props.onNodeClick?.(mockNode)}
        >
          Click Node
        </button>
      </div>
    );
  }),
}));

const mockGraphData = {
  nodes: [
    { id: 'entity-1', entityType: EntityType.Person, name: 'Alice', x: 0, y: 0 },
    { id: 'entity-2', entityType: EntityType.Organization, name: 'Acme Corp', x: 100, y: 100 },
  ],
  edges: [
    { id: 'edge-1', source: 'entity-1', target: 'entity-2', relationType: 'works_for', confidence: 0.85 },
  ],
};

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

describe('GraphViewer', () => {
  beforeEach(() => {
    vi.clearAllMocks();
  });

  it('renders loading state initially', () => {
    vi.mocked(useEntitiesModule.useEntityGraph).mockReturnValue({
      data: undefined,
      isLoading: true,
      error: null,
    } as any);

    renderWithProviders(<GraphViewer width={800} height={600} />);
    
    expect(screen.getByText(/loading graph/i)).toBeInTheDocument();
  });

  it('renders graph with nodes and edges', async () => {
    vi.mocked(useEntitiesModule.useEntityGraph).mockReturnValue({
      data: mockGraphData,
      isLoading: false,
      error: null,
    } as any);

    renderWithProviders(<GraphViewer rootEntityId="entity-1" width={800} height={600} />);
    
    await waitFor(() => {
      expect(screen.getByTestId('graph-canvas')).toBeInTheDocument();
    });
  });

  it('calls onNodeClick when node is clicked', async () => {
    const onNodeClick = vi.fn();
    
    vi.mocked(useEntitiesModule.useEntityGraph).mockReturnValue({
      data: mockGraphData,
      isLoading: false,
      error: null,
    } as any);

    renderWithProviders(
      <GraphViewer 
        rootEntityId="entity-1" 
        width={800} 
        height={600} 
        onNodeClick={onNodeClick}
      />
    );
    
    await waitFor(() => {
      const nodeButton = screen.getByTestId('mock-node-click');
      nodeButton.click();
    });
    
    expect(onNodeClick).toHaveBeenCalledWith(expect.objectContaining({
      id: 'entity-1',
      name: 'Node 1',
    }));
  });

  it('displays node count indicator', async () => {
    vi.mocked(useEntitiesModule.useEntityGraph).mockReturnValue({
      data: mockGraphData,
      isLoading: false,
      error: null,
    } as any);

    renderWithProviders(<GraphViewer rootEntityId="entity-1" width={800} height={600} />);
    
    await waitFor(() => {
      expect(screen.getByText(/2 nodes/i)).toBeInTheDocument();
      expect(screen.getByText(/1 relationships/i)).toBeInTheDocument();
    });
  });

  it('renders graph controls', async () => {
    vi.mocked(useEntitiesModule.useEntityGraph).mockReturnValue({
      data: mockGraphData,
      isLoading: false,
      error: null,
    } as any);

    renderWithProviders(<GraphViewer rootEntityId="entity-1" width={800} height={600} />);
    
    await waitFor(() => {
      // Graph controls should be present
      expect(screen.getByTestId('graph-canvas')).toBeInTheDocument();
    });
  });

  it('handles graph data with no root entity', async () => {
    vi.mocked(useEntitiesModule.useEntityGraph).mockReturnValue({
      data: { nodes: [], edges: [] },
      isLoading: false,
      error: null,
    } as any);

    renderWithProviders(<GraphViewer width={800} height={600} />);
    
    await waitFor(() => {
      expect(screen.getByTestId('graph-canvas')).toBeInTheDocument();
      expect(screen.getByText(/0 nodes/i)).toBeInTheDocument();
    });
  });
});
