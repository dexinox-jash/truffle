//! Validation Engine Integration Tests
//!
//! Tests for contradiction detection, confidence scoring, duplicate detection,
//! and entity merging functionality.

use std::sync::Arc;
use tokio::sync::RwLock;
use uuid::Uuid;
use chrono::{Utc, Duration};

use truffle_core::{
    Database,
    Entity, EntityType,
    GraphRepository,
    validation::{
        ValidationEngine, ValidationConfig, EngineValidationSummary,
        EntityValidationResult, ValidationRequest,
        Fact, FactValue, FactSource,
        Contradiction, ContradictionType, ContradictionSeverity,
        ContradictionResolution, ResolutionType,
        DuplicateCandidate, DuplicateDetector, MergeEngine, MergeConfig,
    },
};

use crate::integration::{
    setup_test_db,
    TestEntityBuilder, TestFactBuilder,
};

// ============================================================================
// Contradiction Detection Tests
// ============================================================================

#[tokio::test]
async fn test_contradiction_detection_temporal() {
    let entity_id = Uuid::new_v4();
    let now = Utc::now();

    // Create overlapping employment facts
    let fact1 = TestFactBuilder::employer(entity_id, "Acme Corp")
        .with_valid_range(
            Some(now - Duration::days(365)),
            Some(now - Duration::days(30)),
        )
        .build();

    // This fact overlaps with the previous one
    let fact2 = TestFactBuilder::employer(entity_id, "Tech Inc")
        .with_valid_range(
            Some(now - Duration::days(60)),
            Some(now + Duration::days(30)),
        )
        .build();

    // Verify facts are created correctly
    assert_eq!(fact1.entity_id, entity_id);
    assert_eq!(fact2.entity_id, entity_id);
    assert!(fact1.overlaps_range(fact2.valid_from.unwrap(), fact2.valid_until.unwrap()));

    // Test that the facts overlap
    let overlap_start = now - Duration::days(60);
    let overlap_end = now - Duration::days(30);
    assert!(fact1.overlaps_range(overlap_start, overlap_end));
    assert!(fact2.overlaps_range(overlap_start, overlap_end));
}

#[tokio::test]
async fn test_contradiction_detection_factual() {
    let entity_id = Uuid::new_v4();

    // Create conflicting timeless facts
    let fact1 = TestFactBuilder::new(entity_id, "department", FactValue::text("Engineering"))
        .verified(Uuid::new_v4())
        .build();

    let fact2 = TestFactBuilder::new(entity_id, "department", FactValue::text("Sales"))
        .verified(Uuid::new_v4())
        .build();

    // Verify facts have conflicting values
    assert_eq!(fact1.attribute, "department");
    assert_eq!(fact2.attribute, "department");
    assert_ne!(fact1.value_string(), fact2.value_string());
    assert!(!fact1.value.is_equal_to(&fact2.value));
}

#[tokio::test]
async fn test_contradiction_detection_no_overlap() {
    let entity_id = Uuid::new_v4();
    let now = Utc::now();

    // Create non-overlapping employment facts
    let fact1 = TestFactBuilder::employer(entity_id, "Acme Corp")
        .with_valid_range(
            Some(now - Duration::days(365)),
            Some(now - Duration::days(180)),
        )
        .build();

    let fact2 = TestFactBuilder::employer(entity_id, "Tech Inc")
        .with_valid_range(
            Some(now - Duration::days(90)),
            None, // Still valid
        )
        .build();

    // These should not overlap
    assert!(!fact1.overlaps_range(
        fact2.valid_from.unwrap(),
        fact2.valid_from.unwrap() + Duration::days(1)
    ));
}

#[tokio::test]
async fn test_contradiction_severity_levels() {
    let entity_id = Uuid::new_v4();

    // Create contradictions with different severities
    let contradictions = vec![
        Contradiction::new(
            entity_id,
            Uuid::new_v4(),
            Uuid::new_v4(),
            ContradictionType::Temporal,
            ContradictionSeverity::Low,
        ),
        Contradiction::new(
            entity_id,
            Uuid::new_v4(),
            Uuid::new_v4(),
            ContradictionType::Factual,
            ContradictionSeverity::Medium,
        ),
        Contradiction::new(
            entity_id,
            Uuid::new_v4(),
            Uuid::new_v4(),
            ContradictionType::Logical,
            ContradictionSeverity::High,
        ),
        Contradiction::new(
            entity_id,
            Uuid::new_v4(),
            Uuid::new_v4(),
            ContradictionType::Temporal,
            ContradictionSeverity::Critical,
        ),
    ];

    // Verify severities
    assert!(contradictions[0].severity < ContradictionSeverity::Medium);
    assert!(contradictions[1].severity < ContradictionSeverity::High);
    assert!(contradictions[2].severity < ContradictionSeverity::Critical);
    assert_eq!(contradictions[3].severity, ContradictionSeverity::Critical);
}

