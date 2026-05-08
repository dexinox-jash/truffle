//! Graph CRUD and Traversal Integration Tests
//!
//! Tests for entity and relationship CRUD operations, graph traversal,
//! semantic search, and entity filtering.

use tokio::sync::RwLock;
use std::sync::Arc;
use uuid::Uuid;
use chrono::{Utc, Duration};

use truffle_core::{
    Database,
    Entity, EntityType, EntityMetadata, PrivacyLevel,
    Relationship,
    GraphRepository, EntityRepository, RelationshipRepository,
    GraphQuery, EntityFilter, RelationshipFilter,
    GraphTraversal, TraversalConfig, PathFinder,
};

use crate::integration::{
    setup_test_db, setup_test_db_file,
    TestEntityBuilder, TestRelationshipBuilder,
    generate_test_embedding, generate_similar_embeddings,
    assertions::*,
};

// ============================================================================
// Entity CRUD Tests
// ============================================================================

#[tokio::test]
async fn test_create_entity() {
    let db = setup_test_db().await;
    let entity = TestEntityBuilder::person("Alice")
        .with_metadata(EntityMetadata {
            title: Some("Engineer".to_string()),
            email: Some("alice@example.com".to_string()),
            ..Default::default()
        })
        .build();

    assert_eq!(entity.name, "Alice");
    assert_eq!(entity.entity_type, EntityType::Person);
    assert_eq!(entity.metadata.title, Some("Engineer".to_string()));
    assert!(!entity.verified);
}

#[tokio::test]
async fn test_entity_crud_lifecycle() {
    let db = setup_test_db().await;
    let repo = GraphRepository::new(db.clone());

    // Create
    let entity = TestEntityBuilder::person("Bob Johnson")
        .with_role("Manager")
        .with_email("bob@example.com")
        .build();

    let entity_id = entity.id;

    // Read (through repository - will return None since it's not persisted in mock)
    // In a real implementation, this would test actual persistence
    let found = repo.entities.get(&entity_id).await.unwrap();
    // Note: Since repository methods are placeholders, this returns None
    // This test validates the API contract

    // Update
    let mut updated = entity.clone();
    updated.name = "Robert Johnson".to_string();
    updated.last_updated_at = Utc::now();

    assert_eq!(updated.id, entity_id);
    assert_eq!(updated.name, "Robert Johnson");

    // Delete
    // In a real implementation, this would test actual deletion
    let deleted = repo.entities.delete(&entity_id).await.unwrap();
    // Note: Placeholder returns false
}

#[tokio::test]
async fn test_create_multiple_entities() {
    let db = setup_test_db().await;

    let entities: Vec<Entity> = vec![
        TestEntityBuilder::person("Alice").build(),
        TestEntityBuilder::person("Bob").build(),
        TestEntityBuilder::person("Charlie").build(),
        TestEntityBuilder::organization("Acme Corp").build(),
        TestEntityBuilder::organization("Tech Inc").build(),
    ];

    assert_eq!(entities.len(), 5);
    assert_eq!(entities[0].entity_type, EntityType::Person);
    assert_eq!(entities[3].entity_type, EntityType::Organization);

    // Verify all have unique IDs
    let ids: std::collections::HashSet<_> = entities.iter().map(|e| e.id).collect();
    assert_eq!(ids.len(), 5);
}

