//! Database Queries
//!
//! CRUD operations for RawArtifacts, WikiNodes, and IngestionQueue.
//! All operations use ACID transactions for data integrity.

use rusqlite::{Connection, Row, params, OptionalExtension};
use uuid::Uuid;
use chrono::{DateTime, Utc};
use serde_json;
use tracing::{info, debug, error};

use crate::models::{
    RawArtifact, RawArtifactMetadata, AppContext, IngestionStatus,
    IngestionQueueEntry, DeviceFingerprint, Geohash, BlobPointer,
    WikiNode, WikiNodeType, WikiLink, Provenance, TemporalVectors,
    PrivacyClassification, EncryptionStatus, VectorClock,
};

use super::DatabaseError;

/// Repository trait for common operations
pub trait Repository<T> {
    type Error;
    
    fn find_by_id(&self, id: Uuid) -> Result<Option<T>, Self::Error>;
    fn save(&self, item: &T) -> Result<(), Self::Error>;
    fn delete(&self, id: Uuid) -> Result<bool, Self::Error>;
}

/// Raw Artifact Repository
pub struct RawArtifactRepository<'a> {
    conn: &'a Connection,
}

impl<'a> RawArtifactRepository<'a> {
    pub fn new(conn: &'a Connection) -> Self {
        Self { conn }
    }
    
    /// Find raw artifact by UUID
    pub fn find_by_id(&self, uuid: Uuid) -> Result<Option<RawArtifact>, DatabaseError> {
        let mut stmt = self.conn.prepare(
            "SELECT * FROM raw_artifacts WHERE uuid = ?1"
        )?;
        
        let result = stmt.query_row([uuid.as_bytes()], |row| {
            Self::row_to_artifact(row)
        }).optional()?;
        
        Ok(result)
    }
    
    /// Find artifacts by status
    pub fn find_by_status(&self, status: IngestionStatus) -> Result<Vec<RawArtifact>, DatabaseError> {
        let mut stmt = self.conn.prepare(
            "SELECT * FROM raw_artifacts WHERE status = ?1 ORDER BY captured_at DESC"
        )?;
        
        let status_str = format!("{:?}", status).to_lowercase();
        let rows = stmt.query_map([&status_str], |row| {
            Self::row_to_artifact(row)
        })?;
        
        let mut artifacts = Vec::new();
        for row in rows {
            artifacts.push(row?);
        }
        
        Ok(artifacts)
    }
    
    /// Find pending artifacts ordered by priority
    pub fn find_pending_ordered(&self, limit: usize) -> Result<Vec<RawArtifact>, DatabaseError> {
        let mut stmt = self.conn.prepare(
            "SELECT ra.* FROM raw_artifacts ra
             LEFT JOIN ingestion_queue iq ON ra.uuid = iq.raw_uuid
             WHERE ra.status = 'pending' OR ra.status = 'failed'
             ORDER BY COALESCE(iq.priority, 2), ra.captured_at DESC
             LIMIT ?1"
        )?;
        
        let rows = stmt.query_map([limit as i64], |row| {
            Self::row_to_artifact(row)
        })?;
        
        let mut artifacts = Vec::new();
        for row in rows {
            artifacts.push(row?);
        }
        
        Ok(artifacts)
    }
    
    /// Save a raw artifact
    pub fn save(&self, artifact: &RawArtifact) -> Result<(), DatabaseError> {
        let app_bundle_id = artifact.app_context.as_ref().map(|c| c.bundle_id.clone());
        let app_name = artifact.app_context.as_ref().map(|c| c.app_name.clone());
        let window_title = artifact.app_context.as_ref().and_then(|c| c.window_title.clone());
        let geohash = artifact.geohash.as_ref().map(|g| g.as_str().to_string());
        
        self.conn.execute(
            "INSERT OR REPLACE INTO raw_artifacts (
                uuid, filename, storage_path, content_hash, captured_at,
                device_id, app_bundle_id, app_name, window_title, geohash,
                size_bytes, width, height, ocr_text, status, retry_count,
                created_at, updated_at
            ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14, ?15, ?16, ?17, ?18)",
            params![
                artifact.uuid.as_bytes(),
                &artifact.filename,
                artifact.binary.path.to_string_lossy().as_ref(),
                &artifact.binary.content_hash,
                artifact.captured_at.to_rfc3339(),
                artifact.device_id.as_str(),
                app_bundle_id,
                app_name,
                window_title,
                geohash,
                artifact.metadata.size_bytes as i64,
                artifact.metadata.width as i64,
                artifact.metadata.height as i64,
                artifact.metadata.ocr_text,
                format!("{:?}", artifact.status).to_lowercase(),
                artifact.retry_count as i64,
                artifact.created_at.to_rfc3339(),
                artifact.updated_at.to_rfc3339(),
            ],
        )?;
        
