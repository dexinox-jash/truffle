//! Stage 1: Ingestion Queue
//!
//! Priority-based queue management for raw artifacts awaiting compilation.
//! Implements the queue schema from Section 2.2.1:
//! - P0: User-initiated "Compile Now" (highest)
//! - P1: Screenshots with messaging context
//! - P2: Background batch (lowest)

use crate::models::{RawArtifact, IngestionQueueEntry, IngestionStatus};
use crate::database::{Database, RawArtifactRepository, IngestionQueueRepository};
use std::sync::Arc;
use tokio::sync::RwLock;
use uuid::Uuid;
use chrono::Utc;
use tracing::{info, debug, warn, error};

/// Queue configuration
#[derive(Debug, Clone)]
pub struct QueueConfig {
    /// Maximum queue size
    pub max_size: usize,
    /// Maximum retries for failed items
    pub max_retries: u32,
    /// Retry delay in seconds (exponential backoff)
    pub retry_delay_secs: u64,
    /// Batch size for background processing
    pub batch_size: usize,
    /// Priority boost for messaging apps
    pub messaging_priority_boost: bool,
}

impl Default for QueueConfig {
    fn default() -> Self {
        Self {
            max_size: 10000,
            max_retries: 3,
            retry_delay_secs: 60,
            batch_size: 10,
            messaging_priority_boost: true,
        }
    }
}

/// Queue statistics
#[derive(Debug, Clone, Default)]
pub struct QueueStats {
    /// Total items in queue
    pub total: usize,
    /// Pending items
    pub pending: usize,
    /// Processing items
    pub processing: usize,
    /// Failed items (awaiting retry)
    pub failed: usize,
    /// Completed items (since last reset)
    pub completed: u64,
}

/// Ingestion Queue Manager
pub struct IngestionQueue {
    config: QueueConfig,
    database: Arc<Database>,
    stats: QueueStats,
}

impl IngestionQueue {
    /// Create a new ingestion queue
    pub fn new(config: QueueConfig, database: Arc<Database>) -> Self {
        Self {
            config,
            database,
            stats: QueueStats::default(),
        }
    }
    
    /// Enqueue a raw artifact for processing
    /// 
    /// # Arguments
    /// * `artifact` - The raw artifact to process
    /// * `priority` - Priority (0 = highest, 2 = lowest)
    /// 
    /// # Returns
    /// The queue entry ID
    pub async fn enqueue(&mut self, artifact: RawArtifact, priority: u32) -> anyhow::Result<IngestionQueueEntry> {
        debug!("Enqueuing artifact: {} with priority {}", artifact.uuid, priority);
        
        // Save artifact to database first
        self.database.with_connection(|conn| {
            let repo = RawArtifactRepository::new(conn);
            repo.save(&artifact)?;
            Ok(())
        }).map_err(|e| anyhow::anyhow!("Failed to save artifact: {}", e))?;
        
        // Create queue entry
        let entry = IngestionQueueEntry::new(artifact.uuid, priority);
        
        // Save to database
        self.database.with_connection(|conn| {
            let repo = IngestionQueueRepository::new(conn);
            repo.save(&entry)?;
            Ok(())
        }).map_err(|e| anyhow::anyhow!("Failed to save queue entry: {}", e))?;
        
        self.stats.total += 1;
        self.stats.pending += 1;
        
        info!("Artifact enqueued: {} (entry: {})", artifact.uuid, entry.id);
        Ok(entry)
    }
    
    /// Enqueue with automatic priority based on artifact context
    pub async fn enqueue_auto(&mut self, artifact: RawArtifact) -> anyhow::Result<IngestionQueueEntry> {
        let priority = if self.config.messaging_priority_boost {
            artifact.priority_score()
        } else {
            2 // Default to background priority
        };
        
        self.enqueue(artifact, priority).await
    }
    
