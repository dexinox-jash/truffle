//! WebSocket Relay Client for Device Pairing
//!
//! This module implements the WebSocket client for communicating
//! with the relay server during device pairing and sync.

use crate::error::{CryptoError, CryptoResult};
use crate::protocol::message::SyncMessage;
use crate::types::DeviceId;
use futures::{SinkExt, StreamExt};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use std::time::Duration;
use tokio::net::TcpStream;
use tokio::sync::{mpsc, Mutex, RwLock};
use tokio::time::{interval, timeout};
use tokio_tungstenite::{
    connect_async,
    tungstenite::protocol::Message as WsMessage,
    MaybeTlsStream, WebSocketStream,
};

/// WebSocket configuration
#[derive(Clone, Debug)]
pub struct WebSocketConfig {
    /// Connection timeout
    pub connect_timeout: Duration,
    /// Message send timeout
    pub send_timeout: Duration,
    /// Ping interval
    pub ping_interval: Duration,
    /// Reconnect attempts
    pub reconnect_attempts: u32,
    /// Reconnect delay
    pub reconnect_delay: Duration,
}

impl Default for WebSocketConfig {
    fn default() -> Self {
        Self {
            connect_timeout: Duration::from_secs(10),
            send_timeout: Duration::from_secs(5),
            ping_interval: Duration::from_secs(30),
            reconnect_attempts: 3,
            reconnect_delay: Duration::from_secs(5),
        }
    }
}

/// WebSocket message types
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum RelayMessage {
    /// Pairing request
    #[serde(rename = "pairing_request")]
    PairingRequest {
        session_id: String,
        device_id: String,
        identity_fingerprint: String,
    },

    /// Pairing response
    #[serde(rename = "pairing_response")]
    PairingResponse {
        session_id: String,
        device_id: String,
        accepted: bool,
    },

    /// X3DH handshake message
    #[serde(rename = "x3dh_message")]
    X3dhMessage {
        session_id: String,
        #[serde(with = "serde_bytes")]
        payload: Vec<u8>,
    },

    /// Sync message
    #[serde(rename = "sync")]
    Sync {
        #[serde(with = "serde_bytes")]
        payload: Vec<u8>,
    },

    /// Acknowledgment
    #[serde(rename = "ack")]
    Ack { message_id: String },

    /// Ping
    #[serde(rename = "ping")]
    Ping,

    /// Pong
    #[serde(rename = "pong")]
    Pong,

    /// Error
    #[serde(rename = "error")]
    Error { code: u16, message: String },
}

/// WebSocket relay client
pub struct RelayClient {
    /// WebSocket URL
    url: String,
    /// Configuration
    config: WebSocketConfig,
    /// Device ID
    device_id: DeviceId,
    /// WebSocket connection
    ws: Option<WebSocketStream<MaybeTlsStream<TcpStream>>>,
    /// Message channel
    message_tx: Option<mpsc::Sender<RelayMessage>>,
    /// Receive channel
    message_rx: Option<mpsc::Receiver<RelayMessage>>,
    /// Connection state
    connected: Arc<RwLock<bool>>,
}

impl RelayClient {
    /// Create a new relay client
    pub fn new(url: String, device_id: DeviceId, config: WebSocketConfig) -> Self {
        Self {
            url,
            config,
            device_id,
            ws: None,
            message_tx: None,
            message_rx: None,
            connected: Arc::new(RwLock::new(false)),
        }
    }

    /// Connect to the relay server
    pub async fn connect(&mut self) -> CryptoResult<()> {
        // Try to connect with timeout
        let ws_stream = timeout(
            self.config.connect_timeout,
            connect_async(&self.url)
        )
        .await
        .map_err(|_| CryptoError::timeout("WebSocket connection timed out"))?
        .map_err(|e| CryptoError::websocket_error(format!("Connection failed: {}", e)))?;

        self.ws = Some(ws_stream.0);

        // Create message channels
        let (tx, rx) = mpsc::channel(100);
        self.message_tx = Some(tx);
        self.message_rx = Some(rx);

        // Mark as connected
        *self.connected.write().await = true;

        // Start background tasks
        self.start_background_tasks().await?;

        Ok(())
    }

