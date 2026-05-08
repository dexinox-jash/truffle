// SPEC v2.0 SECTION 2.1.3: Schema Module
// Compilation rules and schema validation

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Schema {
    pub version: String,
    pub compilation_rules: Vec<CompilationRule>,
    pub prohibited_extractions: Vec<ProhibitedExtraction>,
    pub linking_rules: LinkingRules,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompilationRule {
    pub trigger: String,
    pub action: String,
    pub extract: Vec<String>,
    pub link_to: Vec<String>,
    pub privacy: String,
    pub temporal_awareness: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProhibitedExtraction {
    pub field: String,
    pub action: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LinkingRules {
    pub temporal_proximity: u64,
    pub semantic_similarity_threshold: f32,
    pub entity_resolution: EntityResolution,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EntityResolution {
    pub levenshtein_threshold: usize,
}

impl Default for Schema {
    fn default() -> Self {
        Self {
            version: "1.0.0".to_string(),
            compilation_rules: vec![
                CompilationRule {
                    trigger: "concept_detected('receipt')".to_string(),
                    action: "create_entity(merchant_name)".to_string(),
                    extract: vec![
                        "total_amount".to_string(),
                        "currency".to_string(),
                        "tax_amount".to_string(),
                        "items_list".to_string(),
                        "payment_method".to_string(),
                    ],
                    link_to: vec![
                        "concepts/expenses".to_string(),
                        "chronology/{YYYY-MM}".to_string(),
                    ],
                    privacy: "financial".to_string(),
                    temporal_awareness: Some(true),
                },
                CompilationRule {
                    trigger: "entity_type('person') AND context('contact_card')".to_string(),
                    action: "merge_or_create(entities/{firstname-lastname}.md)".to_string(),
                    extract: vec![
                        "phone".to_string(),
                        "email".to_string(),
                        "company".to_string(),
                        "role".to_string(),
                    ],
                    link_to: vec![],
                    privacy: "personal".to_string(),
                    temporal_awareness: None,
                },
                CompilationRule {
                    trigger: "pattern('flight_confirmation')".to_string(),
                    action: "create_travel_itinerary".to_string(),
                    extract: vec![
                        "airline".to_string(),
                        "flight_number".to_string(),
                        "departure".to_string(),
                        "arrival".to_string(),
                        "confirmation_code".to_string(),
                    ],
                    link_to: vec![],
                    privacy: "personal".to_string(),
                    temporal_awareness: Some(true),
                },
            ],
            prohibited_extractions: vec![
                ProhibitedExtraction {
                    field: "password".to_string(),
                    action: "redact_and_log".to_string(),
                },
                ProhibitedExtraction {
                    field: "credit_card_number".to_string(),
                    action: "tokenize_last4_only".to_string(),
                },
                ProhibitedExtraction {
                    field: "ssn".to_string(),
                    action: "refuse_and_quarantine".to_string(),
                },
            ],
            linking_rules: LinkingRules {
                temporal_proximity: 300,
                semantic_similarity_threshold: 0.82,
                entity_resolution: EntityResolution {
                    levenshtein_threshold: 2,
                },
            },
        }
    }
}

impl Schema {
    pub fn from_yaml(yaml: &str) -> Result<Self> {
        serde_yaml::from_str(yaml).context("Failed to parse schema YAML")
    }

    pub fn to_yaml(&self) -> Result<String> {
        serde_yaml::to_string(self).context("Failed to serialize schema to YAML")
    }

    pub fn validate(&self) -> SchemaValidationResult {
        let mut errors = vec![];
        let mut warnings = vec![];

        // Validate version
        if self.version.is_empty() {
            errors.push("Schema version is required".to_string());
        }

        // Validate compilation rules
        for (i, rule) in self.compilation_rules.iter().enumerate() {
            if rule.trigger.is_empty() {
                errors.push(format!("Rule {}: trigger is required", i));
            }
            if rule.action.is_empty() {
                errors.push(format!("Rule {}: action is required", i));
            }
            if rule.extract.is_empty() {
                warnings.push(format!("Rule {}: no extract fields defined", i));
            }
        }

        // Validate linking rules
        if self.linking_rules.semantic_similarity_threshold < 0.0
            || self.linking_rules.semantic_similarity_threshold > 1.0
        {
            errors.push("Semantic similarity threshold must be between 0 and 1".to_string());
        }

        SchemaValidationResult {
            valid: errors.is_empty(),
            errors,
            warnings,
        }
    }

    pub fn find_matching_rule(&self, content: &str) -> Option<&CompilationRule> {
        // Simple keyword matching (would be more sophisticated in production)
        for rule in &self.compilation_rules {
            if rule.trigger.contains("receipt") && content.to_lowercase().contains("receipt") {
                return Some(rule);
            }
            if rule.trigger.contains("contact") && content.to_lowercase().contains("contact") {
                return Some(rule);
            }
            if rule.trigger.contains("flight") && content.to_lowercase().contains("flight") {
                return Some(rule);
            }
        }
        None
    }
}

#[derive(Debug, Clone)]
pub struct SchemaValidationResult {
    pub valid: bool,
    pub errors: Vec<String>,
    pub warnings: Vec<String>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_schema() {
        let schema = Schema::default();
        assert!(!schema.version.is_empty());
        assert!(!schema.compilation_rules.is_empty());
    }

    #[test]
    fn test_schema_validation() {
        let schema = Schema::default();
        let result = schema.validate();
        assert!(result.valid);
    }

    #[test]
    fn test_invalid_schema() {
        let schema = Schema {
            version: "".to_string(),
            ..Schema::default()
        };
        let result = schema.validate();
        assert!(!result.valid);
        assert!(!result.errors.is_empty());
    }
}
