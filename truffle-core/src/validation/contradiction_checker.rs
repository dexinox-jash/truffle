//! Contradiction Checker
//!
//! Detects factual and temporal contradictions in the knowledge graph.
//! Performs pairwise comparison of facts to identify conflicting information.

use chrono::{DateTime, Utc};
use std::collections::HashMap;
use tracing::{debug, instrument};
use uuid::Uuid;

use crate::validation::models::{Contradiction, ContradictionSeverity, ContradictionType, Fact};

/// Contradiction detection engine
#[derive(Debug, Clone, Default)]
pub struct ContradictionChecker;

impl ContradictionChecker {
    /// Create a new contradiction checker
    pub fn new() -> Self {
        Self
    }

    /// Check if two facts contradict each other
    #[instrument(skip(self, fact_a, fact_b), level = "debug")]
    pub fn facts_contradict(&self, fact_a: &Fact, fact_b: &Fact) -> Option<ContradictionType> {
        // Must be about the same entity
        if fact_a.entity_id != fact_b.entity_id {
            return None;
        }

        // Must be about the same attribute
        if fact_a.attribute != fact_b.attribute {
            return None;
        }

        // If values are equal, no contradiction
        if fact_a.value == fact_b.value {
            return None;
        }

        // Check for temporal overlap
        let has_overlap = self.temporal_overlap(fact_a, fact_b);

        // Determine contradiction type
        let contradiction_type = if has_overlap {
            ContradictionType::Temporal
        } else if fact_a.valid_from.is_none() && fact_b.valid_from.is_none() {
            // Both are timeless facts with different values
            ContradictionType::Factual
        } else {
            // No temporal overlap and at least one has time bounds
            // These might be valid state changes over time
            return None;
        };

        Some(contradiction_type)
    }

    /// Check if two facts have overlapping time periods
    pub fn temporal_overlap(&self, fact_a: &Fact, fact_b: &Fact) -> bool {
        // If neither has time bounds, they overlap (both always valid)
        if fact_a.valid_from.is_none()
            && fact_a.valid_until.is_none()
            && fact_b.valid_from.is_none()
            && fact_b.valid_until.is_none()
        {
            return true;
        }

        // Get effective time ranges
        let a_start = fact_a.valid_from.unwrap_or_else(|| {
            DateTime::from_timestamp(0, 0).unwrap_or(DateTime::UNIX_EPOCH)
        });
        let a_end = fact_a
            .valid_until
            .unwrap_or(DateTime::MAX_UTC);

        let b_start = fact_b.valid_from.unwrap_or_else(|| {
            DateTime::from_timestamp(0, 0).unwrap_or(DateTime::UNIX_EPOCH)
        });
        let b_end = fact_b
            .valid_until
            .unwrap_or(DateTime::MAX_UTC);

        // Check for overlap: intervals [a_start, a_end] and [b_start, b_end]
        a_start < b_end && b_start < a_end
    }

    /// Check an entity for internal contradictions
    #[instrument(skip(self, facts), level = "debug")]
    pub fn check_entity_facts(&self, entity_id: Uuid, facts: &[Fact]) -> Vec<Contradiction> {
        let mut contradictions = Vec::new();

        // Group facts by attribute
        let mut facts_by_attribute: HashMap<String, Vec<&Fact>> = HashMap::new();
        for fact in facts {
            if fact.entity_id == entity_id {
                facts_by_attribute
                    .entry(fact.attribute.clone())
                    .or_default()
                    .push(fact);
            }
        }

        // Check each attribute group for contradictions
        for (attribute, attr_facts) in facts_by_attribute {
            debug!(
                "Checking {} facts for attribute '{}'",
                attr_facts.len(),
                attribute
            );

            // Pairwise comparison
            for i in 0..attr_facts.len() {
                for j in (i + 1)..attr_facts.len() {
                    if let Some(contradiction_type) =
                        self.facts_contradict(attr_facts[i], attr_facts[j])
                    {
                        let severity = self.calculate_severity(attr_facts[i], attr_facts[j]);

                        let contradiction = Contradiction::new(
                            entity_id,
                            attr_facts[i].id,
                            attr_facts[j].id,
                            contradiction_type,
                            severity,
                        )
                        .with_description(format!(
                            "Conflicting values for '{}': '{:?}' vs '{:?}'",
                            attribute, attr_facts[i].value, attr_facts[j].value
                        ));

                        contradictions.push(contradiction);
                    }
                }
            }
        }

        contradictions
    }

    /// Calculate severity based on confidence difference and impact
    fn calculate_severity(&self, fact_a: &Fact, fact_b: &Fact) -> ContradictionSeverity {
        // High severity if both facts have high confidence
        if fact_a.confidence > 0.8 && fact_b.confidence > 0.8 {
            let confidence_diff = (fact_a.confidence - fact_b.confidence).abs();
            if confidence_diff < 0.1 {
                // Similar high confidence - needs investigation
                ContradictionSeverity::Critical
            } else {
                // One is clearly more reliable
                ContradictionSeverity::High
            }
        } else if fact_a.confidence > 0.6 || fact_b.confidence > 0.6 {
            ContradictionSeverity::Medium
        } else {
            ContradictionSeverity::Low
        }
    }
}

