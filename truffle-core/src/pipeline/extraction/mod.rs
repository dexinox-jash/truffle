//! Entity Extraction Pipeline
//!
//! Extracts entities and relationships from meeting transcripts.
//! 5-stage pipeline: Preprocessing → Tokenization → NER → Resolution → Enrichment

pub mod ner;
pub mod resolution;
pub mod enrichment;

pub use ner::{NerEngine, NerConfig, RawEntity, ExtractedEntity};
pub use resolution::{EntityResolver, ResolutionConfig, ResolvedEntity};
pub use enrichment::{ContextAnalyzer, EnrichmentConfig, EnrichedEntity};

use crate::models::{Entity, Relationship, Meeting, EntityType};
use crate::Result;

/// Extraction pipeline configuration
#[derive(Debug, Clone)]
pub struct ExtractionConfig {
    /// NER engine configuration
    pub ner: NerConfig,
    /// Entity resolution configuration
    pub resolution: ResolutionConfig,
    /// Context enrichment configuration
    pub enrichment: EnrichmentConfig,
    /// Maximum transcript chunk size (characters)
    pub chunk_size: usize,
    /// Chunk overlap (characters)
    pub chunk_overlap: usize,
}

impl Default for ExtractionConfig {
    fn default() -> Self {
        Self {
            ner: NerConfig::default(),
            resolution: ResolutionConfig::default(),
            enrichment: EnrichmentConfig::default(),
            chunk_size: 2000,
            chunk_overlap: 200,
        }
    }
}

/// Extraction pipeline
pub struct ExtractionPipeline {
    config: ExtractionConfig,
    ner_engine: NerEngine,
    entity_resolver: EntityResolver,
    context_analyzer: ContextAnalyzer,
}

impl ExtractionPipeline {
    /// Create a new extraction pipeline
    pub fn new(config: ExtractionConfig) -> Result<Self> {
        let ner_engine = NerEngine::new(&config.ner)?;
        let entity_resolver = EntityResolver::new(&config.resolution)?;
        let context_analyzer = ContextAnalyzer::new(&config.enrichment)?;
        
        Ok(Self {
            config,
            ner_engine,
            entity_resolver,
            context_analyzer,
        })
    }
    
    /// Process a meeting transcript and extract entities/relationships
    pub async fn process(&self, meeting: &Meeting) -> Result<ExtractionResult> {
        // Stage 1: Preprocessing - split transcript into chunks
        let chunks = self.preprocess(&meeting.transcript)?;
        
        let mut all_entities: Vec<EnrichedEntity> = Vec::new();
        let mut all_relationships: Vec<Relationship> = Vec::new();
        
        // Process each chunk
        for (chunk_idx, chunk) in chunks.iter().enumerate() {
            // Stage 2: Tokenization
            let tokens = self.tokenize(chunk)?;
            
            // Stage 3: Named Entity Recognition
            let raw_entities = self.ner_engine.extract(&tokens).await?;
            
            // Stage 4: Entity Resolution
            let resolved = self.entity_resolver.resolve(raw_entities).await?;
            
            // Stage 5: Context Enrichment
            let enriched = self.context_analyzer.enrich(
                resolved, 
                meeting, 
                chunk_idx,
                chunk
            ).await?;
            
            all_entities.extend(enriched);
        }
        
        // Extract relationships between entities
        all_relationships = self.extract_relationships(&all_entities, &chunks)?;
        
        // Extract decisions and action items
        let decisions = self.extract_decisions(&all_entities, meeting)?;
        let action_items = self.extract_action_items(&all_entities, meeting)?;
        
        Ok(ExtractionResult {
            entities: all_entities,
            relationships: all_relationships,
            decisions,
            action_items,
        })
    }
    
    /// Preprocess transcript into chunks
    fn preprocess(&self, transcript: &str) -> Result<Vec<String>> {
        if transcript.is_empty() {
            return Ok(Vec::new());
        }
        
        // Normalize whitespace
        let normalized = transcript
            .replace('\t', " ")
            .replace("\r\n", "\n")
            .replace('\r', "\n");
        
        // Split into chunks with overlap
        let mut chunks = Vec::new();
        let mut start = 0;
        
        while start < normalized.len() {
            let end = (start + self.config.chunk_size).min(normalized.len());
            let chunk = &normalized[start..end];
            chunks.push(chunk.to_string());
            
            // Move start with overlap
            start = end.saturating_sub(self.config.chunk_overlap);
            
            // Prevent infinite loop
            if start >= normalized.len() {
                break;
            }
        }
        
        Ok(chunks)
    }
    