        debug!("Saved raw artifact: {}", artifact.uuid);
        Ok(())
    }
    
    /// Update artifact status
    pub fn update_status(&self, uuid: Uuid, status: IngestionStatus) -> Result<(), DatabaseError> {
        self.conn.execute(
            "UPDATE raw_artifacts SET status = ?1, updated_at = ?2 WHERE uuid = ?3",
            params![
                format!("{:?}", status).to_lowercase(),
                Utc::now().to_rfc3339(),
                uuid.as_bytes(),
            ],
        )?;
        
        Ok(())
    }
    
    /// Update OCR text
    pub fn update_ocr(&self, uuid: Uuid, ocr_text: &str) -> Result<(), DatabaseError> {
        self.conn.execute(
            "UPDATE raw_artifacts SET ocr_text = ?1, updated_at = ?2 WHERE uuid = ?3",
            params![
                ocr_text,
                Utc::now().to_rfc3339(),
                uuid.as_bytes(),
            ],
        )?;
        
        Ok(())
    }
    
    /// Delete a raw artifact
    pub fn delete(&self, uuid: Uuid) -> Result<bool, DatabaseError> {
        let rows = self.conn.execute(
            "DELETE FROM raw_artifacts WHERE uuid = ?1",
            [uuid.as_bytes()],
        )?;
        
        Ok(rows > 0)
    }
    
    /// Count artifacts by status
    pub fn count_by_status(&self, status: IngestionStatus) -> Result<i64, DatabaseError> {
        let count: i64 = self.conn.query_row(
            "SELECT COUNT(*) FROM raw_artifacts WHERE status = ?1",
            [format!("{:?}", status).to_lowercase()],
            |row| row.get(0),
        )?;
        
        Ok(count)
    }
    
    /// Search by OCR text (using FTS if available)
    pub fn search_ocr(&self, query: &str, limit: usize) -> Result<Vec<RawArtifact>, DatabaseError> {
        // Try FTS first, fall back to LIKE
        let mut stmt = self.conn.prepare(
            "SELECT ra.* FROM raw_artifacts ra
             JOIN ocr_fts fts ON ra.uuid = fts.uuid
             WHERE ocr_fts MATCH ?1
             LIMIT ?2"
        )?;
        
        let rows = stmt.query_map(params![query, limit as i64], |row| {
            Self::row_to_artifact(row)
        });
        
        match rows {
            Ok(rows) => {
                let mut artifacts = Vec::new();
                for row in rows {
                    artifacts.push(row?);
                }
                Ok(artifacts)
            }
            Err(_) => {
                // Fallback to LIKE
                let mut stmt = self.conn.prepare(
                    "SELECT * FROM raw_artifacts WHERE ocr_text LIKE ?1 LIMIT ?2"
                )?;
                
                let pattern = format!("%{}%", query);
                let rows = stmt.query_map(params![pattern, limit as i64], |row| {
                    Self::row_to_artifact(row)
                })?;
                
                let mut artifacts = Vec::new();
                for row in rows {
                    artifacts.push(row?);
                }
                Ok(artifacts)
            }
        }
    }
    
    /// Convert database row to RawArtifact
    fn row_to_artifact(row: &Row) -> Result<RawArtifact, rusqlite::Error> {
        let uuid_bytes: Vec<u8> = row.get("uuid")?;
        let uuid = Uuid::from_slice(&uuid_bytes)
            .map_err(|e| rusqlite::Error::FromSqlConversionFailure(
                0,
                rusqlite::types::Type::Blob,
                Box::new(e),
            ))?;
        
        let storage_path: String = row.get("storage_path")?;
        let content_hash: String = row.get("content_hash")?;
        
        let app_bundle_id: Option<String> = row.get("app_bundle_id")?;
        let app_context = app_bundle_id.map(|bundle_id| AppContext {
            bundle_id,
            app_name: row.get("app_name")?.unwrap_or_default(),
            window_title: row.get("window_title")?,
        });
        
        let geohash: Option<String> = row.get("geohash")?;
        let geohash = geohash.map(Geohash::from_string);
        
        let status_str: String = row.get("status")?;
        let status = parse_status(&status_str);
        
        let captured_at: String = row.get("captured_at")?;
        let created_at: String = row.get("created_at")?;
        let updated_at: String = row.get("updated_at")?;
        
        Ok(RawArtifact {
            uuid,
            filename: row.get("filename")?,
            binary: BlobPointer {
                path: std::path::PathBuf::from(storage_path),
                content_hash,
            },
            captured_at: DateTime::parse_from_rfc3339(&captured_at)
                .map_err(|e| rusqlite::Error::FromSqlConversionFailure(
                    0,
                    rusqlite::types::Type::Text,
                    Box::new(e),
                ))?.with_timezone(&Utc),
            device_id: DeviceFingerprint::from_string(row.get("device_id")?),
            app_context,
            geohash,
            metadata: RawArtifactMetadata {
                size_bytes: row.get::<_, i64>("size_bytes")? as u64,
                width: row.get::<_, i64>("width")? as u32,
                height: row.get::<_, i64>("height")? as u32,
                ocr_text: row.get("ocr_text")?,
                embedding_vector: None, // Loaded separately
            },
            status,
            retry_count: row.get::<_, i64>("retry_count")? as u32,
            created_at: DateTime::parse_from_rfc3339(&created_at)
                .map_err(|e| rusqlite::Error::FromSqlConversionFailure(
                    0,
                    rusqlite::types::Type::Text,
                    Box::new(e),
                ))?.with_timezone(&Utc),
            updated_at: DateTime::parse_from_rfc3339(&updated_at)
                .map_err(|e| rusqlite::Error::FromSqlConversionFailure(
                    0,
                    rusqlite::types::Type::Text,
                    Box::new(e),
                ))?.with_timezone(&Utc),
        })
    }
}

