//! Schema-Based Compilation
//! 
//! Implements the AI Constitution from schema.md (SPEC v2.0 Section 2.1.3)
//! 
//! # Schema Rules
//! 
//! - Trigger-based compilation rules
//! - Field extraction specifications
//! - Privacy classifications
//! - Linking rules

use std::collections::HashMap;
use serde::{Deserialize, Serialize};

pub mod loader;
pub mod matcher;
pub mod extractor;

pub use loader::{SchemaLoader, SchemaConfig};
pub use matcher::{SchemaMatcher, TriggerMatch};
pub use extractor::{FieldExtractor, ExtractedField};

/// Schema version
pub const SCHEMA_VERSION: &str = "2.0";

/// Compilation rule from schema
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompilationRule {
    /// Rule ID
    pub id: String,
    /// Trigger condition
    pub trigger: TriggerCondition,
    /// Action to take
    pub action: CompilationAction,
    /// Fields to extract
    pub extract: Vec<String>,
    /// Nodes to link to
    pub link_to: Vec<LinkTarget>,
    /// Privacy level
    pub privacy: PrivacyLevel,
    /// Temporal awareness
    pub temporal_awareness: bool,
}

/// Trigger condition types
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TriggerCondition {
    /// Concept detected (e.g., "receipt")
    ConceptDetected { concept: String },
    /// Entity type detected
    EntityType { entity_type: String },
    /// Pattern matched
    Pattern { pattern: String },
    /// Context condition
    Context { context: String },
    /// Combined condition (AND)
    And(Vec<TriggerCondition>),
    /// Combined condition (OR)
    Or(Vec<TriggerCondition>),
}

/// Compilation actions
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CompilationAction {
    /// Create a new entity
    CreateEntity { entity_type: String },
    /// Merge with existing or create new
    MergeOrCreate { path_template: String },
    /// Create travel itinerary
    CreateTravelItinerary,
    /// Create index node
    CreateIndex { index_type: String },
}

/// Link target specification
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LinkTarget {
    /// Target node type
    pub node_type: String,
    /// Target path template
    pub path_template: String,
    /// Link context
    pub context: Option<String>,
}

/// Privacy levels
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PrivacyLevel {
    /// Public information
    Public,
    /// Personal information
    Personal,
    /// Sensitive information
    Sensitive,
    /// Financial information
    Financial,
}

/// Prohibited extraction rule
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProhibitedExtraction {
    /// Field name
    pub field: String,
    /// Action to take
    pub action: ProhibitedAction,
}

/// Actions for prohibited fields
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ProhibitedAction {
    /// Redact and log
    RedactAndLog,
    /// Tokenize last 4 only
    TokenizeLast4,
    /// Refuse and quarantine
    RefuseAndQuarantine,
}

/// Linking rules configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LinkingRules {
    /// Temporal proximity in seconds
    pub temporal_proximity_secs: u64,
    /// Semantic similarity threshold
    pub semantic_similarity_threshold: f32,
    /// Entity resolution method
    pub entity_resolution: EntityResolution,
}

/// Entity resolution methods
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum EntityResolution {
    /// Exact match
    Exact,
    /// Fuzzy match with Levenshtein distance
    FuzzyMatch { max_distance: usize },
    /// Phonetic match (Metaphone)
    Phonetic,
}

/// Complete schema definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Schema {
    /// Schema version
    pub version: String,
    /// Compilation rules
    pub compilation_rules: Vec<CompilationRule>,
    /// Prohibited extractions
    pub prohibited_extractions: Vec<ProhibitedExtraction>,
    /// Linking rules
    pub linking_rules: LinkingRules,
    /// Field type definitions
    pub field_types: HashMap<String, FieldType>,
}

/// Field type definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FieldType {
    /// Field name
    pub name: String,
    /// Data type
    pub data_type: DataType,
    /// Validation regex
    pub validation: Option<String>,
    /// Normalization rules
    pub normalization: Vec<NormalizationRule>,
}

/// Data types for fields
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DataType {
    /// String value
    String,
    /// Integer
    Integer,
    /// Float/decimal
    Float,
    /// Date
    Date,
    /// DateTime
    DateTime,
    /// Currency amount
    Currency,
    /// Phone number
    Phone,
    /// Email address
    Email,
    /// URL
    Url,
    /// List of values
    List(Box<DataType>),
    /// Structured object
    Object(HashMap<String, DataType>),
}

/// Normalization rules
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum NormalizationRule {
    /// Convert to lowercase
    Lowercase,
    /// Convert to uppercase
    Uppercase,
    /// Trim whitespace
    Trim,
    /// Remove special characters
    RemoveSpecialChars,
    /// Format as phone number
    FormatPhone,
    /// Format as currency
    FormatCurrency { currency: String },
}

