//! CRDT Document Implementation using Yrs (Yjs for Rust)
//!
//! Implements the YATA algorithm for conflict-free replicated data types.
//! Provides automatic merge capabilities for concurrent edits.
//!
//! Per Section 2.3.3 of the Enterprise Specification

use yrs::{Doc, Text, Map, ReadTxn, StateVector, Update, Transact, updates::encoder::Encode};
use yrs::types::ToJson;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use parking_lot::Mutex;
use tracing::{info, debug, error};

use crate::models::{WikiNode, VectorClock};

/// A CRDT-backed document for wiki nodes
pub struct CrdtDocument {
    /// The Yjs document
    doc: Doc,
    /// Document ID (maps to wiki node ID)
    doc_id: String,
    /// Vector clock for versioning
    vector_clock: VectorClock,
}

impl CrdtDocument {
    /// Create a new CRDT document
    pub fn new(doc_id: impl Into<String>) -> Self {
        let doc = Doc::new();
        
        Self {
            doc,
            doc_id: doc_id.into(),
            vector_clock: VectorClock::new(),
        }
    }
    
    /// Create from existing wiki node
    pub fn from_node(node: &WikiNode) -> anyhow::Result<Self> {
        let mut doc = Self::new(node.id.to_string());
        doc.apply_node(node)?;
        doc.vector_clock = node.version.clone();
        Ok(doc)
    }
    
    /// Apply a wiki node to the document
    pub fn apply_node(&mut self, node: &WikiNode) -> anyhow::Result<()> {
        let mut txn = self.doc.transact_mut();
        
        // Get or create the root map
        let root = txn.get_or_insert_map("wiki");
        
        // Set node properties
        root.insert(&mut txn, "id", node.id.to_string());
        root.insert(&mut txn, "title", node.title.clone());
        root.insert(&mut txn, "node_type", format!("{:?}", node.node_type));
        root.insert(&mut txn, "content", node.raw_content.clone());
        root.insert(&mut txn, "slug", node.slug());
        
        // Set provenance
        let provenance = txn.get_or_insert_map("provenance");
        provenance.insert(&mut txn, "model_version", node.provenance.model_version.clone());
        provenance.insert(&mut txn, "confidence_score", node.provenance.confidence_score.to_string());
        
        // Set privacy
        root.insert(&mut txn, "privacy", format!("{:?}", node.privacy_classification));
        
        txn.commit();
        
        debug!("Applied node to CRDT document: {}", node.id);
        Ok(())
    }
    
    /// Extract wiki node from document
    pub fn to_node(&self) -> anyhow::Result<WikiNode> {
        let txn = self.doc.transact();
        let root = txn.get_map("wiki")
            .ok_or_else(|| anyhow::anyhow!("Missing 'wiki' map in document"))?;
        
        // This is a simplified extraction - in production, implement full mapping
        let id_str: String = root.get(&txn, "id")
            .and_then(|v| v.as_string().map(|s| s.to_string()))
            .ok_or_else(|| anyhow::anyhow!("Missing 'id' field"))?;
        
        let title: String = root.get(&txn, "title")
            .and_then(|v| v.as_string().map(|s| s.to_string()))
            .unwrap_or_default();
        
        let content: String = root.get(&txn, "content")
            .and_then(|v| v.as_string().map(|s| s.to_string()))
            .unwrap_or_default();
        
        // Create minimal node (production would extract all fields)
        let provenance = Provenance::new(vec![], "crdt", 1.0);
        let node = WikiNode::new(&title, crate::models::WikiNodeType::Entity, &content, provenance);
        
        Ok(node)
    }
    
    /// Encode state as update (for sending to other devices)
    pub fn encode_state_as_update(&self) -> Vec<u8> {
        let txn = self.doc.transact();
        let update = txn.encode_state_as_update_v1(&StateVector::default());
        update
    }
    
    /// Encode state as update with specific state vector
    pub fn encode_state_as_update_from(&self, state_vector: &[u8]) -> anyhow::Result<Vec<u8>> {
        let sv = StateVector::decode_v1(state_vector)
            .map_err(|e| anyhow::anyhow!("Failed to decode state vector: {:?}", e))?;
        
        let txn = self.doc.transact();
        let update = txn.encode_state_as_update_v1(&sv);
        Ok(update)
    }
    
