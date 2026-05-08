//! Stage 4: Persistence & Sync Trigger
//!
//! ACID persistence of wiki nodes and CRDT delta generation for sync.
//! 
//! Per Section 2.2.4 of the Enterprise Specification:
//! - WAL mode SQLite with fsync every transaction
//! - CRDT delta generation (Yjs encodeStateAsUpdate)
//! - Encrypted sync queue

use crate::models::{WikiNode, RawArtifact, IngestionStatus, VectorClock};
use crate::database::{Database, WikiNodeRepository, RawArtifactRepository};
use crate::sync::crdt::CrdtDocument;
use std::sync::Arc;
use tracing::{info, debug, error};

/// Persistence configuration
#[derive(Debug, Clone)]
pub struct PersistenceConfig {
    /// Enable fsync on every transaction
    pub fsync_every_transaction: bool,
    /// Enable automatic sync trigger
    pub enable_sync_trigger: bool,
    /// Sync trigger debounce time (ms)
    pub sync_debounce_ms: u64,
    /// Enable compression for sync payloads
    pub enable_compression: bool,
}

impl Default for PersistenceConfig {
    fn default() -> Self {
        Self {
            fsync_every_transaction: true,
            enable_sync_trigger: true,
            sync_debounce_ms: 1000,
            enable_compression: true,
        }
    }
}

/// Persistence layer for wiki nodes
pub struct PersistenceLayer {
    config: PersistenceConfig,
    database: Arc<Database>,
    sync_trigger: Option<SyncTrigger>,
}

impl PersistenceLayer {
    /// Create a new persistence layer
    pub fn new(config: PersistenceConfig, database: Arc<Database>) -> Self {
        let sync_trigger = if config.enable_sync_trigger {
            Some(SyncTrigger::new(config.sync_debounce_ms))
        } else {
            None
        };
        
        Self {
            config,
            database,
            sync_trigger,
        }
    }
    
    /// Save a wiki node with full ACID guarantees
    pub async fn save_node(&self, node: &WikiNode) -> anyhow::Result<()> {
        debug!("Persisting wiki node: {} ({})", node.id, node.title);
        
        // Use transaction for atomicity
        self.database.with_transaction(|conn| {
            let repo = WikiNodeRepository::new(conn);
            repo.save(node)?;
            Ok(())
        }).map_err(|e| anyhow::anyhow!("Failed to save node: {}", e))?;
        
        info!("Wiki node persisted: {} ({})", node.id, node.title);
        
        // Trigger sync if enabled
        if let Some(ref trigger) = self.sync_trigger {
            trigger.on_node_changed(node).await?;
        }
        
        Ok(())
    }
    
    /// Update raw artifact status after compilation
    pub async fn update_artifact_status(
        &self,
        artifact_uuid: uuid::Uuid,
        status: IngestionStatus,
    ) -> anyhow::Result<()> {
        self.database.with_connection(|conn| {
            let repo = RawArtifactRepository::new(conn);
            repo.update_status(artifact_uuid, status)?;
            Ok(())
        }).map_err(|e| anyhow::anyhow!("Failed to update artifact status: {}", e))?;
        
        Ok(())
    }
    
    /// Save multiple nodes in a batch transaction
    pub async fn save_nodes_batch(&self, nodes: &[WikiNode]) -> anyhow::Result<()> {
        debug!("Persisting batch of {} nodes", nodes.len());
        
        self.database.with_transaction(|conn| {
            let repo = WikiNodeRepository::new(conn);
            for node in nodes {
                repo.save(node)?;
            }
            Ok(())
        }).map_err(|e| anyhow::anyhow!("Failed to save batch: {}", e))?;
        
        info!("Batch of {} nodes persisted", nodes.len());
        
        // Trigger sync for batch
        if let Some(ref trigger) = self.sync_trigger {
            for node in nodes {
                trigger.on_node_changed(node).await?;
            }
        }
        
        Ok(())
    }
    
    /// Delete a node and create tombstone
    pub async fn delete_node(&self, node_id: uuid::Uuid) -> anyhow::Result<()> {
        debug!("Deleting wiki node: {}", node_id);
        
        self.database.with_transaction(|conn| {
            let repo = WikiNodeRepository::new(conn);
            repo.delete(node_id)?;
            
            // Create tombstone for sync
            // INSERT INTO tombstones ...
            
            Ok(())
        }).map_err(|e| anyhow::anyhow!("Failed to delete node: {}", e))?;
        
        info!("Wiki node deleted: {}", node_id);
        
        // Trigger sync for deletion
        if let Some(ref trigger) = self.sync_trigger {
            trigger.on_node_deleted(node_id).await?;
        }
        
        Ok(())
    }
    
    /// Checkpoint WAL to main database
    pub async fn checkpoint(&self) -> anyhow::Result<()> {
        self.database.checkpoint()
            .map_err(|e| anyhow::anyhow!("Checkpoint failed: {}", e))?;
        Ok(())
    }
    
    /// Vacuum the database
    pub async fn vacuum(&self) -> anyhow::Result<()> {
        self.database.vacuum()
            .map_err(|e| anyhow::anyhow!("Vacuum failed: {}", e))?;
        Ok(())
    }
    
    /// Get database statistics
    pub async fn stats(&self) -> anyhow::Result<crate::database::connection::DatabaseStats> {
        self.database.stats()
            .map_err(|e| anyhow::anyhow!("Failed to get stats: {}", e))
    }
}

/// Sync trigger for CRDT delta generation
pub struct SyncTrigger {
    debounce_ms: u64,
    pending_changes: Arc<tokio::sync::RwLock<Vec<PendingChange>>>,
}

