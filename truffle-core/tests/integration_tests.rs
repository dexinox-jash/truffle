//! Integration Tests for Truffle Core
//!
//! These tests verify the end-to-end functionality of the library,
//! including database operations, pipeline processing, and sync.
//!
//! This file serves as the main entry point for all integration tests.
//! Module-specific tests are organized in the `integration/` subdirectory.

// Include the integration test modules
mod integration;
mod fixtures;

// Re-export test utilities for use in integration tests
pub use integration::*;
pub use fixtures::*;

// ============================================================================
// Legacy Integration Tests (preserved from original)
// ============================================================================

use truffle_core::*;
use std::sync::Arc;
use tempfile::tempdir;
use chrono::Utc;

/// Create a test database
fn create_test_db() -> Arc<Database> {
    let db = Database::open_in_memory().unwrap();
    Arc::new(db)
}

/// Create a test raw artifact
fn create_test_artifact() -> RawArtifact {
    let content = b"test image content";
    let uuid = content_addressable_uuid(content);
    
    RawArtifact {
        uuid,
        filename: "test_screenshot.png".to_string(),
        binary: BlobPointer {
            path: std::path::PathBuf::from("/tmp/test.png"),
            content_hash: "abc123".to_string(),
        },
        captured_at: Utc::now(),
        device_id: DeviceFingerprint::new("test_device", b"test_salt"),
        app_context: Some(AppContext {
            bundle_id: "com.apple.Safari".to_string(),
            app_name: "Safari".to_string(),
            window_title: Some("Test Page".to_string()),
        }),
        geohash: Some(Geohash::new(40.7128, -74.0060)),
        metadata: RawArtifactMetadata::new(1024, 1920, 1080),
        status: IngestionStatus::Pending,
        retry_count: 0,
        created_at: Utc::now(),
        updated_at: Utc::now(),
    }
}

/// Create a test wiki node
fn create_test_node() -> WikiNode {
    let provenance = Provenance::new(
        vec![uuid::Uuid::new_v4()],
        "gemma-4-2b-it-Q4_K_M",
        0.92,
    );
    
    WikiNode::new(
        "Test Entity",
        WikiNodeType::Entity,
        "This is test content with [[Another Page]] link.",
        provenance,
    )
}

#[test]
fn test_content_addressable_uuid() {
    let content1 = b"test content";
    let content2 = b"test content";
    let content3 = b"different content";
    
    let uuid1 = content_addressable_uuid(content1);
    let uuid2 = content_addressable_uuid(content2);
    let uuid3 = content_addressable_uuid(content3);
    
    // Same content = same UUID
    assert_eq!(uuid1, uuid2);
    
    // Different content = different UUID
    assert_ne!(uuid1, uuid3);
}

#[test]
fn test_database_connection() {
    let db = create_test_db();
    
    // Check WAL mode
    let stats = db.stats().unwrap();
    assert_eq!(stats.journal_mode.to_lowercase(), "wal");
}

#[test]
fn test_raw_artifact_repository() {
    let db = create_test_db();
    let artifact = create_test_artifact();
    
    // Save artifact
    db.with_connection(|conn| {
        let repo = RawArtifactRepository::new(conn);
        repo.save(&artifact)?;
        Ok(())
    }).unwrap();
    
    // Find by ID
    let found = db.with_connection(|conn| {
        let repo = RawArtifactRepository::new(conn);
        repo.find_by_id(artifact.uuid)
            .map_err(|e| database::DatabaseError::Query(e.to_string()))
    }).unwrap();
    
    assert!(found.is_some());
    let found = found.unwrap();
    assert_eq!(found.filename, artifact.filename);
    assert_eq!(found.metadata.width, 1920);
    assert_eq!(found.metadata.height, 1080);
}

#[test]
fn test_wiki_node_repository() {
    let db = create_test_db();
    let node = create_test_node();
    
    // Save node
    db.with_connection(|conn| {
        let repo = WikiNodeRepository::new(conn);
        repo.save(&node)?;
        Ok(())
    }).unwrap();
    
    // Find by ID
    let found = db.with_connection(|conn| {
        let repo = WikiNodeRepository::new(conn);
        repo.find_by_id(node.id)
            .map_err(|e| database::DatabaseError::Query(e.to_string()))
    }).unwrap();
    
    assert!(found.is_some());
    let found = found.unwrap();
    assert_eq!(found.title, node.title);
    assert_eq!(found.slug(), node.slug());
}