#[tokio::test]
async fn test_entity_builder_patterns() {
    // Test person builder
    let person = TestEntityBuilder::person("John Doe")
        .with_role("Senior Engineer")
        .with_email("john@example.com")
        .with_description("A senior software engineer")
        .verified()
        .with_confidence(0.95)
        .with_privacy(PrivacyLevel::Internal)
        .build();

    assert_eq!(person.name, "John Doe");
    assert_eq!(person.entity_type, EntityType::Person);
    assert_eq!(person.metadata.title, Some("Senior Engineer".to_string()));
    assert_eq!(person.metadata.email, Some("john@example.com".to_string()));
    assert_eq!(person.description, Some("A senior software engineer".to_string()));
    assert!(person.verified);
    assert_eq!(person.extraction_confidence, 0.95);
    assert_eq!(person.privacy_level, PrivacyLevel::Internal);

    // Test organization builder
    let org = TestEntityBuilder::organization("Example Inc")
        .with_industry("Software")
        .with_description("A software company")
        .with_privacy(PrivacyLevel::Public)
        .build();

    assert_eq!(org.name, "Example Inc");
    assert_eq!(org.entity_type, EntityType::Organization);
    assert_eq!(org.metadata.industry, Some("Software".to_string()));
    assert_eq!(org.privacy_level, PrivacyLevel::Public);

    // Test location builder
    let location = TestEntityBuilder::location("San Francisco", 37.7749, -122.4194)
        .build();

    assert_eq!(location.name, "San Francisco");
    assert_eq!(location.entity_type, EntityType::Location);
    assert_eq!(location.metadata.latitude, Some(37.7749));
    assert_eq!(location.metadata.longitude, Some(-122.4194));

    // Test event builder
    let start = Utc::now();
    let end = start + Duration::hours(2);
    let event = TestEntityBuilder::event("Team Meeting", start, end)
        .build();

    assert_eq!(event.name, "Team Meeting");
    assert_eq!(event.entity_type, EntityType::Event);
    assert_eq!(event.metadata.event_start, Some(start));
    assert_eq!(event.metadata.event_end, Some(end));
}

// ============================================================================
// Relationship Tests
// ============================================================================

#[tokio::test]
async fn test_create_relationship() {
    let person = TestEntityBuilder::person("Alice").build();
    let org = TestEntityBuilder::organization("Acme Corp").build();

    let rel = TestRelationshipBuilder::works_at(person.id, org.id)
        .verified()
        .build();

    assert_eq!(rel.source_id, person.id);
    assert_eq!(rel.target_id, org.id);
    assert_eq!(rel.relation_type, "works_at");
    assert!(rel.verified);
    assert!(!rel.archived);
}

#[tokio::test]
async fn test_relationship_with_validity_period() {
    let person = TestEntityBuilder::person("Bob").build();
    let org = TestEntityBuilder::organization("Old Corp").build();

    let start = Utc::now() - Duration::days(365);
    let end = Utc::now() - Duration::days(30);

    let rel = TestRelationshipBuilder::works_at(person.id, org.id)
        .with_validity(start, Some(end))
        .build();

    assert_eq!(rel.valid_from, Some(start));
    assert_eq!(rel.valid_until, Some(end));

    // Should not be valid at current time
    assert!(!rel.is_valid_at(Utc::now()));

    // Should be valid at a time within the range
    let mid = start + Duration::days(100);
    assert!(rel.is_valid_at(mid));
}

#[tokio::test]
async fn test_create_relationship_chain() {
    // Create a chain: Person -> Organization -> Location
    let person = TestEntityBuilder::person("Alice").build();
    let org = TestEntityBuilder::organization("Acme Corp").build();
    let location = TestEntityBuilder::location("New York", 40.7128, -74.0060).build();

    let works_at = TestRelationshipBuilder::works_at(person.id, org.id).build();
    let located_in = TestRelationshipBuilder::located_in(org.id, location.id).build();

    assert_eq!(works_at.source_id, person.id);
    assert_eq!(works_at.target_id, org.id);
    assert_eq!(located_in.source_id, org.id);
    assert_eq!(located_in.target_id, location.id);

    // Verify the chain
    assert_eq!(works_at.target_id, located_in.source_id);
}

#[tokio::test]
async fn test_relationship_equivalence() {
    let a = Uuid::new_v4();
    let b = Uuid::new_v4();

    let rel1 = TestRelationshipBuilder::works_at(a, b).build();
    let rel2 = TestRelationshipBuilder::works_at(a, b).build();
    let rel3 = TestRelationshipBuilder::new(a, b, "knows").build();
    let rel4 = TestRelationshipBuilder::new(b, a, "works_at").build();

    // Same source, target, and type = equivalent
    assert!(rel1.is_equivalent_to(&rel2));

    // Different type = not equivalent
    assert!(!rel1.is_equivalent_to(&rel3));

    // Different direction = not equivalent
    assert!(!rel1.is_equivalent_to(&rel4));
}