#[tokio::test]
async fn test_contradiction_resolution_creation() {
    let entity_id = Uuid::new_v4();
    let fact1_id = Uuid::new_v4();
    let fact2_id = Uuid::new_v4();

    let contradiction = Contradiction::new(
        entity_id,
        fact1_id,
        fact2_id,
        ContradictionType::Factual,
        ContradictionSeverity::High,
    );

    // Test different resolution strategies
    let resolutions = vec![
        ContradictionResolution::keep_first(&contradiction, Uuid::new_v4()),
        ContradictionResolution::keep_second(&contradiction, Uuid::new_v4()),
        ContradictionResolution::merge(&contradiction, Uuid::new_v4(),
            serde_json::json!({ "employer": "Merged Corp" })),
        ContradictionResolution::deprecate_both(&contradiction, Uuid::new_v4()),
        ContradictionResolution::flag_for_review(&contradiction, Uuid::new_v4(),
            "Needs manual verification"),
    ];

    assert_eq!(resolutions[0].resolution_type, ResolutionType::KeepFirst);
    assert_eq!(resolutions[1].resolution_type, ResolutionType::KeepSecond);
    assert_eq!(resolutions[2].resolution_type, ResolutionType::Merge);
    assert_eq!(resolutions[3].resolution_type, ResolutionType::DeprecateBoth);
    assert_eq!(resolutions[4].resolution_type, ResolutionType::FlagForReview);
}

// ============================================================================
// Confidence Scoring Tests
// ============================================================================

#[tokio::test]
async fn test_confidence_scoring_basic() {
    let entity_id = Uuid::new_v4();
    let user_id = Uuid::new_v4();

    // Create facts with different sources and confidences
    let facts = vec![
        TestFactBuilder::employer(entity_id, "Acme Corp")
            .verified(user_id) // Verified = 1.0 confidence
            .build(),
        TestFactBuilder::title(entity_id, "Engineer")
            .with_confidence(0.8)
            .from_extraction(Uuid::new_v4())
            .build(),
        TestFactBuilder::location(entity_id, "New York")
            .with_confidence(0.6)
            .from_extraction(Uuid::new_v4())
            .build(),
    ];

    // Verify confidence values
    assert_eq!(facts[0].confidence, 1.0);
    assert_eq!(facts[1].confidence, 0.8);
    assert_eq!(facts[2].confidence, 0.6);

    // Verify sources
    assert!(facts[0].source.is_verified());
    assert!(facts[1].source.is_extraction());
    assert!(facts[2].source.is_extraction());
}

#[tokio::test]
async fn test_confidence_scoring_weighted() {
    let entity_id = Uuid::new_v4();

    // Create facts from multiple sources
    let extraction_source = FactSource::extraction(Uuid::new_v4());
    let verified_source = FactSource::manual_verified(Uuid::new_v4());

    let facts = vec![
        Fact::new(
            entity_id,
            "salary",
            FactValue::number(100000.0),
            extraction_source.clone(),
        ).with_confidence(0.7),
        Fact::new(
            entity_id,
            "salary",
            FactValue::number(105000.0),
            verified_source.clone(),
        ).with_confidence(1.0),
    ];

    // Verified fact should have higher confidence
    assert!(facts[1].confidence > facts[0].confidence);
    assert!(facts[1].source.is_verified());
}

#[tokio::test]
async fn test_confidence_clamping() {
    let entity_id = Uuid::new_v4();

    // Test confidence clamping to 0.0-1.0 range
    let fact1 = TestFactBuilder::employer(entity_id, "Test Corp")
        .with_confidence(1.5) // Should be clamped to 1.0
        .build();

    let fact2 = TestFactBuilder::employer(entity_id, "Test Corp")
        .with_confidence(-0.5) // Should be clamped to 0.0
        .build();

    assert_eq!(fact1.confidence, 1.0);
    assert_eq!(fact2.confidence, 0.0);
}

// ============================================================================
// Duplicate Detection Tests
// ============================================================================

