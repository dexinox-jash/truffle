import { invokeCommand } from './client';
import type { 
  Entity, 
  EntitySummary, 
  EntityWithRelationships,
  CreateEntityInput, 
  UpdateEntityInput, 
  EntityFilter,
  GraphData
} from '../types/knowledge-graph';
import type { PaginationParams, PaginatedResponse } from './client';

export const entityApi = {
  getEntities: (filter?: EntityFilter, pagination?: PaginationParams) =>
    invokeCommand<PaginatedResponse<EntitySummary>>('get_entities', { filter, ...pagination }),

  getAllEntities: (filter?: EntityFilter) =>
    invokeCommand<EntitySummary[]>('get_all_entities', { filter }),

  getEntity: (id: string) =>
    invokeCommand<Entity>('get_entity', { id }),

  getEntityWithRelationships: (id: string) =>
    invokeCommand<EntityWithRelationships>('get_entity_with_relationships', { id }),

  searchEntities: (query: string, limit?: number) =>
    invokeCommand<EntitySummary[]>('search_entities', { query, limit }),

  semanticSearch: (query: string, limit?: number, threshold?: number) =>
    invokeCommand<EntitySummary[]>('semantic_search_entities', { query, limit, threshold }),

  createEntity: (input: CreateEntityInput) =>
    invokeCommand<Entity>('create_entity', { input }),

  updateEntity: (id: string, input: UpdateEntityInput) =>
    invokeCommand<Entity>('update_entity', { id, input }),

  deleteEntity: (id: string) =>
    invokeCommand<void>('delete_entity', { id }),

  getEntityGraph: (entityId: string, depth?: number) =>
    invokeCommand<GraphData>('get_entity_graph', { entityId, depth }),

  mergeEntities: (sourceId: string, targetId: string) =>
    invokeCommand<Entity>('merge_entities', { sourceId, targetId }),
};
