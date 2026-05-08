//! Performance Benchmarks for Truffle Core Graph Operations
//!
//! These benchmarks measure the performance of core graph operations:
//! - Entity creation and persistence
//! - Graph traversal (BFS, DFS)
//! - Semantic search
//! - Contradiction detection

use criterion::{black_box, criterion_group, criterion_main, Criterion, BenchmarkId, AsyncBencher};
use std::sync::Arc;
use tokio::runtime::Runtime;
use uuid::Uuid;

use truffle_core::{
    Database,
    Entity, EntityType,
    GraphRepository, GraphQuery,
    GraphTraversal, TraversalConfig, PathFinder,
    validation::{Fact, FactValue, FactSource, Contradiction, ContradictionType, ContradictionSeverity},
};

// Import test utilities
use crate::integration::{
    TestEntityBuilder, TestRelationshipBuilder,
    generate_test_embedding, generate_similar_embeddings,
};

/// Setup function for benchmarks
fn setup_bench_db() -> Arc<Database> {
    Database::open_in_memory().expect("Failed to create in-memory database")
}

/// Benchmark entity creation
fn benchmark_entity_creation(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();

    let mut group = c.benchmark_group("entity_operations");

    // Benchmark single entity creation
    group.bench_function("create_single_entity", |b| {
        b.iter(|| {
            let entity = TestEntityBuilder::person("Benchmark Person")
                .with_role("Engineer")
                .with_email("bench@example.com")
                .build();
            black_box(entity);
        });
    });

    // Benchmark batch entity creation
    for batch_size in [10, 100, 1000].iter() {
        group.bench_with_input(
            BenchmarkId::new("create_entity_batch", batch_size),
            batch_size,
            |b, &size| {
                b.iter(|| {
                    let entities: Vec<_> = (0..size)
                        .map(|i| {
                            TestEntityBuilder::person(&format!("Person {}", i))
                                .with_confidence(0.85)
                                .build()
                        })
                        .collect();
                    black_box(entities);
                });
            },
        );
    }

    group.finish();
}

/// Benchmark entity creation with embeddings
fn benchmark_entity_creation_with_embeddings(c: &mut Criterion) {
    let mut group = c.benchmark_group("entity_with_embeddings");

    for dim in [128, 384, 768].iter() {
        group.bench_with_input(
            BenchmarkId::new("create_with_embedding", dim),
            dim,
            |b, &dim| {
                b.iter(|| {
                    let embedding = generate_test_embedding(42, dim);
                    let entity = TestEntityBuilder::person("Embedded Person")
                        .with_embedding(embedding)
                        .build();
                    black_box(entity);
                });
            },
        );
    }

    // Benchmark batch creation with embeddings
    group.bench_function("create_batch_100_with_384dim_embedding", |b| {
        b.iter(|| {
            let embeddings = generate_similar_embeddings(42, 100, 384, 0.1);
            let entities: Vec<_> = embeddings
                .into_iter()
                .enumerate()
                .map(|(i, emb)| {
                    TestEntityBuilder::person(&format!("Person {}", i))
                        .with_embedding(emb)
                        .build()
                })
                .collect();
            black_box(entities);
        });
    });

    group.finish();
}

/// Benchmark relationship creation
fn benchmark_relationship_creation(c: &mut Criterion) {
    let mut group = c.benchmark_group("relationship_operations");

    // Pre-create entities for relationship benchmarks
    let entity_a = TestEntityBuilder::person("Entity A").build();
    let entity_b = TestEntityBuilder::organization("Entity B").build();

    group.bench_function("create_single_relationship", |b| {
        b.iter(|| {
            let rel = TestRelationshipBuilder::works_at(entity_a.id, entity_b.id)
                .with_confidence(0.9)
                .build();
            black_box(rel);
        });
    });

    // Benchmark batch relationship creation
    for batch_size in [10, 100, 500].iter() {
        group.bench_with_input(
            BenchmarkId::new("create_relationship_batch", batch_size),
            batch_size,
            |b, &size| {
                // Create pool of entities
                let entities: Vec<_> = (0..size + 1)
                    .map(|i| TestEntityBuilder::person(&format!("Entity {}", i)).build())
                    .collect();

                b.iter(|| {
                    let relationships: Vec<_> = (0..size)
                        .map(|i| {
                            TestRelationshipBuilder::works_at(entities[i].id, entities[i + 1].id)
                                .build()
                        })
                        .collect();
                    black_box(relationships);
                });
            },
        );
    }

    group.finish();
}

