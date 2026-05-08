// SPEC v2.0 SECTION 2.1: Database Module
// SQLite database operations for Truffle Desktop

use anyhow::{Context, Result};
use rusqlite::{params, Connection, OptionalExtension};
use std::path::PathBuf;
use std::sync::Arc;
use tokio::sync::Mutex;
use uuid::Uuid;

use crate::models::*;

pub struct Database {
    conn: Arc<Mutex<Connection>>,
}

impl Database {
    pub async fn new() -> Result<Self> {
        let db_path = get_db_path()?;
        let conn = Connection::open(&db_path)?;
        
        // Initialize schema
        Self::init_schema(&conn)?;
        
        Ok(Self {
            conn: Arc::new(Mutex::new(conn)),
        })
    }

    fn init_schema(conn: &Connection) -> Result<()> {
        // Raw artifacts table
        conn.execute(
            "CREATE TABLE IF NOT EXISTS raw_artifacts (
                uuid TEXT PRIMARY KEY,
                filename TEXT NOT NULL,
                path TEXT NOT NULL,
                captured_at TEXT NOT NULL,
                device_id TEXT NOT NULL,
                app_context TEXT,
                geohash TEXT,
                metadata TEXT,
                status TEXT NOT NULL DEFAULT 'pending',
                created_at TEXT NOT NULL DEFAULT (datetime('now')),
                updated_at TEXT NOT NULL DEFAULT (datetime('now'))
            )",
            [],
        )?;

