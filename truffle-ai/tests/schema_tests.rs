//! Schema Tests

use truffle_ai::schema::{
    Schema, CompilationRule, TriggerCondition, CompilationAction, LinkTarget,
    PrivacyLevel, ProhibitedExtraction, ProhibitedAction, LinkingRules,
    EntityResolution, FieldType, DataType, NormalizationRule, SCHEMA_VERSION,
};

#[test]
fn test_schema_version() {
    assert_eq!(SCHEMA_VERSION, "2.0");
}

#[test]
fn test_schema_default() {
    let schema = Schema::default();
    
    assert_eq!(schema.version, SCHEMA_VERSION);
    assert!(!schema.compilation_rules.is_empty());
    assert!(!schema.prohibited_extractions.is_empty());
}

#[test]
fn test_compilation_rule_creation() {
    let rule = CompilationRule {
        id: "test_rule".to_string(),
        trigger: TriggerCondition::ConceptDetected { concept: "receipt".to_string() },
        action: CompilationAction::CreateEntity { entity_type: "merchant".to_string() },
        extract: vec!["total".to_string(), "date".to_string()],
        link_to: vec![],
        privacy: PrivacyLevel::Financial,
        temporal_awareness: true,
    };
    
    assert_eq!(rule.id, "test_rule");
    assert_eq!(rule.privacy, PrivacyLevel::Financial);
    assert!(rule.temporal_awareness);
}

#[test]
fn test_trigger_condition_variants() {
    let concept = TriggerCondition::ConceptDetected { concept: "test".to_string() };
    let entity = TriggerCondition::EntityType { entity_type: "person".to_string() };
    let pattern = TriggerCondition::Pattern { pattern: "flight_.*".to_string() };
    let context = TriggerCondition::Context { context: "email".to_string() };
    
    // Test that all variants can be created
    assert!(matches!(concept, TriggerCondition::ConceptDetected { .. }));
    assert!(matches!(entity, TriggerCondition::EntityType { .. }));
    assert!(matches!(pattern, TriggerCondition::Pattern { .. }));
    assert!(matches!(context, TriggerCondition::Context { .. }));
}

#[test]
fn test_trigger_condition_and() {
    let and_condition = TriggerCondition::And(vec![
        TriggerCondition::ConceptDetected { concept: "person".to_string() },
        TriggerCondition::Context { context: "contact".to_string() },
    ]);
    
    assert!(matches!(and_condition, TriggerCondition::And(_)));
}

#[test]
fn test_trigger_condition_or() {
    let or_condition = TriggerCondition::Or(vec![
        TriggerCondition::ConceptDetected { concept: "receipt".to_string() },
        TriggerCondition::ConceptDetected { concept: "invoice".to_string() },
    ]);
    
    assert!(matches!(or_condition, TriggerCondition::Or(_)));
}

#[test]
fn test_compilation_action_variants() {
    let create = CompilationAction::CreateEntity { entity_type: "test".to_string() };
    let merge = CompilationAction::MergeOrCreate { path_template: "path".to_string() };
    let travel = CompilationAction::CreateTravelItinerary;
    let index = CompilationAction::CreateIndex { index_type: "list".to_string() };
    
    assert!(matches!(create, CompilationAction::CreateEntity { .. }));
    assert!(matches!(merge, CompilationAction::MergeOrCreate { .. }));
    assert!(matches!(travel, CompilationAction::CreateTravelItinerary));
    assert!(matches!(index, CompilationAction::CreateIndex { .. }));
}

#[test]
fn test_privacy_level_ordering() {
    assert_ne!(PrivacyLevel::Public, PrivacyLevel::Financial);
    assert_eq!(PrivacyLevel::Personal, PrivacyLevel::Personal);
}

#[test]
fn test_prohibited_extraction() {
    let prohibited = ProhibitedExtraction {
        field: "password".to_string(),
        action: ProhibitedAction::RedactAndLog,
    };
    
    assert_eq!(prohibited.field, "password");
    assert!(matches!(prohibited.action, ProhibitedAction::RedactAndLog));
}

