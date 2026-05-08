//! Schema Matcher
//! 
//! Matches content against schema triggers to determine compilation rules.

use regex::Regex;
use tracing::{debug, info};

use crate::schema::{CompilationRule, TriggerCondition, EntityResolution};
use crate::{AiError, AiResult};

/// Result of trigger matching
#[derive(Debug, Clone)]
pub struct TriggerMatch {
    /// Matched rule
    pub rule: CompilationRule,
    /// Confidence score (0-1)
    pub confidence: f32,
    /// Matched keywords/concepts
    pub matched_keywords: Vec<String>,
    /// Match timestamp
    pub matched_at: chrono::DateTime<chrono::Utc>,
}

/// Matches content against schema triggers
pub struct SchemaMatcher {
    /// Pre-compiled regex patterns
    patterns: Vec<(String, Regex)>,
    /// Entity resolution method
    entity_resolution: EntityResolution,
}

impl SchemaMatcher {
    /// Create a new schema matcher
    pub fn new(entity_resolution: EntityResolution) -> AiResult<Self> {
        Ok(Self {
            patterns: Vec::new(),
            entity_resolution,
        })
    }
    
    /// Add a pattern for matching
    pub fn add_pattern(&mut self, name: &str, pattern: &str) -> AiResult<()> {
        let regex = Regex::new(pattern)
            .map_err(|e| AiError::Other(format!("Invalid regex pattern: {}", e)))?;
        
        self.patterns.push((name.to_string(), regex));
        debug!("Added pattern '{}' for matching", name);
        
        Ok(())
    }
    
    /// Match text content against all rules
    pub fn match_text(&self, text: &str, rules: &[CompilationRule]) -> Vec<TriggerMatch> {
        let mut matches = Vec::new();
        
        for rule in rules {
            if let Some(confidence) = self.evaluate_trigger(&rule.trigger, text) {
                let matched_keywords = self.extract_keywords(&rule.trigger, text);
                
                matches.push(TriggerMatch {
                    rule: rule.clone(),
                    confidence,
                    matched_keywords,
                    matched_at: chrono::Utc::now(),
                });
            }
        }
        
        // Sort by confidence (highest first)
        matches.sort_by(|a, b| b.confidence.partial_cmp(&a.confidence).unwrap());
        
        matches
    }
    
    /// Evaluate a trigger condition against text
    fn evaluate_trigger(&self, trigger: &TriggerCondition, text: &str) -> Option<f32> {
        let text_lower = text.to_lowercase();
        
        match trigger {
            TriggerCondition::ConceptDetected { concept } => {
                let concept_lower = concept.to_lowercase();
                
                // Direct match
                if text_lower.contains(&concept_lower) {
                    return Some(0.9);
                }
                
                // Check against patterns
                for (name, regex) in &self.patterns {
                    if name.to_lowercase() == concept_lower {
                        if regex.is_match(text) {
                            return Some(0.85);
                        }
                    }
                }
                
                None
            }
            
            TriggerCondition::EntityType { entity_type } => {
                let entity_lower = entity_type.to_lowercase();
                
                if text_lower.contains(&entity_lower) {
                    Some(0.8)
                } else {
                    None
                }
            }
            
            TriggerCondition::Pattern { pattern } => {
                // Try to match as regex
                if let Ok(regex) = Regex::new(pattern) {
                    if regex.is_match(text) {
                        return Some(0.95);
                    }
                }
                
                // Fallback to substring match
                if text_lower.contains(&pattern.to_lowercase()) {
                    Some(0.7)
                } else {
                    None
                }
            }
            
            TriggerCondition::Context { context } => {
                if text_lower.contains(&context.to_lowercase()) {
                    Some(0.75)
                } else {
                    None
                }
            }
            
            TriggerCondition::And(conditions) => {
                let mut min_confidence = 1.0f32;
                
                for condition in conditions {
                    match self.evaluate_trigger(condition, text) {
                        Some(conf) => {
                            min_confidence = min_confidence.min(conf);
                        }
                        None => return None,
                    }
                }
                
                Some(min_confidence)
            }
            
            TriggerCondition::Or(conditions) => {
                let mut max_confidence = 0.0f32;
                
                for condition in conditions {
                    if let Some(conf) = self.evaluate_trigger(condition, text) {
                        max_confidence = max_confidence.max(conf);
                    }
                }
                
                if max_confidence > 0.0 {
                    Some(max_confidence)
                } else {
                    None
                }
            }
        }
    }
    
    /// Extract matched keywords from trigger
    fn extract_keywords(&self, trigger: &TriggerCondition, text: &str) -> Vec<String> {
        let mut keywords = Vec::new();
        let text_lower = text.to_lowercase();
        
        match trigger {
            TriggerCondition::ConceptDetected { concept } => {
                let concept_lower = concept.to_lowercase();
                if text_lower.contains(&concept_lower) {
                    keywords.push(concept.clone());
                }
            }
            TriggerCondition::EntityType { entity_type } => {
                if text_lower.contains(&entity_type.to_lowercase()) {
                    keywords.push(entity_type.clone());
                }
            }
            TriggerCondition::Pattern { pattern } => {
                if let Ok(regex) = Regex::new(pattern) {
                    for cap in regex.captures_iter(text) {
                        if let Some(m) = cap.get(0) {
                            keywords.push(m.as_str().to_string());
                        }
                    }
                }
            }
            TriggerCondition::Context { context } => {
                if text_lower.contains(&context.to_lowercase()) {
                    keywords.push(context.clone());
                }
            }
            TriggerCondition::And(conditions) | TriggerCondition::Or(conditions) => {
                for condition in conditions {
                    keywords.extend(self.extract_keywords(condition, text));
                }
            }
        }
        
        keywords
    }
    