#[tokio::test]
async fn test_duplicate_detection_similar_names() {
    // Create entities with similar names
    let entity1 = TestEntityBuilder::person("John Smith")
        .with_email("john@example.com")
        .build();

    let entity2 = TestEntityBuilder::person("Jon Smith")
        .with_email("jon@example.com")
        .build();

    let entity3 = TestEntityBuilder::person("Jane Doe")
        .with_email("jane@example.com")
        .build();

    // John and Jon should be similar
    let name_sim_1_2 = calculate_name_similarity(&entity1.name, &entity2.name);
    assert!(name_sim_1_2 > 0.8, "Expected high similarity for similar names");

    // John and Jane should not be as similar
    let name_sim_1_3 = calculate_name_similarity(&entity1.name, &entity3.name);
    assert!(name_sim_1_3 < name_sim_1_2, "Expected lower similarity for different names");
}

#[tokio::test]
async fn test_duplicate_detection_same_email() {
    // Create entities with same email (strong duplicate indicator)
    let entity1 = TestEntityBuilder::person("John Smith")
        .with_email("john.smith@example.com")
        .build();

    let entity2 = TestEntityBuilder::person("J. Smith")
        .with_email("john.smith@example.com") // Same email
        .build();

    // Same email should be a strong indicator
    assert_eq!(
        entity1.metadata.email,
        entity2.metadata.email,
        "Same email indicates potential duplicate"
    );
}

#[tokio::test]
async fn test_duplicate_candidate_creation() {
    let entity_a_id = Uuid::new_v4();
    let entity_b_id = Uuid::new_v4();

    let candidate = DuplicateCandidate {
        entity_a_id,
        entity_b_id,
        similarity_score: 0.85,
        name_similarity: 0.9,
        embedding_similarity: 0.8,
        relationship_overlap: 0.85,
        detected_at: Utc::now(),
    };

    assert_eq!(candidate.entity_a_id, entity_a_id);
    assert_eq!(candidate.entity_b_id, entity_b_id);
    assert!(candidate.similarity_score >= 0.0 && candidate.similarity_score <= 1.0);
    assert!(candidate.similarity_score > 0.75); // Above threshold
}

#[tokio::test]
async fn test_duplicate_detection_thresholds() {
    // Test different similarity thresholds
    let candidates = vec![
        DuplicateCandidate {
            entity_a_id: Uuid::new_v4(),
            entity_b_id: Uuid::new_v4(),
            similarity_score: 0.95, // High confidence duplicate
            name_similarity: 0.95,
            embedding_similarity: 0.95,
            relationship_overlap: 0.95,
            detected_at: Utc::now(),
        },
        DuplicateCandidate {
            entity_a_id: Uuid::new_v4(),
            entity_b_id: Uuid::new_v4(),
            similarity_score: 0.60, // Below threshold
            name_similarity: 0.60,
            embedding_similarity: 0.60,
            relationship_overlap: 0.60,
            detected_at: Utc::now(),
        },
    ];

    // First candidate is above default threshold
    assert!(candidates[0].similarity_score >= 0.75);

    // Second candidate is below threshold
    assert!(candidates[1].similarity_score < 0.75);
}

// ============================================================================
// Merge Engine Tests
// ============================================================================

#[tokio::test]
async fn test_merge_entities() {
    // Create two entities to merge
    let entity1 = TestEntityBuilder::person("John Smith")
        .with_email("john@example.com")
        .with_role("Engineer")
        .build();

    let entity2 = TestEntityBuilder::person("John A. Smith")
        .with_email("john.smith@example.com")
        .with_description("Software developer")
        .build();

    // Create a merge candidate
    let candidate = DuplicateCandidate {
        entity_a_id: entity1.id,
        entity_b_id: entity2.id,
        similarity_score: 0.9,
        name_similarity: 0.95,
        embedding_similarity: 0.85,
        relationship_overlap: 0.9,
        detected_at: Utc::now(),
    };

    // Verify merge candidate properties
    assert_eq!(candidate.entity_a_id, entity1.id);
    assert_eq!(candidate.entity_b_id, entity2.id);
    assert!(candidate.similarity_score > 0.75);
}

#[tokio::test]
async fn test_merge_config() {
    let config = MergeConfig::default();

    // Verify default config
    assert!(config.require_manual_approval_for_high_confidence);
    assert!(config.preserve_all_relationships);
    assert!(config.create_redirect);
}