/// Benchmark graph traversal operations
fn benchmark_graph_traversal(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();
    let mut group = c.benchmark_group("graph_traversal");

    // Benchmark BFS with different graph sizes and depths
    for (num_entities, max_depth) in [
        (10, 2),
        (50, 3),
        (100, 5),
    ].iter() {
        group.bench_with_input(
            BenchmarkId::new("bfs_traversal", format!("{}_entities_depth_{}", num_entities, max_depth)),
            &(*num_entities, *max_depth),
            |b, (num, depth)| {
                b.to_async(&rt).iter(|| async move {
                    let db = setup_bench_db();
                    let repo = GraphRepository::new(Arc::new(RwLock::new(db)));

                    let config = TraversalConfig {
                        max_depth: *depth,
                        relationship_types: None,
                        min_confidence: 0.5,
                    };

                    let mut traversal = GraphTraversal::new(&repo, config);

                    // Use a dummy ID for traversal (repository is mocked)
                    let start_id = Uuid::new_v4();

                    let mut count = 0;
                    traversal
                        .bfs(&start_id, |_entity, _depth| {
                            count += 1;
                            count < *num
                        })
                        .await
                        .unwrap();

                    black_box(count);
                });
            },
        );
    }

    // Benchmark DFS
    for (num_entities, max_depth) in [
        (10, 2),
        (50, 3),
    ].iter() {
        group.bench_with_input(
            BenchmarkId::new("dfs_traversal", format!("{}_entities_depth_{}", num_entities, max_depth)),
            &(*num_entities, *max_depth),
            |b, (num, depth)| {
                b.to_async(&rt).iter(|| async move {
                    let db = setup_bench_db();
                    let repo = GraphRepository::new(Arc::new(RwLock::new(db)));

                    let config = TraversalConfig {
                        max_depth: *depth,
                        relationship_types: None,
                        min_confidence: 0.5,
                    };

                    let mut traversal = GraphTraversal::new(&repo, config);

                    let start_id = Uuid::new_v4();

                    let mut count = 0;
                    traversal
                        .dfs(&start_id, |_entity, _depth| {
                            count += 1;
                            count < *num
                        })
                        .await
                        .unwrap();

                    black_box(count);
                });
            },
        );
    }

    group.finish();
}

/// Benchmark path finding algorithms
fn benchmark_path_finding(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();
    let mut group = c.benchmark_group("path_finding");

    // Benchmark BFS path finding
    for max_depth in [2, 5, 10].iter() {
        group.bench_with_input(
            BenchmarkId::new("bfs_path_find", max_depth),
            max_depth,
            |b, &depth| {
                b.to_async(&rt).iter(|| async move {
                    let db = setup_bench_db();
                    let repo = GraphRepository::new(Arc::new(RwLock::new(db)));

                    let finder = PathFinder::new(&repo);
                    let start = Uuid::new_v4();
                    let end = Uuid::new_v4();

                    let path = finder
                        .find_path_bfs(&start, &end, depth)
                        .await
                        .unwrap();

                    black_box(path);
                });
            },
        );
    }

    // Benchmark bidirectional path finding
    for max_depth in [2, 5, 10].iter() {
        group.bench_with_input(
            BenchmarkId::new("bidirectional_path_find", max_depth),
            max_depth,
            |b, &depth| {
                b.to_async(&rt).iter(|| async move {
                    let db = setup_bench_db();
                    let repo = GraphRepository::new(Arc::new(RwLock::new(db)));

                    let finder = PathFinder::new(&repo);
                    let start = Uuid::new_v4();
                    let end = Uuid::new_v4();

                    let path = finder
                        .find_path_bidirectional(&start, &end, depth)
                        .await
                        .unwrap();

                    black_box(path);
                });
            },
        );
    }

    group.finish();
}