    /// Dequeue the next item for processing (highest priority first)
    pub async fn dequeue(&mut self) -> Option<IngestionQueueEntry> {
        // Get pending items from database, ordered by priority
        let entries = self.database.with_connection(|conn| {
            let repo = IngestionQueueRepository::new(conn);
            repo.get_pending(self.config.batch_size)
                .map_err(|e| crate::database::DatabaseError::Query(e.to_string()))
        });
        
        match entries {
            Ok(entries) => {
                for mut entry in entries {
                    // Check if we should retry failed items
                    if entry.status == IngestionStatus::Failed {
                        if entry.retry_count >= self.config.max_retries {
                            debug!("Entry {} exceeded max retries, skipping", entry.id);
                            continue;
                        }
                        
                        // Check retry delay
                        if let Some(completed_at) = entry.completed_at {
                            let retry_after = chrono::Duration::seconds(
                                self.config.retry_delay_secs as i64 * (entry.retry_count as i64 + 1)
                            );
                            if Utc::now() < completed_at + retry_after {
                                continue;
                            }
                        }
                    }
                    
                    // Mark as processing
                    entry.mark_started();
                    
                    if let Err(e) = self.database.with_connection(|conn| {
                        let repo = IngestionQueueRepository::new(conn);
                        repo.save(&entry)?;
                        Ok(())
                    }) {
                        error!("Failed to update queue entry status: {}", e);
                        continue;
                    }
                    
                    // Update stats
                    self.stats.pending = self.stats.pending.saturating_sub(1);
                    
                    debug!("Dequeued entry: {} (artifact: {})", entry.id, entry.raw_uuid);
                    return Some(entry);
                }
                None
            }
            Err(e) => {
                error!("Failed to get pending entries: {}", e);
                None
            }
        }
    }
    
    /// Mark an entry as completed
    pub async fn mark_completed(&mut self, entry_id: Uuid) -> anyhow::Result<()> {
        self.database.with_connection(|conn| {
            let repo = IngestionQueueRepository::new(conn);
            repo.update_status(entry_id, IngestionStatus::Compiled)?;
            Ok(())
        }).map_err(|e| anyhow::anyhow!("Failed to mark completed: {}", e))?;
        
        self.stats.completed += 1;
        info!("Entry marked completed: {}", entry_id);
        
        Ok(())
    }
    
    /// Mark an entry as failed
    pub async fn mark_failed(&mut self, entry_id: Uuid, error: impl Into<String>) -> anyhow::Result<()> {
        let error_msg = error.into();
        
        self.database.with_connection(|conn| {
            let repo = IngestionQueueRepository::new(conn);
            
            // Get current entry
            if let Some(mut entry) = repo.find_by_id(entry_id)? {
                entry.mark_failed(&error_msg);
                repo.save(&entry)?;
                
                // Also update artifact status
                let artifact_repo = RawArtifactRepository::new(conn);
                artifact_repo.update_status(entry.raw_uuid, IngestionStatus::Failed)?;
            }
            Ok(())
        }).map_err(|e| anyhow::anyhow!("Failed to mark failed: {}", e))?;
        
        self.stats.failed += 1;
        warn!("Entry marked failed: {} - {}", entry_id, error_msg);
        
        Ok(())
    }
    
    /// Get queue statistics
    pub async fn stats(&self) -> QueueStats {
        // Refresh stats from database
        match self.database.with_connection(|conn| {
            let artifact_repo = RawArtifactRepository::new(conn);
            let queue_repo = IngestionQueueRepository::new(conn);
            
            let pending = artifact_repo.count_by_status(IngestionStatus::Pending)?;
            let processing = artifact_repo.count_by_status(IngestionStatus::Processing)?;
            let failed = artifact_repo.count_by_status(IngestionStatus::Failed)?;
            
            Ok::<_, crate::database::DatabaseError>((pending, processing, failed))
        }) {
            Ok((pending, processing, failed)) => {
                QueueStats {
                    total: (pending + processing + failed) as usize,
                    pending: pending as usize,
                    processing: processing as usize,
                    failed: failed as usize,
                    completed: self.stats.completed,
                }
            }
            Err(e) => {
                error!("Failed to refresh queue stats: {}", e);
                self.stats.clone()
            }
        }
    }
    
    /// Get all pending entries
    pub async fn get_pending(&self, limit: usize) -> anyhow::Result<Vec<IngestionQueueEntry>> {
        self.database.with_connection(|conn| {
            let repo = IngestionQueueRepository::new(conn);
            repo.get_pending(limit)
                .map_err(|e| crate::database::DatabaseError::Query(e.to_string()))
        }).map_err(|e| anyhow::anyhow!("Failed to get pending: {}", e))
    }
    
