import { describe, it, expect, vi } from 'vitest';
import { render, screen, fireEvent } from '@testing-library/react';
import { QueryClient, QueryClientProvider } from '@tanstack/react-query';
import { EntityDetail } from '../../src/components/Entity/EntityDetail';
import { Entity, EntityType } from '../../src/types/knowledge-graph';

const mockEntity: Entity = {
  id: '1',
  name: 'Alice Smith',
  entityType: EntityType.Person,
  slug: 'alice-smith',
  description: 'Software Engineer at Acme Corp',
  metadata: { role: 'Senior Engineer', email: 'alice@example.com' },
  extractionConfidence: 0.92,
  privacyLevel: 'private',
  createdAt: new Date().toISOString(),
  updatedAt: new Date().toISOString(),
};

const createTestQueryClient = () => new QueryClient({
  defaultOptions: { queries: { retry: false } }
});

describe('EntityDetail', () => {
  it('renders entity information', () => {
    render(
      <QueryClientProvider client={createTestQueryClient()}>
        <EntityDetail entity={mockEntity} />
      </QueryClientProvider>
    );
    
    expect(screen.getByText('Alice Smith')).toBeInTheDocument();
    expect(screen.getByText('Software Engineer at Acme Corp')).toBeInTheDocument();
    expect(screen.getByText('Senior Engineer')).toBeInTheDocument();
  });
  
  it('calls onEdit when edit button clicked', () => {
    const onEdit = vi.fn();
    render(
      <QueryClientProvider client={createTestQueryClient()}>
        <EntityDetail entity={mockEntity} onEdit={onEdit} />
      </QueryClientProvider>
    );
    
    fireEvent.click(screen.getByText(/edit/i));
    expect(onEdit).toHaveBeenCalled();
  });
  
  it('calls onDelete when delete confirmed', () => {
    const onDelete = vi.fn();
    render(
      <QueryClientProvider client={createTestQueryClient()}>
        <EntityDetail entity={mockEntity} onDelete={onDelete} />
      </QueryClientProvider>
    );
    
    fireEvent.click(screen.getByText(/delete/i));
    fireEvent.click(screen.getByText(/confirm/i));
    expect(onDelete).toHaveBeenCalled();
  });
  
  it('displays confidence score', () => {
    render(
      <QueryClientProvider client={createTestQueryClient()}>
        <EntityDetail entity={mockEntity} />
      </QueryClientProvider>
    );
    
    expect(screen.getByText(/92%/i)).toBeInTheDocument();
  });
});