#[tokio::test]
async fn test_relationship_inverse() {
    let person = TestEntityBuilder::person("Alice").build();
    let org = TestEntityBuilder::organization("Acme Corp").build();

    let works_at = TestRelationshipBuilder::works_at(person.id, org.id)
        .with_confidence(0.9)
        .build();

    let employs = works_at.inverse("employs");

    assert_eq!(employs.source_id, org.id);
    assert_eq!(employs.target_id, person.id);
    assert_eq!(employs.relation_type, "employs");
    assert_eq!(employs.extraction_confidence, 0.9);
}

// ============================================================================
// Graph Traversal Tests
// ============================================================================

#[tokio::test]
async fn test_graph_traversal_bfs() {
    let db = setup_test_db().await;
    let repo = GraphRepository::new(db.clone());

    // Create entities: A -> B -> C
    let entity_a = TestEntityBuilder::person("A").build();
    let entity_b = TestEntityBuilder::person("B").build();
    let entity_c = TestEntityBuilder::person("C").build();

    // Note: Since repository methods are placeholders, traversal will not find neighbors
    // This test validates the traversal API and configuration

    let config = TraversalConfig {
        max_depth: 3,
        relationship_types: None,
        min_confidence: 0.5,
    };

    let mut traversal = GraphTraversal::new(&repo, config);

    let mut visited = Vec::new();
    traversal
        .bfs(&entity_a.id, |entity, depth| {
            visited.push((entity.name.clone(), depth));
            true // Continue traversal
        })
        .await
        .unwrap();

    // With placeholder repository, only the starting entity is visited
    assert!(!visited.is_empty());
    assert_eq!(visited[0].0, "A");
    assert_eq!(visited[0].1, 0);
}

#[tokio::test]
async fn test_graph_traversal_dfs() {
    let db = setup_test_db().await;
    let repo = GraphRepository::new(db.clone());

    let entity_a = TestEntityBuilder::person("A").build();

    let config = TraversalConfig {
        max_depth: 5,
        relationship_types: None,
        min_confidence: 0.5,
    };

    let mut traversal = GraphTraversal::new(&repo, config);

    let mut visited = Vec::new();
    traversal
        .dfs(&entity_a.id, |entity, depth| {
            visited.push((entity.name.clone(), depth));
            true
        })
        .await
        .unwrap();

    assert!(!visited.is_empty());
    assert_eq!(visited[0].0, "A");
}

#[tokio::test]
async fn test_graph_traversal_with_depth_limit() {
    let db = setup_test_db().await;
    let repo = GraphRepository::new(db.clone());

    let entity_a = TestEntityBuilder::person("A").build();

    // Test with depth limit of 2
    let config = TraversalConfig {
        max_depth: 2,
        relationship_types: None,
        min_confidence: 0.5,
    };

    let mut traversal = GraphTraversal::new(&repo, config);

    let mut max_depth_observed = 0;
    traversal
        .bfs(&entity_a.id, |_entity, depth| {
            max_depth_observed = max_depth_observed.max(depth);
            depth < 2 // Stop if depth >= 2
        })
        .await
        .unwrap();

    // Depth should not exceed the limit
    assert!(max_depth_observed <= 2);
}

#[tokio::test]
async fn test_graph_traversal_with_type_filter() {
    let db = setup_test_db().await;
    let repo = GraphRepository::new(db.clone());

    let entity_a = TestEntityBuilder::person("A").build();

    let config = TraversalConfig {
        max_depth: 3,
        relationship_types: Some(vec!["works_at".to_string(), "knows".to_string()]),
        min_confidence: 0.5,
    };

    let mut traversal = GraphTraversal::new(&repo, config);

    let mut visited = 0;
    traversal
        .bfs(&entity_a.id, |_entity, _depth| {
            visited += 1;
            true
        })
        .await
        .unwrap();

    // Should traverse with type filter applied
    assert!(visited >= 1);
}

