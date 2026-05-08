import { useQuery, useMutation, useQueryClient } from '@tanstack/react-query';
import { entityApi } from '../api/entities';
import type { CreateEntityInput, UpdateEntityInput, EntityFilter, PaginationParams } from '../types/knowledge-graph';

const ENTITIES_KEY = 'entities';
const ENTITY_KEY = 'entity';
const ENTITY_SEARCH_KEY = 'entitySearch';
const ENTITY_GRAPH_KEY = 'entityGraph';

export function useEntities(filter?: EntityFilter, pagination?: PaginationParams) {
  return useQuery({
    queryKey: [ENTITIES_KEY, filter, pagination],
    queryFn: () => entityApi.getEntities(filter, pagination),
  });
}

export function useAllEntities(filter?: EntityFilter) {
  return useQuery({
    queryKey: [ENTITIES_KEY, 'all', filter],
    queryFn: () => entityApi.getAllEntities(filter),
  });
}

export function useEntity(id: string) {
  return useQuery({
    queryKey: [ENTITY_KEY, id],
    queryFn: () => entityApi.getEntity(id),
    enabled: !!id,
  });
}

export function useEntityWithRelationships(id: string) {
  return useQuery({
    queryKey: [ENTITY_KEY, id, 'with-relationships'],
    queryFn: () => entityApi.getEntityWithRelationships(id),
    enabled: !!id,
  });
}

export function useSearchEntities(query: string, limit?: number) {
  return useQuery({
    queryKey: [ENTITY_SEARCH_KEY, query, limit],
    queryFn: () => entityApi.searchEntities(query, limit),
    enabled: query.length > 0,
  });
}

export function useSemanticSearch(query: string, limit?: number, threshold?: number) {
  return useQuery({
    queryKey: [ENTITY_SEARCH_KEY, 'semantic', query, limit, threshold],
    queryFn: () => entityApi.semanticSearch(query, limit, threshold),
    enabled: query.length > 0,
  });
}

export function useEntityGraph(entityId: string, depth?: number) {
  return useQuery({
    queryKey: [ENTITY_GRAPH_KEY, entityId, depth],
    queryFn: () => entityApi.getEntityGraph(entityId, depth),
    enabled: !!entityId,
  });
}

export function useCreateEntity() {
  const queryClient = useQueryClient();
  return useMutation({
    mutationFn: entityApi.createEntity,
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: [ENTITIES_KEY] });
    },
  });
}

export function useUpdateEntity() {
  const queryClient = useQueryClient();
  return useMutation({
    mutationFn: ({ id, input }: { id: string; input: UpdateEntityInput }) =>
      entityApi.updateEntity(id, input),
    onSuccess: (_, { id }) => {
      queryClient.invalidateQueries({ queryKey: [ENTITY_KEY, id] });
      queryClient.invalidateQueries({ queryKey: [ENTITIES_KEY] });
    },
  });
}

export function useDeleteEntity() {
  const queryClient = useQueryClient();
  return useMutation({
    mutationFn: entityApi.deleteEntity,
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: [ENTITIES_KEY] });
      queryClient.invalidateQueries({ queryKey: [ENTITY_KEY] });
    },
  });
}

export function useMergeEntities() {
  const queryClient = useQueryClient();
  return useMutation({
    mutationFn: ({ sourceId, targetId }: { sourceId: string; targetId: string }) =>
      entityApi.mergeEntities(sourceId, targetId),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: [ENTITIES_KEY] });
      queryClient.invalidateQueries({ queryKey: [ENTITY_KEY] });
    },
  });
}
