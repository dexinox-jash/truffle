//! Compilation Pipeline
//!
//! The pipeline transforms RawArtifacts into WikiNodes through 4 stages:
//! 1. Ingestion Queue - Priority-based queue management
//! 2. Multimodal Processing - AI-powered content analysis
//! 3. Knowledge Graph Construction - Entity linking and graph building
//! 4. Persistence & Sync Trigger - ACID persistence and CRDT generation
//!
//! Per Section 2.2 of the Enterprise Specification

pub mod ingestion;
pub mod processor;
pub mod graph;
pub mod persistence;
pub mod extraction;

pub use ingestion::{IngestionQueue, QueueConfig, QueueStats};
pub use processor::{Processor, ProcessorConfig, ProcessingResult, ProcessingError};
pub use graph::{GraphBuilder, GraphConfig, LinkingEngine};
pub use persistence::{PersistenceLayer, PersistenceConfig, SyncTrigger};

/// Pipeline error types
#[derive(Debug, thiserror::Error)]
pub enum PipelineError {
    #[error("Configuration error: {0}")]
    Configuration(String),
    
    #[error("Processing error: {0}")]
    Processing(String),
    
    #[error("Extraction error: {0}")]
    Extraction(String),
    
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
}

use crate::models::{RawArtifact, WikiNode, IngestionQueueEntry, IngestionStatus};
use crate::database::Database;
use std::sync::Arc;
use tokio::sync::{mpsc, RwLock};
use tracing::{info, error, debug};

/// Pipeline configuration
#[derive(Debug, Clone)]
pub struct PipelineConfig {
    /// Queue configuration
    pub queue: QueueConfig,
    /// Processor configuration
    pub processor: ProcessorConfig,
    /// Graph builder configuration
    pub graph: GraphConfig,
    /// Persistence configuration
    pub persistence: PersistenceConfig,
    /// Maximum concurrent processing jobs
    pub max_concurrent_jobs: usize,
    /// Enable background processing
    pub enable_background: bool,
}

impl Default for PipelineConfig {
    fn default() -> Self {
        Self {
            queue: QueueConfig::default(),
            processor: ProcessorConfig::default(),
            graph: GraphConfig::default(),
            persistence: PersistenceConfig::default(),
            max_concurrent_jobs: 2,
            enable_background: true,
        }
    }
}

/// Pipeline statistics
#[derive(Debug, Clone, Default)]
pub struct PipelineStats {
    /// Total items processed
    pub total_processed: u64,
    /// Successful compilations
    pub successful: u64,
    /// Failed compilations
    pub failed: u64,
    /// Currently processing
    pub in_progress: u64,
    /// Average processing time (ms)
    pub avg_processing_time_ms: u64,
    /// Queue depth
    pub queue_depth: usize,
}

/// The main compilation pipeline
pub struct CompilationPipeline {
    config: PipelineConfig,
    database: Arc<Database>,
    ingestion_queue: Arc<RwLock<IngestionQueue>>,
    processor: Arc<Processor>,
    graph_builder: Arc<GraphBuilder>,
    persistence: Arc<PersistenceLayer>,
    stats: Arc<RwLock<PipelineStats>>,
    shutdown_tx: Option<mpsc::Sender<()>>,
}

impl CompilationPipeline {
    /// Create a new compilation pipeline
    pub fn new(
        config: PipelineConfig,
        database: Arc<Database>,
    ) -> anyhow::Result<Self> {
        let ingestion_queue = Arc::new(RwLock::new(
            IngestionQueue::new(config.queue.clone(), database.clone())
        ));
        
        let processor = Arc::new(Processor::new(config.processor.clone()));
        let graph_builder = Arc::new(GraphBuilder::new(config.graph.clone()));
        let persistence = Arc::new(PersistenceLayer::new(
            config.persistence.clone(),
            database.clone(),
        ));
        
        Ok(Self {
            config,
            database,
            ingestion_queue,
            processor,
            graph_builder,
            persistence,
            stats: Arc::new(RwLock::new(PipelineStats::default())),
            shutdown_tx: None,
        })
    }
    