#[test]
fn test_prohibited_action_variants() {
    let redact = ProhibitedAction::RedactAndLog;
    let tokenize = ProhibitedAction::TokenizeLast4;
    let refuse = ProhibitedAction::RefuseAndQuarantine;
    
    assert!(matches!(redact, ProhibitedAction::RedactAndLog));
    assert!(matches!(tokenize, ProhibitedAction::TokenizeLast4));
    assert!(matches!(refuse, ProhibitedAction::RefuseAndQuarantine));
}

#[test]
fn test_linking_rules() {
    let rules = LinkingRules {
        temporal_proximity_secs: 300,
        semantic_similarity_threshold: 0.82,
        entity_resolution: EntityResolution::FuzzyMatch { max_distance: 2 },
    };
    
    assert_eq!(rules.temporal_proximity_secs, 300);
    assert_eq!(rules.semantic_similarity_threshold, 0.82);
}

#[test]
fn test_entity_resolution_variants() {
    let exact = EntityResolution::Exact;
    let fuzzy = EntityResolution::FuzzyMatch { max_distance: 2 };
    let phonetic = EntityResolution::Phonetic;
    
    assert!(matches!(exact, EntityResolution::Exact));
    assert!(matches!(fuzzy, EntityResolution::FuzzyMatch { .. }));
    assert!(matches!(phonetic, EntityResolution::Phonetic));
}

#[test]
fn test_field_type_creation() {
    let field = FieldType {
        name: "email".to_string(),
        data_type: DataType::Email,
        validation: Some(r"^[\w.-]+@[\w.-]+\.\w+$".to_string()),
        normalization: vec![NormalizationRule::Lowercase, NormalizationRule::Trim],
    };
    
    assert_eq!(field.name, "email");
    assert!(matches!(field.data_type, DataType::Email));
    assert_eq!(field.normalization.len(), 2);
}

#[test]
fn test_data_type_variants() {
    let string = DataType::String;
    let integer = DataType::Integer;
    let float = DataType::Float;
    let date = DataType::Date;
    let currency = DataType::Currency;
    let phone = DataType::Phone;
    let email = DataType::Email;
    
    assert!(matches!(string, DataType::String));
    assert!(matches!(integer, DataType::Integer));
    assert!(matches!(float, DataType::Float));
    assert!(matches!(date, DataType::Date));
    assert!(matches!(currency, DataType::Currency));
    assert!(matches!(phone, DataType::Phone));
    assert!(matches!(email, DataType::Email));
}

#[test]
fn test_data_type_list() {
    let list = DataType::List(Box::new(DataType::String));
    assert!(matches!(list, DataType::List(_)));
}

#[test]
fn test_normalization_rule_variants() {
    let lower = NormalizationRule::Lowercase;
    let upper = NormalizationRule::Uppercase;
    let trim = NormalizationRule::Trim;
    let remove_special = NormalizationRule::RemoveSpecialChars;
    let format_phone = NormalizationRule::FormatPhone;
    let format_currency = NormalizationRule::FormatCurrency { currency: "USD".to_string() };
    
    assert!(matches!(lower, NormalizationRule::Lowercase));
    assert!(matches!(upper, NormalizationRule::Uppercase));
    assert!(matches!(trim, NormalizationRule::Trim));
    assert!(matches!(remove_special, NormalizationRule::RemoveSpecialChars));
    assert!(matches!(format_phone, NormalizationRule::FormatPhone));
    assert!(matches!(format_currency, NormalizationRule::FormatCurrency { .. }));
}

#[test]
fn test_link_target_creation() {
    let target = LinkTarget {
        node_type: "concept".to_string(),
        path_template: "concepts/expenses".to_string(),
        context: Some("financial".to_string()),
    };
    
    assert_eq!(target.node_type, "concept");
    assert_eq!(target.path_template, "concepts/expenses");
    assert_eq!(target.context, Some("financial".to_string()));
}