/// Report from a validation run
#[derive(Debug, Clone)]
pub struct ValidationReport {
    /// Number of entities checked
    pub checked_entities: usize,
    /// Total facts examined
    pub total_facts_checked: usize,
    /// Contradictions found
    pub contradictions_found: Vec<Contradiction>,
    /// Count by severity
    pub contradictions_by_severity: HashMap<ContradictionSeverity, usize>,
    /// When the check was performed
    pub checked_at: DateTime<Utc>,
    /// Duration in milliseconds
    pub duration_ms: u64,
}

impl ValidationReport {
    /// Create a new validation report
    pub fn new(checked_entities: usize, facts_checked: usize) -> Self {
        Self {
            checked_entities,
            total_facts_checked: facts_checked,
            contradictions_found: Vec::new(),
            contradictions_by_severity: HashMap::new(),
            checked_at: Utc::now(),
            duration_ms: 0,
        }
    }

    /// Add a contradiction to the report
    pub fn add_contradiction(&mut self, contradiction: Contradiction) {
        let severity = contradiction.severity.clone();
        self.contradictions_found.push(contradiction);
        *self.contradictions_by_severity.entry(severity).or_insert(0) += 1;
    }

    /// Get total contradiction count
    pub fn total_contradictions(&self) -> usize {
        self.contradictions_found.len()
    }

    /// Get critical contradictions
    pub fn critical_count(&self) -> usize {
        self.contradictions_by_severity
            .get(&ContradictionSeverity::Critical)
            .copied()
            .unwrap_or(0)
    }

    /// Check if report has contradictions
    pub fn has_contradictions(&self) -> bool {
        !self.contradictions_found.is_empty()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::validation::models::{FactSource, FactValue};

    fn create_test_fact(
        entity_id: Uuid,
        attribute: &str,
        value: FactValue,
        confidence: f32,
    ) -> Fact {
        Fact::new(entity_id, attribute, value, FactSource::manual_confirmed())
            .with_confidence(confidence)
    }

    #[test]
    fn test_facts_contradict_different_values() {
        let checker = ContradictionChecker::new();
        let entity_id = Uuid::new_v4();

        let fact_a = create_test_fact(
            entity_id,
            "employer",
            FactValue::Text("Google".to_string()),
            0.9,
        );
        let fact_b = create_test_fact(
            entity_id,
            "employer",
            FactValue::Text("Meta".to_string()),
            0.9,
        );

        let result = checker.facts_contradict(&fact_a, &fact_b);
        assert!(result.is_some());
        assert_eq!(result.expect("Expected contradiction"), ContradictionType::Factual);
    }

    #[test]
    fn test_facts_no_contradiction_same_value() {
        let checker = ContradictionChecker::new();
        let entity_id = Uuid::new_v4();

        let fact_a = create_test_fact(
            entity_id,
            "employer",
            FactValue::Text("Google".to_string()),
            0.9,
        );
        let fact_b = create_test_fact(
            entity_id,
            "employer",
            FactValue::Text("Google".to_string()),
            0.8,
        );

        let result = checker.facts_contradict(&fact_a, &fact_b);
        assert!(result.is_none());
    }

    #[test]
    fn test_temporal_overlap_detected() {
        let checker = ContradictionChecker::new();
        let entity_id = Uuid::new_v4();
        let now = Utc::now();

        let fact_a = create_test_fact(
            entity_id,
            "employer",
            FactValue::Text("Google".to_string()),
            0.9,
        )
        .with_valid_range(Some(now - chrono::Duration::days(365)), Some(now));

        let fact_b = create_test_fact(
            entity_id,
            "employer",
            FactValue::Text("Meta".to_string()),
            0.9,
        )
        .with_valid_range(Some(now - chrono::Duration::days(180)), None);

        let result = checker.facts_contradict(&fact_a, &fact_b);
        assert!(result.is_some());
        assert_eq!(result.expect("Expected contradiction"), ContradictionType::Temporal);
    }

    #[test]
    fn test_validation_report_counts() {
        let mut report = ValidationReport::new(10, 50);

        let entity_id = Uuid::new_v4();
        let fact_a = create_test_fact(
            entity_id,
            "location",
            FactValue::Text("NYC".to_string()),
            0.9,
        );
        let fact_b = create_test_fact(
            entity_id,
            "location",
            FactValue::Text("SF".to_string()),
            0.9,
        );

        let contradiction =
            Contradiction::new(entity_id, fact_a.id, fact_b.id, ContradictionType::Factual, ContradictionSeverity::High);
        report.add_contradiction(contradiction);

        assert_eq!(report.total_contradictions(), 1);
        assert!(report.has_contradictions());
    }
}