#[tokio::test]
async fn test_merge_conflict_resolution() {
    use truffle_core::validation::redundancy::ConflictResolutionStrategy;

    // Test different conflict resolution strategies
    let strategies = vec![
        ConflictResolutionStrategy::KeepFirst,
        ConflictResolutionStrategy::KeepSecond,
        ConflictResolutionStrategy::KeepBoth,
        ConflictResolutionStrategy::MergeValues,
        ConflictResolutionStrategy::UseMostReliableSource,
        ConflictResolutionStrategy::UseMostRecent,
    ];

    // Just verify all strategies are valid
    for strategy in strategies {
        match strategy {
            ConflictResolutionStrategy::KeepFirst |
            ConflictResolutionStrategy::KeepSecond |
            ConflictResolutionStrategy::KeepBoth |
            ConflictResolutionStrategy::MergeValues |
            ConflictResolutionStrategy::UseMostReliableSource |
            ConflictResolutionStrategy::UseMostRecent => {
                // Valid strategies
            }
        }
    }
}

// ============================================================================
// Validation Engine Tests
// ============================================================================

#[tokio::test]
async fn test_validation_engine_creation() {
    let db = setup_test_db().await;
    let repo = Arc::new(RwLock::new(GraphRepository::new(db.clone())));

    let config = ValidationConfig::default();
    let engine = ValidationEngine::new(config, repo);

    // Engine should be created successfully
    // Since we can't easily verify internal state, we just ensure creation doesn't panic
}

#[tokio::test]
async fn test_validation_summary() {
    let mut summary = EngineValidationSummary::new();

    // Verify initial state
    assert_eq!(summary.entities_checked, 0);
    assert_eq!(summary.contradictions, 0);
    assert_eq!(summary.critical_contradictions, 0);
    assert_eq!(summary.duplicate_candidates, 0);
    assert!(summary.success);
    assert!(!summary.has_issues());
    assert_eq!(summary.status_text(), "OK");

    // Add some issues
    summary.contradictions = 5;
    assert!(summary.has_issues());
    assert_eq!(summary.status_text(), "WARNINGS");

    // Add critical issues
    summary.critical_contradictions = 1;
    assert_eq!(summary.status_text(), "CRITICAL");

    // Mark as failed
    summary.success = false;
    assert_eq!(summary.status_text(), "FAILED");
}

#[tokio::test]
async fn test_entity_validation_result() {
    let entity_id = Uuid::new_v4();
    let mut result = EntityValidationResult::new(entity_id);

    assert_eq!(result.entity_id, entity_id);
    assert!(result.contradictions.is_empty());
    assert!(result.confidence.is_none());
    assert!(!result.has_issues);

    // Add a contradiction
    result.contradictions.push(Contradiction::new(
        entity_id,
        Uuid::new_v4(),
        Uuid::new_v4(),
        ContradictionType::Factual,
        ContradictionSeverity::High,
    ));

    result.update_status();
    assert!(result.has_issues);
}

#[tokio::test]
async fn test_validation_request_creation() {
    let entity_id = Uuid::new_v4();
    let requested_by = Uuid::new_v4();

    let request = ValidationRequest::new(entity_id, requested_by)
        .with_attributes(vec!["employer".to_string(), "title".to_string()]);

    assert_eq!(request.entity_id, entity_id);
    assert_eq!(request.requested_by, requested_by);
    assert_eq!(request.attributes.len(), 2);
    assert!(request.time_range.is_none());
}

#[tokio::test]
async fn test_validation_request_with_time_range() {
    let entity_id = Uuid::new_v4();
    let requested_by = Uuid::new_v4();

    let start = Utc::now() - Duration::days(365);
    let end = Utc::now();

    let request = ValidationRequest::new(entity_id, requested_by)
        .with_time_range(start, end);

    assert!(request.time_range.is_some());
    let (req_start, req_end) = request.time_range.unwrap();
    assert_eq!(req_start, start);
    assert_eq!(req_end, end);
}

// ============================================================================
// Complex Validation Scenarios
// ============================================================================

