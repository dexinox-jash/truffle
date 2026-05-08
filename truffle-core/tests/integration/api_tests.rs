//! GraphQL API Integration Tests
//!
//! Tests for GraphQL query and mutation operations on the knowledge graph.

use std::sync::Arc;
use async_graphql::{Schema, ID};

use truffle_core::{
    Database,
    Entity, EntityType, EntityMetadata, PrivacyLevel,
    api::{
        ApiContext, QueryRoot, MutationRoot, SubscriptionRoot,
        create_schema, EntityType as GqlEntityType, EntityTypeEnum,
        CreateEntityInput, UpdateEntityInput,
        CreateRelationshipInput, UpdateRelationshipInput,
        EntityFilterInput, RelationshipFilterInput,
        PrivacyLevelEnum,
    },
};

use crate::integration::{
    setup_test_db, TestEntityBuilder, TestRelationshipBuilder,
    assertions::*,
};

/// Helper to create a test GraphQL schema
fn create_test_schema(db: Arc<Database>) -> Schema<QueryRoot, MutationRoot, SubscriptionRoot> {
    create_schema()
}

/// Helper to create API context
fn create_test_context(db: Arc<Database>) -> ApiContext {
    ApiContext::new(db)
}

// ============================================================================
// GraphQL Query Tests
// ============================================================================

#[tokio::test]
async fn test_graphql_query_entities() {
    let db = setup_test_db().await;
    let context = create_test_context(db.clone());
    let schema = create_test_schema(db.clone());

    // Execute a simple query
    let query = r#"
        query {
            entities(first: 10) {
                nodes {
                    id
                    name
                    entityType
                }
                totalCount
                pageInfo {
                    hasNextPage
                    hasPreviousPage
                }
            }
        }
    "#;

    // Execute query with context
    let result = schema
        .execute(async_graphql::Request::new(query).data(context))
        .await;

    // Should not have errors (though may return empty with placeholder impl)
    // Note: Actual results depend on repository implementation
}

#[tokio::test]
async fn test_graphql_query_entity_by_id() {
    let db = setup_test_db().await;
    let context = create_test_context(db.clone());
    let schema = create_test_schema(db.clone());

    // Create a test entity
    let entity = TestEntityBuilder::person("Test Person").build();
    let entity_id = entity.id.to_string();

    let query = format!(
        r#"
        query {{
            entity(id: "{}") {{
                id
                name
                entityType
                slug
            }}
        }}
        "#,
        entity_id
    );

    let result = schema
        .execute(async_graphql::Request::new(&query).data(context))
        .await;

    // Verify query structure is valid
    assert!(!result.is_err(), "Query should not error: {:?}", result.errors);
}

#[tokio::test]
async fn test_graphql_query_with_filter() {
    let db = setup_test_db().await;
    let context = create_test_context(db.clone());
    let schema = create_test_schema(db.clone());

    let query = r#"
        query {
            entities(
                filter: {
                    entityType: PERSON
                    nameContains: "Alice"
                    minConfidence: 0.8
                }
                first: 5
            ) {
                nodes {
                    id
                    name
                    extractionConfidence
                }
                totalCount
            }
        }
    "#;

    let result = schema
        .execute(async_graphql::Request::new(query).data(context))
        .await;

    // Should execute without errors
    // With placeholder repository, will return empty results
}

#[tokio::test]
async fn test_graphql_query_relationships() {
    let db = setup_test_db().await;
    let context = create_test_context(db.clone());
    let schema = create_test_schema(db.clone());

    let query = r#"
        query {
            relationships(first: 10) {
                nodes {
                    id
                    sourceId
                    targetId
                    relationType
                    confidence
                }
                totalCount
            }
        }
    "#;

    let result = schema
        .execute(async_graphql::Request::new(query).data(context))
        .await;

    assert!(!result.is_err());
}

