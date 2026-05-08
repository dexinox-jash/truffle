//! Named Entity Recognition (NER) Engine
//!
//! Rule-based NER for extracting people, organizations, locations, dates, and more.

use crate::pipeline::extraction::Token;
use crate::models::{Entity, EntityType, EntityMetadata};
use crate::pipeline::PipelineError;
use regex::Regex;

/// NER engine configuration
#[derive(Debug, Clone)]
pub struct NerConfig {
    /// Enable person extraction
    pub extract_persons: bool,
    /// Enable organization extraction
    pub extract_organizations: bool,
    /// Enable location extraction
    pub extract_locations: bool,
    /// Enable date extraction
    pub extract_dates: bool,
    /// Enable metric extraction (money, percentages)
    pub extract_metrics: bool,
    /// Confidence threshold
    pub confidence_threshold: f32,
}

impl Default for NerConfig {
    fn default() -> Self {
        Self {
            extract_persons: true,
            extract_organizations: true,
            extract_locations: true,
            extract_dates: true,
            extract_metrics: true,
            confidence_threshold: 0.7,
        }
    }
}

/// NER Engine
pub struct NerEngine {
    config: NerConfig,
    person_patterns: Vec<Regex>,
    org_patterns: Vec<Regex>,
    date_patterns: Vec<Regex>,
    metric_patterns: Vec<Regex>,
}

/// Raw extracted entity
#[derive(Debug, Clone)]
pub struct RawEntity {
    /// Entity text
    pub text: String,
    /// Entity type
    pub entity_type: EntityType,
    /// Confidence score
    pub confidence: f32,
    /// Start position in token stream
    pub start_token: usize,
    /// End position in token stream
    pub end_token: usize,
}

/// Extracted entity with embeddings
#[derive(Debug, Clone)]
pub struct ExtractedEntity {
    /// Raw entity
    pub raw: RawEntity,
    /// Vector embedding
    pub embedding: Option<Vec<f32>>,
}

impl NerEngine {
    /// Create a new NER engine
    pub fn new(config: &NerConfig) -> Result<Self, PipelineError> {
        // Compile patterns
        let person_patterns = vec![
            Regex::new(r"\b[A-Z][a-z]+\s+[A-Z][a-z]+\b")
                .map_err(|e| PipelineError::Configuration(format!("Invalid person pattern: {}", e)))?, // John Smith
        ];
        
        let org_patterns = vec![
            Regex::new(r"\b[A-Z][a-z]*\s+(?:Corp|Inc|Ltd|LLC|Company)\b")
                .map_err(|e| PipelineError::Configuration(format!("Invalid org pattern: {}", e)))?,
            Regex::new(r"\b[A-Z][a-z]*\s+(?:University|College|Institute)\b")
                .map_err(|e| PipelineError::Configuration(format!("Invalid org pattern: {}", e)))?,
        ];
        
        let date_patterns = vec![
            Regex::new(r"\b(?:January|February|March|April|May|June|July|August|September|October|November|December)\s+\d{1,2}(?:st|nd|rd|th)?\b")
                .map_err(|e| PipelineError::Configuration(format!("Invalid date pattern: {}", e)))?,
            Regex::new(r"\b\d{1,2}/\d{1,2}/\d{2,4}\b")
                .map_err(|e| PipelineError::Configuration(format!("Invalid date pattern: {}", e)))?,
            Regex::new(r"\bQ[1-4]\s+\d{4}\b")
                .map_err(|e| PipelineError::Configuration(format!("Invalid date pattern: {}", e)))?,
        ];
        
        let metric_patterns = vec![
            Regex::new(r"\$\d+(?:\.\d+)?(?:M|K|B)?\b")
                .map_err(|e| PipelineError::Configuration(format!("Invalid metric pattern: {}", e)))?, // $5M, $100K
            Regex::new(r"\b\d+(?:\.\d+)?%\b")
                .map_err(|e| PipelineError::Configuration(format!("Invalid metric pattern: {}", e)))?, // 50%
        ];
        
        Ok(Self {
            config: config.clone(),
            person_patterns,
            org_patterns,
            date_patterns,
            metric_patterns,
        })
    }
    
    /// Extract entities from tokens
    pub async fn extract(&self, tokens: &[Token]) -> Result<Vec<ExtractedEntity>, PipelineError> {
        let mut entities = Vec::new();
        
        // Reconstruct text from tokens for pattern matching
        let text = tokens.iter()
            .map(|t| t.original.as_str())
            .collect::<Vec<_>>()
            .join(" ");
        
        // Extract persons
        if self.config.extract_persons {
            for pattern in &self.person_patterns {
                for mat in pattern.find_iter(&text) {
                    if let Some(entity) = self.create_entity_from_match(mat, EntityType::Person, tokens) {
                        entities.push(ExtractedEntity {
                            raw: entity,
                            embedding: None,
                        });
                    }
                }
            }
        }
        
        // Extract organizations
        if self.config.extract_organizations {
            for pattern in &self.org_patterns {
                for mat in pattern.find_iter(&text) {
                    if let Some(entity) = self.create_entity_from_match(mat, EntityType::Organization, tokens) {
                        entities.push(ExtractedEntity {
                            raw: entity,
                            embedding: None,
                        });
                    }
                }
            }
        }
        
        // Extract dates
        if self.config.extract_dates {
            for pattern in &self.date_patterns {
                for mat in pattern.find_iter(&text) {
                    if let Some(entity) = self.create_entity_from_match(mat, EntityType::Event, tokens) {
                        entities.push(ExtractedEntity {
                            raw: entity,
                            embedding: None,
                        });
                    }
                }
            }
        }
        
        // Deduplicate entities
        entities = self.deduplicate(entities);
        
        Ok(entities)
    }
    
    /// Create entity from regex match
    fn create_entity_from_match(
        &self,
        mat: regex::Match,
        entity_type: EntityType,
        tokens: &[Token],
    ) -> Option<RawEntity> {
        let text = mat.as_str().to_string();
        
        // Find corresponding tokens
        let start_token = self.find_token_at_position(tokens, mat.start())?;
        let end_token = self.find_token_at_position(tokens, mat.end()).unwrap_or(start_token);
        
        Some(RawEntity {
            text,
            entity_type,
            confidence: 0.8,
            start_token,
            end_token,
        })
    }
    
    /// Find token index at character position
    fn find_token_at_position(&self, tokens: &[Token], pos: usize) -> Option<usize> {
        let mut current_pos = 0;
        
        for (i, token) in tokens.iter().enumerate() {
            if current_pos >= pos {
                return Some(i);
            }
            current_pos += token.original.len() + 1; // +1 for space
        }
        
        None
    }
    
    /// Deduplicate entities by text
    fn deduplicate(&self, entities: Vec<ExtractedEntity>) -> Vec<ExtractedEntity> {
        let mut seen = std::collections::HashSet::new();
        entities.into_iter()
            .filter(|e| seen.insert(e.raw.text.clone()))
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_ner_engine_creation() {
        let config = NerConfig::default();
        let engine = NerEngine::new(&config).expect("Failed to create NER engine");
        assert!(engine.config.extract_persons);
    }
}