#[test]
fn test_wiki_node_links() {
    let node1 = create_test_node();
    let mut node2 = create_test_node();
    
    // Add backlink
    let link = WikiLink::new(node1.id, &node1.title);
    node2.add_backlink(link);
    
    assert_eq!(node2.backlinks.len(), 1);
    assert_eq!(node2.backlinks[0].target_id, node1.id);
    
    // Duplicate should not be added
    let link2 = WikiLink::new(node1.id, &node1.title);
    node2.add_backlink(link2);
    assert_eq!(node2.backlinks.len(), 1);
}

#[test]
fn test_schema_ruleset() {
    // Load default ruleset
    let ruleset = SchemaRuleset::default_ruleset();
    
    // Verify rules exist
    assert!(!ruleset.compilation_rules.is_empty());
    
    // Verify prohibited extractions
    assert!(!ruleset.prohibited_extractions.is_empty());
    
    // Validate ruleset
    let errors = ruleset.validate();
    assert!(errors.is_empty(), "Validation errors: {:?}", errors);
}

#[test]
fn test_trigger_evaluation() {
    use models::{Trigger, TriggerContext};
    
    let trigger = Trigger::ConceptDetected {
        concept: "receipt".to_string(),
    };
    
    let ctx = TriggerContext::new()
        .with_concepts(vec!["receipt".to_string(), "invoice".to_string()]);
    
    assert!(trigger.evaluate(&ctx));
    
    let ctx_no_match = TriggerContext::new()
        .with_concepts(vec!["photo".to_string()]);
    
    assert!(!trigger.evaluate(&ctx_no_match));
}

#[test]
fn test_vector_clock() {
    let mut clock1 = VectorClock::new();
    let mut clock2 = VectorClock::new();
    
    // Increment clocks
    clock1.increment("device_a");
    clock1.increment("device_a");
    clock2.increment("device_b");
    
    // Concurrent - neither happens before the other
    assert_eq!(clock1.compare(&clock2), None);
    
    // Merge
    clock1.merge(&clock2);
    assert_eq!(clock1.clocks.get("device_a"), Some(&2));
    assert_eq!(clock1.clocks.get("device_b"), Some(&1));
    
    // Now clock1 > clock2
    assert_eq!(clock1.compare(&clock2), Some(true));
}

#[test]
fn test_crdt_document() {
    use sync::CrdtDocument;
    
    let node = create_test_node();
    let mut doc = CrdtDocument::from_node(&node).unwrap();
    
    // Encode state
    let update = doc.encode_state_as_update();
    assert!(!update.is_empty());
    
    // Create new document and apply update
    let mut doc2 = CrdtDocument::new(node.id.to_string());
    doc2.apply_update(&update).unwrap();
    
    // State vectors should match
    assert_eq!(doc.state_vector(), doc2.state_vector());
}

#[test]
fn test_crdt_manager() {
    use sync::CrdtManager;
    
    let manager = CrdtManager::new("device1");
    
    // Create document
    let doc = manager.get_or_create("doc1");
    assert_eq!(doc.doc_id(), "doc1");
    
    // Retrieve existing
    let doc2 = manager.get_or_create("doc1");
    assert_eq!(doc.doc_id(), doc2.doc_id());
    
    assert_eq!(manager.document_count(), 1);
}

#[test]
fn test_sync_delta() {
    use sync::{SyncDelta, CompressionType};
    
    let clock = VectorClock::new();
    let mut delta = SyncDelta::new(
        "doc1",
        "device1",
        vec![1, 2, 3, 4, 5],
        clock,
    );
    
    assert_eq!(delta.doc_id, "doc1");
    assert_eq!(delta.original_size, 5);
    
    // Compression is a no-op in test (no actual compression)
    delta.compress(CompressionType::Zstd).unwrap();
    assert_eq!(delta.compression, CompressionType::Zstd);
}

#[test]
fn test_sync_crypto() {
    use sync::{SyncCrypto, PublicKeyBundle};
    
    let crypto1 = SyncCrypto::generate();
    let crypto2 = SyncCrypto::generate();
    
    let public1 = crypto1.export_public();
    let public2 = crypto2.export_public();
    
    // Derive shared secrets
    let mut crypto1 = crypto1;
    let mut crypto2 = crypto2;
    
    crypto1.derive_shared_secret("device2", &public2).unwrap();
    crypto2.derive_shared_secret("device1", &public1).unwrap();
    
    // Encrypt/decrypt
    let nonce = SyncCrypto::generate_nonce();
    let plaintext = b"secret message";
    
    let encrypted = crypto1.encrypt("device2", plaintext, &nonce).unwrap();
    let decrypted = crypto2.decrypt("device1", &encrypted, &nonce).unwrap();
    
    assert_eq!(plaintext.to_vec(), decrypted);
}

