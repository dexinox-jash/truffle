//! JSON Parser and Validator
//! 
//! Validates model output against WikiNode schema.
//! 
//! # JSON Mode Enforcement (SPEC v2.0 Section 4.2)
//! 
//! - Model MUST output valid JSON only
//! - No free-text generation allowed
//! - Schema validation with detailed error reporting
//! - Automatic retry on parse failure

use serde::{Deserialize, Serialize};
use serde_json::Value;
use tracing::{info, warn, debug, error};

use crate::AiError;
use crate::AiResult;

/// JSON parser with validation
pub struct JsonParser {
    /// Maximum JSON depth
    max_depth: usize,
    /// Maximum string length
    max_string_length: usize,
    /// Required fields
    required_fields: Vec<String>,
}

/// Result of JSON validation
#[derive(Debug, Clone)]
pub enum ValidationResult {
    /// Valid JSON matching schema
    Valid(Value),
    /// Invalid with list of errors
    Invalid(Vec<ValidationError>),
}

/// Validation error details
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationError {
    /// Error type
    pub error_type: ErrorType,
    /// Field path (e.g., "node_type" or "content.title")
    pub field_path: String,
    /// Error message
    pub message: String,
    /// Suggested fix
    pub suggestion: Option<String>,
}

/// Types of validation errors
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ErrorType {
    /// Missing required field
    MissingField,
    /// Invalid field type
    InvalidType,
    /// Value out of range
    OutOfRange,
    /// Invalid format
    InvalidFormat,
    /// JSON syntax error
    SyntaxError,
    /// Schema violation
    SchemaViolation,
    /// Unknown field
    UnknownField,
}

/// Parsed and normalized WikiNode JSON
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ParsedWikiNode {
    /// Raw JSON value
    pub raw: Value,
    /// Normalized fields
    pub normalized: NormalizedFields,
    /// Validation errors (if any)
    pub errors: Vec<ValidationError>,
    /// Whether parsing succeeded
    pub is_valid: bool,
}

/// Normalized field values
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct NormalizedFields {
    /// Node type
    pub node_type: Option<String>,
    /// Title
    pub title: Option<String>,
    /// Content
    pub content: Option<String>,
    /// Privacy classification
    pub privacy_classification: Option<String>,
    /// Confidence score
    pub confidence_score: Option<f32>,
}

impl JsonParser {
    /// Create a new JSON parser
    pub fn new() -> Self {
        Self {
            max_depth: 10,
            max_string_length: 100_000, // 100KB max per string
            required_fields: vec![
                "node_type".to_string(),
                "title".to_string(),
                "content".to_string(),
            ],
        }
    }
    
    /// Parse and validate JSON output from model
    /// 
    /// # Arguments
    /// 
    /// * `json_str` - Raw JSON string from model
    /// 
    /// # Returns
    /// 
    /// Returns `ValidationResult::Valid` if parsing succeeds,
    /// or `ValidationResult::Invalid` with error details.
    pub fn parse_and_validate(&self, json_str: &str) -> AiResult<ValidationResult> {
        debug!("Parsing JSON output ({} chars)", json_str.len());
        
        // Step 1: Parse JSON syntax
        let value = match serde_json::from_str::<Value>(json_str) {
            Ok(v) => v,
            Err(e) => {
                warn!("JSON parse error: {}", e);
                
                // Try to extract partial JSON if wrapped in markdown
                if let Some(extracted) = self.extract_json_from_markdown(json_str) {
                    debug!("Extracted JSON from markdown");
                    return self.parse_and_validate(&extracted);
                }
                
                return Ok(ValidationResult::Invalid(vec![ValidationError {
                    error_type: ErrorType::SyntaxError,
                    field_path: "root".to_string(),
                    message: format!("JSON syntax error: {}", e),
                    suggestion: Some("Check for trailing commas or missing quotes".to_string()),
                }]));
            }
        };
        
        // Step 2: Validate structure
        let errors = self.validate_structure(&value);
        
        if errors.is_empty() {
            debug!("JSON validation passed");
            Ok(ValidationResult::Valid(value))
        } else {
            warn!("JSON validation failed with {} errors", errors.len());
            Ok(ValidationResult::Invalid(errors))
        }
    }
    
