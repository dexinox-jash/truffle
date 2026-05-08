//! Embedding service for semantic search

use fastembed::{EmbeddingModel, TextEmbedding};
use std::sync::Arc;
use tokio::sync::Mutex;

pub struct EmbeddingService {
    model: Arc<Mutex<TextEmbedding>>,
}

impl EmbeddingService {
    pub async fn new() -> anyhow::Result<Self> {
        // Initialize the embedding model
        let model = TextEmbedding::try_new(
            fastembed::InitOptions::new(EmbeddingModel::BGESmallENV15)
        )?;

        Ok(Self {
            model: Arc::new(Mutex::new(model)),
        })
    }

    /// Create embedding for a single text
    pub async fn embed(&self, text: &str) -> anyhow::Result<Vec<f32>> {
        let model = self.model.lock().await;
        let embeddings = model.embed(vec![text], None)?;
        Ok(embeddings.into_iter().next().unwrap_or_default())
    }

    /// Create embeddings for multiple texts
    pub async fn embed_batch(&self, texts: Vec<&str>) -> anyhow::Result<Vec<Vec<f32>>> {
        let model = self.model.lock().await;
        let embeddings = model.embed(texts, None)?;
        Ok(embeddings)
    }

    /// Calculate cosine similarity between two embeddings
    pub fn cosine_similarity(a: &[f32], b: &[f32]) -> f32 {
        let dot_product: f32 = a.iter().zip(b.iter()).map(|(x, y)| x * y).sum();
        let norm_a: f32 = a.iter().map(|x| x * x).sum::<f32>().sqrt();
        let norm_b: f32 = b.iter().map(|x| x * x).sum::<f32>().sqrt();
        
        if norm_a == 0.0 || norm_b == 0.0 {
            return 0.0;
        }
        
        dot_product / (norm_a * norm_b)
    }
}

#[derive(Debug, Clone)]
pub struct SearchResult {
    pub id: String,
    pub text: String,
    pub score: f32,
}

impl EmbeddingService {
    /// Semantic search over a corpus
    pub async fn search(
        &self,
        query: &str,
        corpus: Vec<(String, String)>, // (id, text)
        top_k: usize,
    ) -> anyhow::Result<Vec<SearchResult>> {
        let query_embedding = self.embed(query).await?;
        
        // Get embeddings for all corpus texts
        let texts: Vec<&str> = corpus.iter().map(|(_, text)| text.as_str()).collect();
        let corpus_embeddings = self.embed_batch(texts).await?;
        
        // Calculate similarities
        let mut results: Vec<SearchResult> = corpus
            .into_iter()
            .zip(corpus_embeddings.iter())
            .map(|((id, text), embedding)| {
                let score = Self::cosine_similarity(&query_embedding, embedding);
                SearchResult { id, text, score }
            })
            .collect();
        
        // Sort by score descending
        results.sort_by(|a, b| b.score.partial_cmp(&a.score).unwrap());
        
        // Return top k
        Ok(results.into_iter().take(top_k).collect())
    }
}