    /// Tokenize text into sentences and words
    fn tokenize(&self, text: &str) -> Result<Vec<Token>> {
        let mut tokens = Vec::new();
        
        // Simple sentence tokenization
        let sentences: Vec<&str> = text.split(['.', '!', '?', '\n']).collect();
        
        for (sent_idx, sentence) in sentences.iter().enumerate() {
            let sentence = sentence.trim();
            if sentence.is_empty() {
                continue;
            }
            
            // Word tokenization
            let words: Vec<&str> = sentence.split_whitespace().collect();
            
            for (word_idx, word) in words.iter().enumerate() {
                // Clean word (remove punctuation)
                let clean_word = word
                    .trim_matches(|c: char| !c.is_alphanumeric())
                    .to_lowercase();
                
                if !clean_word.is_empty() {
                    tokens.push(Token {
                        text: clean_word,
                        original: word.to_string(),
                        sentence_idx: sent_idx,
                        word_idx,
                        is_capitalized: word.chars().next().map(|c| c.is_uppercase()).unwrap_or(false),
                    });
                }
            }
        }
        
        Ok(tokens)
    }
    
    /// Extract relationships between entities
    fn extract_relationships(
        &self,
        entities: &[EnrichedEntity],
        chunks: &[String],
    ) -> Result<Vec<Relationship>> {
        let mut relationships = Vec::new();
        
        // Simple relationship extraction based on proximity and patterns
        for (i, entity1) in entities.iter().enumerate() {
            for entity2 in entities.iter().skip(i + 1) {
                // Check if entities appear in same chunk
                if entity1.chunk_idx == entity2.chunk_idx {
                    // Check for relationship patterns
                    if let Some(rel_type) = self.detect_relationship_pattern(
                        entity1, entity2, &chunks[entity1.chunk_idx]
                    ) {
                        relationships.push(Relationship {
                            id: uuid::Uuid::new_v4(),
                            source_id: entity1.entity.id,
                            target_id: entity2.entity.id,
                            relation_type: rel_type.to_string(),
                            valid_from: None,
                            valid_until: None,
                            confidence: 0.7,
                            metadata: None,
                            source_meeting_id: entity1.entity.source_meeting_id,
                            extraction_method: "rule_based".to_string(),
                            created_at: chrono::Utc::now(),
                            updated_at: chrono::Utc::now(),
                        });
                    }
                }
            }
        }
        
        Ok(relationships)
    }
    
    /// Detect relationship type from context
    fn detect_relationship_pattern(
        &self,
        entity1: &EnrichedEntity,
        entity2: &EnrichedEntity,
        context: &str,
    ) -> Option<&'static str> {
        let context_lower = context.to_lowercase();
        
        // Person + Organization patterns
        if entity1.entity.entity_type == EntityType::Person 
            && entity2.entity.entity_type == EntityType::Organization {
            let name_lower = entity1.entity.name.to_lowercase();
            if context_lower.contains(&format!("{} works at", name_lower))
                || context_lower.contains(&format!("{} is at", name_lower))
                || context_lower.contains(&format!("{} from", name_lower)) {
                return Some("works_at");
            }
            if context_lower.contains(&format!("{} manages", name_lower))
                || context_lower.contains(&format!("{} leads", name_lower)) {
                return Some("manages");
            }
        }
        
        // Person + Person collaboration
        if entity1.entity.entity_type == EntityType::Person 
            && entity2.entity.entity_type == EntityType::Person {
            if context_lower.contains("working with")
                || context_lower.contains("collaborating")
                || context_lower.contains("together") {
                return Some("collaborated_with");
            }
            if context_lower.contains("reports to")
                || context_lower.contains("manages")
                || context_lower.contains("supervises") {
                return Some("reports_to");
            }
        }
        
        // Organization + Location
        if entity1.entity.entity_type == EntityType::Organization 
            && entity2.entity.entity_type == EntityType::Location {
            if context_lower.contains("located in")
                || context_lower.contains("based in")
                || context_lower.contains("headquartered") {
                return Some("located_in");
            }
        }
        