    /// Validate JSON structure against WikiNode schema
    fn validate_structure(&self, value: &Value) -> Vec<ValidationError> {
        let mut errors = Vec::new();
        
        // Check if root is an object
        let obj = match value.as_object() {
            Some(o) => o,
            None => {
                errors.push(ValidationError {
                    error_type: ErrorType::InvalidType,
                    field_path: "root".to_string(),
                    message: "Root must be an object".to_string(),
                    suggestion: Some("Wrap output in curly braces {}".to_string()),
                });
                return errors;
            }
        };
        
        // Check required fields
        for field in &self.required_fields {
            if !obj.contains_key(field) {
                errors.push(ValidationError {
                    error_type: ErrorType::MissingField,
                    field_path: field.clone(),
                    message: format!("Required field '{}' is missing", field),
                    suggestion: Some(format!("Add '{}' field to output", field)),
                });
            }
        }
        
        // Validate node_type
        if let Some(node_type) = obj.get("node_type") {
            if let Some(type_str) = node_type.as_str() {
                let valid_types = ["entity", "concept", "chronology", "index"];
                if !valid_types.contains(&type_str) {
                    errors.push(ValidationError {
                        error_type: ErrorType::InvalidType,
                        field_path: "node_type".to_string(),
                        message: format!("Invalid node_type: {}", type_str),
                        suggestion: Some(format!("Use one of: {:?}", valid_types)),
                    });
                }
            } else {
                errors.push(ValidationError {
                    error_type: ErrorType::InvalidType,
                    field_path: "node_type".to_string(),
                    message: "node_type must be a string".to_string(),
                    suggestion: Some("Use a string value like 'entity'".to_string()),
                });
            }
        }
        
        // Validate title
        if let Some(title) = obj.get("title") {
            if let Some(title_str) = title.as_str() {
                if title_str.is_empty() {
                    errors.push(ValidationError {
                        error_type: ErrorType::InvalidFormat,
                        field_path: "title".to_string(),
                        message: "Title cannot be empty".to_string(),
                        suggestion: Some("Provide a descriptive title".to_string()),
                    });
                }
                if title_str.len() > 200 {
                    errors.push(ValidationError {
                        error_type: ErrorType::OutOfRange,
                        field_path: "title".to_string(),
                        message: format!("Title too long: {} chars (max 200)", title_str.len()),
                        suggestion: Some("Shorten the title".to_string()),
                    });
                }
            } else {
                errors.push(ValidationError {
                    error_type: ErrorType::InvalidType,
                    field_path: "title".to_string(),
                    message: "Title must be a string".to_string(),
                    suggestion: None,
                });
            }
        }
        
        // Validate content
        if let Some(content) = obj.get("content") {
            if let Some(content_str) = content.as_str() {
                if content_str.len() > self.max_string_length {
                    errors.push(ValidationError {
                        error_type: ErrorType::OutOfRange,
                        field_path: "content".to_string(),
                        message: format!(
                            "Content too long: {} chars (max {})",
                            content_str.len(),
                            self.max_string_length
                        ),
                        suggestion: Some("Truncate or summarize content".to_string()),
                    });
                }
            } else {
                errors.push(ValidationError {
                    error_type: ErrorType::InvalidType,
                    field_path: "content".to_string(),
                    message: "Content must be a string".to_string(),
                    suggestion: None,
                });
            }
        }
        
        // Validate privacy_classification
        if let Some(privacy) = obj.get("privacy_classification") {
            if let Some(privacy_str) = privacy.as_str() {
                let valid_levels = ["public", "personal", "sensitive", "financial"];
                if !valid_levels.contains(&privacy_str) {
                    errors.push(ValidationError {
                        error_type: ErrorType::InvalidType,
                        field_path: "privacy_classification".to_string(),
                        message: format!("Invalid privacy level: {}", privacy_str),
                        suggestion: Some(format!("Use one of: {:?}", valid_levels)),
                    });
                }
            }
        }
        
        // Validate confidence_score
        if let Some(confidence) = obj.get("confidence_score") {
            if let Some(confidence_num) = confidence.as_f64() {
                if confidence_num < 0.0 || confidence_num > 1.0 {
                    errors.push(ValidationError {
                        error_type: ErrorType::OutOfRange,
                        field_path: "confidence_score".to_string(),
                        message: format!("Confidence must be between 0 and 1, got {}", confidence_num),
                        suggestion: Some("Use a value in range [0.0, 1.0]".to_string()),
                    });
                }
            } else {
                errors.push(ValidationError {
                    error_type: ErrorType::InvalidType,
                    field_path: "confidence_score".to_string(),
                    message: "Confidence score must be a number".to_string(),
                    suggestion: Some("Use a float value like 0.95".to_string()),
                });
            }
        }
        
        // Check JSON depth
        if self.get_json_depth(value) > self.max_depth {
            errors.push(ValidationError {
                error_type: ErrorType::SchemaViolation,
                field_path: "root".to_string(),
                message: format!("JSON too deeply nested (max {} levels)", self.max_depth),
                suggestion: Some("Flatten nested structures".to_string()),
            });
        }
        
        errors
    }
    
