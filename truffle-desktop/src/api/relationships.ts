import { invokeCommand } from './client';
import type { 
  Relationship, 
  CreateRelationshipInput,
  UpdateRelationshipInput,
  RelationshipFilter
} from '../types/knowledge-graph';
import type { PaginationParams, PaginatedResponse } from './client';

export const relationshipApi = {
  getRelationships: (filter?: RelationshipFilter, pagination?: PaginationParams) =>
    invokeCommand<PaginatedResponse<Relationship>>('get_relationships', { filter, ...pagination }),

  getAllRelationships: (filter?: RelationshipFilter) =>
    invokeCommand<Relationship[]>('get_all_relationships', { filter }),

  getRelationship: (id: string) =>
    invokeCommand<Relationship>('get_relationship', { id }),

  getEntityRelationships: (entityId: string) =>
    invokeCommand<Relationship[]>('get_entity_relationships', { entityId }),

  createRelationship: (input: CreateRelationshipInput) =>
    invokeCommand<Relationship>('create_relationship', { input }),

  updateRelationship: (id: string, input: UpdateRelationshipInput) =>
    invokeCommand<Relationship>('update_relationship', { id, input }),

  deleteRelationship: (id: string) =>
    invokeCommand<void>('delete_relationship', { id }),

  deleteRelationshipsByEntity: (entityId: string) =>
    invokeCommand<void>('delete_relationships_by_entity', { entityId }),

  getRelationshipTypes: () =>
    invokeCommand<string[]>('get_relationship_types', {}),

  inferRelationships: (entityId: string) =>
    invokeCommand<Relationship[]>('infer_relationships', { entityId }),
};