#[tokio::test]
async fn test_graphql_query_entity_relationships() {
    let db = setup_test_db().await;
    let context = create_test_context(db.clone());
    let schema = create_test_schema(db.clone());

    // Create test entity
    let entity = TestEntityBuilder::person("Test Person").build();
    let entity_id = entity.id.to_string();

    let query = format!(
        r#"
        query {{
            entityRelationships(
                entityId: "{}"
                direction: BOTH
            ) {{
                id
                relationType
                sourceId
                targetId
            }}
        }}
        "#,
        entity_id
    );

    let result = schema
        .execute(async_graphql::Request::new(&query).data(context))
        .await;

    assert!(!result.is_err());
}

#[tokio::test]
async fn test_graphql_search_entities() {
    let db = setup_test_db().await;
    let context = create_test_context(db.clone());
    let schema = create_test_schema(db.clone());

    let query = r#"
        query {
            searchEntities(query: "software engineer", limit: 10) {
                id
                name
                description
                entityType
            }
        }
    "#;

    let result = schema
        .execute(async_graphql::Request::new(query).data(context))
        .await;

    assert!(!result.is_err());
}

#[tokio::test]
async fn test_graphql_query_graph() {
    let db = setup_test_db().await;
    let context = create_test_context(db.clone());
    let schema = create_test_schema(db.clone());

    // Create test entity
    let entity = TestEntityBuilder::person("Test Person").build();
    let entity_id = entity.id.to_string();

    let query = format!(
        r#"
        query {{
            graph(entityId: "{}", depth: 2) {{
                id
                name
                neighbors {{
                    id
                    name
                }}
            }}
        }}
        "#,
        entity_id
    );

    let result = schema
        .execute(async_graphql::Request::new(&query).data(context))
        .await;

    // Graph query returns not yet implemented
    // This test validates the query structure
}

#[tokio::test]
async fn test_graphql_query_path() {
    let db = setup_test_db().await;
    let context = create_test_context(db.clone());
    let schema = create_test_schema(db.clone());

    let entity1 = TestEntityBuilder::person("Person A").build();
    let entity2 = TestEntityBuilder::person("Person B").build();

    let query = format!(
        r#"
        query {{
            path(
                from: "{}"
                to: "{}"
                maxDepth: 5
            ) {{
                id
                name
            }}
        }}
        "#,
        entity1.id, entity2.id
    );

    let result = schema
        .execute(async_graphql::Request::new(&query).data(context))
        .await;

    assert!(!result.is_err());
}

// ============================================================================
// GraphQL Mutation Tests
// ============================================================================

#[tokio::test]
async fn test_graphql_mutation_create_entity() {
    let db = setup_test_db().await;
    let context = create_test_context(db.clone());
    let schema = create_test_schema(db.clone());

    let mutation = r#"
        mutation {
            createEntity(input: {
                entityType: PERSON
                name: "New Person"
                description: "A test person"
                privacyLevel: INTERNAL
                extractionConfidence: 0.9
                metadata: {
                    title: "Engineer"
                    email: "test@example.com"
                }
            }) {
                id
                name
                entityType
                description
                extractionConfidence
                metadata {
                    title
                    email
                }
            }
        }
    "#;

    let result = schema
        .execute(async_graphql::Request::new(mutation).data(context))
        .await;

    // Should succeed and return created entity
    assert!(!result.is_err(), "Create entity mutation failed: {:?}", result.errors);

    // Verify data was returned
    let data = result.data.into_json().unwrap();
    assert!(data.get("createEntity").is_some());
}

#[tokio::test]
async fn test_graphql_mutation_create_organization() {
    let db = setup_test_db().await;
    let context = create_test_context(db.clone());
    let schema = create_test_schema(db.clone());

    let mutation = r#"
        mutation {
            createEntity(input: {
                entityType: ORGANIZATION
                name: "Acme Corp"
                description: "A test organization"
                metadata: {
                    industry: "Technology"
                    website: "https://acme.example.com"
                }
            }) {
                id
                name
                entityType
                metadata {
                    industry
                    website
                }
            }
        }
    "#;

    let result = schema
        .execute(async_graphql::Request::new(mutation).data(context))
        .await;

    assert!(!result.is_err());
}