#[tokio::test]
async fn test_path_finder_bfs() {
    let db = setup_test_db().await;
    let repo = GraphRepository::new(db.clone());

    let entity_a = TestEntityBuilder::person("A").build();
    let entity_b = TestEntityBuilder::person("B").build();

    let finder = PathFinder::new(&repo);

    // Test path finding (will return empty with placeholder repository)
    let path = finder
        .find_path_bfs(&entity_a.id, &entity_b.id, 5)
        .await
        .unwrap();

    // With placeholder implementation, no path is found
    assert!(path.is_empty());
}

#[tokio::test]
async fn test_path_finder_bidirectional() {
    let db = setup_test_db().await;
    let repo = GraphRepository::new(db.clone());

    let entity_a = TestEntityBuilder::person("A").build();
    let entity_b = TestEntityBuilder::person("B").build();

    let finder = PathFinder::new(&repo);

    // Test bidirectional path finding
    let path = finder
        .find_path_bidirectional(&entity_a.id, &entity_b.id, 5)
        .await
        .unwrap();

    // With placeholder implementation, no path is found
    assert!(path.is_empty());

    // Test same entity
    let path_same = finder
        .find_path_bidirectional(&entity_a.id, &entity_a.id, 5)
        .await
        .unwrap();

    // Should return single entity when start == end
    assert_eq!(path_same.len(), 1);
    assert_eq!(path_same[0].id, entity_a.id);
}

// ============================================================================
// Semantic Search Tests
// ============================================================================

#[tokio::test]
async fn test_semantic_search_basic() {
    let db = setup_test_db().await;
    let repo = GraphRepository::new(db.clone());

    // Create entities with embeddings
    let emb1 = generate_test_embedding(1, 384);
    let emb2 = generate_test_embedding(2, 384);

    let entity1 = TestEntityBuilder::person("Alice")
        .with_embedding(emb1)
        .build();

    let entity2 = TestEntityBuilder::person("Bob")
        .with_embedding(emb2)
        .build();

    // Test semantic search API (placeholder returns empty)
    let results = repo.semantic_search("software engineer", 10).await.unwrap();

    // Placeholder implementation returns empty
    assert!(results.is_empty());
}

#[tokio::test]
async fn test_entity_similarity_with_embeddings() {
    // Generate similar embeddings
    let embeddings = generate_similar_embeddings(42, 3, 384, 0.1);

    let entity1 = TestEntityBuilder::person("Person A")
        .with_embedding(embeddings[0].clone())
        .build();

    let entity2 = TestEntityBuilder::person("Person B")
        .with_embedding(embeddings[1].clone())
        .build();

    let entity3 = TestEntityBuilder::person("Person C")
        .with_embedding(embeddings[2].clone())
        .build();

    // All should have embeddings
    assert!(entity1.embedding.is_some());
    assert!(entity2.embedding.is_some());
    assert!(entity3.embedding.is_some());

    // Embeddings should be similar but not identical
    let emb1 = entity1.embedding.unwrap();
    let emb2 = entity2.embedding.unwrap();
    assert_ne!(emb1, emb2);
}

// ============================================================================
// Entity Filtering Tests
// ============================================================================

#[tokio::test]
async fn test_entity_filtering_by_type() {
    let db = setup_test_db().await;
    let repo = GraphRepository::new(db.clone());

    // Create filter for Person type
    let filter = EntityFilter::new()
        .with_type(EntityType::Person)
        .with_pagination(0, 10);

    let results = repo.entities.search(&filter).await.unwrap();

    // Placeholder returns empty
    assert!(results.items.is_empty());
    assert_eq!(results.offset, 0);
    assert_eq!(results.limit, 10);
}