    /// Apply update from another device
    pub fn apply_update(&mut self, update: &[u8]) -> anyhow::Result<()> {
        let update = Update::decode_v1(update)
            .map_err(|e| anyhow::anyhow!("Failed to decode update: {:?}", e))?;
        
        let mut txn = self.doc.transact_mut();
        txn.apply_update(update)
            .map_err(|e| anyhow::anyhow!("Failed to apply update: {:?}", e))?;
        txn.commit();
        
        debug!("Applied CRDT update to document: {}", self.doc_id);
        Ok(())
    }
    
    /// Get current state vector
    pub fn state_vector(&self) -> Vec<u8> {
        let txn = self.doc.transact();
        txn.state_vector().encode_v1()
    }
    
    /// Get document ID
    pub fn doc_id(&self) -> &str {
        &self.doc_id
    }
    
    /// Get vector clock
    pub fn vector_clock(&self) -> &VectorClock {
        &self.vector_clock
    }
    
    /// Increment vector clock for device
    pub fn increment_clock(&mut self, device_id: &str) -> u64 {
        self.vector_clock.increment(device_id)
    }
    
    /// Merge another document into this one
    pub fn merge(&mut self, other: &CrdtDocument) -> anyhow::Result<()> {
        let update = other.encode_state_as_update();
        self.apply_update(&update)?;
        
        // Merge vector clocks
        self.vector_clock.merge(&other.vector_clock);
        
        info!("Merged CRDT document: {} <- {}", self.doc_id, other.doc_id);
        Ok(())
    }
    
    /// Get document size in bytes (approximate)
    pub fn size_bytes(&self) -> usize {
        self.encode_state_as_update().len()
    }
}

impl Clone for CrdtDocument {
    fn clone(&self) -> Self {
        // Clone by encoding and re-decoding
        let update = self.encode_state_as_update();
        let mut new_doc = Self::new(self.doc_id.clone());
        let _ = new_doc.apply_update(&update);
        new_doc.vector_clock = self.vector_clock.clone();
        new_doc
    }
}

/// Manager for multiple CRDT documents
pub struct CrdtManager {
    /// Document cache
    documents: Arc<Mutex<std::collections::HashMap<String, CrdtDocument>>>,
    /// Device ID for this instance
    device_id: String,
}

impl CrdtManager {
    /// Create a new CRDT manager
    pub fn new(device_id: impl Into<String>) -> Self {
        Self {
            documents: Arc::new(Mutex::new(std::collections::HashMap::new())),
            device_id: device_id.into(),
        }
    }
    
    /// Get or create a document
    pub fn get_or_create(&self, doc_id: impl Into<String>) -> CrdtDocument {
        let id = doc_id.into();
        let mut docs = self.documents.lock();
        
        if let Some(doc) = docs.get(&id) {
            doc.clone()
        } else {
            let doc = CrdtDocument::new(&id);
            docs.insert(id.clone(), doc.clone());
            doc
        }
    }
    
    /// Get a document if it exists
    pub fn get(&self, doc_id: &str) -> Option<CrdtDocument> {
        let docs = self.documents.lock();
        docs.get(doc_id).cloned()
    }
    
    /// Store a document
    pub fn store(&self, doc: CrdtDocument) {
        let mut docs = self.documents.lock();
        docs.insert(doc.doc_id().to_string(), doc);
    }
    
    /// Remove a document
    pub fn remove(&self, doc_id: &str) -> Option<CrdtDocument> {
        let mut docs = self.documents.lock();
        docs.remove(doc_id)
    }
    
    /// Apply update to a document
    pub fn apply_update(&self, doc_id: &str, update: &[u8]) -> anyhow::Result<()> {
        let mut docs = self.documents.lock();
        
        if let Some(doc) = docs.get_mut(doc_id) {
            doc.apply_update(update)?;
            Ok(())
        } else {
            // Create new document and apply update
            let mut doc = CrdtDocument::new(doc_id);
            doc.apply_update(update)?;
            docs.insert(doc_id.to_string(), doc);
            Ok(())
        }
    }
    
