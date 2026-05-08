//! Connection state management

use tokio::sync::mpsc;
use uuid::Uuid;

use crate::message::ServerMessage;

/// Represents a single WebSocket connection
pub struct Connection {
    pub id: Uuid,
    pub user_id: Uuid,
    pub tenant_id: Uuid,
    pub user_name: String,
    pub meeting_id: Option<Uuid>,
    /// Channel for sending messages to this connection
    pub tx: mpsc::UnboundedSender<ServerMessage>,
    /// Whether this connection has completed authentication
    pub authenticated: bool,
    /// Join timestamp
    pub connected_at: chrono::DateTime<chrono::Utc>,
}

impl Connection {
    pub fn new(user_id: Uuid, tenant_id: Uuid, user_name: String) -> (Self, mpsc::UnboundedReceiver<ServerMessage>) {
        let (tx, rx) = mpsc::unbounded_channel();
        
        let conn = Self {
            id: Uuid::new_v4(),
            user_id,
            tenant_id,
            user_name,
            meeting_id: None,
            tx,
            authenticated: false,
            connected_at: chrono::Utc::now(),
        };
        
        (conn, rx)
    }

    pub fn set_meeting(&mut self, meeting_id: Uuid) {
        self.meeting_id = Some(meeting_id);
    }

    pub fn authenticate(&mut self) {
        self.authenticated = true;
    }

    /// Send a message to this connection
    pub fn send(&self, message: ServerMessage) -> anyhow::Result<()> {
        self.tx.send(message)
            .map_err(|_| anyhow::anyhow!("Connection closed"))
    }
}