    /// Disconnect from the relay server
    pub async fn disconnect(&mut self) -> CryptoResult<()> {
        *self.connected.write().await = false;

        if let Some(ws) = self.ws.take() {
            let _ = ws.close(None).await;
        }

        self.message_tx = None;
        self.message_rx = None;

        Ok(())
    }

    /// Check if connected
    pub async fn is_connected(&self) -> bool {
        *self.connected.read().await
    }

    /// Send a message
    pub async fn send(&mut self, message: RelayMessage) -> CryptoResult<()> {
        if !self.is_connected().await {
            return Err(CryptoError::websocket_error("Not connected"));
        }

        let tx = self.message_tx.as_ref()
            .ok_or_else(|| CryptoError::websocket_error("Message channel not available"))?;

        timeout(
            self.config.send_timeout,
            tx.send(message)
        )
        .await
        .map_err(|_| CryptoError::timeout("Send timed out"))?
        .map_err(|_| CryptoError::websocket_error("Channel closed"))?;

        Ok(())
    }

    /// Receive a message
    pub async fn receive(&mut self) -> CryptoResult<Option<RelayMessage>> {
        let rx = self.message_rx.as_mut()
            .ok_or_else(|| CryptoError::websocket_error("Message channel not available"))?;

        match timeout(Duration::from_millis(100), rx.recv()).await {
            Ok(Some(msg)) => Ok(Some(msg)),
            Ok(None) => Ok(None),
            Err(_) => Ok(None), // Timeout, no message available
        }
    }

    /// Send a pairing request
    pub async fn send_pairing_request(
        &mut self,
        session_id: String,
        identity_fingerprint: String,
    ) -> CryptoResult<()> {
        let message = RelayMessage::PairingRequest {
            session_id,
            device_id: self.device_id.to_string(),
            identity_fingerprint,
        };
        self.send(message).await
    }

    /// Send a sync message
    pub async fn send_sync(&mut self, sync_message: &SyncMessage) -> CryptoResult<()> {
        let payload = sync_message.to_bytes()?;
        let message = RelayMessage::Sync { payload };
        self.send(message).await
    }

    /// Start background tasks for WebSocket handling
    async fn start_background_tasks(&mut self) -> CryptoResult<()> {
        // This would spawn tasks for:
        // - Reading from WebSocket and forwarding to channel
        // - Writing from channel to WebSocket
        // - Ping/pong handling
        // - Reconnection logic

        // For now, this is a placeholder
        Ok(())
    }
}

/// Pairing-specific WebSocket handler
pub struct PairingWebSocket {
    /// Underlying relay client
    client: RelayClient,
    /// Session ID
    session_id: Option<String>,
    /// Callback for pairing events
    event_callback: Option<Box<dyn Fn(PairingEvent) + Send + Sync>>,
}

/// Pairing events
#[derive(Clone, Debug)]
pub enum PairingEvent {
    /// Connected to relay
    Connected,
    /// Disconnected from relay
    Disconnected,
    /// Pairing request received
    PairingRequest { device_id: String, fingerprint: String },
    /// Pairing accepted
    PairingAccepted,
    /// Pairing rejected
    PairingRejected,
    /// X3DH message received
    X3dhMessage { payload: Vec<u8> },
    /// Error occurred
    Error { message: String },
}

impl PairingWebSocket {
    /// Create a new pairing WebSocket
    pub fn new(url: String, device_id: DeviceId) -> Self {
        let config = WebSocketConfig::default();
        let client = RelayClient::new(url, device_id, config);

        Self {
            client,
            session_id: None,
            event_callback: None,
        }
    }

    /// Set event callback
    pub fn on_event<F>(&mut self, callback: F)
    where
        F: Fn(PairingEvent) + Send + Sync + 'static,
    {
        self.event_callback = Some(Box::new(callback));
    }

    /// Connect and start pairing
    pub async fn connect(&mut self) -> CryptoResult<()> {
        self.client.connect().await?;
        self.emit_event(PairingEvent::Connected);
        Ok(())
    }

