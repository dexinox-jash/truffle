//! Context Enrichment
//!
//! Enriches resolved entities with contextual information.

use crate::pipeline::extraction::resolution::ResolvedEntity;
use crate::models::{Entity, Meeting, EntityMetadata};
use crate::Result;

/// Enrichment configuration
#[derive(Debug, Clone)]
pub struct EnrichmentConfig {
    /// Extract temporal context
    pub extract_temporal: bool,
    /// Extract spatial context
    pub extract_spatial: bool,
    /// Extract sentiment
    pub extract_sentiment: bool,
}

impl Default for EnrichmentConfig {
    fn default() -> Self {
        Self {
            extract_temporal: true,
            extract_spatial: false,
            extract_sentiment: false,
        }
    }
}

/// Context analyzer
pub struct ContextAnalyzer {
    config: EnrichmentConfig,
}

/// Enriched entity with context
#[derive(Debug, Clone)]
pub struct EnrichedEntity {
    /// The entity
    pub entity: Entity,
    /// Chunk index in transcript
    pub chunk_idx: usize,
    /// Contextual metadata
    pub context: ContextMetadata,
}

/// Context metadata
#[derive(Debug, Clone, Default)]
pub struct ContextMetadata {
    /// Temporal context (when mentioned)
    pub temporal: Option<TemporalContext>,
    /// Spatial context (where mentioned)
    pub spatial: Option<String>,
    /// Sentiment (-1.0 to 1.0)
    pub sentiment: Option<f32>,
}

/// Temporal context
#[derive(Debug, Clone)]
pub struct TemporalContext {
    /// Mentioned date/time
    pub mentioned_at: chrono::DateTime<chrono::Utc>,
    /// Temporal expression in text
    pub temporal_expression: Option<String>,
}

impl ContextAnalyzer {
    /// Create a new context analyzer
    pub fn new(config: &EnrichmentConfig) -> Result<Self> {
        Ok(Self {
            config: config.clone(),
        })
    }
    
    /// Enrich resolved entities with context
    pub async fn enrich(
        &self,
        resolved: Vec<ResolvedEntity>,
        meeting: &Meeting,
        chunk_idx: usize,
        chunk_text: &str,
    ) -> Result<Vec<EnrichedEntity>> {
        let mut enriched = Vec::new();
        
        for resolved_entity in resolved {
            let entity = match resolved_entity {
                ResolvedEntity::Linked { raw, canonical_id, confidence } => {
                    // Fetch existing entity from database
                    // For now, create a placeholder
                    Entity::new(&raw.text, raw.entity_type, Some(meeting.id))
                        .with_confidence(confidence)
                }
                ResolvedEntity::New(raw) => {
                    Entity::new(&raw.text, raw.entity_type, Some(meeting.id))
                        .with_confidence(raw.confidence)
                }
            };
            
            // Build context metadata
            let mut context = ContextMetadata::default();
            
            if self.config.extract_temporal {
                context.temporal = Some(TemporalContext {
                    mentioned_at: meeting.started_at.unwrap_or_else(chrono::Utc::now),
                    temporal_expression: self.extract_temporal_expression(chunk_text),
                });
            }
            
            enriched.push(EnrichedEntity {
                entity,
                chunk_idx,
                context,
            });
        }
        
        Ok(enriched)
    }
    
    /// Extract temporal expression from context
    fn extract_temporal_expression(&self, text: &str) -> Option<String> {
        // Simple temporal patterns
        let patterns = [
            "yesterday", "today", "tomorrow",
            "last week", "this week", "next week",
            "last month", "this month", "next month",
            "Q1", "Q2", "Q3", "Q4",
        ];
        
        let text_lower = text.to_lowercase();
        for pattern in &patterns {
            if text_lower.contains(pattern) {
                return Some(pattern.to_string());
            }
        }
        
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_analyzer_creation() {
        let config = EnrichmentConfig::default();
        let analyzer = ContextAnalyzer::new(&config).unwrap();
        assert!(analyzer.config.extract_temporal);
    }
    
    #[test]
    fn test_temporal_extraction() {
        let config = EnrichmentConfig::default();
        let analyzer = ContextAnalyzer::new(&config).unwrap();
        
        assert_eq!(
            analyzer.extract_temporal_expression("We met yesterday"),
            Some("yesterday".to_string())
        );
        
        assert_eq!(
            analyzer.extract_temporal_expression("Launch is next week"),
            Some("next week".to_string())
        );
    }
}