#[tokio::test]
async fn test_entity_filtering_by_name() {
    let db = setup_test_db().await;
    let repo = GraphRepository::new(db.clone());

    let filter = EntityFilter::new()
        .with_name_contains("Alice")
        .with_pagination(0, 20);

    let results = repo.entities.search(&filter).await.unwrap();

    // Placeholder returns empty
    assert!(results.items.is_empty());
}

#[tokio::test]
async fn test_entity_filtering_by_confidence() {
    let db = setup_test_db().await;
    let repo = GraphRepository::new(db.clone());

    let filter = EntityFilter::new()
        .with_min_confidence(0.8)
        .verified_only()
        .with_pagination(0, 50);

    let results = repo.entities.search(&filter).await.unwrap();

    // Verify filter was set correctly
    assert_eq!(filter.min_confidence, Some(0.8));
    assert!(filter.verified_only);
}

#[tokio::test]
async fn test_entity_filtering_by_date_range() {
    let db = setup_test_db().await;
    let repo = GraphRepository::new(db.clone());

    let start = Utc::now() - Duration::days(30);
    let end = Utc::now();

    let filter = EntityFilter::new()
        .with_date_range(start, end)
        .with_pagination(0, 100);

    let results = repo.entities.search(&filter).await.unwrap();

    // Verify filter was set correctly
    assert_eq!(filter.created_after, Some(start));
    assert_eq!(filter.created_before, Some(end));
}

#[tokio::test]
async fn test_entity_filter_builder() {
    let filter = EntityFilter::new()
        .with_type(EntityType::Organization)
        .with_name_contains("Tech")
        .with_min_confidence(0.75)
        .verified_only()
        .with_pagination(10, 25);

    assert_eq!(filter.entity_type, Some(EntityType::Organization));
    assert_eq!(filter.name_contains, Some("Tech".to_string()));
    assert_eq!(filter.min_confidence, Some(0.75));
    assert!(filter.verified_only);
    assert_eq!(filter.offset, 10);
    assert_eq!(filter.limit, 25);
}

#[tokio::test]
async fn test_relationship_filter_builder() {
    let source_id = Uuid::new_v4();
    let target_id = Uuid::new_v4();

    let filter = RelationshipFilter::new()
        .with_source(source_id)
        .with_target(target_id)
        .with_type("works_at")
        .valid_at(Utc::now())
        .with_pagination(0, 10);

    assert_eq!(filter.source_id, Some(source_id));
    assert_eq!(filter.target_id, Some(target_id));
    assert_eq!(filter.relation_type, Some("works_at".to_string()));
    assert!(filter.valid_at.is_some());
    assert_eq!(filter.offset, 0);
    assert_eq!(filter.limit, 10);
}

// ============================================================================
// Complex Graph Scenarios
// ============================================================================

#[tokio::test]
async fn test_create_organization_hierarchy() {
    // Create an org hierarchy
    let ceo = TestEntityBuilder::person("CEO Person").build();
    let vp = TestEntityBuilder::person("VP Person").build();
    let manager = TestEntityBuilder::person("Manager Person").build();
    let employee = TestEntityBuilder::person("Employee Person").build();

    let company = TestEntityBuilder::organization("Big Corp").build();
    let department = TestEntityBuilder::organization("Engineering").build();

    // Create relationships
    let rels = vec![
        TestRelationshipBuilder::works_at(ceo.id, company.id).build(),
        TestRelationshipBuilder::works_at(vp.id, company.id).build(),
        TestRelationshipBuilder::works_at(manager.id, department.id).build(),
        TestRelationshipBuilder::works_at(employee.id, department.id).build(),
        TestRelationshipBuilder::new(vp.id, ceo.id, "reports_to").build(),
        TestRelationshipBuilder::new(manager.id, vp.id, "reports_to").build(),
        TestRelationshipBuilder::new(employee.id, manager.id, "reports_to").build(),
        TestRelationshipBuilder::new(department.id, company.id, "part_of").build(),
    ];

    assert_eq!(rels.len(), 8);

    // Verify hierarchy
    assert_eq!(rels[4].relation_type, "reports_to");
    assert_eq!(rels[4].source_id, vp.id);
    assert_eq!(rels[4].target_id, ceo.id);
}