/// Wiki Node Repository
pub struct WikiNodeRepository<'a> {
    conn: &'a Connection,
}

impl<'a> WikiNodeRepository<'a> {
    pub fn new(conn: &'a Connection) -> Self {
        Self { conn }
    }
    
    /// Find wiki node by ID
    pub fn find_by_id(&self, id: Uuid) -> Result<Option<WikiNode>, DatabaseError> {
        let mut stmt = self.conn.prepare(
            "SELECT * FROM wiki_nodes WHERE id = ?1"
        )?;
        
        let result = stmt.query_row([id.as_bytes()], |row| {
            Self::row_to_node(row)
        }).optional()?;
        
        Ok(result)
    }
    
    /// Find node by slug
    pub fn find_by_slug(&self, slug: &str) -> Result<Option<WikiNode>, DatabaseError> {
        let mut stmt = self.conn.prepare(
            "SELECT * FROM wiki_nodes WHERE slug = ?1"
        )?;
        
        let result = stmt.query_row([slug], |row| {
            Self::row_to_node(row)
        }).optional()?;
        
        Ok(result)
    }
    
    /// Find nodes by type
    pub fn find_by_type(&self, node_type: WikiNodeType, limit: usize) -> Result<Vec<WikiNode>, DatabaseError> {
        let mut stmt = self.conn.prepare(
            "SELECT * FROM wiki_nodes WHERE node_type = ?1 LIMIT ?2"
        )?;
        
        let type_str = format!("{:?}", node_type).to_lowercase();
        let rows = stmt.query_map(params![type_str, limit as i64], |row| {
            Self::row_to_node(row)
        })?;
        
        let mut nodes = Vec::new();
        for row in rows {
            nodes.push(row?);
        }
        
        Ok(nodes)
    }
    
