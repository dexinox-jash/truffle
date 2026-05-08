//! Graph Traversal
//!
//! Pathfinding and traversal algorithms for the knowledge graph.

use std::collections::{HashMap, HashSet, VecDeque};
use uuid::Uuid;

use crate::graph::repository::GraphRepository;
use crate::models::{Entity, Relationship};
use crate::Result;

/// Traversal configuration
#[derive(Debug, Clone)]
pub struct TraversalConfig {
    /// Maximum traversal depth
    pub max_depth: usize,
    /// Relationship types to follow (None = all)
    pub relationship_types: Option<Vec<String>>,
    /// Minimum confidence threshold
    pub min_confidence: f32,
}

impl Default for TraversalConfig {
    fn default() -> Self {
        Self {
            max_depth: 5,
            relationship_types: None,
            min_confidence: 0.5,
        }
    }
}

/// Graph traversal
pub struct GraphTraversal<'a> {
    repository: &'a GraphRepository,
    config: TraversalConfig,
    visited: HashSet<Uuid>,
}

impl<'a> GraphTraversal<'a> {
    /// Create a new graph traversal
    pub fn new(repository: &'a GraphRepository, config: TraversalConfig) -> Self {
        Self {
            repository,
            config,
            visited: HashSet::new(),
        }
    }
    
    /// Breadth-first traversal
    pub async fn bfs<F>(&mut self, start: &Uuid, mut visitor: F) -> Result<()>
    where
        F: FnMut(&Entity, usize) -> bool, // Return false to stop traversal
    {
        let mut queue = VecDeque::new();
        
        // Get starting entity
        if let Some(entity) = self.repository.entities.get(start).await? {
            queue.push_back((entity, 0));
            self.visited.insert(*start);
        }
        
        while let Some((entity, depth)) = queue.pop_front() {
            if depth > self.config.max_depth {
                continue;
            }
            
            // Visit entity
            let should_continue = visitor(&entity, depth);
            if !should_continue {
                break;
            }
            
            // Get neighbors
            let relationships = self.repository.relationships
                .get_outgoing(&entity.id)
                .await?;
            
            for rel in relationships {
                if self.should_follow_relationship(&rel) && !self.visited.contains(&rel.target_id) {
                    self.visited.insert(rel.target_id);
                    
                    if let Some(neighbor) = self.repository.entities.get(&rel.target_id).await? {
                        queue.push_back((neighbor, depth + 1));
                    }
                }
            }
        }
        
        Ok(())
    }
    
    /// Depth-first traversal
    pub async fn dfs<F>(&mut self, start: &Uuid, mut visitor: F) -> Result<()>
    where
        F: FnMut(&Entity, usize) -> bool,
    {
        self.visited.clear();
        
        if let Some(entity) = self.repository.entities.get(start).await? {
            self.dfs_recursive(&entity, 0, &mut visitor).await?;
        }
        
        Ok(())
    }
    
    /// Recursive DFS helper
    async fn dfs_recursive<F>(&mut self, entity: &Entity, depth: usize, visitor: &mut F) -> Result<()>
    where
        F: FnMut(&Entity, usize) -> bool,
    {
        if depth > self.config.max_depth || self.visited.contains(&entity.id) {
            return Ok(());
        }
        
        self.visited.insert(entity.id);
        
        let should_continue = visitor(entity, depth);
        if !should_continue {
            return Ok(());
        }
        
        // Get neighbors
        let relationships = self.repository.relationships
            .get_outgoing(&entity.id)
            .await?;
        
        for rel in relationships {
            if self.should_follow_relationship(&rel) {
                if let Some(neighbor) = self.repository.entities.get(&rel.target_id).await? {
                    Box::pin(self.dfs_recursive(&neighbor, depth + 1, visitor)).await?;
                }
            }
        }
        
        Ok(())
    }
    
    /// Check if relationship should be followed
    fn should_follow_relationship(&self, rel: &Relationship) -> bool {
        // Check extraction confidence
        if rel.extraction_confidence < self.config.min_confidence {
            return false;
        }
        
        // Check relationship type
        if let Some(ref types) = self.config.relationship_types {
            return types.contains(&rel.relation_type);
        }
        
        true
    }
}

/// Path finder
pub struct PathFinder<'a> {
    repository: &'a GraphRepository,
}

impl<'a> PathFinder<'a> {
    /// Create a new path finder
    pub fn new(repository: &'a GraphRepository) -> Self {
        Self { repository }
    }
    
    /// Find path using BFS
    pub async fn find_path_bfs(
        &self,
        start: &Uuid,
        end: &Uuid,
        max_depth: usize,
    ) -> Result<Vec<Entity>> {
        let mut queue = VecDeque::new();
        let mut parents: HashMap<Uuid, Uuid> = HashMap::new();
        let mut visited = HashSet::new();
        
        queue.push_back(*start);
        visited.insert(*start);
        
        let mut depth = 0;
        while !queue.is_empty() && depth < max_depth {
            let level_size = queue.len();
            
            for _ in 0..level_size {
                let current = queue.pop_front().unwrap();
                
                // Check if we reached the target
                if current == *end {
                    // Reconstruct path
                    return self.reconstruct_path(&parents, *end).await;
                }
                
                // Get neighbors
                let relationships = self.repository.relationships
                    .get_outgoing(&current)
                    .await?;
                
                for rel in relationships {
                    if !visited.contains(&rel.target_id) {
                        visited.insert(rel.target_id);
                        parents.insert(rel.target_id, current);
                        queue.push_back(rel.target_id);
                    }
                }
            }
            
            depth += 1;
        }
        
        // No path found
        Ok(Vec::new())
    }
    
