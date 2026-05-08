//! WebSocket connection handlers

use axum::extract::ws::{WebSocket, Message};
use futures::{sink::SinkExt, stream::StreamExt};
use std::net::SocketAddr;
use std::sync::Arc;
use uuid::Uuid;

use crate::{
    auth::validate_token,
    connection::Connection,
    hub::MeetingHub,
    message::{ClientMessage, ServerMessage},
};

/// Handle a meeting WebSocket connection
pub async fn handle_meeting_socket(
    hub: Arc<MeetingHub>,
    meeting_id: Uuid,
    mut socket: WebSocket,
    addr: SocketAddr,
) {
    tracing::info!("New WebSocket connection from {} for meeting {}", addr, meeting_id);

    // Wait for authentication message
    let (user_id, tenant_id, user_name) = match wait_for_auth(&mut socket).await {
        Ok(auth) => auth,
        Err(e) => {
            tracing::error!("Authentication failed: {}", e);
            let _ = socket.send(Message::Text(
                ServerMessage::Error {
                    code: "AUTH_FAILED".to_string(),
                    message: e.to_string(),
                }.to_json().unwrap()
            )).await;
            return;
        }
    };

    // Create connection
    let (mut connection, mut rx) = Connection::new(user_id, tenant_id, user_name.clone());
    connection.authenticate();
    connection.set_meeting(meeting_id);
    let connection_id = connection.id;

    // Join the meeting room
    let mut room_rx = match hub.join_room(meeting_id, connection).await {
        Ok(rx) => rx,
        Err(e) => {
            tracing::error!("Failed to join room: {}", e);
            return;
        }
    };

    // Send authentication success
    let auth_msg = ServerMessage::Authenticated {
        user_id,
        success: true,
        error: None,
    };
    if socket.send(Message::Text(auth_msg.to_json().unwrap())).await.is_err() {
        return;
    }

    // Send joined confirmation
    let participants = hub.get_participants(meeting_id);
    let joined_msg = ServerMessage::Joined {
        meeting_id,
        participant_id: user_id,
        participants: participants.iter().map(|(id, name)| crate::message::ParticipantInfo {
            id: *id,
            name: name.clone(),
            avatar_url: None,
            is_speaking: false,
            is_screen_sharing: false,
            hand_raised: false,
            joined_at: chrono::Utc::now(),
        }).collect(),
    };
    if socket.send(Message::Text(joined_msg.to_json().unwrap())).await.is_err() {
        return;
    }

    // Handle messages
    loop {
        tokio::select! {
            // Messages from client
            Some(msg) = socket.next() => {
                match msg {
                    Ok(Message::Text(text)) => {
                        match handle_client_message(&hub, meeting_id, connection_id, &text).await {
                            Ok(Some(response)) => {
                                if socket.send(Message::Text(response.to_json().unwrap())).await.is_err() {
                                    break;
                                }
                            }
                            Err(e) => {
                                tracing::error!("Error handling message: {}", e);
                            }
                            _ => {}
                        }
                    }
                    Ok(Message::Binary(data)) => {
                        // Handle binary audio data
                        handle_audio_data(&hub, meeting_id, connection_id, data).await;
                    }
                    Ok(Message::Close(_)) => {
                        tracing::info!("Client disconnected");
                        break;
                    }
                    _ => {}
                }
            }

            // Messages from room broadcast
            Ok(msg) = room_rx.recv() => {
                if socket.send(Message::Text(msg.to_json().unwrap())).await.is_err() {
                    break;
                }
            }

            // Direct messages to this connection
            Some(msg) = rx.recv() => {
                if socket.send(Message::Text(msg.to_json().unwrap())).await.is_err() {
                    break;
                }
            }
        }
    }

    // Cleanup
    hub.leave_room(meeting_id, connection_id).await;
    tracing::info!("Connection {} closed for meeting {}", connection_id, meeting_id);
}

/// Handle signaling WebSocket for WebRTC
pub async fn handle_signaling_socket(
    hub: Arc<MeetingHub>,
    meeting_id: Uuid,
    mut socket: WebSocket,
    addr: SocketAddr,
) {
    tracing::info!("New signaling connection from {} for meeting {}", addr, meeting_id);

    // Similar to handle_meeting_socket but focused on WebRTC signaling
    // Simplified for brevity
    while let Some(msg) = socket.next().await {
        if let Ok(Message::Text(text)) = msg {
            // Parse and relay signaling messages
            tracing::debug!("Signaling message: {}", text);
        }
    }
}

/// Wait for and validate authentication message
async fn wait_for_auth(socket: &mut WebSocket) -> anyhow::Result<(Uuid, Uuid, String)> {
    if let Some(Ok(Message::Text(text))) = socket.next().await {
        let msg: ClientMessage = serde_json::from_str(&text)?;
        
        if let ClientMessage::Authenticate { token } = msg {
            return validate_token(&token).await;
        }
    }
    
    Err(anyhow::anyhow!("Expected authentication message"))
}

/// Handle a client message
async fn handle_client_message(
    hub: &Arc<MeetingHub>,
    meeting_id: Uuid,
    connection_id: Uuid,
    text: &str,
) -> anyhow::Result<Option<ServerMessage>> {
    let msg: ClientMessage = ClientMessage::from_json(text)?;
    
    match msg {
        ClientMessage::Chat { content, reply_to } => {
            // Broadcast chat message
            let chat_msg = ServerMessage::ChatMessage {
                id: Uuid::new_v4(),
                participant_id: connection_id,
                participant_name: "User".to_string(), // TODO: Get from connection
                content,
                reply_to,
                timestamp: chrono::Utc::now(),
            };
            hub.broadcast(meeting_id, chat_msg);
            Ok(None)
        }
        
        ClientMessage::HandRaise { raised } => {
            let hand_msg = ServerMessage::HandRaised {
                participant_id: connection_id,
                raised,
            };
            hub.broadcast(meeting_id, hand_msg);
            Ok(None)
        }
        
        ClientMessage::Reaction { emoji } => {
            let reaction_msg = ServerMessage::ReactionReceived {
                participant_id: connection_id,
                emoji,
                timestamp: chrono::Utc::now(),
            };
            hub.broadcast(meeting_id, reaction_msg);
            Ok(None)
        }
        
        ClientMessage::Signal { target_user_id, signal } => {
            // Relay signal to target user
            // This would need connection lookup by user_id
            Ok(None)
        }
        
        ClientMessage::Ping => {
            Ok(Some(ServerMessage::Pong))
        }
        
        _ => Ok(None),
    }
}

/// Handle binary audio data
async fn handle_audio_data(
    _hub: &Arc<MeetingHub>,
    _meeting_id: Uuid,
    _connection_id: Uuid,
    _data: Vec<u8>,
) {
    // Process audio data
    // Could forward to ASR service or other participants
    tracing::debug!("Received audio data: {} bytes", _data.len());
}
