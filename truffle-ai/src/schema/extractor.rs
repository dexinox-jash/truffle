//! Field Extractor
//! 
//! Extracts fields from content according to schema field type definitions.

use regex::Regex;
use tracing::{debug, warn};

use crate::schema::{FieldType, DataType, NormalizationRule, ProhibitedExtraction, ProhibitedAction};
use crate::{AiError, AiResult};

/// Extracted field with metadata
#[derive(Debug, Clone)]
pub struct ExtractedField {
    /// Field name
    pub name: String,
    /// Raw value
    pub raw_value: String,
    /// Normalized value
    pub normalized_value: String,
    /// Data type
    pub data_type: DataType,
    /// Whether extraction succeeded
    pub is_valid: bool,
    /// Validation errors (if any)
    pub errors: Vec<String>,
}

/// Field extractor for schema-defined fields
pub struct FieldExtractor {
    /// Field type definitions
    field_types: std::collections::HashMap<String, FieldType>,
    /// Prohibited extractions
    prohibited: Vec<ProhibitedExtraction>,
}

impl FieldExtractor {
    /// Create a new field extractor
    pub fn new(
        field_types: std::collections::HashMap<String, FieldType>,
        prohibited: Vec<ProhibitedExtraction>,
    ) -> Self {
        Self {
            field_types,
            prohibited,
        }
    }
    
    /// Extract a field from text content
    /// 
    /// # Arguments
    /// 
    /// * `field_name` - Name of the field to extract
    /// * `content` - Text content to extract from
    /// 
    /// # Returns
    /// 
    /// Returns the extracted field or None if extraction failed.
    pub fn extract(&self, field_name: &str, content: &str) -> AiResult<Option<ExtractedField>> {
        debug!("Extracting field '{}' from content", field_name);
        
        // Check if field is prohibited
        if let Some(prohibited) = self.prohibited.iter().find(|p| p.field == field_name) {
            warn!("Attempted to extract prohibited field: {}", field_name);
            return self.handle_prohibited(field_name, prohibited);
        }
        
        // Get field type definition
        let field_type = match self.field_types.get(field_name) {
            Some(ft) => ft,
            None => {
                // Try generic extraction for unknown fields
                return self.generic_extract(field_name, content);
            }
        };
        
        // Extract based on data type
        let raw_value = match &field_type.data_type {
            DataType::Phone => self.extract_phone(content),
            DataType::Email => self.extract_email(content),
            DataType::Currency => self.extract_currency(content),
            DataType::Date => self.extract_date(content),
            DataType::DateTime => self.extract_datetime(content),
            DataType::Url => self.extract_url(content),
            DataType::String => self.extract_string(field_name, content),
            DataType::Integer => self.extract_integer(content),
            DataType::Float => self.extract_float(content),
            _ => self.extract_string(field_name, content),
        };
        
        let Some(raw_value) = raw_value else {
            return Ok(None);
        };
        
        // Validate
        let mut errors = Vec::new();
        if let Some(ref validation) = field_type.validation {
            if let Ok(regex) = Regex::new(validation) {
                if !regex.is_match(&raw_value) {
                    errors.push(format!("Value '{}' doesn't match pattern '{}'", raw_value, validation));
                }
            }
        }
        
        // Normalize
        let normalized_value = self.normalize(&raw_value, &field_type.normalization);
        
        Ok(Some(ExtractedField {
            name: field_name.to_string(),
            raw_value,
            normalized_value,
            data_type: field_type.data_type.clone(),
            is_valid: errors.is_empty(),
            errors,
        }))
    }
    
    /// Extract multiple fields
    pub fn extract_multiple(
        &self,
        field_names: &[String],
        content: &str,
    ) -> AiResult<Vec<ExtractedField>> {
        let mut results = Vec::new();
        
        for field_name in field_names {
            if let Some(field) = self.extract(field_name, content)? {
                results.push(field);
            }
        }
        
        Ok(results)
    }
    
    /// Extract phone number
    fn extract_phone(&self, content: &str) -> Option<String> {
        // Common phone number patterns
        let patterns = [
            r"\b\+?1?[-.\s]?\(?\d{3}\)?[-.\s]?\d{3}[-.\s]?\d{4}\b", // US
            r"\b\+?\d{1,3}[-.\s]?\d{1,4}[-.\s]?\d{1,4}[-.\s]?\d{1,4}\b", // International
        ];
        
        for pattern in &patterns {
            if let Ok(regex) = Regex::new(pattern) {
                if let Some(m) = regex.find(content) {
                    return Some(m.as_str().to_string());
                }
            }
        }
        
        None
    }
    