    /// Find shortest path using bidirectional BFS
    pub async fn find_path_bidirectional(
        &self,
        start: &Uuid,
        end: &Uuid,
        max_depth: usize,
    ) -> Result<Vec<Entity>> {
        if start == end {
            // Return single entity if start == end
            if let Some(entity) = self.repository.entities.get(start).await? {
                return Ok(vec![entity]);
            }
            return Ok(Vec::new());
        }
        
        // Frontiers for bidirectional search
        let mut forward_queue = VecDeque::new();
        let mut backward_queue = VecDeque::new();
        
        forward_queue.push_back(*start);
        backward_queue.push_back(*end);
        
        let mut forward_visited = HashSet::new();
        let mut backward_visited = HashSet::new();
        
        forward_visited.insert(*start);
        backward_visited.insert(*end);
        
        let mut forward_parents: HashMap<Uuid, Uuid> = HashMap::new();
        let mut backward_parents: HashMap<Uuid, Uuid> = HashMap::new();
        
        let mut depth = 0;
        
        while !forward_queue.is_empty() && !backward_queue.is_empty() && depth < max_depth {
            // Expand forward frontier
            if let Some(meeting_point) = self.expand_frontier(
                &mut forward_queue,
                &mut forward_visited,
                &backward_visited,
                &mut forward_parents,
                true,
            ).await? {
                return self.merge_paths(
                    &forward_parents,
                    &backward_parents,
                    meeting_point,
                ).await;
            }
            
            // Expand backward frontier
            if let Some(meeting_point) = self.expand_frontier(
                &mut backward_queue,
                &mut backward_visited,
                &forward_visited,
                &mut backward_parents,
                false,
            ).await? {
                return self.merge_paths(
                    &forward_parents,
                    &backward_parents,
                    meeting_point,
                ).await;
            }
            
            depth += 1;
        }
        
        // No path found
        Ok(Vec::new())
    }
    
    /// Expand one frontier step
    async fn expand_frontier(
        &self,
        queue: &mut VecDeque<Uuid>,
        visited: &mut HashSet<Uuid>,
        other_visited: &HashSet<Uuid>,
        parents: &mut HashMap<Uuid, Uuid>,
        is_forward: bool,
    ) -> Result<Option<Uuid>> {
        if let Some(current) = queue.pop_front() {
            // Get neighbors (outgoing for forward, incoming for backward)
            let relationships = if is_forward {
                self.repository.relationships.get_outgoing(&current).await?
            } else {
                self.repository.relationships.get_incoming(&current).await?
            };
            
            for rel in relationships {
                let neighbor = if is_forward {
                    rel.target_id
                } else {
                    rel.source_id
                };
                
                if !visited.contains(&neighbor) {
                    visited.insert(neighbor);
                    parents.insert(neighbor, current);
                    queue.push_back(neighbor);
                    
                    // Check if we met the other frontier
                    if other_visited.contains(&neighbor) {
                        return Ok(Some(neighbor));
                    }
                }
            }
        }
        
        Ok(None)
    }
    
    /// Merge two paths into one
    async fn merge_paths(
        &self,
        forward_parents: &HashMap<Uuid, Uuid>,
        backward_parents: &HashMap<Uuid, Uuid>,
        meeting_point: Uuid,
    ) -> Result<Vec<Entity>> {
        // Reconstruct forward path
        let mut forward_path = Vec::new();
        let mut current = meeting_point;
        
        while let Some(&parent) = forward_parents.get(&current) {
            forward_path.push(current);
            current = parent;
        }
        forward_path.push(current); // Add start
        
        forward_path.reverse();
        
        // Reconstruct backward path (excluding meeting point)
        current = meeting_point;
        while let Some(&parent) = backward_parents.get(&current) {
            forward_path.push(parent);
            current = parent;
        }
        
        // Convert IDs to entities
        let mut entities = Vec::new();
        for id in forward_path {
            if let Some(entity) = self.repository.entities.get(&id).await? {
                entities.push(entity);
            }
        }
        
        Ok(entities)
    }
    
    /// Reconstruct path from parents map
    async fn reconstruct_path(
        &self,
        parents: &HashMap<Uuid, Uuid>,
        end: Uuid,
    ) -> Result<Vec<Entity>> {
        let mut path = Vec::new();
        let mut current = end;
        
        // Build path from end to start
        let mut ids = vec![current];
        while let Some(&parent) = parents.get(&current) {
            ids.push(parent);
            current = parent;
        }
        
        // Reverse to get start -> end
        ids.reverse();
        
        // Convert IDs to entities
        for id in ids {
            if let Some(entity) = self.repository.entities.get(&id).await? {
                path.push(entity);
            }
        }
        
        Ok(path)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    // Note: Tests would require a mock repository
    
    #[test]
    fn test_traversal_config_default() {
        let config = TraversalConfig::default();
        assert_eq!(config.max_depth, 5);
        assert_eq!(config.min_confidence, 0.5);
    }
}