#[test]
fn test_pairing_ceremony() {
    use sync::{PairingCeremony, PairingState};
    
    let crypto1 = SyncCrypto::generate();
    let crypto2 = SyncCrypto::generate();
    
    let mut ceremony1 = PairingCeremony::new(crypto1);
    let mut ceremony2 = PairingCeremony::new(crypto2);
    
    // Start pairing
    let public1 = ceremony1.start_primary().unwrap();
    assert_eq!(ceremony1.state(), PairingState::WaitingForScan);
    
    ceremony2.start_secondary(&public1).unwrap();
    assert_eq!(ceremony2.state(), PairingState::SasVerification);
    
    // Generate and verify SAS
    let public2 = ceremony2.crypto.export_public();
    let sas1 = ceremony1.generate_sas(&public2);
    let sas2 = ceremony2.generate_sas(&public1);
    
    assert_eq!(sas1.code, sas2.code);
    
    assert!(ceremony1.verify_sas(&sas1, &sas2));
    assert!(ceremony1.is_paired());
}

#[test]
fn test_privacy_classification() {
    assert!(PrivacyClassification::Financial.requires_encryption());
    assert!(PrivacyClassification::Sensitive.requires_encryption());
    assert!(!PrivacyClassification::Personal.requires_encryption());
    assert!(!PrivacyClassification::Public.requires_encryption());
}

#[test]
fn test_provenance_confidence() {
    let prov = Provenance::new(vec![], "test", 0.96);
    assert!(prov.meets_confidence_threshold(0.95));
    assert!(!prov.meets_confidence_threshold(0.97));
}

#[test]
fn test_wiki_node_slug() {
    let provenance = Provenance::new(vec![], "test", 0.9);
    let node = WikiNode::new(
        "Hello World Test!!!",
        WikiNodeType::Entity,
        "Content",
        provenance,
    );
    
    assert_eq!(node.slug(), "hello-world-test");
}

#[test]
fn test_ingestion_status_transitions() {
    let mut status = IngestionStatus::Pending;
    
    assert_eq!(status, IngestionStatus::Pending);
    
    // Simulate transition
    status = IngestionStatus::Processing;
    assert_eq!(status, IngestionStatus::Processing);
    
    status = IngestionStatus::Compiled;
    assert_eq!(status, IngestionStatus::Compiled);
}

#[test]
fn test_database_transaction() {
    let db = create_test_db();
    
    // Execute in transaction
    let result = db.with_transaction(|conn| {
        conn.execute(
            "CREATE TABLE test_table (id INTEGER PRIMARY KEY, name TEXT)",
            [],
        )?;
        conn.execute(
            "INSERT INTO test_table (name) VALUES (?1)",
            ["test_value"],
        )?;
        Ok(42)
    });
    
    assert_eq!(result.unwrap(), 42);
    
    // Verify data persisted
    let count: i64 = db.with_connection(|conn| {
        conn.query_row(
            "SELECT COUNT(*) FROM test_table",
            [],
            |row| row.get(0),
        ).map_err(database::DatabaseError::Sqlite)
    }).unwrap();
    
    assert_eq!(count, 1);
}

#[tokio::test]
async fn test_pipeline_creation() {
    let db = create_test_db();
    let config = PipelineConfig::default();
    
    let pipeline = CompilationPipeline::new(config, db);
    assert!(pipeline.is_ok());
}

#[test]
fn test_device_fingerprint() {
    let fp1 = DeviceFingerprint::new("device_info_123", b"salt_value");
    let fp2 = DeviceFingerprint::new("device_info_123", b"salt_value");
    let fp3 = DeviceFingerprint::new("different_info", b"salt_value");
    
    // Same input = same fingerprint
    assert_eq!(fp1, fp2);
    
    // Different input = different fingerprint
    assert_ne!(fp1, fp3);
}

#[test]
fn test_geohash() {
    let geohash1 = Geohash::new(40.7128, -74.0060); // NYC
    let geohash2 = Geohash::new(40.7129, -74.0061); // Very close to NYC
    
    // Should produce same 4-char geohash (20km precision)
    assert_eq!(geohash1.as_str().len(), 4);
    assert_eq!(geohash1.as_str(), geohash2.as_str());
    
    // Different location
    let geohash3 = Geohash::new(51.5074, -0.1278); // London
    assert_ne!(geohash1.as_str(), geohash3.as_str());
}

#[test]
fn test_blob_pointer() {
    use std::io::Write;
    
    let temp_dir = tempdir().unwrap();
    let file_path = temp_dir.path().join("test.txt");
    
    let content = b"test content for hashing";
    
    // Write file
    {
        let mut file = std::fs::File::create(&file_path).unwrap();
        file.write_all(content).unwrap();
    }
    
    // Create blob pointer
    let blob = BlobPointer::new(file_path.clone(), content);
    
    // Verify integrity
    assert!(blob.verify().unwrap());
    
    // Corrupt file
    {
        let mut file = std::fs::File::create(&file_path).unwrap();
        file.write_all(b"corrupted").unwrap();
    }
    
    // Verification should fail
    assert!(!blob.verify().unwrap());
}