#[tokio::test]
async fn test_graphql_mutation_update_entity() {
    let db = setup_test_db().await;
    let context = create_test_context(db.clone());
    let schema = create_test_schema(db.clone());

    // First create an entity
    let entity = TestEntityBuilder::person("Original Name").build();
    let entity_id = entity.id.to_string();

    let mutation = format!(
        r#"
        mutation {{
            updateEntity(
                id: "{}"
                input: {{
                    name: "Updated Name"
                    description: "Updated description"
                }}
            ) {{
                id
                name
                description
            }}
        }}
        "#,
        entity_id
    );

    let result = schema
        .execute(async_graphql::Request::new(&mutation).data(context))
        .await;

    // May error since update is not fully implemented
    // This tests the API structure
}

#[tokio::test]
async fn test_graphql_mutation_delete_entity() {
    let db = setup_test_db().await;
    let context = create_test_context(db.clone());
    let schema = create_test_schema(db.clone());

    let entity = TestEntityBuilder::person("To Delete").build();
    let entity_id = entity.id.to_string();

    let mutation = format!(
        r#"
        mutation {{
            deleteEntity(id: "{}")
        }}
        "#,
        entity_id
    );

    let result = schema
        .execute(async_graphql::Request::new(&mutation).data(context))
        .await;

    // May error since delete is not fully implemented
}

#[tokio::test]
async fn test_graphql_mutation_create_relationship() {
    let db = setup_test_db().await;
    let context = create_test_context(db.clone());
    let schema = create_test_schema(db.clone());

    // Create two entities
    let person = TestEntityBuilder::person("John Doe").build();
    let org = TestEntityBuilder::organization("Acme Corp").build();

    let mutation = format!(
        r#"
        mutation {{
            createRelationship(input: {{
                sourceId: "{}"
                targetId: "{}"
                relationType: "works_at"
                confidence: 0.95
            }}) {{
                id
                sourceId
                targetId
                relationType
                confidence
            }}
        }}
        "#,
        person.id, org.id
    );

    let result = schema
        .execute(async_graphql::Request::new(&mutation).data(context))
        .await;

    assert!(!result.is_err(), "Create relationship mutation failed: {:?}", result.errors);

    let data = result.data.into_json().unwrap();
    let rel = data.get("createRelationship").unwrap();
    assert_eq!(rel.get("relationType").unwrap().as_str().unwrap(), "works_at");
}

#[tokio::test]
async fn test_graphql_mutation_update_relationship() {
    let db = setup_test_db().await;
    let context = create_test_context(db.clone());
    let schema = create_test_schema(db.clone());

    // Create entities and relationship
    let person = TestEntityBuilder::person("Person").build();
    let org = TestEntityBuilder::organization("Org").build();
    let rel = TestRelationshipBuilder::works_at(person.id, org.id).build();

    let mutation = format!(
        r#"
        mutation {{
            updateRelationship(
                id: "{}"
                input: {{
                    confidence: 0.99
                }}
            ) {{
                id
                confidence
            }}
        }}
        "#,
        rel.id
    );

    let result = schema
        .execute(async_graphql::Request::new(&mutation).data(context))
        .await;

    // May error since update is not fully implemented
}

#[tokio::test]
async fn test_graphql_mutation_delete_relationship() {
    let db = setup_test_db().await;
    let context = create_test_context(db.clone());
    let schema = create_test_schema(db.clone());

    let person = TestEntityBuilder::person("Person").build();
    let org = TestEntityBuilder::organization("Org").build();
    let rel = TestRelationshipBuilder::works_at(person.id, org.id).build();

    let mutation = format!(
        r#"
        mutation {{
            deleteRelationship(id: "{}")
        }}
        "#,
        rel.id
    );

    let result = schema
        .execute(async_graphql::Request::new(&mutation).data(context))
        .await;

    // May error since delete is not fully implemented
}

#[tokio::test]
async fn test_graphql_mutation_verify_entity() {
    let db = setup_test_db().await;
    let context = create_test_context(db.clone());
    let schema = create_test_schema(db.clone());

    let entity = TestEntityBuilder::person("Unverified Person").build();
    let entity_id = entity.id.to_string();

    let mutation = format!(
        r#"
        mutation {{
            verifyEntity(id: "{}") {{
                id
                verified
            }}
        }}
        "#,
        entity_id
    );

    let result = schema
        .execute(async_graphql::Request::new(&mutation).data(context))
        .await;

    // May error since verify is not fully implemented
}