#[tokio::test]
async fn test_create_social_network() {
    // Create a small social network
    let people: Vec<_> = (0..5)
        .map(|i| TestEntityBuilder::person(&format!("Person {}", i)).build())
        .collect();

    // Create "knows" relationships (bidirectional)
    let mut relationships = Vec::new();
    for i in 0..people.len() {
        for j in (i + 1)..people.len() {
            relationships.push(
                TestRelationshipBuilder::knows(people[i].id, people[j].id).build()
            );
        }
    }

    // For 5 people, there should be 10 relationships (complete graph)
    assert_eq!(relationships.len(), 10);
}

#[tokio::test]
async fn test_create_event_with_participants() {
    // Create event and participants
    let start = Utc::now();
    let end = start + Duration::hours(2);

    let event = TestEntityBuilder::event("Team Standup", start, end).build();
    let organizer = TestEntityBuilder::person("Organizer").build();
    let participant1 = TestEntityBuilder::person("Participant 1").build();
    let participant2 = TestEntityBuilder::person("Participant 2").build();
    let location = TestEntityBuilder::location("Conference Room A", 0.0, 0.0).build();

    let relationships = vec![
        TestRelationshipBuilder::new(organizer.id, event.id, "organizes").build(),
        TestRelationshipBuilder::involves(event.id, participant1.id).build(),
        TestRelationshipBuilder::involves(event.id, participant2.id).build(),
        TestRelationshipBuilder::located_in(event.id, location.id).build(),
    ];

    assert_eq!(relationships.len(), 4);
    assert_eq!(relationships[0].relation_type, "organizes");
    assert_eq!(relationships[1].relation_type, "involves");
}

#[tokio::test]
async fn test_query_result_pagination() {
    let items: Vec<i32> = (1..=25).collect();
    let result = truffle_core::graph::QueryResult::new(items, 100, 0, 25);

    assert_eq!(result.items.len(), 25);
    assert_eq!(result.total, 100);
    assert_eq!(result.offset, 0);
    assert_eq!(result.limit, 25);
    assert!(result.has_more());
    assert_eq!(result.next_offset(), Some(25));

    // Second page
    let result2 = truffle_core::graph::QueryResult::new(vec![26, 27, 28], 100, 25, 25);
    assert_eq!(result2.offset, 25);
    assert!(result2.has_more());
    assert_eq!(result2.next_offset(), Some(50));

    // Last page
    let last_items: Vec<i32> = (91..=100).collect();
    let result3 = truffle_core::graph::QueryResult::new(last_items, 100, 90, 25);
    assert!(!result3.has_more());
    assert_eq!(result3.next_offset(), None);
}

#[tokio::test]
async fn test_entity_with_embedding_similarity_search() {
    // Create entities with different embeddings
    let base_emb = generate_test_embedding(100, 384);

    // Create variations
    let similar_emb: Vec<f32> = base_emb
        .iter()
        .map(|&v| (v + 0.05).clamp(-1.0, 1.0))
        .collect();

    let different_emb = generate_test_embedding(200, 384);

    let entity1 = TestEntityBuilder::person("Entity 1")
        .with_embedding(base_emb)
        .build();

    let entity2 = TestEntityBuilder::person("Entity 2")
        .with_embedding(similar_emb)
        .build();

    let entity3 = TestEntityBuilder::person("Entity 3")
        .with_embedding(different_emb)
        .build();

    // Verify embeddings are stored
    assert!(entity1.embedding.is_some());
    assert!(entity2.embedding.is_some());
    assert!(entity3.embedding.is_some());

    // Verify they have correct dimensions
    assert_eq!(entity1.embedding.as_ref().unwrap().len(), 384);
    assert_eq!(entity2.embedding.as_ref().unwrap().len(), 384);
    assert_eq!(entity3.embedding.as_ref().unwrap().len(), 384);
}