    /// Start pairing session
    pub async fn start_pairing(&mut self, session_id: String) -> CryptoResult<()> {
        self.session_id = Some(session_id);
        // Additional pairing setup would go here
        Ok(())
    }

    /// Send X3DH message
    pub async fn send_x3dh(&mut self, payload: Vec<u8>) -> CryptoResult<()> {
        let session_id = self.session_id.as_ref()
            .ok_or_else(|| CryptoError::invalid_state("No active pairing session"))?
            .clone();

        let message = RelayMessage::X3dhMessage { session_id, payload };
        self.client.send(message).await
    }

    /// Disconnect
    pub async fn disconnect(&mut self) -> CryptoResult<()> {
        self.client.disconnect().await?;
        self.emit_event(PairingEvent::Disconnected);
        Ok(())
    }

    /// Emit an event
    fn emit_event(&self, event: PairingEvent) {
        if let Some(ref callback) = self.event_callback {
            callback(event);
        }
    }
}

/// Mock relay client for testing
pub struct MockRelayClient {
    messages: Arc<Mutex<Vec<RelayMessage>>>,
    connected: Arc<RwLock<bool>>,
}

impl MockRelayClient {
    /// Create a new mock relay client
    pub fn new() -> Self {
        Self {
            messages: Arc::new(Mutex::new(Vec::new())),
            connected: Arc::new(RwLock::new(false)),
        }
    }

    /// Connect (mock)
    pub async fn connect(&self) -> CryptoResult<()> {
        *self.connected.write().await = true;
        Ok(())
    }

    /// Disconnect (mock)
    pub async fn disconnect(&self) -> CryptoResult<()> {
        *self.connected.write().await = false;
        Ok(())
    }

    /// Check if connected
    pub async fn is_connected(&self) -> bool {
        *self.connected.read().await
    }

    /// Send a message (stores it)
    pub async fn send(&self, message: RelayMessage) -> CryptoResult<()> {
        self.messages.lock().await.push(message);
        Ok(())
    }

    /// Get all sent messages
    pub async fn get_messages(&self) -> Vec<RelayMessage> {
        self.messages.lock().await.clone()
    }

    /// Clear messages
    pub async fn clear_messages(&self) {
        self.messages.lock().await.clear();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_websocket_config_default() {
        let config = WebSocketConfig::default();
        assert_eq!(config.connect_timeout, Duration::from_secs(10));
        assert_eq!(config.reconnect_attempts, 3);
    }

    #[test]
    fn test_relay_message_serialization() {
        let msg = RelayMessage::PairingRequest {
            session_id: "test".to_string(),
            device_id: "device".to_string(),
            identity_fingerprint: "fp".to_string(),
        };

        let json = serde_json::to_string(&msg).unwrap();
        assert!(json.contains("pairing_request"));

        let decoded: RelayMessage = serde_json::from_str(&json).unwrap();
        match decoded {
            RelayMessage::PairingRequest { session_id, .. } => {
                assert_eq!(session_id, "test");
            }
            _ => panic!("Wrong message type"),
        }
    }

    #[test]
    fn test_pairing_event() {
        let event = PairingEvent::Connected;
        assert!(matches!(event, PairingEvent::Connected));

        let event = PairingEvent::Error { message: "test".to_string() };
        match event {
            PairingEvent::Error { message } => assert_eq!(message, "test"),
            _ => panic!("Wrong event type"),
        }
    }

    #[tokio::test]
    async fn test_mock_relay_client() {
        let client = MockRelayClient::new();

        assert!(!client.is_connected().await);
        client.connect().await.unwrap();
        assert!(client.is_connected().await);

        let msg = RelayMessage::Ping;
        client.send(msg).await.unwrap();

        let messages = client.get_messages().await;
        assert_eq!(messages.len(), 1);

        client.clear_messages().await;
        assert!(client.get_messages().await.is_empty());

        client.disconnect().await.unwrap();
        assert!(!client.is_connected().await);
    }
}