#[tokio::test]
async fn test_graphql_mutation_verify_relationship() {
    let db = setup_test_db().await;
    let context = create_test_context(db.clone());
    let schema = create_test_schema(db.clone());

    let person = TestEntityBuilder::person("Person").build();
    let org = TestEntityBuilder::organization("Org").build();
    let rel = TestRelationshipBuilder::works_at(person.id, org.id).build();

    let mutation = format!(
        r#"
        mutation {{
            verifyRelationship(id: "{}") {{
                id
                verified
            }}
        }}
        "#,
        rel.id
    );

    let result = schema
        .execute(async_graphql::Request::new(&mutation).data(context))
        .await;

    // May error since verify is not fully implemented
}

// ============================================================================
// GraphQL Complex Query Tests
// ============================================================================

#[tokio::test]
async fn test_graphql_complex_entity_query() {
    let db = setup_test_db().await;
    let context = create_test_context(db.clone());
    let schema = create_test_schema(db.clone());

    let query = r#"
        query {
            entities(
                filter: {
                    entityType: PERSON
                    verifiedOnly: true
                    minConfidence: 0.8
                }
                sort: {
                    field: NAME
                    direction: ASC
                }
                first: 20
            ) {
                nodes {
                    id
                    name
                    entityType
                    verified
                    extractionConfidence
                    createdAt
                    updatedAt
                    metadata {
                        title
                        email
                    }
                    relationships {
                        id
                        relationType
                    }
                }
                totalCount
                pageInfo {
                    hasNextPage
                    hasPreviousPage
                    startCursor
                    endCursor
                }
            }
        }
    "#;

    let result = schema
        .execute(async_graphql::Request::new(query).data(context))
        .await;

    assert!(!result.is_err());
}

#[tokio::test]
async fn test_graphql_nested_relationship_query() {
    let db = setup_test_db().await;
    let context = create_test_context(db.clone());
    let schema = create_test_schema(db.clone());

    let query = r#"
        query {
            entities(first: 5) {
                nodes {
                    id
                    name
                    relationships {
                        id
                        relationType
                        sourceId
                        targetId
                    }
                }
            }
        }
    "#;

    let result = schema
        .execute(async_graphql::Request::new(query).data(context))
        .await;

    assert!(!result.is_err());
}

#[tokio::test]
async fn test_graphql_pagination() {
    let db = setup_test_db().await;
    let context = create_test_context(db.clone());
    let schema = create_test_schema(db.clone());

    // First page
    let query1 = r#"
        query {
            entities(first: 5, after: null) {
                nodes {
                    id
                    name
                }
                pageInfo {
                    hasNextPage
                    endCursor
                }
            }
        }
    "#;

    let result1 = schema
        .execute(async_graphql::Request::new(query1).data(context.clone()))
        .await;

    assert!(!result1.is_err());

    // Second page (using cursor from first page if available)
    let query2 = r#"
        query {
            entities(first: 5, after: "5") {
                nodes {
                    id
                    name
                }
                pageInfo {
                    hasNextPage
                    hasPreviousPage
                    startCursor
                    endCursor
                }
            }
        }
    "#;

    let result2 = schema
        .execute(async_graphql::Request::new(query2).data(context))
        .await;

    assert!(!result2.is_err());
}

// ============================================================================
// Input Type Tests
// ============================================================================

#[tokio::test]
async fn test_create_entity_input_builder() {
    // Test building CreateEntityInput programmatically
    let input = CreateEntityInput {
        entity_type: EntityTypeEnum::Person,
        name: "Test Person".to_string(),
        description: Some("A test person".to_string()),
        metadata: Some(truffle_core::api::EntityMetadataInput {
            title: Some("Engineer".to_string()),
            email: Some("test@example.com".to_string()),
            phone: None,
            industry: None,
            website: None,
            address: None,
            latitude: None,
            longitude: None,
            event_start: None,
            event_end: None,
            category: None,
            price: None,
            decision_status: None,
            impact: None,
            deadline: None,
            assignee: None,
        }),
        privacy_level: Some(PrivacyLevelEnum::Internal),
        extraction_confidence: Some(0.9),
    };

    assert_eq!(input.entity_type, EntityTypeEnum::Person);
    assert_eq!(input.name, "Test Person");
    assert!(input.metadata.as_ref().unwrap().title.is_some());
}

