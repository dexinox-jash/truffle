import { describe, it, expect, vi } from 'vitest';
import { render, screen, fireEvent, waitFor } from '@testing-library/react';
import { EntityForm } from '../../src/components/Entity/EntityForm';
import { EntityType } from '../../src/types/knowledge-graph';

describe('EntityForm', () => {
  it('submits form with correct data', async () => {
    const onSubmit = vi.fn();
    render(<EntityForm onSubmit={onSubmit} />);
    
    // Fill form
    fireEvent.change(screen.getByLabelText(/name/i), {
      target: { value: 'New Entity' }
    });
    
    // Select entity type
    fireEvent.click(screen.getByLabelText(/person/i));
    
    // Fill metadata
    fireEvent.change(screen.getByLabelText(/role/i), {
      target: { value: 'Engineer' }
    });
    
    // Submit
    fireEvent.click(screen.getByText(/create/i));
    
    // Verify
    await waitFor(() => {
      expect(onSubmit).toHaveBeenCalledWith(expect.objectContaining({
        name: 'New Entity',
        entityType: EntityType.Person,
        metadata: expect.objectContaining({ role: 'Engineer' })
      }));
    });
  });
  
  it('shows validation errors for empty name', async () => {
    const onSubmit = vi.fn();
    render(<EntityForm onSubmit={onSubmit} />);
    
    // Submit empty form
    fireEvent.click(screen.getByText(/create/i));
    
    // Verify error
    expect(await screen.findByText(/name is required/i)).toBeInTheDocument();
    expect(onSubmit).not.toHaveBeenCalled();
  });
  
  it('pre-fills data in edit mode', () => {
    const existingEntity = {
      id: '1',
      name: 'Alice',
      entityType: EntityType.Person,
      description: 'Test description',
      metadata: { role: 'Manager' },
      extractionConfidence: 0.9,
      privacyLevel: 'private' as const,
      createdAt: new Date().toISOString(),
      updatedAt: new Date().toISOString(),
    };
    
    render(<EntityForm entity={existingEntity} onSubmit={vi.fn()} />);
    
    expect(screen.getByDisplayValue('Alice')).toBeInTheDocument();
    expect(screen.getByDisplayValue('Test description')).toBeInTheDocument();
  });
  
  it('calls onCancel when cancel clicked', () => {
    const onCancel = vi.fn();
    render(<EntityForm onSubmit={vi.fn()} onCancel={onCancel} />);
    
    fireEvent.click(screen.getByText(/cancel/i));
    expect(onCancel).toHaveBeenCalled();
  });
});