/// Benchmark semantic search operations
fn benchmark_semantic_search(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();
    let mut group = c.benchmark_group("semantic_search");

    // Benchmark embedding similarity calculation
    for dim in [128, 384, 768, 1536].iter() {
        group.bench_with_input(
            BenchmarkId::new("embedding_similarity", dim),
            dim,
            |b, &dim| {
                let emb1 = generate_test_embedding(1, dim);
                let emb2 = generate_test_embedding(2, dim);

                b.iter(|| {
                    let similarity = calculate_cosine_similarity(&emb1, &emb2);
                    black_box(similarity);
                });
            },
        );
    }

    // Benchmark search with different result limits
    for limit in [10, 50, 100].iter() {
        group.bench_with_input(
            BenchmarkId::new("semantic_search_limit", limit),
            limit,
            |b, &limit| {
                b.to_async(&rt).iter(|| async move {
                    let db = setup_bench_db();
                    let repo = GraphRepository::new(Arc::new(RwLock::new(db)));

                    let results = repo
                        .semantic_search("software engineer", limit)
                        .await
                        .unwrap();

                    black_box(results);
                });
            },
        );
    }

    // Benchmark batch embedding generation
    group.bench_function("generate_1000_embeddings_384dim", |b| {
        b.iter(|| {
            let embeddings = generate_similar_embeddings(42, 1000, 384, 0.1);
            black_box(embeddings);
        });
    });

    group.finish();
}

/// Benchmark contradiction detection
fn benchmark_validation(c: &mut Criterion) {
    let mut group = c.benchmark_group("validation");

    // Benchmark fact creation with validation
    for num_facts in [10, 50, 100].iter() {
        group.bench_with_input(
            BenchmarkId::new("create_facts_batch", num_facts),
            num_facts,
            |b, &num| {
                let entity_id = Uuid::new_v4();
                b.iter(|| {
                    let facts: Vec<_> = (0..num)
                        .map(|i| {
                            TestFactBuilder::employer(
                                entity_id,
                                format!("Company {}", i)
                            )
                            .with_confidence(0.8)
                            .build()
                        })
                        .collect();
                    black_box(facts);
                });
            },
        );
    }

    // Benchmark contradiction detection between fact pairs
    group.bench_function("detect_temporal_contradiction", |b| {
        let entity_id = Uuid::new_v4();
        let now = Utc::now();

        let fact1 = TestFactBuilder::employer(entity_id, "Company A")
            .with_valid_range(Some(now - Duration::days(180)), Some(now))
            .build();

        let fact2 = TestFactBuilder::employer(entity_id, "Company B")
            .with_valid_range(Some(now - Duration::days(90)), Some(now + Duration::days(30)))
            .build();

        b.iter(|| {
            let overlaps = fact1.overlaps_range(
                fact2.valid_from.unwrap(),
                fact2.valid_until.unwrap()
            );
            black_box(overlaps);
        });
    });

    // Benchmark contradiction creation
    group.bench_function("create_contradiction", |b| {
        let entity_id = Uuid::new_v4();
        let fact1_id = Uuid::new_v4();
        let fact2_id = Uuid::new_v4();

        b.iter(|| {
            let contradiction = Contradiction::new(
                entity_id,
                fact1_id,
                fact2_id,
                ContradictionType::Temporal,
                ContradictionSeverity::High,
            );
            black_box(contradiction);
        });
    });

    group.finish();
}