impl Default for Schema {
    fn default() -> Self {
        Self {
            version: SCHEMA_VERSION.to_string(),
            compilation_rules: vec![
                CompilationRule {
                    id: "receipt".to_string(),
                    trigger: TriggerCondition::ConceptDetected { 
                        concept: "receipt".to_string() 
                    },
                    action: CompilationAction::CreateEntity { 
                        entity_type: "merchant".to_string() 
                    },
                    extract: vec![
                        "merchant_name".to_string(),
                        "total_amount".to_string(),
                        "currency".to_string(),
                        "tax_amount".to_string(),
                        "items_list".to_string(),
                        "payment_method".to_string(),
                    ],
                    link_to: vec![
                        LinkTarget {
                            node_type: "concept".to_string(),
                            path_template: "concepts/expenses".to_string(),
                            context: None,
                        },
                        LinkTarget {
                            node_type: "chronology".to_string(),
                            path_template: "chronology/{YYYY-MM}".to_string(),
                            context: None,
                        },
                    ],
                    privacy: PrivacyLevel::Financial,
                    temporal_awareness: true,
                },
                CompilationRule {
                    id: "contact".to_string(),
                    trigger: TriggerCondition::And(vec![
                        TriggerCondition::EntityType { 
                            entity_type: "person".to_string() 
                        },
                        TriggerCondition::Context { 
                            context: "contact_card".to_string() 
                        },
                    ]),
                    action: CompilationAction::MergeOrCreate { 
                        path_template: "entities/{firstname-lastname}.md".to_string() 
                    },
                    extract: vec![
                        "phone".to_string(),
                        "email".to_string(),
                        "company".to_string(),
                        "role".to_string(),
                    ],
                    link_to: vec![],
                    privacy: PrivacyLevel::Personal,
                    temporal_awareness: false,
                },
                CompilationRule {
                    id: "travel".to_string(),
                    trigger: TriggerCondition::Pattern { 
                        pattern: "flight_confirmation".to_string() 
                    },
                    action: CompilationAction::CreateTravelItinerary,
                    extract: vec![
                        "airline".to_string(),
                        "flight_number".to_string(),
                        "departure".to_string(),
                        "arrival".to_string(),
                        "confirmation_code".to_string(),
                    ],
                    link_to: vec![
                        LinkTarget {
                            node_type: "chronology".to_string(),
                            path_template: "chronology/{travel_date}".to_string(),
                            context: Some("travel".to_string()),
                        },
                    ],
                    privacy: PrivacyLevel::Personal,
                    temporal_awareness: true,
                },
            ],
            prohibited_extractions: vec![
                ProhibitedExtraction {
                    field: "password".to_string(),
                    action: ProhibitedAction::RedactAndLog,
                },
                ProhibitedExtraction {
                    field: "credit_card_number".to_string(),
                    action: ProhibitedAction::TokenizeLast4,
                },
                ProhibitedExtraction {
                    field: "ssn".to_string(),
                    action: ProhibitedAction::RefuseAndQuarantine,
                },
            ],
            linking_rules: LinkingRules {
                temporal_proximity_secs: 300, // 5 minutes
                semantic_similarity_threshold: 0.82,
                entity_resolution: EntityResolution::FuzzyMatch { max_distance: 2 },
            },
            field_types: {
                let mut map = HashMap::new();
                map.insert("phone".to_string(), FieldType {
                    name: "phone".to_string(),
                    data_type: DataType::Phone,
                    validation: Some(r"^[\d\s\-\+\(\)]+$".to_string()),
                    normalization: vec![NormalizationRule::RemoveSpecialChars],
                });
                map.insert("email".to_string(), FieldType {
                    name: "email".to_string(),
                    data_type: DataType::Email,
                    validation: Some(r"^[\w.-]+@[\w.-]+\.\w+$".to_string()),
                    normalization: vec![NormalizationRule::Lowercase, NormalizationRule::Trim],
                });
                map.insert("total_amount".to_string(), FieldType {
                    name: "total_amount".to_string(),
                    data_type: DataType::Currency,
                    validation: Some(r"^\d+\.\d{2}$".to_string()),
                    normalization: vec![NormalizationRule::FormatCurrency { currency: "USD".to_string() }],
                });
                map
            },
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_default_schema() {
        let schema = Schema::default();
        assert_eq!(schema.version, SCHEMA_VERSION);
        assert!(!schema.compilation_rules.is_empty());
    }
    
    #[test]
    fn test_trigger_condition() {
        let trigger = TriggerCondition::ConceptDetected { 
            concept: "receipt".to_string() 
        };
        
        match trigger {
            TriggerCondition::ConceptDetected { concept } => {
                assert_eq!(concept, "receipt");
            }
            _ => panic!("Wrong trigger type"),
        }
    }
    
    #[test]
    fn test_privacy_levels() {
        assert_ne!(PrivacyLevel::Public, PrivacyLevel::Financial);
        assert_eq!(PrivacyLevel::Personal, PrivacyLevel::Personal);
    }
}