        None
    }
    
    /// Extract decisions from meeting
    fn extract_decisions(
        &self,
        entities: &[EnrichedEntity],
        meeting: &Meeting,
    ) -> Result<Vec<crate::models::Decision>> {
        use crate::models::Decision;
        
        let mut decisions = Vec::new();
        
        // Pattern matching for decision statements
        let decision_patterns = [
            "we decide to",
            "we decided to",
            "the decision is",
            "let's go with",
            "we'll go with",
            "agreed to",
            "consensus is",
        ];
        
        if let Some(transcript) = &meeting.transcript {
            for pattern in &decision_patterns {
                if let Some(idx) = transcript.to_lowercase().find(pattern) {
                    let start = idx + pattern.len();
                    let end = transcript[start..].find('.').map(|i| start + i).unwrap_or(transcript.len());
                    let decision_text = transcript[start..end].trim().to_string();
                    
                    if !decision_text.is_empty() && decision_text.len() > 10 {
                        decisions.push(Decision {
                            id: uuid::Uuid::new_v4(),
                            decision_text,
                            summary: None,
                            version: 1,
                            previous_version_id: None,
                            decision_chain_id: uuid::Uuid::new_v4(),
                            status: "proposed".to_string(),
                            proposed_by_id: None,
                            decided_by_ids: None,
                            dissenting_ids: None,
                            decided_in_meeting_id: Some(meeting.id),
                            proposed_at: None,
                            decided_at: Some(meeting.started_at.unwrap_or_else(chrono::Utc::now)),
                            implemented_at: None,
                            impact_score: None,
                            dependent_decision_ids: None,
                            created_at: chrono::Utc::now(),
                            updated_at: chrono::Utc::now(),
                        });
                    }
                }
            }
        }
        
        Ok(decisions)
    }
    
    /// Extract action items from meeting
    fn extract_action_items(
        &self,
        entities: &[EnrichedEntity],
        meeting: &Meeting,
    ) -> Result<Vec<crate::models::ActionItem>> {
        use crate::models::ActionItem;
        
        let mut action_items = Vec::new();
        
        // Pattern matching for action items
        let action_patterns = [
            ("will ", "medium"),
            ("going to ", "medium"),
            ("committed to ", "high"),
            ("promise to ", "high"),
            ("need to ", "medium"),
            ("should ", "low"),
        ];
        
        if let Some(transcript) = &meeting.transcript {
            for person in entities.iter().filter(|e| e.entity.entity_type == EntityType::Person) {
                let person_name_lower = person.entity.name.to_lowercase();
                
                for (pattern, priority) in &action_patterns {
                    let search_pattern = format!("{} {}", person_name_lower, pattern);
                    
                    if let Some(idx) = transcript.to_lowercase().find(&search_pattern) {
                        let start = idx + search_pattern.len();
                        let end = transcript[start..].find('.')
                            .map(|i| start + i)
                            .or_else(|| transcript[start..].find('\n').map(|i| start + i))
                            .unwrap_or(transcript.len());
                        
                        let description = transcript[start..end].trim().to_string();
                        
                        if !description.is_empty() && description.len() > 5 {
                            action_items.push(ActionItem {
                                id: uuid::Uuid::new_v4(),
                                description,
                                assignee_id: Some(person.entity.id),
                                creator_id: None,
                                source_meeting_id: Some(meeting.id),
                                source_decision_id: None,
                                status: "open".to_string(),
                                deadline: None,
                                completed_at: None,
                                priority: priority.to_string(),
                                completion_verified_by_id: None,
                                verification_notes: None,
                                reminder_sent: false,
                                reminder_sent_at: None,
                                created_at: chrono::Utc::now(),
                                updated_at: chrono::Utc::now(),
                            });
                        }
                    }
                }
            }
        }
        
        Ok(action_items)
    }
}

/// A token in the text
#[derive(Debug, Clone, PartialEq)]
pub struct Token {
    /// Normalized text (lowercase, no punctuation)
    pub text: String,
    /// Original text (preserving case/punctuation)
    pub original: String,
    /// Sentence index
    pub sentence_idx: usize,
    /// Word index within sentence
    pub word_idx: usize,
    /// Whether original was capitalized
    pub is_capitalized: bool,
}

/// Result of extraction pipeline
#[derive(Debug, Clone)]
pub struct ExtractionResult {
    /// Extracted entities
    pub entities: Vec<EnrichedEntity>,
    /// Extracted relationships
    pub relationships: Vec<Relationship>,
    /// Extracted decisions
    pub decisions: Vec<crate::models::Decision>,
    /// Extracted action items
    pub action_items: Vec<crate::models::ActionItem>,
}

/// Import models needed for extraction
use crate::models::Relationship;

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_preprocess_chunks() {
        let config = ExtractionConfig::default();
        let pipeline = ExtractionPipeline::new(config).unwrap();
        
        let text = "This is a test. ".repeat(100);
        let chunks = pipeline.preprocess(&text).unwrap();
        
        assert!(!chunks.is_empty());
    }
    
    #[test]
    fn test_tokenize() {
        let config = ExtractionConfig::default();
        let pipeline = ExtractionPipeline::new(config).unwrap();
        
        let text = "Hello world. This is a test.";
        let tokens = pipeline.tokenize(text).unwrap();
        
        assert!(!tokens.is_empty());
        assert!(tokens.iter().any(|t| t.text == "hello"));
        assert!(tokens.iter().any(|t| t.text == "world"));
    }
    
    #[test]
    fn test_extract_decisions() {
        let config = ExtractionConfig::default();
        let pipeline = ExtractionPipeline::new(config).unwrap();
        
        let meeting = Meeting {
            id: uuid::Uuid::new_v4(),
            title: "Test Meeting".to_string(),
            transcript: Some("We decide to use PostgreSQL. The decision is to deploy next week.".to_string()),
            started_at: Some(chrono::Utc::now()),
            ..Default::default()
        };
        
        let decisions = pipeline.extract_decisions(&[], &meeting).unwrap();
        assert!(!decisions.is_empty());
    }
}
