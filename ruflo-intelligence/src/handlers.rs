//! HTTP handlers

use axum::{
    extract::State,
    response::{Json, Sse},
    response::sse::Event,
};
use futures::Stream;
use std::sync::Arc;
use serde::Deserialize;

use crate::{
    AppState,
    services::{ChatMessage, TranscriptionSegment},
};

/// Transcribe audio file
pub async fn transcribe_audio(
    State(state): State<AppState>,
) -> Json<serde_json::Value> {
    // Implementation would accept multipart audio upload
    Json(serde_json::json!({
        "text": "Transcription placeholder",
        "segments": [],
    }))
}

/// Stream transcription
pub async fn transcribe_stream(
    State(_state): State<AppState>,
) -> Sse<impl Stream<Item = Result<Event, axum::Error>>> {
    use async_stream::stream;
    
    let s = stream! {
        yield Ok(Event::default().data("transcription started"));
        
        // Stream transcript segments
        for i in 0..10 {
            tokio::time::sleep(tokio::time::Duration::from_millis(500)).await;
            yield Ok(Event::default().data(format!("segment {}", i)));
        }
        
        yield Ok(Event::default().data("[DONE]"));
    };
    
    Sse::new(s)
}

/// Create embedding
#[derive(Deserialize)]
pub struct EmbedRequest {
    pub text: String,
}

pub async fn create_embedding(
    State(state): State<AppState>,
    Json(req): Json<EmbedRequest>,
) -> Json<serde_json::Value> {
    match state.embedding.embed(&req.text).await {
        Ok(embedding) => Json(serde_json::json!({
            "embedding": embedding,
            "dimensions": embedding.len(),
        })),
        Err(e) => Json(serde_json::json!({
            "error": e.to_string(),
        })),
    }
}

/// Create embeddings batch
#[derive(Deserialize)]
pub struct EmbedBatchRequest {
    pub texts: Vec<String>,
}

pub async fn create_embeddings_batch(
    State(state): State<AppState>,
    Json(req): Json<EmbedBatchRequest>,
) -> Json<serde_json::Value> {
    let texts: Vec<&str> = req.texts.iter().map(|s| s.as_str()).collect();
    
    match state.embedding.embed_batch(texts).await {
        Ok(embeddings) => Json(serde_json::json!({
            "embeddings": embeddings,
            "count": embeddings.len(),
        })),
        Err(e) => Json(serde_json::json!({
            "error": e.to_string(),
        })),
    }
}

/// Semantic search
#[derive(Deserialize)]
pub struct SearchRequest {
    pub query: String,
    pub corpus: Vec<(String, String)>,
    #[serde(default = "default_top_k")]
    pub top_k: usize,
}

fn default_top_k() -> usize {
    5
}

pub async fn semantic_search(
    State(state): State<AppState>,
    Json(req): Json<SearchRequest>,
) -> Json<serde_json::Value> {
    match state.embedding.search(&req.query, req.corpus, req.top_k).await {
        Ok(results) => {
            let results_json: Vec<_> = results.into_iter().map(|r| {
                serde_json::json!({
                    "id": r.id,
                    "text": r.text,
                    "score": r.score,
                })
            }).collect();
            
            Json(serde_json::json!({
                "results": results_json,
            }))
        }
        Err(e) => Json(serde_json::json!({
            "error": e.to_string(),
        })),
    }
}

/// Analyze text
#[derive(Deserialize)]
pub struct AnalyzeRequest {
    pub text: String,
    #[serde(default)]
    pub analysis_type: String,
}

pub async fn analyze_text(
    State(state): State<AppState>,
    Json(req): Json<AnalyzeRequest>,
) -> Json<serde_json::Value> {
    match state.insight.analyze(&req.text, &req.analysis_type).await {
        Ok(result) => Json(result),
        Err(e) => Json(serde_json::json!({
            "error": e.to_string(),
        })),
    }
}

/// Summarize text
#[derive(Deserialize)]
pub struct SummarizeRequest {
    pub text: String,
}

pub async fn summarize(
    State(state): State<AppState>,
    Json(req): Json<SummarizeRequest>,
) -> Json<serde_json::Value> {
    match state.insight.summarize(&req.text).await {
        Ok(result) => Json(serde_json::json!({
            "summary": result.summary,
            "key_points": result.key_points,
            "topics": result.topics,
        })),
        Err(e) => Json(serde_json::json!({
            "error": e.to_string(),
        })),
    }
}

/// Extract action items
#[derive(Deserialize)]
pub struct ActionItemsRequest {
    pub text: String,
}

pub async fn extract_action_items(
    State(state): State<AppState>,
    Json(req): Json<ActionItemsRequest>,
) -> Json<serde_json::Value> {
    match state.insight.extract_action_items(&req.text).await {
        Ok(items) => Json(serde_json::json!({
            "action_items": items,
        })),
        Err(e) => Json(serde_json::json!({
            "error": e.to_string(),
        })),
    }
}

/// Analyze sentiment
#[derive(Deserialize)]
pub struct SentimentRequest {
    pub text: String,
}

pub async fn analyze_sentiment(
    State(state): State<AppState>,
    Json(req): Json<SentimentRequest>,
) -> Json<serde_json::Value> {
    match state.insight.analyze_sentiment(&req.text).await {
        Ok(result) => Json(serde_json::json!({
            "overall": result.overall,
            "score": result.score,
            "breakdown": result.breakdown,
        })),
        Err(e) => Json(serde_json::json!({
            "error": e.to_string(),
        })),
    }
}

/// Chat completion
#[derive(Deserialize)]
pub struct ChatRequest {
    pub messages: Vec<ChatMessage>,
    #[serde(default)]
    pub stream: bool,
}

pub async fn chat_completion(
    State(state): State<AppState>,
    Json(req): Json<ChatRequest>,
) -> Json<serde_json::Value> {
    match state.insight.chat(req.messages, false).await {
        Ok(response) => Json(serde_json::json!({
            "response": response,
        })),
        Err(e) => Json(serde_json::json!({
            "error": e.to_string(),
        })),
    }
}

/// Chat completion streaming
pub async fn chat_completion_stream(
    State(_state): State<AppState>,
    Json(_req): Json<ChatRequest>,
) -> Sse<impl Stream<Item = Result<Event, axum::Error>>> {
    use async_stream::stream;
    
    let s = stream! {
        yield Ok(Event::default().data("streaming response..."));
        
        for i in 0..5 {
            tokio::time::sleep(tokio::time::Duration::from_millis(200)).await;
            yield Ok(Event::default().data(format!("chunk {}", i)));
        }
        
        yield Ok(Event::default().data("[DONE]"));
    };
    
    Sse::new(s)
}