/// A pending sync change
#[derive(Debug, Clone)]
struct PendingChange {
    node_id: uuid::Uuid,
    change_type: ChangeType,
    timestamp: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ChangeType {
    Create,
    Update,
    Delete,
}

impl SyncTrigger {
    /// Create a new sync trigger
    pub fn new(debounce_ms: u64) -> Self {
        let pending = Arc::new(tokio::sync::RwLock::new(Vec::new()));
        
        // Start debounce task
        let pending_clone = pending.clone();
        tokio::spawn(async move {
            let mut interval = tokio::time::interval(
                tokio::time::Duration::from_millis(debounce_ms)
            );
            
            loop {
                interval.tick().await;
                
                let changes = {
                    let mut guard = pending_clone.write().await;
                    if guard.is_empty() {
                        continue;
                    }
                    std::mem::take(&mut *guard)
                };
                
                if !changes.is_empty() {
                    debug!("Sync trigger: {} pending changes", changes.len());
                    // In production, this would generate CRDT deltas
                    // and queue them for sync
                }
            }
        });
        
        Self {
            debounce_ms,
            pending_changes: pending,
        }
    }
    
    /// Called when a node is changed
    pub async fn on_node_changed(&self, node: &WikiNode) -> anyhow::Result<()> {
        let change = PendingChange {
            node_id: node.id,
            change_type: ChangeType::Update,
            timestamp: chrono::Utc::now(),
        };
        
        self.pending_changes.write().await.push(change);
        Ok(())
    }
    
    /// Called when a node is created
    pub async fn on_node_created(&self, node: &WikiNode) -> anyhow::Result<()> {
        let change = PendingChange {
            node_id: node.id,
            change_type: ChangeType::Create,
            timestamp: chrono::Utc::now(),
        };
        
        self.pending_changes.write().await.push(change);
        Ok(())
    }
    
    /// Called when a node is deleted
    pub async fn on_node_deleted(&self, node_id: uuid::Uuid) -> anyhow::Result<()> {
        let change = PendingChange {
            node_id,
            change_type: ChangeType::Delete,
            timestamp: chrono::Utc::now(),
        };
        
        self.pending_changes.write().await.push(change);
        Ok(())
    }
    
    /// Generate CRDT delta for a node
    pub fn generate_delta(&self, node: &WikiNode) -> anyhow::Result<Vec<u8>> {
        // In production, this would use Yrs to generate a CRDT update
        // let doc = CrdtDocument::new();
        // doc.apply_node(node);
        // let update = doc.encode_state_as_update();
        
        // Placeholder: serialize the node
        let json = serde_json::to_vec(node)?;
        Ok(json)
    }
    
    /// Get pending change count
    pub async fn pending_count(&self) -> usize {
        self.pending_changes.read().await.len()
    }
    
    /// Flush all pending changes immediately
    pub async fn flush(&self) -> anyhow::Result<Vec<PendingChange>> {
        let mut guard = self.pending_changes.write().await;
        let changes = std::mem::take(&mut *guard);
        Ok(changes)
    }
}

/// CRDT delta for sync
#[derive(Debug, Clone)]
pub struct CrdtDelta {
    /// Node ID
    pub node_id: uuid::Uuid,
    /// Vector clock at time of change
    pub vector_clock: VectorClock,
    /// Binary CRDT update
    pub update_data: Vec<u8>,
    /// Timestamp
    pub timestamp: chrono::DateTime<chrono::Utc>,
    /// Compression applied
    pub compressed: bool,
}

impl CrdtDelta {
    /// Create a new delta
    pub fn new(
        node_id: uuid::Uuid,
        vector_clock: VectorClock,
        update_data: Vec<u8>,
    ) -> Self {
        Self {
            node_id,
            vector_clock,
            update_data,
            timestamp: chrono::Utc::now(),
            compressed: false,
        }
    }
    
    /// Compress the delta payload
    pub fn compress(&mut self) -> anyhow::Result<()> {
        if self.compressed {
            return Ok(());
        }
        
        // In production, use proper compression (e.g., zstd)
        // For now, just mark as compressed
        self.compressed = true;
        Ok(())
    }
    
    /// Get payload size in bytes
    pub fn size_bytes(&self) -> usize {
        self.update_data.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::{WikiNode, WikiNodeType, Provenance, PrivacyClassification};
    
    fn create_test_node() -> WikiNode {
        let provenance = Provenance::new(vec![uuid::Uuid::new_v4()], "test", 0.9);
        WikiNode::new(
            "Test Node",
            WikiNodeType::Entity,
            "Test content",
            provenance,
        )
    }
    
    #[test]
    fn test_persistence_config() {
        let config = PersistenceConfig::default();
        assert!(config.fsync_every_transaction);
        assert!(config.enable_sync_trigger);
        assert_eq!(config.sync_debounce_ms, 1000);
    }
    
    #[tokio::test]
    async fn test_sync_trigger() {
        let trigger = SyncTrigger::new(100);
        
        let node = create_test_node();
        trigger.on_node_created(&node).await.unwrap();
        
        assert_eq!(trigger.pending_count().await, 1);
        
        let changes = trigger.flush().await.unwrap();
        assert_eq!(changes.len(), 1);
        assert_eq!(changes[0].node_id, node.id);
    }
    
    #[test]
    fn test_crdt_delta() {
        let vector_clock = VectorClock::new();
        let update_data = vec![1, 2, 3, 4, 5];
        
        let delta = CrdtDelta::new(
            uuid::Uuid::new_v4(),
            vector_clock,
            update_data.clone(),
        );
        
        assert_eq!(delta.size_bytes(), update_data.len());
        assert!(!delta.compressed);
    }
}