    /// Extract email address
    fn extract_email(&self, content: &str) -> Option<String> {
        let pattern = r"\b[A-Za-z0-9._%+-]+@[A-Za-z0-9.-]+\.[A-Z|a-z]{2,}\b";
        
        if let Ok(regex) = Regex::new(pattern) {
            if let Some(m) = regex.find(content) {
                return Some(m.as_str().to_string());
            }
        }
        
        None
    }
    
    /// Extract currency amount
    fn extract_currency(&self, content: &str) -> Option<String> {
        // Currency patterns: $123.45, 123.45 USD, etc.
        let patterns = [
            r"\$\s*\d{1,3}(?:,\d{3})*\.\d{2}",
            r"\b\d{1,3}(?:,\d{3})*\.\d{2}\s*(?:USD|EUR|GBP|CAD|AUD)\b",
            r"\b(?:total|amount|price)[:\s]*\$?\s*\d{1,3}(?:,\d{3})*\.?\d{0,2}",
        ];
        
        for pattern in &patterns {
            if let Ok(regex) = Regex::new(pattern) {
                if let Some(m) = regex.find(content) {
                    return Some(m.as_str().to_string());
                }
            }
        }
        
        None
    }
    
    /// Extract date
    fn extract_date(&self, content: &str) -> Option<String> {
        // Date patterns
        let patterns = [
            r"\b\d{1,2}[/-]\d{1,2}[/-]\d{2,4}\b",  // MM/DD/YYYY
            r"\b\d{4}[/-]\d{1,2}[/-]\d{1,2}\b",    // YYYY-MM-DD
            r"\b(?:Jan|Feb|Mar|Apr|May|Jun|Jul|Aug|Sep|Oct|Nov|Dec)[a-z]*\.?\s+\d{1,2},?\s+\d{4}\b", // Jan 1, 2024
        ];
        
        for pattern in &patterns {
            if let Ok(regex) = Regex::new(pattern) {
                if let Some(m) = regex.find(content) {
                    return Some(m.as_str().to_string());
                }
            }
        }
        
        None
    }
    
    /// Extract datetime
    fn extract_datetime(&self, content: &str) -> Option<String> {
        // Try to find date + time combination
        let date = self.extract_date(content)?;
        
        // Look for time after the date
        let time_pattern = r"\d{1,2}:\d{2}(?::\d{2})?(?:\s*[AaPp][Mm])?";
        
        if let Ok(regex) = Regex::new(time_pattern) {
            if let Some(m) = regex.find(content) {
                return Some(format!("{} {}", date, m.as_str()));
            }
        }
        
        Some(date)
    }
    
    /// Extract URL
    fn extract_url(&self, content: &str) -> Option<String> {
        let pattern = r"https?://[^\s<>\"{}|\\^`\[\]]+";
        
        if let Ok(regex) = Regex::new(pattern) {
            if let Some(m) = regex.find(content) {
                return Some(m.as_str().to_string());
            }
        }
        
        None
    }
    
    /// Extract string by field name pattern
    fn extract_string(&self, field_name: &str, content: &str) -> Option<String> {
        // Look for "field_name: value" pattern
        let pattern = format!(r"{}[:\s]+([^\n,]+)", regex::escape(field_name));
        
        if let Ok(regex) = Regex::new(&pattern) {
            if let Some(cap) = regex.captures(content) {
                if let Some(m) = cap.get(1) {
                    return Some(m.as_str().trim().to_string());
                }
            }
        }
        
        None
    }
    
    /// Extract integer
    fn extract_integer(&self, content: &str) -> Option<String> {
        let pattern = r"\b\d+\b";
        
        if let Ok(regex) = Regex::new(pattern) {
            if let Some(m) = regex.find(content) {
                return Some(m.as_str().to_string());
            }
        }
        
        None
    }
    
    /// Extract float
    fn extract_float(&self, content: &str) -> Option<String> {
        let pattern = r"\b\d+\.\d+\b";
        
        if let Ok(regex) = Regex::new(pattern) {
            if let Some(m) = regex.find(content) {
                return Some(m.as_str().to_string());
            }
        }
        
        None
    }
    
    /// Generic extraction for unknown fields
    fn generic_extract(&self, field_name: &str, content: &str) -> AiResult<Option<ExtractedField>> {
        if let Some(value) = self.extract_string(field_name, content) {
            Ok(Some(ExtractedField {
                name: field_name.to_string(),
                raw_value: value.clone(),
                normalized_value: value,
                data_type: DataType::String,
                is_valid: true,
                errors: vec![],
            }))
        } else {
            Ok(None)
        }
    }
    