    /// Search nodes by title or content
    pub fn search(&self, query: &str, limit: usize) -> Result<Vec<WikiNode>, DatabaseError> {
        // Try FTS first
        let mut stmt = self.conn.prepare(
            "SELECT wn.* FROM wiki_nodes wn
             JOIN wiki_fts fts ON wn.id = fts.node_id
             WHERE wiki_fts MATCH ?1
             LIMIT ?2"
        )?;
        
        let rows = stmt.query_map(params![query, limit as i64], |row| {
            Self::row_to_node(row)
        });
        
        match rows {
            Ok(rows) => {
                let mut nodes = Vec::new();
                for row in rows {
                    nodes.push(row?);
                }
                Ok(nodes)
            }
            Err(_) => {
                // Fallback to LIKE
                let pattern = format!("%{}%", query);
                let mut stmt = self.conn.prepare(
                    "SELECT * FROM wiki_nodes WHERE title LIKE ?1 OR raw_content LIKE ?1 LIMIT ?2"
                )?;
                
                let rows = stmt.query_map(params![pattern, limit as i64], |row| {
                    Self::row_to_node(row)
                })?;
                
                let mut nodes = Vec::new();
                for row in rows {
                    nodes.push(row?);
                }
                Ok(nodes)
            }
        }
    }
    
    /// Save a wiki node
    pub fn save(&self, node: &WikiNode) -> Result<(), DatabaseError> {
        let source_artifacts = serde_json::to_string(&node.provenance.source_artifacts)
            .map_err(|e| DatabaseError::Query(e.to_string()))?;
        
        let mentioned_dates = serde_json::to_string(&node.temporal_vectors.mentioned_dates)
            .map_err(|e| DatabaseError::Query(e.to_string()))?;
        
        let version_clock = serde_json::to_string(&node.version)
            .map_err(|e| DatabaseError::Query(e.to_string()))?;
        
        let embedding_bytes = node.embedding.as_ref()
            .map(|e| embedding_to_bytes(e));
        
        self.conn.execute(
            "INSERT OR REPLACE INTO wiki_nodes (
                id, node_type, title, slug, raw_content,
                source_artifacts, compiled_at, model_version, confidence_score,
                compilation_duration_ms, schema_version, mentioned_dates, fiscal_quarter,
                privacy_classification, encryption_status, version_clock, embedding,
                created_at, updated_at
            ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14, ?15, ?16, ?17, ?18, ?19)",
            params![
                node.id.as_bytes(),
                format!("{:?}", node.node_type).to_lowercase(),
                &node.title,
                node.slug(),
                &node.raw_content,
                source_artifacts,
                node.provenance.compiled_at.to_rfc3339(),
                &node.provenance.model_version,
                node.provenance.confidence_score,
                node.provenance.compilation_duration_ms.map(|d| d as i64),
                &node.provenance.schema_version,
                mentioned_dates,
                node.temporal_vectors.fiscal_quarter.as_ref(),
                format!("{:?}", node.privacy_classification).to_lowercase(),
                format!("{:?}", node.encryption_status).to_lowercase(),
                version_clock,
                embedding_bytes.as_ref().map(|b| b.as_slice()),
                node.created_at.to_rfc3339(),
                node.updated_at.to_rfc3339(),
            ],
        )?;
        
        // Save links
        self.save_links(node)?;
        
