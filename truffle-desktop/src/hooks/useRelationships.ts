import { useQuery, useMutation, useQueryClient } from '@tanstack/react-query';
import { relationshipApi } from '../api/relationships';
import type { CreateRelationshipInput, UpdateRelationshipInput, RelationshipFilter, PaginationParams } from '../types/knowledge-graph';

const RELATIONSHIPS_KEY = 'relationships';
const RELATIONSHIP_KEY = 'relationship';
const RELATIONSHIP_TYPES_KEY = 'relationshipTypes';

export function useRelationships(filter?: RelationshipFilter, pagination?: PaginationParams) {
  return useQuery({
    queryKey: [RELATIONSHIPS_KEY, filter, pagination],
    queryFn: () => relationshipApi.getRelationships(filter, pagination),
  });
}

export function useAllRelationships(filter?: RelationshipFilter) {
  return useQuery({
    queryKey: [RELATIONSHIPS_KEY, 'all', filter],
    queryFn: () => relationshipApi.getAllRelationships(filter),
  });
}

export function useRelationship(id: string) {
  return useQuery({
    queryKey: [RELATIONSHIP_KEY, id],
    queryFn: () => relationshipApi.getRelationship(id),
    enabled: !!id,
  });
}

export function useEntityRelationships(entityId: string) {
  return useQuery({
    queryKey: [RELATIONSHIPS_KEY, 'entity', entityId],
    queryFn: () => relationshipApi.getEntityRelationships(entityId),
    enabled: !!entityId,
  });
}

export function useRelationshipTypes() {
  return useQuery({
    queryKey: [RELATIONSHIP_TYPES_KEY],
    queryFn: () => relationshipApi.getRelationshipTypes(),
  });
}

export function useInferRelationships(entityId: string) {
  return useQuery({
    queryKey: [RELATIONSHIPS_KEY, 'inferred', entityId],
    queryFn: () => relationshipApi.inferRelationships(entityId),
    enabled: !!entityId,
  });
}

export function useCreateRelationship() {
  const queryClient = useQueryClient();
  return useMutation({
    mutationFn: relationshipApi.createRelationship,
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: [RELATIONSHIPS_KEY] });
      queryClient.invalidateQueries({ queryKey: [RELATIONSHIP_KEY] });
    },
  });
}

export function useUpdateRelationship() {
  const queryClient = useQueryClient();
  return useMutation({
    mutationFn: ({ id, input }: { id: string; input: UpdateRelationshipInput }) =>
      relationshipApi.updateRelationship(id, input),
    onSuccess: (_, { id }) => {
      queryClient.invalidateQueries({ queryKey: [RELATIONSHIP_KEY, id] });
      queryClient.invalidateQueries({ queryKey: [RELATIONSHIPS_KEY] });
    },
  });
}

export function useDeleteRelationship() {
  const queryClient = useQueryClient();
  return useMutation({
    mutationFn: relationshipApi.deleteRelationship,
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: [RELATIONSHIPS_KEY] });
      queryClient.invalidateQueries({ queryKey: [RELATIONSHIP_KEY] });
    },
  });
}

export function useDeleteRelationshipsByEntity() {
  const queryClient = useQueryClient();
  return useMutation({
    mutationFn: relationshipApi.deleteRelationshipsByEntity,
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: [RELATIONSHIPS_KEY] });
      queryClient.invalidateQueries({ queryKey: [RELATIONSHIP_KEY] });
    },
  });
}