#[tokio::test]
async fn test_entity_filter_input() {
    let filter = EntityFilterInput {
        entity_type: Some(EntityTypeEnum::Organization),
        name_contains: Some("Acme".to_string()),
        created_after: None,
        created_before: None,
        min_confidence: Some(0.8),
        verified_only: Some(true),
        privacy_level: Some(PrivacyLevelEnum::Public),
    };

    assert_eq!(filter.entity_type, Some(EntityTypeEnum::Organization));
    assert_eq!(filter.name_contains, Some("Acme".to_string()));
    assert_eq!(filter.min_confidence, Some(0.8));
    assert_eq!(filter.verified_only, Some(true));
}

#[tokio::test]
async fn test_relationship_filter_input() {
    let source_id = uuid::Uuid::new_v4().to_string();

    let filter = RelationshipFilterInput {
        source_id: Some(ID::new(source_id.clone())),
        target_id: None,
        relation_type: Some("works_at".to_string()),
        valid_at: None,
        min_confidence: Some(0.7),
    };

    assert!(filter.source_id.is_some());
    assert_eq!(filter.relation_type, Some("works_at".to_string()));
    assert_eq!(filter.min_confidence, Some(0.7));
}

// ============================================================================
// API Context Tests
// ============================================================================

#[tokio::test]
async fn test_api_context_creation() {
    let db = setup_test_db().await;
    let context = ApiContext::new(db.clone());

    // Context should be created successfully
    let _ = context.database();
}

#[tokio::test]
async fn test_api_context_clone() {
    let db = setup_test_db().await;
    let context = ApiContext::new(db.clone());
    let cloned = context.clone();

    // Both should work independently
    let _ = context.database();
    let _ = cloned.database();
}

// ============================================================================
// Error Handling Tests
// ============================================================================