#[test]
fn test_temporal_vectors() {
    use chrono::TimeZone;
    
    let date = Utc.with_ymd_and_hms(2024, 1, 15, 0, 0, 0).unwrap();
    
    let quarter = TemporalVectors::calculate_fiscal_quarter(date, 4);
    assert_eq!(quarter, "Q4-2023");
    
    let date = Utc.with_ymd_and_hms(2024, 5, 15, 0, 0, 0).unwrap();
    let quarter = TemporalVectors::calculate_fiscal_quarter(date, 4);
    assert_eq!(quarter, "Q1-2024");
}

#[test]
fn test_wikilink_generation() {
    let link = WikiLink::new(uuid::Uuid::new_v4(), "Target Page");
    
    assert_eq!(link.to_wikilink(), "[[Target Page]]");
    assert_eq!(link.to_wikilink_with_alias("Display"), "[[Target Page|Display]]");
}

#[test]
fn test_wiki_node_word_count() {
    let provenance = Provenance::new(vec![], "test", 0.9);
    let node = WikiNode::new(
        "Test",
        WikiNodeType::Entity,
        "This is a test with five words.",
        provenance,
    );
    
    assert_eq!(node.word_count(), 7);
}

#[test]
fn test_wiki_node_summary() {
    let provenance = Provenance::new(vec![], "test", 0.9);
    let node = WikiNode::new(
        "Test",
        WikiNodeType::Entity,
        "This is a long content that should be truncated in the summary.",
        provenance,
    );
    
    let summary = node.summary(20);
    assert!(summary.len() <= 23); // 20 + "..."
    assert!(summary.ends_with("..."));
}

#[test]
fn test_schema_toml_serialization() {
    let ruleset = SchemaRuleset::default_ruleset();
    
    let toml = ruleset.to_toml().unwrap();
    assert!(!toml.is_empty());
    
    let parsed = SchemaRuleset::from_toml(&toml).unwrap();
    assert_eq!(parsed.schema_version, ruleset.schema_version);
}

#[test]
fn test_priority_calculator() {
    let mut artifact = create_test_artifact();
    
    // Default priority
    assert_eq!(PriorityCalculator::calculate(&artifact), 2);
    
    // Messaging app priority
    artifact.app_context = Some(AppContext {
        bundle_id: "com.apple.MobileSMS".to_string(),
        app_name: "Messages".to_string(),
        window_title: None,
    });
    assert_eq!(PriorityCalculator::calculate(&artifact), 1);
}

#[test]
fn test_document_type_inference() {
    use pipeline::graph::GraphBuilder;
    use pipeline::processor::DocumentType;
    
    let builder = GraphBuilder::new(GraphConfig::default());
    
    assert_eq!(builder.infer_node_type(DocumentType::Receipt), WikiNodeType::Entity);
    assert_eq!(builder.infer_node_type(DocumentType::TravelConfirmation), WikiNodeType::Chronology);
    assert_eq!(builder.infer_node_type(DocumentType::Messaging), WikiNodeType::Concept);
}

#[test]
fn test_levenshtein_distance() {
    use pipeline::graph::levenshtein_distance;
    
    assert_eq!(levenshtein_distance("kitten", "sitting"), 3);
    assert_eq!(levenshtein_distance("saturday", "sunday"), 3);
    assert_eq!(levenshtein_distance("", "abc"), 3);
    assert_eq!(levenshtein_distance("abc", "abc"), 0);
}

#[test]
fn test_conflict_resolution() {
    use sync::{ConflictResolver, ConflictStrategy, ConflictResolution};
    
    let resolver = ConflictResolver::new(ConflictStrategy::LastWriteWins);
    
    let mut local = VectorClock::new();
    local.increment("device1");
    local.increment("device1");
    
    let mut remote = VectorClock::new();
    remote.increment("device1");
    
    // Local is newer
    let result = resolver.resolve(&local, &remote);
    assert_eq!(result, ConflictResolution::UseLocal);
    
    // Remote is older
    let result = resolver.resolve(&remote, &local);
    assert_eq!(result, ConflictResolution::UseRemote);
}

#[test]
fn test_library_info() {
    let info = crate::info();
    
    assert!(!info.version.is_empty());
    assert!(!info.name.is_empty());
    assert!(!info.sqlite_version.is_empty());
    assert!(!info.features.is_empty());
}