    /// Match entities using fuzzy matching
    pub fn fuzzy_match(&self, a: &str, b: &str) -> f32 {
        match self.entity_resolution {
            EntityResolution::Exact => {
                if a == b { 1.0 } else { 0.0 }
            }
            EntityResolution::FuzzyMatch { max_distance } => {
                let distance = levenshtein_distance(a, b);
                if distance <= max_distance {
                    1.0 - (distance as f32 / max_distance as f32)
                } else {
                    0.0
                }
            }
            EntityResolution::Phonetic => {
                // Simplified phonetic matching
                let a_code = soundex(a);
                let b_code = soundex(b);
                if a_code == b_code { 0.9 } else { 0.0 }
            }
        }
    }
    
    /// Find best matching rule
    pub fn find_best_match(&self, text: &str, rules: &[CompilationRule]) -> Option<TriggerMatch> {
        let matches = self.match_text(text, rules);
        matches.into_iter().next()
    }
    
    /// Batch match multiple texts
    pub fn batch_match(&self, texts: &[String], rules: &[CompilationRule]) -> Vec<Vec<TriggerMatch>> {
        texts.iter()
            .map(|text| self.match_text(text, rules))
            .collect()
    }
}

/// Calculate Levenshtein distance between two strings
fn levenshtein_distance(a: &str, b: &str) -> usize {
    let a_len = a.chars().count();
    let b_len = b.chars().count();
    
    if a_len == 0 { return b_len; }
    if b_len == 0 { return a_len; }
    
    let mut matrix = vec![vec![0; b_len + 1]; a_len + 1];
    
    for i in 0..=a_len {
        matrix[i][0] = i;
    }
    
    for j in 0..=b_len {
        matrix[0][j] = j;
    }
    
    for (i, a_char) in a.chars().enumerate() {
        for (j, b_char) in b.chars().enumerate() {
            let cost = if a_char == b_char { 0 } else { 1 };
            
            matrix[i + 1][j + 1] = (
                matrix[i][j + 1] + 1,
                matrix[i + 1][j] + 1,
                matrix[i][j] + cost,
            ).min();
        }
    }
    
    matrix[a_len][b_len]
}

/// Simple Soundex implementation for phonetic matching
fn soundex(s: &str) -> String {
    if s.is_empty() {
        return String::new();
    }
    
    let s = s.to_uppercase();
    let mut result = String::with_capacity(4);
    
    // First character
    if let Some(first) = s.chars().next() {
        result.push(first);
    }
    
    // Soundex codes
    let mut prev_code = '0';
    for c in s.chars().skip(1) {
        let code = match c {
            'B' | 'F' | 'P' | 'V' => '1',
            'C' | 'G' | 'J' | 'K' | 'Q' | 'S' | 'X' | 'Z' => '2',
            'D' | 'T' => '3',
            'L' => '4',
            'M' | 'N' => '5',
            'R' => '6',
            _ => '0',
        };
        
        if code != '0' && code != prev_code {
            result.push(code);
            if result.len() == 4 {
                break;
            }
        }
        
        prev_code = code;
    }
    
    // Pad with zeros
    while result.len() < 4 {
        result.push('0');
    }
    
    result
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_levenshtein_distance() {
        assert_eq!(levenshtein_distance("kitten", "sitting"), 3);
        assert_eq!(levenshtein_distance("", "abc"), 3);
        assert_eq!(levenshtein_distance("abc", "abc"), 0);
    }
    
    #[test]
    fn test_soundex() {
        assert_eq!(soundex("Robert"), "R163");
        assert_eq!(soundex("Rupert"), "R163");
    }
    
    #[test]
    fn test_fuzzy_match() {
        let matcher = SchemaMatcher::new(EntityResolution::FuzzyMatch { max_distance: 2 }).unwrap();
        
        assert!(matcher.fuzzy_match("hello", "hello") > 0.9);
        assert!(matcher.fuzzy_match("hello", "helo") > 0.5);
        assert_eq!(matcher.fuzzy_match("hello", "world"), 0.0);
    }
    
    #[test]
    fn test_match_concept() {
        let matcher = SchemaMatcher::new(EntityResolution::Exact).unwrap();
        
        let rule = CompilationRule {
            id: "test".to_string(),
            trigger: TriggerCondition::ConceptDetected { concept: "receipt".to_string() },
            action: crate::schema::CompilationAction::CreateEntity { entity_type: "test".to_string() },
            extract: vec![],
            link_to: vec![],
            privacy: crate::schema::PrivacyLevel::Personal,
            temporal_awareness: false,
        };
        
        let matches = matcher.match_text("This is a receipt from the store", &[rule]);
        assert!(!matches.is_empty());
        assert!(matches[0].confidence > 0.8);
    }
    
    #[test]
    fn test_and_condition() {
        let matcher = SchemaMatcher::new(EntityResolution::Exact).unwrap();
        
        let trigger = TriggerCondition::And(vec![
            TriggerCondition::ConceptDetected { concept: "person".to_string() },
            TriggerCondition::Context { context: "contact".to_string() },
        ]);
        
        let confidence = matcher.evaluate_trigger(&trigger, "person contact info");
        assert!(confidence.is_some());
        
        let confidence = matcher.evaluate_trigger(&trigger, "just a person");
        assert!(confidence.is_none());
    }
}