/// Benchmark name similarity calculation
fn benchmark_name_similarity(c: &mut Criterion) {
    let mut group = c.benchmark_group("name_similarity");

    let test_names = [
        ("John Smith", "John Smith"),           // Exact match
        ("John Smith", "Jon Smith"),            // Typo
        ("Johnathan Smith", "John Smith"),      // Nickname
        ("Robert Johnson", "Bob Johnson"),      // Different first name
        ("Christopher", "Chris"),               // Shortened
    ];

    for (i, (name1, name2)) in test_names.iter().enumerate() {
        group.bench_with_input(
            BenchmarkId::new(format!("similarity_case_{}", i), format!("{}_vs_{}", name1, name2)),
            &(*name1, *name2),
            |b, (n1, n2)| {
                b.iter(|| {
                    let similarity = jaro_winkler_similarity(n1, n2);
                    black_box(similarity);
                });
            },
        );
    }

    // Benchmark batch similarity calculation
    group.bench_function("batch_100_similarity_checks", |b| {
        let names: Vec<_> = (0..100)
            .map(|i| format!("Person {} Name", i))
            .collect();

        b.iter(|| {
            let mut scores = Vec::new();
            for i in 0..names.len() {
                for j in (i + 1)..names.len() {
                    let sim = jaro_winkler_similarity(&names[i], &names[j]);
                    scores.push(sim);
                }
            }
            black_box(scores);
        });
    });

    group.finish();
}

/// Helper function to calculate cosine similarity
fn calculate_cosine_similarity(a: &[f32], b: &[f32]) -> f32 {
    if a.len() != b.len() || a.is_empty() {
        return 0.0;
    }

    let dot_product: f32 = a.iter().zip(b.iter()).map(|(x, y)| x * y).sum();
    let norm_a: f32 = a.iter().map(|x| x * x).sum::<f32>().sqrt();
    let norm_b: f32 = b.iter().map(|x| x * x).sum::<f32>().sqrt();

    if norm_a == 0.0 || norm_b == 0.0 {
        return 0.0;
    }

    (dot_product / (norm_a * norm_b)).clamp(-1.0, 1.0)
}

/// Jaro-Winkler similarity algorithm
fn jaro_winkler_similarity(s1: &str, s2: &str) -> f32 {
    if s1 == s2 {
        return 1.0;
    }
    if s1.is_empty() || s2.is_empty() {
        return 0.0;
    }

    let s1_chars: Vec<char> = s1.chars().collect();
    let s2_chars: Vec<char> = s2.chars().collect();
    let len1 = s1_chars.len();
    let len2 = s2_chars.len();

    let match_window = (len1.max(len2) / 2).saturating_sub(1);
    let mut s1_matches = vec![false; len1];
    let mut s2_matches = vec![false; len2];

    let mut matches = 0;
    for i in 0..len1 {
        let start = i.saturating_sub(match_window);
        let end = (i + match_window + 1).min(len2);
        for j in start..end {
            if !s2_matches[j] && s1_chars[i] == s2_chars[j] {
                s1_matches[i] = true;
                s2_matches[j] = true;
                matches += 1;
                break;
            }
        }
    }

    if matches == 0 {
        return 0.0;
    }

    let mut transpositions = 0;
    let mut k = 0;
    for i in 0..len1 {
        if !s1_matches[i] {
            continue;
        }
        while !s2_matches[k] {
            k += 1;
        }
        if s1_chars[i] != s2_chars[k] {
            transpositions += 1;
        }
        k += 1;
    }

    let matches_f = matches as f32;
    let jaro = ((matches_f / len1 as f32)
        + (matches_f / len2 as f32)
        + ((matches_f - transpositions as f32 / 2.0) / matches_f))
        / 3.0;

    let prefix_len = s1_chars
        .iter()
        .zip(s2_chars.iter())
        .take(4)
        .take_while(|(a, b)| a == b)
        .count();

    jaro + (prefix_len as f32 * 0.1 * (1.0 - jaro))
}

use tokio::sync::RwLock;
use chrono::Duration;

// Define benchmark groups
criterion_group!(
    benches,
    benchmark_entity_creation,
    benchmark_entity_creation_with_embeddings,
    benchmark_relationship_creation,
    benchmark_graph_traversal,
    benchmark_path_finding,
    benchmark_semantic_search,
    benchmark_validation,
    benchmark_name_similarity
);

criterion_main!(benches);