    /// Start the pipeline
    pub async fn start(&mut self) -> anyhow::Result<()> {
        info!("Starting compilation pipeline");
        
        let (shutdown_tx, mut shutdown_rx) = mpsc::channel(1);
        self.shutdown_tx = Some(shutdown_tx);
        
        // Start background worker if enabled
        if self.config.enable_background {
            let queue = self.ingestion_queue.clone();
            let processor = self.processor.clone();
            let graph_builder = self.graph_builder.clone();
            let persistence = self.persistence.clone();
            let stats = self.stats.clone();
            let max_concurrent = self.config.max_concurrent_jobs;
            
            tokio::spawn(async move {
                let semaphore = Arc::new(tokio::sync::Semaphore::new(max_concurrent));
                
                loop {
                    tokio::select! {
                        _ = shutdown_rx.recv() => {
                            info!("Pipeline shutdown signal received");
                            break;
                        }
                        _ = tokio::time::sleep(tokio::time::Duration::from_secs(1)) => {
                            // Process queue
                            let permit = match semaphore.clone().try_acquire_owned() {
                                Ok(p) => p,
                                Err(_) => continue, // Max concurrent reached
                            };
                            
                            let queue = queue.clone();
                            let processor = processor.clone();
                            let graph_builder = graph_builder.clone();
                            let persistence = persistence.clone();
                            let stats = stats.clone();
                            
                            tokio::spawn(async move {
                                let _permit = permit;
                                
                                if let Some(entry) = queue.write().await.dequeue().await {
                                    debug!("Processing queue entry: {}", entry.id);
                                    
                                    let start = std::time::Instant::now();
                                    
                                    // Process the artifact
                                    match process_entry(
                                        &entry,
                                        &processor,
                                        &graph_builder,
                                        &persistence,
                                    ).await {
                                        Ok(_) => {
                                            let elapsed = start.elapsed().as_millis() as u64;
                                            let mut s = stats.write().await;
                                            s.total_processed += 1;
                                            s.successful += 1;
                                            s.avg_processing_time_ms = 
                                                (s.avg_processing_time_ms * (s.total_processed - 1) + elapsed)
                                                / s.total_processed;
                                        }
                                        Err(e) => {
                                            error!("Processing failed: {}", e);
                                            let mut s = stats.write().await;
                                            s.total_processed += 1;
                                            s.failed += 1;
                                        }
                                    }
                                }
                            });
                        }
                    }
                }
                
                info!("Pipeline background worker stopped");
            });
        }
        
        info!("Compilation pipeline started");
        Ok(())
    }
    
    /// Stop the pipeline
    pub async fn stop(&mut self) -> anyhow::Result<()> {
        info!("Stopping compilation pipeline");
        
        if let Some(tx) = self.shutdown_tx.take() {
            let _ = tx.send(()).await;
        }
        
        info!("Compilation pipeline stopped");
        Ok(())
    }
    
    /// Submit an artifact for processing
    pub async fn submit(&self, artifact: RawArtifact, priority: u32) -> anyhow::Result<Uuid> {
        let mut queue = self.ingestion_queue.write().await;
        let entry = queue.enqueue(artifact, priority).await?;
        Ok(entry.id)
    }
    
    /// Process a specific artifact immediately (bypass queue)
    pub async fn process_now(&self, artifact: RawArtifact) -> anyhow::Result<WikiNode> {
        let entry = IngestionQueueEntry::new(artifact.uuid, 0);
        
        process_entry(&entry, &self.processor, &self.graph_builder, &self.persistence).await
    }
    
    /// Get pipeline statistics
    pub async fn stats(&self) -> PipelineStats {
        self.stats.read().await.clone()
    }
    
    /// Get queue statistics
    pub async fn queue_stats(&self) -> QueueStats {
        self.ingestion_queue.read().await.stats().await
    }
}

/// Process a single queue entry through the pipeline
async fn process_entry(
    entry: &IngestionQueueEntry,
    processor: &Processor,
    graph_builder: &GraphBuilder,
    persistence: &PersistenceLayer,
) -> anyhow::Result<WikiNode> {
    debug!("Processing entry: {}", entry.id);
    
    // Stage 2: Multimodal Processing
    let processing_result = processor.process(entry.raw_uuid).await?;
    
    // Stage 3: Knowledge Graph Construction
    let node = graph_builder.build_node(processing_result).await?;
    
    // Stage 4: Persistence & Sync Trigger
    persistence.save_node(&node).await?;
    
    debug!("Entry processed successfully: {}", entry.id);
    Ok(node)
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[tokio::test]
    async fn test_pipeline_config() {
        let config = PipelineConfig::default();
        assert_eq!(config.max_concurrent_jobs, 2);
        assert!(config.enable_background);
    }
}
