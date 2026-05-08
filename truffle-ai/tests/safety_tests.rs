//! Safety Classifier Tests

use std::collections::HashMap;
use tempfile::TempDir;

use truffle_ai::pipeline::safety::{
    SafetyClassifier, SafetyResult, ContentCategory, SafetyAction,
    SafetyPolicy, QuarantineManager,
};

#[test]
fn test_safety_result_safe() {
    let result = SafetyResult::safe();
    
    assert!(result.is_safe);
    assert!(result.categories.is_empty());
    assert_eq!(result.recommended_action, SafetyAction::Allow);
    assert!(!result.is_unsafe());
}

#[test]
fn test_safety_result_unsafe() {
    let mut result = SafetyResult::safe();
    result.is_safe = false;
    result.categories.insert(ContentCategory::Nsfw, 0.85);
    
    assert!(!result.is_safe);
    assert!(result.is_unsafe());
}

#[test]
fn test_safety_result_flagged_categories() {
    let mut result = SafetyResult::safe();
    result.categories.insert(ContentCategory::Banking, 0.9);
    result.categories.insert(ContentCategory::Medical, 0.7);
    
    let flagged = result.flagged_categories(0.75);
    assert_eq!(flagged.len(), 1);
    assert_eq!(flagged[0].0, ContentCategory::Banking);
}

#[test]
fn test_safety_result_highest_confidence() {
    let mut result = SafetyResult::safe();
    result.categories.insert(ContentCategory::Banking, 0.9);
    result.categories.insert(ContentCategory::Medical, 0.7);
    
    let highest = result.highest_confidence();
    assert!(highest.is_some());
    assert_eq!(highest.unwrap().0, ContentCategory::Banking);
}

#[test]
fn test_content_category_description() {
    assert!(!ContentCategory::Nsfw.description().is_empty());
    assert!(!ContentCategory::Banking.description().is_empty());
    assert!(!ContentCategory::Medical.description().is_empty());
}

#[test]
fn test_safety_classifier_creation() {
    let classifier = SafetyClassifier::new();
    assert!(classifier.is_ok());
    
    let classifier = classifier.unwrap();
    let stats = classifier.get_stats();
    assert!(stats.initialized);
    assert!(stats.categories_configured > 0);
}

#[test]
fn test_safety_classifier_thresholds() {
    let mut classifier = SafetyClassifier::new().unwrap();
    
    // Default threshold
    let default_threshold = classifier.get_threshold(ContentCategory::Nsfw);
    assert!(default_threshold > 0.0 && default_threshold <= 1.0);
    
    // Set custom threshold
    classifier.set_threshold(ContentCategory::Nsfw, 0.5);
    assert_eq!(classifier.get_threshold(ContentCategory::Nsfw), 0.5);
    
    // Threshold should be clamped
    classifier.set_threshold(ContentCategory::Nsfw, 1.5);
    assert_eq!(classifier.get_threshold(ContentCategory::Nsfw), 1.0);
}

#[tokio::test]
async fn test_classify_receipt() {
    let classifier = SafetyClassifier::new().unwrap();
    
    let temp_dir = TempDir::new().unwrap();
    let receipt_path = temp_dir.path().join("receipt_grocery.png");
    std::fs::write(&receipt_path, b"fake receipt image data").unwrap();
    
    let result = classifier.classify(&receipt_path).await.unwrap();
    
    // Receipt should trigger banking classification
    assert!(result.categories.contains_key(&ContentCategory::Banking));
}

#[tokio::test]
async fn test_classify_medical() {
    let classifier = SafetyClassifier::new().unwrap();
    
    let temp_dir = TempDir::new().unwrap();
    let medical_path = temp_dir.path().join("medical_prescription.png");
    std::fs::write(&medical_path, b"fake medical image data").unwrap();
    
    let result = classifier.classify(&medical_path).await.unwrap();
    
    // Medical file should trigger medical classification
    assert!(result.categories.contains_key(&ContentCategory::Medical));
}

#[test]
fn test_safety_policy_default() {
    let policy = SafetyPolicy::default();
    
    assert_eq!(policy.name, "default");
    assert!(!policy.thresholds.is_empty());
    assert!(!policy.action_mappings.is_empty());
}

#[test]
fn test_quarantine_manager_creation() {
    let temp_dir = TempDir::new().unwrap();
    let quarantine_dir = temp_dir.path().join("quarantine");
    
    let manager = QuarantineManager::new(quarantine_dir.clone());
    assert!(manager.is_ok());
    assert!(quarantine_dir.exists());
}

#[test]
fn test_quarantine_manager_quarantine() {
    let temp_dir = TempDir::new().unwrap();
    let quarantine_dir = temp_dir.path().join("quarantine");
    let manager = QuarantineManager::new(quarantine_dir).unwrap();
    
    // Create a fake image
    let image_path = temp_dir.path().join("test.png");
    std::fs::write(&image_path, b"fake image data").unwrap();
    
    // Quarantine it
    let safety_result = SafetyResult::safe();
    let quarantined = manager.quarantine(&image_path, &safety_result).unwrap();
    
    assert!(quarantined.exists());
    
    // Check metadata file was created
    let metadata_path = quarantined.with_extension("json");
    assert!(metadata_path.exists());
}

#[test]
fn test_quarantine_manager_list() {
    let temp_dir = TempDir::new().unwrap();
    let quarantine_dir = temp_dir.path().join("quarantine");
    let manager = QuarantineManager::new(quarantine_dir).unwrap();
    
    // Create and quarantine an image
    let image_path = temp_dir.path().join("test.png");
    std::fs::write(&image_path, b"fake").unwrap();
    
    let safety_result = SafetyResult::safe();
    manager.quarantine(&image_path, &safety_result).unwrap();
    
    // List quarantined items
    let list = manager.list_quarantined().unwrap();
    assert_eq!(list.len(), 1);
}