    /// Normalize a value using normalization rules
    fn normalize(&self, value: &str, rules: &[NormalizationRule]) -> String {
        let mut result = value.to_string();
        
        for rule in rules {
            result = match rule {
                NormalizationRule::Lowercase => result.to_lowercase(),
                NormalizationRule::Uppercase => result.to_uppercase(),
                NormalizationRule::Trim => result.trim().to_string(),
                NormalizationRule::RemoveSpecialChars => {
                    result.chars().filter(|c| c.is_alphanumeric() || c.is_whitespace()).collect()
                }
                NormalizationRule::FormatPhone => {
                    // Remove all non-digits
                    let digits: String = result.chars().filter(|c| c.is_digit(10)).collect();
                    // Format as (XXX) XXX-XXXX for US numbers
                    if digits.len() == 10 {
                        format!("({}) {}-{}", 
                            &digits[0..3], 
                            &digits[3..6], 
                            &digits[6..10]
                        )
                    } else {
                        digits
                    }
                }
                NormalizationRule::FormatCurrency { currency } => {
                    // Extract numeric value
                    let numeric: String = result.chars()
                        .filter(|c| c.is_digit(10) || *c == '.')
                        .collect();
                    
                    if let Ok(amount) = numeric.parse::<f64>() {
                        format!("{} {:.2}", currency, amount)
                    } else {
                        result
                    }
                }
            };
        }
        
        result
    }
    
    /// Handle prohibited field extraction
    fn handle_prohibited(
        &self,
        field_name: &str,
        prohibited: &ProhibitedExtraction,
    ) -> AiResult<Option<ExtractedField>> {
        match prohibited.action {
            ProhibitedAction::RedactAndLog => {
                // Log the attempt and return redacted value
                warn!("Prohibited field '{}' extraction redacted", field_name);
                Ok(Some(ExtractedField {
                    name: field_name.to_string(),
                    raw_value: "[REDACTED]".to_string(),
                    normalized_value: "[REDACTED]".to_string(),
                    data_type: DataType::String,
                    is_valid: false,
                    errors: vec!["Field extraction prohibited".to_string()],
                }))
            }
            ProhibitedAction::TokenizeLast4 => {
                // Return last 4 characters only
                Ok(Some(ExtractedField {
                    name: field_name.to_string(),
                    raw_value: "****".to_string(),
                    normalized_value: "****".to_string(),
                    data_type: DataType::String,
                    is_valid: true,
                    errors: vec![],
                }))
            }
            ProhibitedAction::RefuseAndQuarantine => {
                Err(AiError::SafetyViolation {
                    category: format!("Prohibited field: {}", field_name),
                })
            }
        }
    }
    
    /// Add a field type definition
    pub fn add_field_type(&mut self, field_type: FieldType) {
        self.field_types.insert(field_type.name.clone(), field_type);
    }
    
    /// Get statistics
    pub fn get_stats(&self) -> ExtractorStats {
        ExtractorStats {
            field_types_configured: self.field_types.len(),
            prohibited_fields: self.prohibited.len(),
        }
    }
}

/// Extractor statistics
#[derive(Debug, Clone)]
pub struct ExtractorStats {
    /// Number of field types configured
    pub field_types_configured: usize,
    /// Number of prohibited fields
    pub prohibited_fields: usize,
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_extract_phone() {
        let field_types = std::collections::HashMap::new();
        let extractor = FieldExtractor::new(field_types, vec![]);
        
        let content = "Call me at (555) 123-4567";
        let phone = extractor.extract_phone(content);
        assert!(phone.is_some());
        assert!(phone.unwrap().contains("555"));
    }
    
    #[test]
    fn test_extract_email() {
        let field_types = std::collections::HashMap::new();
        let extractor = FieldExtractor::new(field_types, vec![]);
        
        let content = "Contact: john.doe@example.com";
        let email = extractor.extract_email(content);
        assert_eq!(email, Some("john.doe@example.com".to_string()));
    }
    
    #[test]
    fn test_extract_currency() {
        let field_types = std::collections::HashMap::new();
        let extractor = FieldExtractor::new(field_types, vec![]);
        
        let content = "Total: $45.67";
        let currency = extractor.extract_currency(content);
        assert!(currency.is_some());
    }
    
    #[test]
    fn test_normalize_phone() {
        let field_types = std::collections::HashMap::new();
        let extractor = FieldExtractor::new(field_types, vec![]);
        
        let normalized = extractor.normalize(
            "555-123-4567",
            &[NormalizationRule::FormatPhone]
        );
        assert_eq!(normalized, "(555) 123-4567");
    }
    
    #[test]
    fn test_prohibited_field() {
        let field_types = std::collections::HashMap::new();
        let prohibited = vec![
            ProhibitedExtraction {
                field: "password".to_string(),
                action: ProhibitedAction::RedactAndLog,
            }
        ];
        
        let extractor = FieldExtractor::new(field_types, prohibited);
        
        let result = extractor.extract("password", "password: secret123");
        assert!(result.is_ok());
        
        let field = result.unwrap().unwrap();
        assert_eq!(field.normalized_value, "[REDACTED]");
    }
}
