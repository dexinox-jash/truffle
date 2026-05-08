//! Fact Model
//!
//! A verifiable statement about an entity in the knowledge graph.
//! Facts represent atomic pieces of information that can be validated
//! for consistency and contradictions.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// A verifiable statement about an entity
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Fact {
    /// Unique identifier
    pub id: Uuid,

    /// Entity this fact is about
    pub entity_id: Uuid,

    /// Attribute name (e.g., "employer", "location", "title")
    pub attribute: String,

    /// The value of this fact
    pub value: FactValue,

    /// When this fact became valid (None = always valid)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub valid_from: Option<DateTime<Utc>>,

    /// When this fact stopped being valid (None = still valid)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub valid_until: Option<DateTime<Utc>>,

    /// Source of this fact
    pub source: FactSource,

    /// Confidence score (0.0 to 1.0)
    pub confidence: f32,

    /// Link to extraction event (if from pipeline)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub extraction_id: Option<Uuid>,

    /// When this fact was created
    pub created_at: DateTime<Utc>,

    /// When this fact was last updated
    pub updated_at: DateTime<Utc>,
}

impl Fact {
    /// Create a new fact
    pub fn new(
        entity_id: Uuid,
        attribute: impl Into<String>,
        value: FactValue,
        source: FactSource,
    ) -> Self {
        Self {
            id: Uuid::new_v4(),
            entity_id,
            attribute: attribute.into(),
            value,
            valid_from: None,
            valid_until: None,
            source,
            confidence: 0.8,
            extraction_id: None,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        }
    }

    /// Set the valid time range for this fact
    pub fn with_valid_range(
        mut self,
        from: Option<DateTime<Utc>>,
        until: Option<DateTime<Utc>>,
    ) -> Self {
        self.valid_from = from;
        self.valid_until = until;
        self
    }

    /// Set confidence score (clamped to 0.0-1.0)
    pub fn with_confidence(mut self, confidence: f32) -> Self {
        self.confidence = confidence.clamp(0.0, 1.0);
        self
    }

    /// Set extraction ID
    pub fn with_extraction(mut self, extraction_id: Uuid) -> Self {
        self.extraction_id = Some(extraction_id);
        self
    }

    /// Mark this fact as deprecated (set valid_until to now)
    pub fn deprecate(&mut self) {
        self.valid_until = Some(Utc::now());
        self.updated_at = Utc::now();
    }

    /// Check if this fact is currently valid
    pub fn is_currently_valid(&self) -> bool {
        let now = Utc::now();
        
        let after_start = match self.valid_from {
            Some(start) => now >= start,
            None => true,
        };
        
        let before_end = match self.valid_until {
            Some(end) => now < end,
            None => true,
        };
        
        after_start && before_end
    }

    /// Check if this fact overlaps with a time range
    pub fn overlaps_range(&self, from: DateTime<Utc>, until: DateTime<Utc>) -> bool {
        let self_start = self.valid_from.unwrap_or_else(|| DateTime::from_timestamp(0, 0).unwrap());
        let self_end = self.valid_until.unwrap_or_else(|| DateTime::from_timestamp(i64::MAX, 0).unwrap());
        
        self_start < until && self_end > from
    }

    /// Get a string representation of the value
    pub fn value_string(&self) -> String {
        self.value.to_string()
    }
}

/// Possible value types for a fact
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case", tag = "type", content = "value")]
pub enum FactValue {
    /// Text value
    Text(String),
    /// Numeric value
    Number(f64),
    /// Date/time value
    Date(DateTime<Utc>),
    /// Reference to another entity
    EntityRef(Uuid),
}

impl FactValue {
    /// Create a text value
    pub fn text(value: impl Into<String>) -> Self {
        Self::Text(value.into())
    }

    /// Create a number value
    pub fn number(value: f64) -> Self {
        Self::Number(value)
    }

    /// Create a date value
    pub fn date(value: DateTime<Utc>) -> Self {
        Self::Date(value)
    }

    /// Create an entity reference
    pub fn entity_ref(entity_id: Uuid) -> Self {
        Self::EntityRef(entity_id)
    }

    /// Get the type name as a string
    pub fn type_name(&self) -> &'static str {
        match self {
            Self::Text(_) => "text",
            Self::Number(_) => "number",
            Self::Date(_) => "date",
            Self::EntityRef(_) => "entity_ref",
        }
    }

    /// Check if two values are semantically equal
    pub fn is_equal_to(&self, other: &Self) -> bool {
        match (self, other) {
            (Self::Text(a), Self::Text(b)) => a.trim().eq_ignore_ascii_case(b.trim()),
            (Self::Number(a), Self::Number(b)) => (a - b).abs() < f64::EPSILON,
            (Self::Date(a), Self::Date(b)) => a == b,
            (Self::EntityRef(a), Self::EntityRef(b)) => a == b,
            _ => false,
        }
    }
}

impl std::fmt::Display for FactValue {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Text(s) => write!(f, "{}", s),
            Self::Number(n) => write!(f, "{}", n),
            Self::Date(d) => write!(f, "{}", d.to_rfc3339()),
            Self::EntityRef(id) => write!(f, "@{}", id),
        }
    }
}