        debug!("Saved wiki node: {} ({})", node.id, node.title);
        Ok(())
    }
    
    /// Save wiki links
    fn save_links(&self, node: &WikiNode) -> Result<(), DatabaseError> {
        // Delete existing links
        self.conn.execute(
            "DELETE FROM wiki_links WHERE source_id = ?1",
            [node.id.as_bytes()],
        )?;
        
        // Insert forward links
        for link in &node.forward_links {
            self.conn.execute(
                "INSERT INTO wiki_links (source_id, target_id, target_title, context, created_at)
                 VALUES (?1, ?2, ?3, ?4, ?5)",
                params![
                    node.id.as_bytes(),
                    link.target_id.as_bytes(),
                    &link.target_title,
                    link.context.as_ref(),
                    link.created_at.to_rfc3339(),
                ],
            )?;
        }
        
        Ok(())
    }
    
    /// Load backlinks for a node
    pub fn load_backlinks(&self, node_id: Uuid) -> Result<Vec<WikiLink>, DatabaseError> {
        let mut stmt = self.conn.prepare(
            "SELECT target_id, target_title, context, created_at FROM wiki_links
             WHERE target_id = ?1"
        )?;
        
        let rows = stmt.query_map([node_id.as_bytes()], |row| {
            let target_bytes: Vec<u8> = row.get(0)?;
            let target_id = Uuid::from_slice(&target_bytes)
                .map_err(|e| rusqlite::Error::FromSqlConversionFailure(
                    0,
                    rusqlite::types::Type::Blob,
                    Box::new(e),
                ))?;
            
            let created_at: String = row.get(3)?;
            
            Ok(WikiLink {
                target_id,
                target_title: row.get(1)?,
                context: row.get(2)?,
                created_at: DateTime::parse_from_rfc3339(&created_at)
                    .map_err(|e| rusqlite::Error::FromSqlConversionFailure(
                        0,
                        rusqlite::types::Type::Text,
                        Box::new(e),
                    ))?.with_timezone(&Utc),
            })
        })?;
        
        let mut links = Vec::new();
        for row in rows {
            links.push(row?);
        }
        
        Ok(links)
    }
    
    /// Delete a wiki node
    pub fn delete(&self, id: Uuid) -> Result<bool, DatabaseError> {
        let rows = self.conn.execute(
            "DELETE FROM wiki_nodes WHERE id = ?1",
            [id.as_bytes()],
        )?;
        
        Ok(rows > 0)
    }
    
    /// Convert database row to WikiNode
    fn row_to_node(row: &Row) -> Result<WikiNode, rusqlite::Error> {
        let id_bytes: Vec<u8> = row.get("id")?;
        let id = Uuid::from_slice(&id_bytes)
            .map_err(|e| rusqlite::Error::FromSqlConversionFailure(
                0,
                rusqlite::types::Type::Blob,
                Box::new(e),
            ))?;
        
        let node_type_str: String = row.get("node_type")?;
        let node_type = parse_node_type(&node_type_str);
        
        let source_artifacts_str: String = row.get("source_artifacts")?;
        let source_artifacts: Vec<Uuid> = serde_json::from_str(&source_artifacts_str)
            .map_err(|e| rusqlite::Error::FromSqlConversionFailure(
                0,
                rusqlite::types::Type::Text,
                Box::new(e),
            ))?;
        
        let compiled_at: String = row.get("compiled_at")?;
        let mentioned_dates_str: String = row.get("mentioned_dates")?;
        let mentioned_dates: Vec<DateTime<Utc>> = serde_json::from_str(&mentioned_dates_str)
            .map_err(|e| rusqlite::Error::FromSqlConversionFailure(
                0,
                rusqlite::types::Type::Text,
                Box::new(e),
            ))?;
        
        let version_clock_str: String = row.get("version_clock")?;
        let version_clock: VectorClock = serde_json::from_str(&version_clock_str)
            .map_err(|e| rusqlite::Error::FromSqlConversionFailure(
                0,
                rusqlite::types::Type::Text,
                Box::new(e),
            ))?;
        
        let privacy_str: String = row.get("privacy_classification")?;
        let encryption_str: String = row.get("encryption_status")?;
        
        let created_at: String = row.get("created_at")?;
        let updated_at: String = row.get("updated_at")?;
        
        let embedding = match row.get::<_, Option<Vec<u8>>>("embedding")? {
            Some(b) => Some(bytes_to_embedding(&b)?),
            None => None,
        };
        
        Ok(WikiNode {
            id,
            node_type,
            title: row.get("title")?,
            content: Vec::new(), // Parsed on demand
            raw_content: row.get("raw_content")?,
            backlinks: Vec::new(), // Loaded separately
            forward_links: Vec::new(), // Loaded separately
            provenance: Provenance {
                source_artifacts,
                compiled_at: DateTime::parse_from_rfc3339(&compiled_at)
                    .map_err(|e| rusqlite::Error::FromSqlConversionFailure(
                        0,
                        rusqlite::types::Type::Text,
                        Box::new(e),
                    ))?.with_timezone(&Utc),
                model_version: row.get("model_version")?,
                confidence_score: row.get("confidence_score")?,
                compilation_duration_ms: row.get::<_, Option<i64>>("compilation_duration_ms")?
                    .map(|d| d as u64),
                schema_version: row.get("schema_version")?,
            },
            temporal_vectors: TemporalVectors {
                mentioned_dates,
                fiscal_quarter: row.get("fiscal_quarter")?,
                day_of_week: None,
                hour_of_day: None,
            },
            privacy_classification: parse_privacy_classification(&privacy_str),
            encryption_status: parse_encryption_status(&encryption_str),
            version: version_clock,
            created_at: DateTime::parse_from_rfc3339(&created_at)
                .map_err(|e| rusqlite::Error::FromSqlConversionFailure(
                    0,
                    rusqlite::types::Type::Text,
                    Box::new(e),
                ))?.with_timezone(&Utc),
            updated_at: DateTime::parse_from_rfc3339(&updated_at)
                .map_err(|e| rusqlite::Error::FromSqlConversionFailure(
                    0,
                    rusqlite::types::Type::Text,
                    Box::new(e),
                ))?.with_timezone(&Utc),
            embedding,
        })
    }
}