        // Wiki nodes table
        conn.execute(
            "CREATE TABLE IF NOT EXISTS wiki_nodes (
                id TEXT PRIMARY KEY,
                node_type TEXT NOT NULL,
                title TEXT NOT NULL,
                content TEXT NOT NULL,
                backlinks TEXT,
                forward_links TEXT,
                provenance TEXT,
                temporal_vectors TEXT,
                privacy_classification TEXT NOT NULL DEFAULT 'personal',
                encryption_status TEXT NOT NULL DEFAULT 'plaintext',
                version INTEGER NOT NULL DEFAULT 1,
                created_at TEXT NOT NULL DEFAULT (datetime('now')),
                updated_at TEXT NOT NULL DEFAULT (datetime('now'))
            )",
            [],
        )?;

        // Compilation logs table
        conn.execute(
            "CREATE TABLE IF NOT EXISTS compilation_logs (
                id TEXT PRIMARY KEY,
                artifact_id TEXT,
                level TEXT NOT NULL,
                message TEXT NOT NULL,
                timestamp TEXT NOT NULL DEFAULT (datetime('now')),
                FOREIGN KEY (artifact_id) REFERENCES raw_artifacts(uuid)
            )",
            [],
        )?;

        // Create indexes
        conn.execute(
            "CREATE INDEX IF NOT EXISTS idx_artifacts_status ON raw_artifacts(status)",
            [],
        )?;
        conn.execute(
            "CREATE INDEX IF NOT EXISTS idx_artifacts_captured ON raw_artifacts(captured_at)",
            [],
        )?;
        conn.execute(
            "CREATE INDEX IF NOT EXISTS idx_nodes_type ON wiki_nodes(node_type)",
            [],
        )?;
        conn.execute(
            "CREATE INDEX IF NOT EXISTS idx_nodes_title ON wiki_nodes(title)",
            [],
        )?;

        // Entities table
        conn.execute(
            "CREATE TABLE IF NOT EXISTS entities (
                id TEXT PRIMARY KEY,
                entity_type TEXT NOT NULL,
                name TEXT NOT NULL,
                description TEXT,
                attributes TEXT,
                confidence_score REAL NOT NULL DEFAULT 1.0,
                merged_into TEXT,
                created_at TEXT NOT NULL DEFAULT (datetime('now')),
                updated_at TEXT NOT NULL DEFAULT (datetime('now'))
            )",
            [],
        )?;

        // Relationships table
        conn.execute(
            "CREATE TABLE IF NOT EXISTS relationships (
                id TEXT PRIMARY KEY,
                source_id TEXT NOT NULL,
                target_id TEXT NOT NULL,
                relationship_type TEXT NOT NULL,
                properties TEXT,
                confidence_score REAL NOT NULL DEFAULT 1.0,
                created_at TEXT NOT NULL DEFAULT (datetime('now')),
                FOREIGN KEY (source_id) REFERENCES entities(id),
                FOREIGN KEY (target_id) REFERENCES entities(id)
            )",
            [],
        )?;

        // Meetings table
        conn.execute(
            "CREATE TABLE IF NOT EXISTS meetings (
                id TEXT PRIMARY KEY,
                title TEXT NOT NULL,
                occurred_at TEXT NOT NULL,
                participants TEXT,
                source_artifact_id TEXT NOT NULL,
                extracted_at TEXT NOT NULL DEFAULT (datetime('now'))
            )",
            [],
        )?;

        // Decisions table
        conn.execute(
            "CREATE TABLE IF NOT EXISTS decisions (
                id TEXT PRIMARY KEY,
                meeting_id TEXT NOT NULL,
                description TEXT NOT NULL,
                decided_by TEXT,
                related_entities TEXT,
                confidence_score REAL NOT NULL DEFAULT 1.0,
                extracted_at TEXT NOT NULL DEFAULT (datetime('now')),
                FOREIGN KEY (meeting_id) REFERENCES meetings(id)
            )",
            [],
        )?;

        // Action items table
        conn.execute(
            "CREATE TABLE IF NOT EXISTS action_items (
                id TEXT PRIMARY KEY,
                meeting_id TEXT NOT NULL,
                description TEXT NOT NULL,
                assignee TEXT,
                due_date TEXT,
                status TEXT NOT NULL DEFAULT 'pending',
                related_entities TEXT,
                extracted_at TEXT NOT NULL DEFAULT (datetime('now')),
                FOREIGN KEY (meeting_id) REFERENCES meetings(id)
            )",
            [],
        )?;

        // Contradictions table
        conn.execute(
            "CREATE TABLE IF NOT EXISTS contradictions (
                id TEXT PRIMARY KEY,
                entity_id TEXT NOT NULL,
                contradiction_type TEXT NOT NULL,
                description TEXT NOT NULL,
                severity TEXT NOT NULL DEFAULT 'medium',
                related_entity_ids TEXT,
                resolution_type TEXT,
                resolution_note TEXT,
                resolved_at TEXT,
                created_at TEXT NOT NULL DEFAULT (datetime('now')),
                FOREIGN KEY (entity_id) REFERENCES entities(id)
            )",
            [],
        )?;

        // Create indexes for new tables
        conn.execute(
            "CREATE INDEX IF NOT EXISTS idx_entities_type ON entities(entity_type)",
            [],
        )?;
        conn.execute(
            "CREATE INDEX IF NOT EXISTS idx_entities_name ON entities(name)",
            [],
        )?;
        conn.execute(
            "CREATE INDEX IF NOT EXISTS idx_relationships_source ON relationships(source_id)",
            [],
        )?;
        conn.execute(
            "CREATE INDEX IF NOT EXISTS idx_relationships_target ON relationships(target_id)",
            [],
        )?;
        conn.execute(
            "CREATE INDEX IF NOT EXISTS idx_contradictions_entity ON contradictions(entity_id)",
            [],
        )?;
        conn.execute(
            "CREATE INDEX IF NOT EXISTS idx_contradictions_severity ON contradictions(severity)",
            [],
        )?;

        Ok(())
    }

    // ==================== RAW ARTIFACT OPERATIONS ====================

    pub async fn get_raw_artifacts(&self, limit: usize, offset: usize) -> Result<Vec<RawArtifact>> {
        let conn = self.conn.lock().await;
        let mut stmt = conn.prepare(
            "SELECT uuid, filename, path, captured_at, device_id, app_context, 
                    geohash, metadata, status 
             FROM raw_artifacts 
             ORDER BY captured_at DESC 
             LIMIT ? OFFSET ?"
        )?;

        let artifacts = stmt
            .query_map(params![limit, offset], |row| {
                Ok(RawArtifact {
                    uuid: row.get(0)?,
                    filename: row.get(1)?,
                    path: row.get(2)?,
                    captured_at: row.get(3)?,
                    device_id: row.get(4)?,
                    app_context: row.get::<_, Option<String>>(5)?
                        .and_then(|s| serde_json::from_str(&s).ok()),
                    geohash: row.get(6)?,
                    metadata: row.get::<_, Option<String>>(7)?
                        .and_then(|s| serde_json::from_str(&s).ok())
                        .unwrap_or_default(),
                    status: row.get(8)?,
                })
            })?
            .collect::<Result<Vec<_>, _>>()?;

        Ok(artifacts)
    }

    pub async fn get_raw_artifact(&self, id: Uuid) -> Result<RawArtifact> {
        let conn = self.conn.lock().await;
        let artifact = conn.query_row(
            "SELECT uuid, filename, path, captured_at, device_id, app_context, 
                    geohash, metadata, status 
             FROM raw_artifacts 
             WHERE uuid = ?",
            [id.to_string()],
            |row| {
                Ok(RawArtifact {
                    uuid: row.get(0)?,
                    filename: row.get(1)?,
                    path: row.get(2)?,
                    captured_at: row.get(3)?,
                    device_id: row.get(4)?,
                    app_context: row.get::<_, Option<String>>(5)?
                        .and_then(|s| serde_json::from_str(&s).ok()),
                    geohash: row.get(6)?,
                    metadata: row.get::<_, Option<String>>(7)?
                        .and_then(|s| serde_json::from_str(&s).ok())
                        .unwrap_or_default(),
                    status: row.get(8)?,
                })
            },
        )?;

        Ok(artifact)
    }

    pub async fn import_raw_artifact(
        &self,
        path: &str,
        app_context: Option<AppContext>,
    ) -> Result<RawArtifact> {
        let id = Uuid::new_v4();
        let filename = std::path::Path::new(path)
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("unknown")
            .to_string();

        let artifact = RawArtifact {
            uuid: id.to_string(),
            filename,
            path: path.to_string(),
            captured_at: chrono::Utc::now().to_rfc3339(),
            device_id: get_device_id(),
            app_context,
            geohash: None,
            metadata: ArtifactMetadata {
                size_bytes: 0,
                width: 0,
                height: 0,
                ocr_text: None,
                embedding_vector: None,
            },
            status: ArtifactStatus::Pending,
        };

        let conn = self.conn.lock().await;
        conn.execute(
            "INSERT INTO raw_artifacts (uuid, filename, path, captured_at, device_id, 
                                        app_context, geohash, metadata, status)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)",
            params![
                artifact.uuid,
                artifact.filename,
                artifact.path,
                artifact.captured_at,
                artifact.device_id,
                artifact.app_context.as_ref().map(|c| serde_json::to_string(c).unwrap()),
                artifact.geohash,
                serde_json::to_string(&artifact.metadata).unwrap(),
                serde_json::to_string(&artifact.status).unwrap(),
            ],
        )?;

        Ok(artifact)
    }

    pub async fn delete_raw_artifact(&self, id: Uuid) -> Result<()> {
        let conn = self.conn.lock().await;
        conn.execute(
            "DELETE FROM raw_artifacts WHERE uuid = ?",
            [id.to_string()],
        )?;
        Ok(())
    }

    pub async fn get_thumbnail(&self, id: Uuid, width: u32) -> Result<Vec<u8>> {
        // TODO: Generate thumbnail on demand
        Ok(vec![])
    }

    // ==================== WIKI NODE OPERATIONS ====================

    pub async fn get_wiki_nodes(&self, node_type: Option<String>) -> Result<Vec<WikiNodeSummary>> {
        let conn = self.conn.lock().await;
        
        let query = if let Some(nt) = node_type {
            format!(
                "SELECT id, node_type, title, content, updated_at 
                 FROM wiki_nodes 
                 WHERE node_type = '{}' 
                 ORDER BY updated_at DESC",
                nt
            )
        } else {
            "SELECT id, node_type, title, content, updated_at 
             FROM wiki_nodes 
             ORDER BY updated_at DESC".to_string()
        };

        let mut stmt = conn.prepare(&query)?;
        let nodes = stmt
            .query_map([], |row| {
                let content: String = row.get(3)?;
                Ok(WikiNodeSummary {
                    id: row.get(0)?,
                    node_type: row.get(1)?,
                    title: row.get(2)?,
                    excerpt: content.lines().next().unwrap_or("").to_string(),
                    updated_at: row.get(4)?,
                    backlink_count: 0, // TODO: Calculate
                })
            })?
            .collect::<Result<Vec<_>, _>>()?;

        Ok(nodes)
    }

    pub async fn get_wiki_node(&self, id: Uuid) -> Result<WikiNode> {
        let conn = self.conn.lock().await;
        let node = conn.query_row(
            "SELECT * FROM wiki_nodes WHERE id = ?",
            [id.to_string()],
            |row| {
                Ok(WikiNode {
                    id: row.get("id")?,
                    node_type: row.get("node_type")?,
                    title: row.get("title")?,
                    content: row.get("content")?,
                    backlinks: vec![],
                    forward_links: vec![],
                    provenance: None,
                    temporal_vectors: None,
                    privacy_classification: row.get("privacy_classification")?,
                    encryption_status: row.get("encryption_status")?,
                    version: row.get("version")?,
                    created_at: row.get("created_at")?,
                    updated_at: row.get("updated_at")?,
                })
            },
        )?;

        Ok(node)
    }

    pub async fn create_wiki_node(
        &self,
        title: &str,
        content: &str,
        node_type: &str,
    ) -> Result<WikiNode> {
        let id = Uuid::new_v4();
        let now = chrono::Utc::now().to_rfc3339();

        let conn = self.conn.lock().await;
        conn.execute(
            "INSERT INTO wiki_nodes (id, node_type, title, content, created_at, updated_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
            params![id.to_string(), node_type, title, content, now, now],
        )?;

        Ok(WikiNode {
            id: id.to_string(),
            node_type: node_type.to_string(),
            title: title.to_string(),
            content: content.to_string(),
            backlinks: vec![],
            forward_links: vec![],
            provenance: None,
            temporal_vectors: None,
            privacy_classification: PrivacyClassification::Personal,
            encryption_status: EncryptionStatus::Plaintext,
            version: 1,
            created_at: now.clone(),
            updated_at: now,
        })
    }

    pub async fn update_wiki_node(
        &self,
        id: Uuid,
        title: Option<&str>,
        content: Option<&str>,
    ) -> Result<WikiNode> {
        let conn = self.conn.lock().await;
        let now = chrono::Utc::now().to_rfc3339();

        if let Some(t) = title {
            conn.execute(
                "UPDATE wiki_nodes SET title = ?1, updated_at = ?2 WHERE id = ?3",
                params![t, now, id.to_string()],
            )?;
        }

        if let Some(c) = content {
            conn.execute(
                "UPDATE wiki_nodes SET content = ?1, updated_at = ?2, version = version + 1 WHERE id = ?3",
                params![c, now, id.to_string()],
            )?;
        }

        drop(conn);
        self.get_wiki_node(id).await
    }

    pub async fn delete_wiki_node(&self, id: Uuid) -> Result<()> {
        let conn = self.conn.lock().await;
        conn.execute(
            "DELETE FROM wiki_nodes WHERE id = ?",
            [id.to_string()],
        )?;
        Ok(())
    }

    pub async fn search_wiki_nodes(&self, query: &str) -> Result<Vec<WikiNodeSummary>> {
        let conn = self.conn.lock().await;
        let mut stmt = conn.prepare(
            "SELECT id, node_type, title, content, updated_at 
             FROM wiki_nodes 
             WHERE title LIKE ?1 OR content LIKE ?1
             ORDER BY updated_at DESC"
        )?;

        let search_pattern = format!("%{}%", query);
        let nodes = stmt
            .query_map(params![search_pattern], |row| {
                let content: String = row.get(3)?;
                Ok(WikiNodeSummary {
                    id: row.get(0)?,
                    node_type: row.get(1)?,
                    title: row.get(2)?,
                    excerpt: content.lines().next().unwrap_or("").to_string(),
                    updated_at: row.get(4)?,
                    backlink_count: 0,
                })
            })?
            .collect::<Result<Vec<_>, _>>()?;

        Ok(nodes)
    }

    pub async fn get_backlinks(&self, id: Uuid) -> Result<Vec<WikiLink>> {
        // TODO: Implement backlink tracking
        Ok(vec![])
    }

    // ==================== COMPILATION LOGS ====================

    pub async fn get_compilation_logs(&self, artifact_id: Option<Uuid>) -> Result<Vec<CompilationLog>> {
        let conn = self.conn.lock().await;
        
        let query = if artifact_id.is_some() {
            "SELECT id, artifact_id, level, message, timestamp 
             FROM compilation_logs 
             WHERE artifact_id = ?1
             ORDER BY timestamp DESC"
        } else {
            "SELECT id, artifact_id, level, message, timestamp 
             FROM compilation_logs 
             ORDER BY timestamp DESC 
             LIMIT 100"
        };

        let mut stmt = conn.prepare(query)?;
        
        let logs = if let Some(id) = artifact_id {
            stmt.query_map(params![id.to_string()], |row| {
                Ok(CompilationLog {
                    id: row.get(0)?,
                    artifact_id: row.get(1)?,
                    level: row.get(2)?,
                    message: row.get(3)?,
                    timestamp: row.get(4)?,
                })
            })?
            .collect::<Result<Vec<_>, _>>()?
        } else {
            stmt.query_map([], |row| {
                Ok(CompilationLog {
                    id: row.get(0)?,
                    artifact_id: row.get(1)?,
                    level: row.get(2)?,
                    message: row.get(3)?,
                    timestamp: row.get(4)?,
                })
            })?
            .collect::<Result<Vec<_>, _>>()?
        };

        Ok(logs)
    }

    // ==================== SEARCH OPERATIONS ====================

    pub async fn search_content(&self, query: &str, search_type: &str) -> Result<SearchResults> {
        let artifacts = self.get_raw_artifacts(50, 0).await?;
        let wiki_nodes = self.search_wiki_nodes(query).await?;

        Ok(SearchResults {
            query: query.to_string(),
            artifacts,
            wiki_nodes,
            total_count: artifacts.len() + wiki_nodes.len(),
        })
    }

    pub async fn semantic_search(&self, query: &str, limit: usize) -> Result<Vec<SemanticSearchResult>> {
        // TODO: Implement semantic search with sqlite-vec
        Ok(vec![])
    }

    // ==================== SYSTEM OPERATIONS ====================

    pub async fn get_storage_info(&self) -> Result<StorageInfo> {
        let conn = self.conn.lock().await;
        
        let artifact_count: i64 = conn.query_row(
            "SELECT COUNT(*) FROM raw_artifacts",
            [],
            |row| row.get(0),
        )?;

        let node_count: i64 = conn.query_row(
            "SELECT COUNT(*) FROM wiki_nodes",
            [],
            |row| row.get(0),
        )?;

        Ok(StorageInfo {
            raw_storage_bytes: 0,
            wiki_storage_bytes: 0,
            thumbnail_cache_bytes: 0,
            total_bytes: 0,
            artifact_count: artifact_count as usize,
            node_count: node_count as usize,
        })
    }

    pub async fn export_to_markdown(&self, path: &str) -> Result<ExportResult> {
        // TODO: Implement markdown export
        Ok(ExportResult {
            success: true,
            path: path.to_string(),
            files_exported: 0,
            error: None,
        })
    }

    pub async fn get_schema(&self) -> Result<SchemaDefinition> {
        // TODO: Load schema from file
        Ok(SchemaDefinition {
            version: "1.0.0".to_string(),
            content: "".to_string(),
            updated_at: chrono::Utc::now().to_rfc3339(),
        })
    }

    pub async fn update_schema(&self, schema: &str) -> Result<()> {
        // TODO: Save schema to file
        Ok(())
    }

    // ==================== KNOWLEDGE GRAPH OPERATIONS ====================

    pub async fn get_entities(
        &self,
        _filter: Option<EntityFilter>,
        limit: usize,
        offset: usize,
    ) -> Result<Vec<EntitySummary>> {
        let conn = self.conn.lock().await;
        let mut stmt = conn.prepare(
            "SELECT id, entity_type, name, confidence_score, updated_at 
             FROM entities 
             ORDER BY updated_at DESC 
             LIMIT ? OFFSET ?"
        )?;

        let entities = stmt
            .query_map(params![limit, offset], |row| {
                Ok(EntitySummary {
                    id: row.get(0)?,
                    entity_type: row.get(1)?,
                    name: row.get(2)?,
                    confidence_score: row.get(3)?,
                    updated_at: row.get(4)?,
                })
            })?
            .collect::<Result<Vec<_>, _>>()?;

        Ok(entities)
    }

    pub async fn get_entity(&self, id: Uuid) -> Result<Entity> {
        let conn = self.conn.lock().await;
        let entity = conn.query_row(
            "SELECT id, entity_type, name, description, attributes, confidence_score, created_at, updated_at 
             FROM entities WHERE id = ?",
            [id.to_string()],
            |row| {
                Ok(Entity {
                    id: row.get(0)?,
                    entity_type: row.get(1)?,
                    name: row.get(2)?,
                    description: row.get(3)?,
                    attributes: row.get::<_, Option<String>>(4)?
                        .and_then(|s| serde_json::from_str(&s).ok())
                        .unwrap_or(serde_json::Value::Null),
                    confidence_score: row.get(5)?,
                    created_at: row.get(6)?,
                    updated_at: row.get(7)?,
                })
            },
        )?;

        Ok(entity)
    }

    pub async fn search_entities(&self, query: &str, limit: usize) -> Result<Vec<EntitySummary>> {
        let conn = self.conn.lock().await;
        let mut stmt = conn.prepare(
            "SELECT id, entity_type, name, confidence_score, updated_at 
             FROM entities 
             WHERE name LIKE ?1 OR description LIKE ?1
             ORDER BY confidence_score DESC 
             LIMIT ?"
        )?;

        let search_pattern = format!("%{}%", query);
        let entities = stmt
            .query_map(params![search_pattern, limit], |row| {
                Ok(EntitySummary {
                    id: row.get(0)?,
                    entity_type: row.get(1)?,
                    name: row.get(2)?,
                    confidence_score: row.get(3)?,
                    updated_at: row.get(4)?,
                })
            })?
            .collect::<Result<Vec<_>, _>>()?;

        Ok(entities)
    }

    pub async fn create_entity(&self, input: CreateEntityInput) -> Result<Entity> {
        let id = Uuid::new_v4();
        let now = chrono::Utc::now().to_rfc3339();

        let entity = Entity {
            id: id.to_string(),
            entity_type: input.entity_type,
            name: input.name,
            description: input.description,
            attributes: input.attributes.unwrap_or(serde_json::Value::Null),
            confidence_score: 1.0,
            created_at: now.clone(),
            updated_at: now,
        };

        let conn = self.conn.lock().await;
        conn.execute(
            "INSERT INTO entities (id, entity_type, name, description, attributes, confidence_score, created_at, updated_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
            params![
                entity.id,
                serde_json::to_string(&entity.entity_type).unwrap(),
                entity.name,
                entity.description,
                serde_json::to_string(&entity.attributes).unwrap(),
                entity.confidence_score,
                entity.created_at,
                entity.updated_at,
            ],
        )?;

        Ok(entity)
    }

    pub async fn update_entity(&self, id: Uuid, input: UpdateEntityInput) -> Result<Entity> {
        let conn = self.conn.lock().await;
        let now = chrono::Utc::now().to_rfc3339();

        if let Some(name) = input.name {
            conn.execute(
                "UPDATE entities SET name = ?1, updated_at = ?2 WHERE id = ?3",
                params![name, now, id.to_string()],
            )?;
        }

        if let Some(description) = input.description {
            conn.execute(
                "UPDATE entities SET description = ?1, updated_at = ?2 WHERE id = ?3",
                params![description, now, id.to_string()],
            )?;
        }

        if let Some(attributes) = input.attributes {
            conn.execute(
                "UPDATE entities SET attributes = ?1, updated_at = ?2 WHERE id = ?3",
                params![serde_json::to_string(&attributes).unwrap(), now, id.to_string()],
            )?;
        }

        if let Some(confidence) = input.confidence_score {
            conn.execute(
                "UPDATE entities SET confidence_score = ?1, updated_at = ?2 WHERE id = ?3",
                params![confidence, now, id.to_string()],
            )?;
        }

        drop(conn);
        self.get_entity(id).await
    }

    pub async fn delete_entity(&self, id: Uuid) -> Result<()> {
        let conn = self.conn.lock().await;
        conn.execute(
            "DELETE FROM entities WHERE id = ?",
            [id.to_string()],
        )?;
        Ok(())
    }

    pub async fn get_relationships(&self, entity_id: Uuid) -> Result<Vec<Relationship>> {
        let conn = self.conn.lock().await;
        let mut stmt = conn.prepare(
            "SELECT id, source_id, target_id, relationship_type, properties, confidence_score, created_at 
             FROM relationships 
             WHERE source_id = ?1 OR target_id = ?1
             ORDER BY created_at DESC"
        )?;

        let relationships = stmt
            .query_map(params![entity_id.to_string()], |row| {
                Ok(Relationship {
                    id: row.get(0)?,
                    source_id: row.get(1)?,
                    target_id: row.get(2)?,
                    relationship_type: row.get(3)?,
                    properties: row.get::<_, Option<String>>(4)?
                        .and_then(|s| serde_json::from_str(&s).ok()),
                    confidence_score: row.get(5)?,
                    created_at: row.get(6)?,
                })
            })?
            .collect::<Result<Vec<_>, _>>()?;

        Ok(relationships)
    }

    pub async fn create_relationship(&self, input: CreateRelationshipInput) -> Result<Relationship> {
        let id = Uuid::new_v4();
        let now = chrono::Utc::now().to_rfc3339();

        let relationship = Relationship {
            id: id.to_string(),
            source_id: input.source_id,
            target_id: input.target_id,
            relationship_type: input.relationship_type,
            properties: input.properties,
            confidence_score: 1.0,
            created_at: now,
        };

        let conn = self.conn.lock().await;
        conn.execute(
            "INSERT INTO relationships (id, source_id, target_id, relationship_type, properties, confidence_score, created_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
            params![
                relationship.id,
                relationship.source_id,
                relationship.target_id,
                relationship.relationship_type,
                relationship.properties.as_ref().map(|p| serde_json::to_string(p).unwrap()),
                relationship.confidence_score,
                relationship.created_at,
            ],
        )?;

        Ok(relationship)
    }

    pub async fn get_entity_graph(&self, entity_id: Uuid, depth: usize) -> Result<GraphNode> {
        let entity = self.get_entity(entity_id).await?;
        let relationships = self.get_relationships(entity_id).await?;

        let mut graph_edges = Vec::new();
        let mut children = Vec::new();

        if depth > 0 {
            for rel in &relationships {
                let child_id = if rel.source_id == entity_id.to_string() {
                    Uuid::parse_str(&rel.target_id).unwrap_or(entity_id)
                } else {
                    Uuid::parse_str(&rel.source_id).unwrap_or(entity_id)
                };

                if let Ok(child) = self.get_entity_graph(child_id, depth - 1).await {
                    children.push(child);
                }

                let direction = if rel.source_id == entity_id.to_string() {
                    EdgeDirection::Outgoing
                } else {
                    EdgeDirection::Incoming
                };

                graph_edges.push(GraphEdge {
                    relationship: rel.clone(),
                    direction,
                });
            }
        }

        Ok(GraphNode {
            entity,
            relationships: graph_edges,
            children,
        })
    }

    pub async fn find_path(&self, from_id: Uuid, to_id: Uuid, max_depth: usize) -> Result<Vec<Entity>> {
        // Simple BFS implementation for finding path
        use std::collections::{VecDeque, HashSet};

        let mut queue = VecDeque::new();
        let mut visited = HashSet::new();
        let mut parent_map = std::collections::HashMap::new();

        queue.push_back(from_id);
        visited.insert(from_id);

        let mut found = false;
        let mut depth = 0;

        while !queue.is_empty() && depth < max_depth {
            let level_size = queue.len();
            for _ in 0..level_size {
                let current = queue.pop_front().unwrap();

                if current == to_id {
                    found = true;
                    break;
                }

                let relationships = self.get_relationships(current).await?;
                for rel in relationships {
                    let neighbor_id = if rel.source_id == current.to_string() {
                        Uuid::parse_str(&rel.target_id)
                    } else {
                        Uuid::parse_str(&rel.source_id)
                    };

                    if let Ok(neighbor) = neighbor_id {
                        if !visited.contains(&neighbor) {
                            visited.insert(neighbor);
                            parent_map.insert(neighbor, current);
                            queue.push_back(neighbor);
                        }
                    }
                }
            }

            if found {
                break;
            }
            depth += 1;
        }

        if !found {
            return Ok(vec![]);
        }

        // Reconstruct path
        let mut path = Vec::new();
        let mut current = to_id;

        while current != from_id {
            if let Ok(entity) = self.get_entity(current).await {
                path.push(entity);
            }
            current = *parent_map.get(&current).unwrap_or(&from_id);
        }

        if let Ok(entity) = self.get_entity(from_id).await {
            path.push(entity);
        }

        path.reverse();
        Ok(path)
    }

    // ==================== MEETING OPERATIONS ====================

    pub async fn get_meetings(&self, limit: usize) -> Result<Vec<Meeting>> {
        let conn = self.conn.lock().await;
        let mut stmt = conn.prepare(
            "SELECT id, title, occurred_at, participants, source_artifact_id, extracted_at 
             FROM meetings 
             ORDER BY occurred_at DESC 
             LIMIT ?"
        )?;

        let meetings = stmt
            .query_map(params![limit], |row| {
                Ok(Meeting {
                    id: row.get(0)?,
                    title: row.get(1)?,
                    occurred_at: row.get(2)?,
                    participants: row.get::<_, Option<String>>(3)?
                        .and_then(|s| serde_json::from_str(&s).ok())
                        .unwrap_or_default(),
                    source_artifact_id: row.get(4)?,
                    extracted_at: row.get(5)?,
                })
            })?
            .collect::<Result<Vec<_>, _>>()?;

        Ok(meetings)
    }

    pub async fn extract_from_meeting(&self, meeting_id: Uuid) -> Result<ExtractionResult> {
        // TODO: Implement actual extraction logic
        Ok(ExtractionResult {
            meeting_id: meeting_id.to_string(),
            entities_extracted: vec![],
            decisions_extracted: vec![],
            action_items_extracted: vec![],
            extraction_confidence: 0.0,
        })
    }

    pub async fn get_decisions(&self, entity_id: Option<Uuid>) -> Result<Vec<Decision>> {
        let conn = self.conn.lock().await;
        
        let query = if entity_id.is_some() {
            "SELECT id, meeting_id, description, decided_by, related_entities, confidence_score, extracted_at 
             FROM decisions 
             WHERE related_entities LIKE ?1
             ORDER BY extracted_at DESC"
        } else {
            "SELECT id, meeting_id, description, decided_by, related_entities, confidence_score, extracted_at 
             FROM decisions 
             ORDER BY extracted_at DESC"
        };

        let mut stmt = conn.prepare(query)?;

        let decisions = if let Some(id) = entity_id {
            stmt.query_map(params![format!("%{}%", id.to_string())], |row| {
                Ok(Decision {
                    id: row.get(0)?,
                    meeting_id: row.get(1)?,
                    description: row.get(2)?,
                    decided_by: row.get::<_, Option<String>>(3)?
                        .and_then(|s| serde_json::from_str(&s).ok())
                        .unwrap_or_default(),
                    related_entities: row.get::<_, Option<String>>(4)?
                        .and_then(|s| serde_json::from_str(&s).ok())
                        .unwrap_or_default(),
                    confidence_score: row.get(5)?,
                    extracted_at: row.get(6)?,
                })
            })?
            .collect::<Result<Vec<_>, _>>()?
        } else {
            stmt.query_map([], |row| {
                Ok(Decision {
                    id: row.get(0)?,
                    meeting_id: row.get(1)?,
                    description: row.get(2)?,
                    decided_by: row.get::<_, Option<String>>(3)?
                        .and_then(|s| serde_json::from_str(&s).ok())
                        .unwrap_or_default(),
                    related_entities: row.get::<_, Option<String>>(4)?
                        .and_then(|s| serde_json::from_str(&s).ok())
                        .unwrap_or_default(),
                    confidence_score: row.get(5)?,
                    extracted_at: row.get(6)?,
                })
            })?
            .collect::<Result<Vec<_>, _>>()?
        };

        Ok(decisions)
    }

    pub async fn get_action_items(
        &self,
        entity_id: Option<Uuid>,
        status: Option<ActionItemStatus>,
    ) -> Result<Vec<ActionItem>> {
        let conn = self.conn.lock().await;
        
        let query = match (entity_id.is_some(), status.is_some()) {
            (true, true) => "SELECT id, meeting_id, description, assignee, due_date, status, related_entities, extracted_at 
                             FROM action_items 
                             WHERE related_entities LIKE ?1 AND status = ?2
                             ORDER BY due_date ASC",
            (true, false) => "SELECT id, meeting_id, description, assignee, due_date, status, related_entities, extracted_at 
                              FROM action_items 
                              WHERE related_entities LIKE ?1
                              ORDER BY due_date ASC",
            (false, true) => "SELECT id, meeting_id, description, assignee, due_date, status, related_entities, extracted_at 
                              FROM action_items 
                              WHERE status = ?1
                              ORDER BY due_date ASC",
            (false, false) => "SELECT id, meeting_id, description, assignee, due_date, status, related_entities, extracted_at 
                               FROM action_items 
                               ORDER BY due_date ASC",
        };

        let mut stmt = conn.prepare(query)?;

        let action_items = match (entity_id, status) {
            (Some(eid), Some(st)) => stmt.query_map(
                params![format!("%{}%", eid.to_string()), serde_json::to_string(&st).unwrap()],
                |row| Self::map_action_item(row)
            )?.collect::<Result<Vec<_>, _>>()?,
            (Some(eid), None) => stmt.query_map(
                params![format!("%{}%", eid.to_string())],
                |row| Self::map_action_item(row)
            )?.collect::<Result<Vec<_>, _>>()?,
            (None, Some(st)) => stmt.query_map(
                params![serde_json::to_string(&st).unwrap()],
                |row| Self::map_action_item(row)
            )?.collect::<Result<Vec<_>, _>>()?,
            (None, None) => stmt.query_map(
                [],
                |row| Self::map_action_item(row)
            )?.collect::<Result<Vec<_>, _>>()?,
        };

        Ok(action_items)
    }

    fn map_action_item(row: &rusqlite::Row) -> rusqlite::Result<ActionItem> {
        Ok(ActionItem {
            id: row.get(0)?,
            meeting_id: row.get(1)?,
            description: row.get(2)?,
            assignee: row.get(3)?,
            due_date: row.get(4)?,
            status: row.get(5)?,
            related_entities: row.get::<_, Option<String>>(6)?
                .and_then(|s| serde_json::from_str(&s).ok())
                .unwrap_or_default(),
            extracted_at: row.get(7)?,
        })
    }

    // ==================== VALIDATION OPERATIONS ====================

    pub async fn validate_entity(&self, entity_id: Uuid) -> Result<ValidationResult> {
        let contradictions = self.get_contradictions(Some(entity_id), None).await?;
        
        Ok(ValidationResult {
            entity_id: entity_id.to_string(),
            valid: contradictions.is_empty(),
            contradictions,
            warnings: vec![],
            checked_at: chrono::Utc::now().to_rfc3339(),
        })
    }

    pub async fn get_contradictions(
        &self,
        entity_id: Option<Uuid>,
        severity: Option<ContradictionSeverity>,
    ) -> Result<Vec<Contradiction>> {
        let conn = self.conn.lock().await;
        
        let query = match (entity_id.is_some(), severity.is_some()) {
            (true, true) => "SELECT id, entity_id, contradiction_type, description, severity, related_entity_ids, created_at 
                             FROM contradictions 
                             WHERE entity_id = ?1 AND severity = ?2
                             ORDER BY created_at DESC",
            (true, false) => "SELECT id, entity_id, contradiction_type, description, severity, related_entity_ids, created_at 
                              FROM contradictions 
                              WHERE entity_id = ?1
                              ORDER BY created_at DESC",
            (false, true) => "SELECT id, entity_id, contradiction_type, description, severity, related_entity_ids, created_at 
                              FROM contradictions 
                              WHERE severity = ?1
                              ORDER BY created_at DESC",
            (false, false) => "SELECT id, entity_id, contradiction_type, description, severity, related_entity_ids, created_at 
                               FROM contradictions 
                               ORDER BY created_at DESC",
        };

        let mut stmt = conn.prepare(query)?;

        let contradictions = match (entity_id, severity) {
            (Some(eid), Some(sev)) => stmt.query_map(
                params![eid.to_string(), serde_json::to_string(&sev).unwrap()],
                |row| Self::map_contradiction(row)
            )?.collect::<Result<Vec<_>, _>>()?,
            (Some(eid), None) => stmt.query_map(
                params![eid.to_string()],
                |row| Self::map_contradiction(row)
            )?.collect::<Result<Vec<_>, _>>()?,
            (None, Some(sev)) => stmt.query_map(
                params![serde_json::to_string(&sev).unwrap()],
                |row| Self::map_contradiction(row)
            )?.collect::<Result<Vec<_>, _>>()?,
            (None, None) => stmt.query_map(
                [],
                |row| Self::map_contradiction(row)
            )?.collect::<Result<Vec<_>, _>>()?,
        };

        Ok(contradictions)
    }

    fn map_contradiction(row: &rusqlite::Row) -> rusqlite::Result<Contradiction> {
        Ok(Contradiction {
            id: row.get(0)?,
            entity_id: row.get(1)?,
            contradiction_type: row.get(2)?,
            description: row.get(3)?,
            severity: row.get(4)?,
            related_entity_ids: row.get::<_, Option<String>>(5)?
                .and_then(|s| serde_json::from_str(&s).ok())
                .unwrap_or_default(),
            created_at: row.get(6)?,
        })
    }

    pub async fn get_confidence_score(&self, entity_id: Uuid) -> Result<ConfidenceScore> {
        // TODO: Implement actual confidence calculation
        Ok(ConfidenceScore {
            entity_id: entity_id.to_string(),
            overall_score: 1.0,
            source_reliability: 1.0,
            cross_reference_score: 1.0,
            temporal_consistency: 1.0,
            calculated_at: chrono::Utc::now().to_rfc3339(),
        })
    }

    pub async fn find_duplicates(&self, threshold: f32) -> Result<Vec<DuplicateCandidate>> {
        // TODO: Implement duplicate detection algorithm
        let _ = threshold;
        Ok(vec![])
    }

    pub async fn merge_entities(&self, primary_id: Uuid, secondary_id: Uuid) -> Result<Entity> {
        let conn = self.conn.lock().await;
        let now = chrono::Utc::now().to_rfc3339();

        // Mark secondary entity as merged
        conn.execute(
            "UPDATE entities SET merged_into = ?1, updated_at = ?2 WHERE id = ?3",
            params![primary_id.to_string(), now, secondary_id.to_string()],
        )?;

        // Update relationships pointing to secondary
        conn.execute(
            "UPDATE relationships SET target_id = ?1 WHERE target_id = ?2",
            params![primary_id.to_string(), secondary_id.to_string()],
        )?;

        drop(conn);
        self.get_entity(primary_id).await
    }

    pub async fn resolve_contradiction(
        &self,
        contradiction_id: Uuid,
        resolution: ResolutionInput,
    ) -> Result<()> {
        let conn = self.conn.lock().await;
        let now = chrono::Utc::now().to_rfc3339();

        conn.execute(
            "UPDATE contradictions SET resolution_type = ?1, resolution_note = ?2, resolved_at = ?3 WHERE id = ?4",
            params![
                serde_json::to_string(&resolution.resolution_type).unwrap(),
                resolution.note,
                now,
                contradiction_id.to_string(),
            ],
        )?;

        Ok(())
    }
}

// ==================== HELPER FUNCTIONS ====================

fn get_db_path() -> Result<PathBuf> {
    let data_dir = dirs::data_dir()
        .context("Could not determine data directory")?
        .join("Truffle");
    
    std::fs::create_dir_all(&data_dir)?;
    
    Ok(data_dir.join("truffle.db"))
}

fn get_device_id() -> String {
    // TODO: Generate and persist device ID
    "unknown".to_string()
}