    /// Extract JSON from markdown code blocks
    fn extract_json_from_markdown(&self, text: &str) -> Option<String> {
        // Look for JSON in markdown code blocks
        if let Some(start) = text.find("```json") {
            let after_start = &text[start + 7..];
            if let Some(end) = after_start.find("```") {
                return Some(after_start[..end].trim().to_string());
            }
        }
        
        // Look for JSON in generic code blocks
        if let Some(start) = text.find("```") {
            let after_start = &text[start + 3..];
            if let Some(end) = after_start.find("```") {
                let content = after_start[..end].trim();
                // Check if it looks like JSON
                if content.starts_with('{') && content.ends_with('}') {
                    return Some(content.to_string());
                }
            }
        }
        
        // Look for JSON between curly braces
        if let Some(start) = text.find('{') {
            if let Some(end) = text.rfind('}') {
                if end > start {
                    return Some(text[start..=end].to_string());
                }
            }
        }
        
        None
    }
    
    /// Calculate JSON nesting depth
    fn get_json_depth(&self, value: &Value) -> usize {
        match value {
            Value::Object(map) => {
                if map.is_empty() {
                    1
                } else {
                    1 + map.values()
                        .map(|v| self.get_json_depth(v))
                        .max()
                        .unwrap_or(0)
                }
            }
            Value::Array(arr) => {
                if arr.is_empty() {
                    1
                } else {
                    1 + arr.iter()
                        .map(|v| self.get_json_depth(v))
                        .max()
                        .unwrap_or(0)
                }
            }
            _ => 1,
        }
    }
    
    /// Normalize JSON to standard format
    pub fn normalize(&self, value: &Value) -> AiResult<NormalizedFields> {
        let mut normalized = NormalizedFields::default();
        
        if let Some(obj) = value.as_object() {
            normalized.node_type = obj.get("node_type")
                .and_then(|v| v.as_str())
                .map(|s| s.to_lowercase());
            
            normalized.title = obj.get("title")
                .and_then(|v| v.as_str())
                .map(|s| s.trim().to_string());
            
            normalized.content = obj.get("content")
                .and_then(|v| v.as_str())
                .map(|s| s.trim().to_string());
            
            normalized.privacy_classification = obj.get("privacy_classification")
                .and_then(|v| v.as_str())
                .map(|s| s.to_lowercase());
            
            normalized.confidence_score = obj.get("confidence_score")
                .and_then(|v| v.as_f64())
                .map(|f| f as f32);
        }
        
        Ok(normalized)
    }
    
    /// Create a minimal valid WikiNode JSON for fallback
    pub fn create_fallback(&self, error_message: &str) -> Value {
        serde_json::json!({
            "node_type": "entity",
            "title": "Compilation Error",
            "content": format!("Failed to compile screenshot: {}", error_message),
            "privacy_classification": "personal",
            "confidence_score": 0.0
        })
    }
}

impl Default for JsonParser {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_parse_valid_json() {
        let parser = JsonParser::new();
        
        let valid_json = r#"{
            "node_type": "entity",
            "title": "Test Receipt",
            "content": "Test content",
            "privacy_classification": "financial",
            "confidence_score": 0.95
        }"#;
        
        let result = parser.parse_and_validate(valid_json).unwrap();
        assert!(matches!(result, ValidationResult::Valid(_)));
    }
    
    #[test]
    fn test_parse_invalid_json() {
        let parser = JsonParser::new();
        
        let invalid_json = r#"{"node_type": "invalid_type", "title": ""}"#;
        
        let result = parser.parse_and_validate(invalid_json).unwrap();
        assert!(matches!(result, ValidationResult::Invalid(_)));
    }
    
    #[test]
    fn test_extract_from_markdown() {
        let parser = JsonParser::new();
        
        let markdown = r#"Here's the result:
```json
{"node_type": "entity", "title": "Test"}
```
"#;
        
        let extracted = parser.extract_json_from_markdown(markdown);
        assert!(extracted.is_some());
        assert!(extracted.unwrap().contains("node_type"));
    }
    
    #[test]
    fn test_missing_required_field() {
        let parser = JsonParser::new();
        
        let incomplete = r#"{"title": "Test"}"#;
        
        let result = parser.parse_and_validate(incomplete).unwrap();
        match result {
            ValidationResult::Invalid(errors) => {
                assert!(errors.iter().any(|e| e.field_path == "node_type"));
                assert!(errors.iter().any(|e| e.field_path == "content"));
            }
            _ => panic!("Expected validation to fail"),
        }
    }
    
    #[test]
    fn test_confidence_out_of_range() {
        let parser = JsonParser::new();
        
        let invalid = r#"{
            "node_type": "entity",
            "title": "Test",
            "content": "Test",
            "confidence_score": 1.5
        }"#;
        
        let result = parser.parse_and_validate(invalid).unwrap();
        match result {
            ValidationResult::Invalid(errors) => {
                assert!(errors.iter().any(|e| e.field_path == "confidence_score"));
            }
            _ => panic!("Expected validation to fail"),
        }
    }
}