    /// Remove an entry from the queue
    pub async fn remove(&mut self, entry_id: Uuid) -> anyhow::Result<bool> {
        self.database.with_connection(|conn| {
            let repo = IngestionQueueRepository::new(conn);
            repo.delete(entry_id)
                .map_err(|e| crate::database::DatabaseError::Query(e.to_string()))
        }).map_err(|e| anyhow::anyhow!("Failed to remove entry: {}", e))
    }
    
    /// Clear all completed entries
    pub async fn clear_completed(&mut self) -> anyhow::Result<u64> {
        // In a real implementation, this would delete old completed entries
        // For now, just return 0
        Ok(0)
    }
    
    /// Requeue a failed entry (manual retry)
    pub async fn retry(&mut self, entry_id: Uuid) -> anyhow::Result<()> {
        self.database.with_connection(|conn| {
            let repo = IngestionQueueRepository::new(conn);
            
            if let Some(mut entry) = repo.find_by_id(entry_id)? {
                entry.status = IngestionStatus::Pending;
                entry.retry_count = 0;
                entry.error_message = None;
                repo.save(&entry)?;
                
                // Also reset artifact status
                let artifact_repo = RawArtifactRepository::new(conn);
                artifact_repo.update_status(entry.raw_uuid, IngestionStatus::Pending)?;
            }
            Ok(())
        }).map_err(|e| anyhow::anyhow!("Failed to retry entry: {}", e))?;
        
        info!("Entry requeued for retry: {}", entry_id);
        Ok(())
    }
}

/// Priority calculator for artifacts
pub struct PriorityCalculator;

impl PriorityCalculator {
    /// Calculate priority based on artifact properties
    /// 
    /// Priority levels:
    /// - 0 (P0): User-initiated, urgent
    /// - 1 (P1): Messaging apps (temporal relevance decays fast)
    /// - 2 (P2): Background batch processing
    pub fn calculate(artifact: &RawArtifact) -> u32 {
        // Check for messaging context
        if let Some(ref ctx) = artifact.app_context {
            let bundle_lower = ctx.bundle_id.to_lowercase();
            if Self::is_messaging_app(&bundle_lower) {
                return 1; // P1: Messaging
            }
        }
        
        // Default to background priority
        2 // P2: Background
    }
    
    /// Check if bundle ID is a messaging app
    fn is_messaging_app(bundle_id: &str) -> bool {
        const MESSAGING_APPS: &[&str] = &[
            "message",
            "slack",
            "telegram",
            "whatsapp",
            "signal",
            "discord",
            "teams",
            "imessage",
            "sms",
        ];
        
        MESSAGING_APPS.iter().any(|app| bundle_id.contains(app))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::{RawArtifact, RawArtifactMetadata, AppContext, DeviceFingerprint, BlobPointer};
    use crate::database::init_database;
    
    fn create_test_artifact() -> RawArtifact {
        RawArtifact {
            uuid: Uuid::new_v4(),
            filename: "test.png".to_string(),
            binary: BlobPointer {
                path: std::path::PathBuf::from("/tmp/test.png"),
                content_hash: "abc123".to_string(),
            },
            captured_at: Utc::now(),
            device_id: DeviceFingerprint::new("test", b"salt"),
            app_context: None,
            geohash: None,
            metadata: RawArtifactMetadata::new(1024, 100, 100),
            status: IngestionStatus::Pending,
            retry_count: 0,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        }
    }
    
    #[test]
    fn test_priority_calculator() {
        let mut artifact = create_test_artifact();
        
        // Default priority
        assert_eq!(PriorityCalculator::calculate(&artifact), 2);
        
        // Messaging app priority
        artifact.app_context = Some(AppContext {
            bundle_id: "com.apple.MobileSMS".to_string(),
            app_name: "Messages".to_string(),
            window_title: None,
        });
        assert_eq!(PriorityCalculator::calculate(&artifact), 1);
        
        // Slack priority
        artifact.app_context = Some(AppContext {
            bundle_id: "com.tinyspeck.slackmacgap".to_string(),
            app_name: "Slack".to_string(),
            window_title: None,
        });
        assert_eq!(PriorityCalculator::calculate(&artifact), 1);
    }
}