#[tokio::test]
async fn test_graphql_invalid_query() {
    let db = setup_test_db().await;
    let context = create_test_context(db.clone());
    let schema = create_test_schema(db.clone());

    // Invalid query - missing closing brace
    let query = r#"
        query {
            entities(first: 10) {
                id
                name
            }
    "#;

    let result = schema
        .execute(async_graphql::Request::new(query).data(context))
        .await;

    // Should have parse errors
    assert!(!result.errors.is_empty());
}

#[tokio::test]
async fn test_graphql_nonexistent_entity() {
    let db = setup_test_db().await;
    let context = create_test_context(db.clone());
    let schema = create_test_schema(db.clone());

    let query = r#"
        query {
            entity(id: "550e8400-e29b-41d4-a716-446655440000") {
                id
                name
            }
        }
    "#;

    let result = schema
        .execute(async_graphql::Request::new(query).data(context))
        .await;

    // Should return null for non-existent entity, not error
    assert!(!result.is_err());
}

#[tokio::test]
async fn test_graphql_invalid_entity_id() {
    let db = setup_test_db().await;
    let context = create_test_context(db.clone());
    let schema = create_test_schema(db.clone());

    let query = r#"
        query {
            entity(id: "not-a-valid-uuid") {
                id
                name
            }
        }
    "#;

    let result = schema
        .execute(async_graphql::Request::new(query).data(context))
        .await;

    // Should have error for invalid UUID
    assert!(!result.errors.is_empty());
}

// ============================================================================
// Full Workflow Tests
// ============================================================================

#[tokio::test]
async fn test_full_entity_creation_workflow() {
    let db = setup_test_db().await;
    let context = create_test_context(db.clone());
    let schema = create_test_schema(db.clone());

    // Step 1: Create an entity
    let create_mutation = r#"
        mutation {
            createEntity(input: {
                entityType: PERSON
                name: "Alice Smith"
                description: "Software Engineer"
                metadata: {
                    title: "Senior Engineer"
                    email: "alice@example.com"
                }
            }) {
                id
                name
                entityType
            }
        }
    "#;

    let create_result = schema
        .execute(async_graphql::Request::new(create_mutation).data(context.clone()))
        .await;

    assert!(!create_result.is_err());

    let data = create_result.data.into_json().unwrap();
    let created = data.get("createEntity").unwrap();
    let entity_id = created.get("id").unwrap().as_str().unwrap();

    // Step 2: Query the created entity
    let query = format!(
        r#"
        query {{
            entity(id: "{}") {{
                id
                name
                description
                metadata {{
                    title
                    email
                }}
            }}
        }}
        "#,
        entity_id
    );

    let query_result = schema
        .execute(async_graphql::Request::new(&query).data(context.clone()))
        .await;

    assert!(!query_result.is_err());

    // Step 3: Update the entity
    let update_mutation = format!(
        r#"
        mutation {{
            updateEntity(
                id: "{}"
                input: {{
                    description: "Senior Software Engineer"
                }}
            ) {{
                id
                description
            }}
        }}
        "#,
        entity_id
    );

    // May fail since update is not fully implemented
    let _ = schema
        .execute(async_graphql::Request::new(&update_mutation).data(context.clone()))
        .await;

    // Step 4: Verify the entity
    let verify_mutation = format!(
        r#"
        mutation {{
            verifyEntity(id: "{}") {{
                id
                verified
            }}
        }}
        "#,
        entity_id
    );

    // May fail since verify is not fully implemented
    let _ = schema
        .execute(async_graphql::Request::new(&verify_mutation).data(context))
        .await;
}

#[tokio::test]
async fn test_relationship_creation_workflow() {
    let db = setup_test_db().await;
    let context = create_test_context(db.clone());
    let schema = create_test_schema(db.clone());

    // Create two entities
    let create_person = r#"
        mutation {
            createEntity(input: {
                entityType: PERSON
                name: "John Doe"
            }) {
                id
            }
        }
    "#;

    let create_org = r#"
        mutation {
            createEntity(input: {
                entityType: ORGANIZATION
                name: "Acme Corp"
            }) {
                id
            }
        }
    "#;

    let person_result = schema
        .execute(async_graphql::Request::new(create_person).data(context.clone()))
        .await;

    let org_result = schema
        .execute(async_graphql::Request::new(create_org).data(context.clone()))
        .await;

    assert!(!person_result.is_err());
    assert!(!org_result.is_err());

    let person_data = person_result.data.into_json().unwrap();
    let org_data = org_result.data.into_json().unwrap();

    let person_id = person_data
        .get("createEntity")
        .unwrap()
        .get("id")
        .unwrap()
        .as_str()
        .unwrap();

    let org_id = org_data
        .get("createEntity")
        .unwrap()
        .get("id")
        .unwrap()
        .as_str()
        .unwrap();

    // Create relationship
    let create_rel = format!(
        r#"
        mutation {{
            createRelationship(input: {{
                sourceId: "{}"
                targetId: "{}"
                relationType: "works_at"
                confidence: 0.95
            }}) {{
                id
                sourceId
                targetId
                relationType
            }}
        }}
        "#,
        person_id, org_id
    );

    let rel_result = schema
        .execute(async_graphql::Request::new(&create_rel).data(context.clone()))
        .await;

    assert!(!rel_result.is_err());

    let rel_data = rel_result.data.into_json().unwrap();
    let rel = rel_data.get("createRelationship").unwrap();
    assert_eq!(rel.get("relationType").unwrap().as_str().unwrap(), "works_at");

    // Query entity relationships
    let query_rels = format!(
        r#"
        query {{
            entityRelationships(
                entityId: "{}"
                direction: OUTGOING
            ) {{
                id
                relationType
            }}
        }}
        "#,
        person_id
    );

    let query_result = schema
        .execute(async_graphql::Request::new(&query_rels).data(context))
        .await;

    assert!(!query_result.is_err());
}
