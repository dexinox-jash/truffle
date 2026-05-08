//! Meeting Hub - Central coordination for all meeting connections

use dashmap::DashMap;
use std::sync::Arc;
use tokio::sync::broadcast;
use uuid::Uuid;

use crate::{
    config::AppConfig,
    connection::Connection,
    message::{ClientMessage, ServerMessage},
};

/// Manages all active meeting rooms and connections
pub struct MeetingHub {
    config: Arc<AppConfig>,
    /// Meeting ID -> Room
    rooms: DashMap<Uuid, MeetingRoom>,
    /// User ID -> Connection mapping for presence
    user_connections: DashMap<Uuid, Vec<Uuid>>, // user_id -> connection_ids
    /// NATS client for cross-instance messaging
    nats_client: Option<async_nats::Client>,
}

/// A meeting room containing all participant connections
pub struct MeetingRoom {
    pub meeting_id: Uuid,
    /// Connection ID -> Connection
    pub connections: DashMap<Uuid, Connection>,
    /// Broadcast channel for room messages
    pub tx: broadcast::Sender<ServerMessage>,
    /// Meeting metadata
    pub metadata: MeetingMetadata,
}

#[derive(Clone, Debug)]
pub struct MeetingMetadata {
    pub tenant_id: Uuid,
    pub started_at: Option<chrono::DateTime<chrono::Utc>>,
    pub is_recording: bool,
    pub is_transcribing: bool,
}

impl MeetingHub {
    pub async fn new(config: Arc<AppConfig>) -> anyhow::Result<Self> {
        // Connect to NATS for clustering
        let nats_client = if let Ok(client) = async_nats::connect(&config.nats_url).await {
            tracing::info!("Connected to NATS");
            Some(client)
        } else {
            tracing::warn!("NATS not available - running in standalone mode");
            None
        };

        Ok(Self {
            config,
            rooms: DashMap::new(),
            user_connections: DashMap::new(),
            nats_client,
        })
    }

    /// Get or create a meeting room
    pub fn get_or_create_room(&self, meeting_id: Uuid, tenant_id: Uuid) -> Arc<MeetingRoom> {
        if let Some(room) = self.rooms.get(&meeting_id) {
            return Arc::clone(&room);
        }

        let (tx, _rx) = broadcast::channel(1000);
        
        let room = Arc::new(MeetingRoom {
            meeting_id,
            connections: DashMap::new(),
            tx,
            metadata: MeetingMetadata {
                tenant_id,
                started_at: None,
                is_recording: false,
                is_transcribing: false,
            },
        });

        self.rooms.insert(meeting_id, Arc::clone(&room));
        tracing::info!("Created room for meeting {}", meeting_id);
        
        room
    }

    /// Join a connection to a room
    pub async fn join_room(
        &self,
        meeting_id: Uuid,
        connection: Connection,
    ) -> anyhow::Result<broadcast::Receiver<ServerMessage>> {
        let room = self.get_or_create_room(meeting_id, connection.tenant_id);
        
        let conn_id = connection.id;
        let user_id = connection.user_id;
        
        // Add connection to room
        room.connections.insert(conn_id, connection);
        
        // Track user connection
        self.user_connections
            .entry(user_id)
            .or_insert_with(Vec::new)
            .push(conn_id);

        // Subscribe to room messages
        let rx = room.tx.subscribe();

        // Broadcast join message
        let join_msg = ServerMessage::ParticipantJoined {
            participant_id: user_id,
            participant_name: "User".to_string(), // TODO: Get from DB
            joined_at: chrono::Utc::now(),
        };
        let _ = room.tx.send(join_msg);

        tracing::info!("Connection {} joined meeting {}", conn_id, meeting_id);
        
        Ok(rx)
    }

    /// Remove a connection from a room
    pub async fn leave_room(&self, meeting_id: Uuid, connection_id: Uuid) {
        if let Some(room) = self.rooms.get(&meeting_id) {
            if let Some((_, conn)) = room.connections.remove(&connection_id) {
                // Remove from user connections
                if let Some(mut user_conns) = self.user_connections.get_mut(&conn.user_id) {
                    user_conns.retain(|&id| id != connection_id);
                    if user_conns.is_empty() {
                        drop(user_conns);
                        self.user_connections.remove(&conn.user_id);
                        
                        // Broadcast leave message
                        let leave_msg = ServerMessage::ParticipantLeft {
                            participant_id: conn.user_id,
                            left_at: chrono::Utc::now(),
                        };
                        let _ = room.tx.send(leave_msg);
                    }
                }

                tracing::info!("Connection {} left meeting {}", connection_id, meeting_id);
            }

            // Clean up empty rooms
            if room.connections.is_empty() {
                drop(room);
                self.rooms.remove(&meeting_id);
                tracing::info!("Removed empty room for meeting {}", meeting_id);
            }
        }
    }

    /// Broadcast a message to all connections in a room
    pub fn broadcast(&self, meeting_id: Uuid, message: ServerMessage) {
        if let Some(room) = self.rooms.get(&meeting_id) {
            let _ = room.tx.send(message);
        }
    }

    /// Send a direct message to a specific connection
    pub fn send_direct(&self, meeting_id: Uuid, connection_id: Uuid, message: ServerMessage) {
        if let Some(room) = self.rooms.get(&meeting_id) {
            if let Some(conn) = room.connections.get(&connection_id) {
                // Direct messages go through the connection's individual channel
                // This is handled by the connection's message handler
                let _ = conn.tx.try_send(message);
            }
        }
    }

    /// Get participant count for a meeting
    pub fn get_participant_count(&self, meeting_id: Uuid) -> usize {
        self.rooms
            .get(&meeting_id)
            .map(|r| r.connections.len())
            .unwrap_or(0)
    }

    /// Get list of participants in a meeting
    pub fn get_participants(&self, meeting_id: Uuid) -> Vec<(Uuid, String)> {
        self.rooms
            .get(&meeting_id)
            .map(|r| {
                r.connections
                    .iter()
                    .map(|c| (c.user_id, c.user_name.clone()))
                    .collect()
            })
            .unwrap_or_default()
    }
}