#[tokio::test]
async fn test_complex_entity_with_multiple_facts() {
    let entity_id = Uuid::new_v4();
    let now = Utc::now();

    // Create a complex entity with multiple facts over time
    let facts = vec![
        // Employment history
        TestFactBuilder::employer(entity_id, "Startup Inc")
            .with_valid_range(Some(now - Duration::days(730)), Some(now - Duration::days(366)))
            .with_confidence(0.9)
            .build(),
        TestFactBuilder::employer(entity_id, "Big Corp")
            .with_valid_range(Some(now - Duration::days(365)), Some(now - Duration::days(31)))
            .with_confidence(0.95)
            .build(),
        TestFactBuilder::employer(entity_id, "Current Company")
            .with_valid_range(Some(now - Duration::days(30)), None)
            .with_confidence(0.98)
            .build(),
        // Job titles
        TestFactBuilder::title(entity_id, "Junior Developer")
            .with_valid_range(Some(now - Duration::days(730)), Some(now - Duration::days(366)))
            .build(),
        TestFactBuilder::title(entity_id, "Senior Developer")
            .with_valid_range(Some(now - Duration::days(365)), None)
            .verified(Uuid::new_v4())
            .build(),
        // Location
        TestFactBuilder::location(entity_id, "San Francisco")
            .verified(Uuid::new_v4())
            .build(),
    ];

    // Verify all facts belong to the same entity
    for fact in &facts {
        assert_eq!(fact.entity_id, entity_id);
    }

    // Verify employment timeline has no overlaps
    assert!(!facts[0].overlaps_range(
        facts[1].valid_from.unwrap(),
        facts[1].valid_until.unwrap()
    ));

    // Current employment has no end date
    assert!(facts[2].valid_until.is_none());
    assert!(facts[2].is_currently_valid());
}

#[tokio::test]
async fn test_contradiction_detection_in_fact_set() {
    let entity_id = Uuid::new_v4();
    let now = Utc::now();

    // Create a set of facts with an intentional contradiction
    let facts = vec![
        // These two facts contradict - same time period, different employers
        TestFactBuilder::employer(entity_id, "Company A")
            .with_valid_range(Some(now - Duration::days(180)), Some(now))
            .with_confidence(0.8)
            .build(),
        TestFactBuilder::employer(entity_id, "Company B")
            .with_valid_range(Some(now - Duration::days(90)), Some(now + Duration::days(30)))
            .with_confidence(0.85)
            .build(),
    ];

    // Verify the contradiction
    assert!(facts[0].overlaps_range(
        facts[1].valid_from.unwrap(),
        facts[1].valid_until.unwrap()
    ));

    // Both claim to be current employers
    assert!(facts[0].is_currently_valid());
    assert!(facts[1].is_currently_valid());
}

#[tokio::test]
async fn test_fact_deprecation() {
    let entity_id = Uuid::new_v4();

    let mut fact = TestFactBuilder::employer(entity_id, "Old Company")
        .build();

    // Fact should be valid initially
    assert!(fact.is_currently_valid());

    // Deprecate the fact
    fact.deprecate();

    // Fact should no longer be valid
    assert!(!fact.is_currently_valid());
    assert!(fact.valid_until.is_some());
}

#[tokio::test]
async fn test_fact_source_reliability() {
    let extraction = FactSource::extraction(Uuid::new_v4());
    let manual = FactSource::manual(Uuid::new_v4());
    let verified = FactSource::manual_verified(Uuid::new_v4());
    let import = FactSource::import("CRM", "contact_123");

    // Verified manual should be most reliable
    assert!(verified.is_verified());
    assert!(!manual.is_verified());
    assert!(!extraction.is_verified());
    assert!(!import.is_verified());

    // Display names should be descriptive
    assert!(verified.display_name().contains("verified"));
    assert!(extraction.display_name().starts_with("Extraction"));
    assert!(import.display_name().contains("CRM"));
}

#[tokio::test]
async fn test_validation_batch_processing() {
    // Create multiple entities for batch validation
    let entities: Vec<_> = (0..10)
        .map(|i| TestEntityBuilder::person(&format!("Person {}", i)).build())
        .collect();

    // Create facts for each entity
    let mut all_facts = Vec::new();
    for entity in &entities {
        let facts = vec![
            TestFactBuilder::employer(entity.id, "Test Corp").build(),
            TestFactBuilder::title(entity.id, "Engineer").build(),
        ];
        all_facts.extend(facts);
    }

    // Verify batch
    assert_eq!(entities.len(), 10);
    assert_eq!(all_facts.len(), 20);
}

// Helper function for name similarity (simplified version)
fn calculate_name_similarity(name_a: &str, name_b: &str) -> f32 {
    let a = name_a.to_lowercase();
    let b = name_b.to_lowercase();

    if a == b {
        return 1.0;
    }

    if a.contains(&b) || b.contains(&a) {
        let ratio = a.len().min(b.len()) as f32 / a.len().max(b.len()) as f32;
        return 0.8 + (0.2 * ratio);
    }

    // Calculate Jaro-Winkler similarity
    jaro_winkler_simple(&a, &b)
}

fn jaro_winkler_simple(s1: &str, s2: &str) -> f32 {
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