    /// Get all document IDs
    pub fn document_ids(&self) -> Vec<String> {
        let docs = self.documents.lock();
        docs.keys().cloned().collect()
    }
    
    /// Get document count
    pub fn document_count(&self) -> usize {
        let docs = self.documents.lock();
        docs.len()
    }
    
    /// Clear all documents
    pub fn clear(&self) {
        let mut docs = self.documents.lock();
        docs.clear();
    }
    
    /// Generate sync delta for a document
    pub fn generate_delta(&self, doc_id: &str, target_state: Option<&[u8]>) -> anyhow::Result<Vec<u8>> {
        let docs = self.documents.lock();
        
        if let Some(doc) = docs.get(doc_id) {
            match target_state {
                Some(sv) => doc.encode_state_as_update_from(sv),
                None => Ok(doc.encode_state_as_update()),
            }
        } else {
            Err(anyhow::anyhow!("Document not found: {}", doc_id))
        }
    }
    
    /// Get total size of all documents
    pub fn total_size_bytes(&self) -> usize {
        let docs = self.documents.lock();
        docs.values().map(|d| d.size_bytes()).sum()
    }
}

/// Trait for types that can be synced via CRDT
pub trait SyncDocument {
    /// Get document ID
    fn doc_id(&self) -> String;
    
    /// Convert to CRDT document
    fn to_crdt(&self) -> anyhow::Result<CrdtDocument>;
    
    /// Apply CRDT document
    fn from_crdt(&mut self, doc: &CrdtDocument) -> anyhow::Result<()>;
}

impl SyncDocument for WikiNode {
    fn doc_id(&self) -> String {
        self.id.to_string()
    }
    
    fn to_crdt(&self) -> anyhow::Result<CrdtDocument> {
        CrdtDocument::from_node(self)
    }
    
    fn from_crdt(&mut self, doc: &CrdtDocument) -> anyhow::Result<()> {
        // Extract data from CRDT and apply to self
        let extracted = doc.to_node()?;
        
        self.title = extracted.title;
        self.raw_content = extracted.raw_content;
        // ... apply other fields
        
        Ok(())
    }
}

use crate::models::Provenance;

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::{WikiNodeType, Provenance};
    
    fn create_test_node() -> WikiNode {
        let provenance = Provenance::new(vec![uuid::Uuid::new_v4()], "test", 0.9);
        WikiNode::new(
            "Test Node",
            WikiNodeType::Entity,
            "This is test content.",
            provenance,
        )
    }
    
    #[test]
    fn test_crdt_document_creation() {
        let doc = CrdtDocument::new("test-doc");
        assert_eq!(doc.doc_id(), "test-doc");
    }
    
    #[test]
    fn test_crdt_from_node() {
        let node = create_test_node();
        let doc = CrdtDocument::from_node(&node).unwrap();
        
        assert_eq!(doc.doc_id(), node.id.to_string());
        assert!(doc.size_bytes() > 0);
    }
    
    #[test]
    fn test_crdt_update_roundtrip() {
        let node = create_test_node();
        let mut doc1 = CrdtDocument::from_node(&node).unwrap();
        
        // Get state vector
        let sv = doc1.state_vector();
        
        // Create second document and apply update
        let mut doc2 = CrdtDocument::new(node.id.to_string());
        let update = doc1.encode_state_as_update_from(&sv).unwrap();
        doc2.apply_update(&update).unwrap();
        
        // Documents should now be equivalent
        assert_eq!(doc1.state_vector(), doc2.state_vector());
    }
    
    #[test]
    fn test_crdt_manager() {
        let manager = CrdtManager::new("device1");
        
        let doc = manager.get_or_create("doc1");
        assert_eq!(doc.doc_id(), "doc1");
        
        assert_eq!(manager.document_count(), 1);
        
        let ids = manager.document_ids();
        assert!(ids.contains(&"doc1".to_string()));
        
        manager.clear();
        assert_eq!(manager.document_count(), 0);
    }
    
    #[test]
    fn test_vector_clock() {
        let mut clock = VectorClock::new();
        
        clock.increment("device1");
        clock.increment("device1");
        clock.increment("device2");
        
        assert_eq!(clock.clocks.get("device1"), Some(&2));
        assert_eq!(clock.clocks.get("device2"), Some(&1));
    }
}