/// Ingestion Queue Repository
pub struct IngestionQueueRepository<'a> {
    conn: &'a Connection,
}

impl<'a> IngestionQueueRepository<'a> {
    pub fn new(conn: &'a Connection) -> Self {
        Self { conn }
    }
    
    /// Find entry by ID
    pub fn find_by_id(&self, id: Uuid) -> Result<Option<IngestionQueueEntry>, DatabaseError> {
        let mut stmt = self.conn.prepare(
            "SELECT * FROM ingestion_queue WHERE id = ?1"
        )?;
        
        let result = stmt.query_row([id.as_bytes()], |row| {
            Self::row_to_entry(row)
        }).optional()?;
        
        Ok(result)
    }
    
    /// Find entry by raw UUID
    pub fn find_by_raw_uuid(&self, raw_uuid: Uuid) -> Result<Option<IngestionQueueEntry>, DatabaseError> {
        let mut stmt = self.conn.prepare(
            "SELECT * FROM ingestion_queue WHERE raw_uuid = ?1"
        )?;
        
        let result = stmt.query_row([raw_uuid.as_bytes()], |row| {
            Self::row_to_entry(row)
        }).optional()?;
        
        Ok(result)
    }
    
    /// Get pending entries ordered by priority
    pub fn get_pending(&self, limit: usize) -> Result<Vec<IngestionQueueEntry>, DatabaseError> {
        let mut stmt = self.conn.prepare(
            "SELECT * FROM ingestion_queue 
             WHERE status = 'pending' OR (status = 'failed' AND retry_count < 3)
             ORDER BY priority ASC, created_at ASC
             LIMIT ?1"
        )?;
        
        let rows = stmt.query_map([limit as i64], |row| {
            Self::row_to_entry(row)
        })?;
        
        let mut entries = Vec::new();
        for row in rows {
            entries.push(row?);
        }
        
        Ok(entries)
    }
    
    /// Save queue entry
    pub fn save(&self, entry: &IngestionQueueEntry) -> Result<(), DatabaseError> {
        self.conn.execute(
            "INSERT OR REPLACE INTO ingestion_queue (
                id, raw_uuid, status, retry_count, priority,
                created_at, started_at, completed_at, error_message
            ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)",
            params![
                entry.id.as_bytes(),
                entry.raw_uuid.as_bytes(),
                format!("{:?}", entry.status).to_lowercase(),
                entry.retry_count as i64,
                entry.priority as i64,
                entry.created_at.to_rfc3339(),
                entry.started_at.map(|d| d.to_rfc3339()),
                entry.completed_at.map(|d| d.to_rfc3339()),
                entry.error_message.as_ref(),
            ],
        )?;
        