/// Source of a fact
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case", tag = "source_type")]
pub enum FactSource {
    /// Extracted from pipeline processing
    Extraction {
        /// Pipeline instance that extracted this fact
        pipeline_id: Uuid,
        /// Optional meeting ID if from transcript
        #[serde(skip_serializing_if = "Option::is_none")]
        meeting_id: Option<Uuid>,
    },
    /// Manually entered by a user
    Manual {
        /// User who entered this fact
        user_id: Uuid,
        /// Whether this fact has been verified
        verified: bool,
    },
    /// Imported from an external system
    Import {
        /// Name of the source system
        source_system: String,
        /// ID in the source system
        import_id: String,
    },
}

impl FactSource {
    /// Create an extraction source
    pub fn extraction(pipeline_id: Uuid) -> Self {
        Self::Extraction {
            pipeline_id,
            meeting_id: None,
        }
    }

    /// Create an extraction source with meeting
    pub fn extraction_with_meeting(pipeline_id: Uuid, meeting_id: Uuid) -> Self {
        Self::Extraction {
            pipeline_id,
            meeting_id: Some(meeting_id),
        }
    }

    /// Create a manual source
    pub fn manual(user_id: Uuid) -> Self {
        Self::Manual {
            user_id,
            verified: false,
        }
    }

    /// Create a verified manual source
    pub fn manual_verified(user_id: Uuid) -> Self {
        Self::Manual {
            user_id,
            verified: true,
        }
    }

    /// Create an import source
    pub fn import(source_system: impl Into<String>, import_id: impl Into<String>) -> Self {
        Self::Import {
            source_system: source_system.into(),
            import_id: import_id.into(),
        }
    }

    /// Get a display name for this source
    pub fn display_name(&self) -> String {
        match self {
            Self::Extraction { pipeline_id, meeting_id } => {
                if let Some(mid) = meeting_id {
                    format!("Extraction ({}/{})", pipeline_id, mid)
                } else {
                    format!("Extraction ({})", pipeline_id)
                }
            }
            Self::Manual { user_id, verified } => {
                if *verified {
                    format!("Manual by {} (verified)", user_id)
                } else {
                    format!("Manual by {}", user_id)
                }
            }
            Self::Import { source_system, import_id } => {
                format!("Import from {} ({})", source_system, import_id)
            }
        }
    }

    /// Check if this source is verified
    pub fn is_verified(&self) -> bool {
        match self {
            Self::Manual { verified, .. } => *verified,
            _ => false,
        }
    }

    /// Check if this source is from an extraction pipeline
    pub fn is_extraction(&self) -> bool {
        matches!(self, Self::Extraction { .. })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_fact_creation() {
        let entity_id = Uuid::new_v4();
        let source = FactSource::extraction(Uuid::new_v4());
        
        let fact = Fact::new(
            entity_id,
            "employer",
            FactValue::text("Acme Corp"),
            source,
        );
        
        assert_eq!(fact.attribute, "employer");
        assert_eq!(fact.value_string(), "Acme Corp");
        assert_eq!(fact.confidence, 0.8);
        assert!(fact.is_currently_valid());
    }

    #[test]
    fn test_fact_with_valid_range() {
        let entity_id = Uuid::new_v4();
        let source = FactSource::extraction(Uuid::new_v4());
        
        let from = Utc::now();
        let until = from + chrono::Duration::days(30);
        
        let fact = Fact::new(
            entity_id,
            "title",
            FactValue::text("Engineer"),
            source,
        ).with_valid_range(Some(from), Some(until));
        
        assert!(fact.is_currently_valid());
        assert!(fact.overlaps_range(from, until));
    }

    #[test]
    fn test_fact_deprecation() {
        let entity_id = Uuid::new_v4();
        let source = FactSource::extraction(Uuid::new_v4());
        
        let mut fact = Fact::new(
            entity_id,
            "location",
            FactValue::text("New York"),
            source,
        );
        
        assert!(fact.is_currently_valid());
        fact.deprecate();
        assert!(!fact.is_currently_valid());
    }

    #[test]
    fn test_fact_value_equality() {
        let v1 = FactValue::text("Hello World");
        let v2 = FactValue::text("hello world");
        let v3 = FactValue::text("Different");
        
        assert!(v1.is_equal_to(&v2));
        assert!(!v1.is_equal_to(&v3));
        
        let n1 = FactValue::number(3.14);
        let n2 = FactValue::number(3.14);
        let n3 = FactValue::number(2.71);
        
        assert!(n1.is_equal_to(&n2));
        assert!(!n1.is_equal_to(&n3));
    }

    #[test]
    fn test_fact_source_display() {
        let extraction = FactSource::extraction(Uuid::new_v4());
        assert!(extraction.display_name().starts_with("Extraction"));
        
        let manual = FactSource::manual_verified(Uuid::new_v4());
        assert!(manual.display_name().contains("verified"));
        assert!(manual.is_verified());
        
        let import = FactSource::import("CRM", "contact_123");
        assert!(import.display_name().contains("CRM"));
    }

    #[test]
    fn test_confidence_clamping() {
        let entity_id = Uuid::new_v4();
        let source = FactSource::extraction(Uuid::new_v4());
        
        let fact = Fact::new(
            entity_id,
            "test",
            FactValue::text("test"),
            source,
        ).with_confidence(1.5);
        
        assert_eq!(fact.confidence, 1.0);
        
        let fact2 = Fact::new(
            entity_id,
            "test2",
            FactValue::text("test"),
            FactSource::extraction(Uuid::new_v4()),
        ).with_confidence(-0.5);
        
        assert_eq!(fact2.confidence, 0.0);
    }
}
