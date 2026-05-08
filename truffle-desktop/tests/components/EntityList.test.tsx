import { describe, it, expect, vi } from 'vitest';
import { render, screen, fireEvent, waitFor } from '@testing-library/react';
import { QueryClient, QueryClientProvider } from '@tanstack/react-query';
import { EntityList } from '../../src/components/Entity/EntityList';
import { EntityType, EntitySummary } from '../../src/types/knowledge-graph';

const mockEntities: EntitySummary[] = [
  { id: '1', name: 'Alice', entityType: EntityType.Person, extractionConfidence: 0.9 },
  { id: '2', name: 'Acme Corp', entityType: EntityType.Organization, extractionConfidence: 0.85 },
  { id: '3', name: 'New York', entityType: EntityType.Location, extractionConfidence: 0.95 },
];

const createTestQueryClient = () => new QueryClient({
  defaultOptions: { queries: { retry: false } }
});

describe('EntityList', () => {
  it('renders list of entities', () => {
    render(
      <QueryClientProvider client={createTestQueryClient()}>
        <EntityList entities={mockEntities} />
      </QueryClientProvider>
    );
    
    expect(screen.getByText('Alice')).toBeInTheDocument();
    expect(screen.getByText('Acme Corp')).toBeInTheDocument();
    expect(screen.getByText('New York')).toBeInTheDocument();
  });
  
  it('calls onSelect when entity clicked', () => {
    const onSelect = vi.fn();
    render(
      <QueryClientProvider client={createTestQueryClient()}>
        <EntityList entities={mockEntities} onSelect={onSelect} />
      </QueryClientProvider>
    );
    
    fireEvent.click(screen.getByText('Alice'));
    expect(onSelect).toHaveBeenCalledWith(mockEntities[0]);
  });
  
  it('renders empty state when no entities', () => {
    render(
      <QueryClientProvider client={createTestQueryClient()}>
        <EntityList entities={[]} />
      </QueryClientProvider>
    );
    
    expect(screen.getByText(/no entities found/i)).toBeInTheDocument();
  });
  
  it('displays entity type badges', () => {
    render(
      <QueryClientProvider client={createTestQueryClient()}>
        <EntityList entities={mockEntities} />
      </QueryClientProvider>
    );
    
    expect(screen.getByText('Person')).toBeInTheDocument();
    expect(screen.getByText('Organization')).toBeInTheDocument();
    expect(screen.getByText('Location')).toBeInTheDocument();
  });
  
  it('shows confidence indicators', () => {
    render(
      <QueryClientProvider client={createTestQueryClient()}>
        <EntityList entities={mockEntities} />
      </QueryClientProvider>
    );
    
    // Confidence should be displayed (90%, 85%, 95%)
    expect(screen.getByText('90%')).toBeInTheDocument();
  });
});