        Ok(())
    }
    
    /// Update status
    pub fn update_status(&self, id: Uuid, status: IngestionStatus) -> Result<(), DatabaseError> {
        let now = Utc::now();
        
        let (started_at, completed_at) = match status {
            IngestionStatus::Processing => (Some(now), None),
            IngestionStatus::Compiled | IngestionStatus::Failed => {
                let existing = self.find_by_id(id)?;
                (existing.and_then(|e| e.started_at), Some(now))
            }
            _ => (None, None),
        };
        
        self.conn.execute(
            "UPDATE ingestion_queue SET 
                status = ?1, started_at = ?2, completed_at = ?3
             WHERE id = ?4",
            params![
                format!("{:?}", status).to_lowercase(),
                started_at.map(|d| d.to_rfc3339()),
                completed_at.map(|d| d.to_rfc3339()),
                id.as_bytes(),
            ],
        )?;
        
        Ok(())
    }
    
    /// Delete entry
    pub fn delete(&self, id: Uuid) -> Result<bool, DatabaseError> {
        let rows = self.conn.execute(
            "DELETE FROM ingestion_queue WHERE id = ?1",
            [id.as_bytes()],
        )?;
        
        Ok(rows > 0)
    }
    
    /// Convert row to entry
    fn row_to_entry(row: &Row) -> Result<IngestionQueueEntry, rusqlite::Error> {
        let id_bytes: Vec<u8> = row.get("id")?;
        let id = Uuid::from_slice(&id_bytes)
            .map_err(|e| rusqlite::Error::FromSqlConversionFailure(
                0,
                rusqlite::types::Type::Blob,
                Box::new(e),
            ))?;
        
        let raw_uuid_bytes: Vec<u8> = row.get("raw_uuid")?;
        let raw_uuid = Uuid::from_slice(&raw_uuid_bytes)
            .map_err(|e| rusqlite::Error::FromSqlConversionFailure(
                0,
                rusqlite::types::Type::Blob,
                Box::new(e),
            ))?;
        
        let status_str: String = row.get("status")?;
        let status = parse_status(&status_str);
        
        let created_at: String = row.get("created_at")?;
        let started_at: Option<String> = row.get("started_at")?;
        let completed_at: Option<String> = row.get("completed_at")?;
        
        Ok(IngestionQueueEntry {
            id,
            raw_uuid,
            status,
            retry_count: row.get::<_, i64>("retry_count")? as u32,
            priority: row.get::<_, i64>("priority")? as u32,
            created_at: DateTime::parse_from_rfc3339(&created_at)
                .map_err(|e| rusqlite::Error::FromSqlConversionFailure(
                    0,
                    rusqlite::types::Type::Text,
                    Box::new(e),
                ))?.with_timezone(&Utc),
            started_at: started_at.map(|s| DateTime::parse_from_rfc3339(&s)
                .map_err(|e| rusqlite::Error::FromSqlConversionFailure(
                    0,
                    rusqlite::types::Type::Text,
                    Box::new(e),
                ))
                .map(|dt| dt.with_timezone(&Utc))
                .transpose()?,
            completed_at: completed_at.map(|s| DateTime::parse_from_rfc3339(&s)
                .map_err(|e| rusqlite::Error::FromSqlConversionFailure(
                    0,
                    rusqlite::types::Type::Text,
                    Box::new(e),
                ))
                .map(|dt| dt.with_timezone(&Utc))
                .transpose()?,)
            error_message: row.get("error_message")?,
        })
    }
}

// Helper functions

fn parse_status(s: &str) -> IngestionStatus {
    match s {
        "pending" => IngestionStatus::Pending,
        "processing" => IngestionStatus::Processing,
        "compiled" => IngestionStatus::Compiled,
        "failed" => IngestionStatus::Failed,
        "excluded" => IngestionStatus::Excluded,
        _ => IngestionStatus::Pending,
    }
}

fn parse_node_type(s: &str) -> WikiNodeType {
    match s {
        "entity" => WikiNodeType::Entity,
        "concept" => WikiNodeType::Concept,
        "chronology" => WikiNodeType::Chronology,
        "index" => WikiNodeType::Index,
        _ => WikiNodeType::Entity,
    }
}

fn parse_privacy_classification(s: &str) -> PrivacyClassification {
    match s {
        "public" => PrivacyClassification::Public,
        "personal" => PrivacyClassification::Personal,
        "sensitive" => PrivacyClassification::Sensitive,
        "financial" => PrivacyClassification::Financial,
        _ => PrivacyClassification::Personal,
    }
}

fn parse_encryption_status(s: &str) -> EncryptionStatus {
    match s {
        "plaintext" => EncryptionStatus::Plaintext,
        "aes256_gcm" => EncryptionStatus::Aes256Gcm,
        _ => EncryptionStatus::Plaintext,
    }
}

fn embedding_to_bytes(embedding: &[f32]) -> Vec<u8> {
    embedding.iter()
        .flat_map(|f| f.to_le_bytes())
        .collect()
}

fn bytes_to_embedding(bytes: &[u8]) -> Result<Vec<f32>, DatabaseError> {
    if bytes.len() % 4 != 0 {
        return Err(DatabaseError::Query("Invalid embedding byte length".to_string()));
    }
    bytes.chunks_exact(4)
        .map(|chunk| {
            let arr: [u8; 4] = chunk.try_into()
                .map_err(|_| DatabaseError::Query("Invalid byte chunk".to_string()))?;
            Ok(f32::from_le_bytes(arr))
        })
        .collect()
}

// Helper trait for Geohash
impl Geohash {
    fn from_string(s: String) -> Self {
        Self(s)
    }
    
    fn as_str(&self) -> &str {
        &self.0
    }
}

// Helper trait for DeviceFingerprint
impl DeviceFingerprint {
    fn from_string(s: String) -> Self {
        Self(s)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::database::init_database;
    
    fn create_test_artifact() -> RawArtifact {
        use crate::models::content_addressable_uuid;
        
        let content = b"test image data";
        RawArtifact {
            uuid: content_addressable_uuid(content),
            filename: "test.png".to_string(),
            binary: BlobPointer {
                path: std::path::PathBuf::from("/tmp/test.png"),
                content_hash: "abc123".to_string(),
            },
            captured_at: Utc::now(),
            device_id: DeviceFingerprint::new("test_device", b"salt"),
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
    fn test_raw_artifact_crud() {
        let mut conn = Connection::open_in_memory().expect("Failed to open in-memory database");
        run_migrations(&mut conn).expect("Failed to run migrations");
        
        let repo = RawArtifactRepository::new(&conn);
        let artifact = create_test_artifact();
        
        // Save
        repo.save(&artifact).expect("Failed to save artifact");
        
        // Find
        let found = repo.find_by_id(artifact.uuid).expect("Failed to find artifact");
        assert!(found.is_some());
        assert_eq!(found.expect("Artifact not found").filename, artifact.filename);
        
        // Update status
        repo.update_status(artifact.uuid, IngestionStatus::Compiled).expect("Failed to update status");
        let updated = repo.find_by_id(artifact.uuid).expect("Failed to find artifact").expect("Artifact not found");
        assert_eq!(updated.status, IngestionStatus::Compiled);
        
        // Delete
        assert!(repo.delete(artifact.uuid).expect("Failed to delete artifact"));
        assert!(repo.find_by_id(artifact.uuid).expect("Failed to find artifact").is_none());
    }
    
    #[test]
    fn test_wiki_node_crud() {
        let mut conn = Connection::open_in_memory().expect("Failed to open in-memory database");
        run_migrations(&mut conn).expect("Failed to run migrations");
        
        let repo = WikiNodeRepository::new(&conn);
        
        let provenance = Provenance::new(vec![Uuid::new_v4()], "test_model", 0.95);
        let node = WikiNode::new(
            "Test Node",
            WikiNodeType::Entity,
            "This is test content.",
            provenance,
        );
        
        // Save
        repo.save(&node).expect("Failed to save node");
        
        // Find
        let found = repo.find_by_id(node.id).expect("Failed to find node");
        assert!(found.is_some());
        
        // Find by slug
        let by_slug = repo.find_by_slug(&node.slug()).expect("Failed to find by slug");
        assert!(by_slug.is_some());
        
        // Delete
        assert!(repo.delete(node.id).expect("Failed to delete node"));
    }
    
    #[test]
    fn test_ingestion_queue() {
        let mut conn = Connection::open_in_memory().expect("Failed to open in-memory database");
        run_migrations(&mut conn).expect("Failed to run migrations");
        
        let repo = IngestionQueueRepository::new(&conn);
        let raw_uuid = Uuid::new_v4();
        let entry = IngestionQueueEntry::new(raw_uuid, 1);
        
        // Save
        repo.save(&entry).expect("Failed to save entry");
        
        // Find
        let found = repo.find_by_id(entry.id).expect("Failed to find entry");
        assert!(found.is_some());
        
        // Get pending
        let pending = repo.get_pending(10).expect("Failed to get pending entries");
        assert_eq!(pending.len(), 1);
        
        // Update status
        repo.update_status(entry.id, IngestionStatus::Processing).expect("Failed to update status");
        let updated = repo.find_by_id(entry.id).expect("Failed to find entry").expect("Entry not found");
        assert_eq!(updated.status, IngestionStatus::Processing);
        assert!(updated.started_at.is_some());
    }
}
