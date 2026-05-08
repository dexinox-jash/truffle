/**
 * WebSocket Handler - Store-and-Forward Encrypted Blobs
 * SPEC v2.0 SECTION 2.3.4 - Relay Server
 * SPEC v2.0 SECTION 2.3.2 - Device Pairing Ceremony
 * 
 * CRITICAL: Zero-access infrastructure. We only route encrypted messages.
 * We CANNOT decrypt any content.
 */

import type { Env } from '../index';
import { verifyDeviceToken } from './auth';
import { checkRateLimit } from './rate-limit';
import { logAccess } from '../crypto/audit';

// WebSocket message types
interface WSMessage {
  type: 'pairing' | 'sync' | 'ack' | 'ping' | 'pong' | 'error';
  payload?: ArrayBuffer | Uint8Array;
  device_id?: string;
  target_device?: string;
  message_id?: string;
  timestamp?: number;
}

// WebSocket connection state
interface WSConnectionState {
  deviceId: string;
  connectedAt: number;
  lastActivity: number;
  pairedDevices: string[];
  isAuthenticated: boolean;
}

// Active connections (in-memory per-worker)
const activeConnections = new Map<WebSocket, WSConnectionState>();

/**
 * Handle WebSocket upgrade request
 */
export async function handleWebSocketUpgrade(
  request: Request,
  env: Env
): Promise<Response> {
  // Verify device token before upgrade
  const auth = await verifyDeviceToken(request, env);
  if (!auth.valid) {
    await logAccess(request, env, 'websocket_upgrade', false, auth.error);
    return new Response(JSON.stringify({ error: auth.error || 'Unauthorized' }), {
      status: 401,
      headers: { 'Content-Type': 'application/json' },
    });
  }
  
  // Check rate limit for WebSocket connections
  const rateLimit = await checkRateLimit(auth.deviceId!, env, 'websocket');
  if (!rateLimit.allowed) {
    await logAccess(request, env, 'websocket_upgrade', false, 'rate_limit_exceeded');
    return new Response(JSON.stringify({ error: 'Rate limit exceeded' }), {
      status: 429,
      headers: { 'Content-Type': 'application/json' },
    });
  }
  
  // Create WebSocket pair
  const [client, server] = Object.values(new WebSocketPair()) as [WebSocket, WebSocket];
  
  // Initialize connection state
  const connectionState: WSConnectionState = {
    deviceId: auth.deviceId!,
    connectedAt: Date.now(),
    lastActivity: Date.now(),
    pairedDevices: [],
    isAuthenticated: true,
  };
  
  activeConnections.set(server, connectionState);
  
  // Accept the WebSocket
  server.accept();
  
  // Send welcome message
  server.send(JSON.stringify({
    type: 'connected',
    device_id: auth.deviceId,
    protocol_version: env.PROTOCOL_VERSION,
    timestamp: Date.now(),
  }));
  
  // Log successful connection
  await logAccess(request, env, 'websocket_connected', true);
  
  // Handle incoming messages
  server.addEventListener('message', async (event) => {
    try {
      await handleWebSocketMessage(server, event, env, connectionState);
    } catch (err) {
      console.error('WebSocket message error:', err);
      server.send(JSON.stringify({
        type: 'error',
        error: 'Message processing failed',
        message_id: crypto.randomUUID(),
      }));
    }
  });
  
  // Handle close
  server.addEventListener('close', async () => {
    activeConnections.delete(server);
    
    // Store disconnect state in KV for potential reconnection
    await env.WEBSOCKET_STATE.put(
      `disconnect:${connectionState.deviceId}`,
      JSON.stringify({
        disconnected_at: Date.now(),
        device_id: connectionState.deviceId,
      }),
      { expirationTtl: 3600 } // 1 hour
    );
    
    await logAccess(request, env, 'websocket_disconnected', true);
  });
  
  // Handle errors
  server.addEventListener('error', async (error) => {
    console.error('WebSocket error:', error);
    activeConnections.delete(server);
    await logAccess(request, env, 'websocket_error', false, error.toString());
  });
  
  // Set up idle timeout
  const idleTimeout = parseInt(env.WEBSOCKET_IDLE_TIMEOUT_MS || '300000');
  const intervalId = setInterval(() => {
    const state = activeConnections.get(server);
    if (state && Date.now() - state.lastActivity > idleTimeout) {
      server.close(1001, 'Idle timeout');
      clearInterval(intervalId);
    }
  }, 60000); // Check every minute
  
  return new Response(null, {
    status: 101,
    webSocket: client,
  });
}

/**
 * Handle WebSocket message
 * CRITICAL: All payloads are encrypted - we only route them
 */
export async function handleWebSocketMessage(
  socket: WebSocket,
  event: MessageEvent,
  env: Env,
  state: WSConnectionState
): Promise<void> {
  // Update last activity
  state.lastActivity = Date.now();
  
  let message: WSMessage;
  
  try {
    // Parse message
    if (typeof event.data === 'string') {
      message = JSON.parse(event.data);
    } else if (event.data instanceof ArrayBuffer) {
      // Binary messages are encrypted payloads - forward as-is
      message = { type: 'sync', payload: event.data };
    } else {
      throw new Error('Invalid message format');
    }
  } catch (err) {
    socket.send(JSON.stringify({
      type: 'error',
      error: 'Invalid message format',
    }));
    return;
  }
  
  // Handle message types
  switch (message.type) {
    case 'ping':
      socket.send(JSON.stringify({ type: 'pong', timestamp: Date.now() }));
      break;
      
    case 'pong':
      // Pong received, connection is alive
      break;
      
    case 'pairing':
      await handlePairingMessage(socket, message, env, state);
      break;
      
    case 'sync':
      await handleSyncMessage(socket, message, env, state);
      break;
      
    case 'ack':
      // Acknowledgment received - could track delivery status
      break;
      
    default:
      socket.send(JSON.stringify({
        type: 'error',
        error: 'Unknown message type',
      }));
  }
}

/**
 * Handle device pairing message
 * SPEC: X3DH key exchange via WebSocket relay
 */
async function handlePairingMessage(
  socket: WebSocket,
  message: WSMessage,
  env: Env,
  state: WSConnectionState
): Promise<void> {
  // Pairing messages contain public key material for X3DH handshake
  // We only forward these to the target device - we NEVER process the keys
  
  if (!message.target_device) {
    socket.send(JSON.stringify({
      type: 'error',
      error: 'target_device required for pairing',
    }));
    return;
  }
  
  // Check if target device is connected
  const targetSocket = findSocketByDeviceId(message.target_device);
  
  if (targetSocket) {
    // Forward pairing message to target device
    targetSocket.send(JSON.stringify({
      type: 'pairing',
      from_device: state.deviceId,
      payload: message.payload ? Array.from(new Uint8Array(message.payload)) : undefined,
      timestamp: Date.now(),
    }));
    
    // Send acknowledgment
    socket.send(JSON.stringify({
      type: 'ack',
      message_id: message.message_id,
      status: 'delivered',
    }));
  } else {
    // Target device offline - store for later delivery
    await storePendingMessage(message.target_device, {
      type: 'pairing',
      from_device: state.deviceId,
      payload: message.payload,
      timestamp: Date.now(),
    }, env);
    
    socket.send(JSON.stringify({
      type: 'ack',
      message_id: message.message_id,
      status: 'stored',
    }));
  }
}

/**
 * Handle sync message (encrypted CRDT updates)
 * CRITICAL: These are encrypted blobs - we only route them
 */
async function handleSyncMessage(
  socket: WebSocket,
  message: WSMessage,
  env: Env,
  state: WSConnectionState
): Promise<void> {
  // Check rate limit for sync messages
  const rateLimit = await checkRateLimit(state.deviceId, env, 'sync');
  if (!rateLimit.allowed) {
    socket.send(JSON.stringify({
      type: 'error',
      error: 'Rate limit exceeded',
      retry_after: 60,
    }));
    return;
  }
  
  // Sync messages can be:
  // 1. Broadcast to all paired devices
  // 2. Targeted to a specific device
  
  if (message.target_device) {
    // Targeted sync
    const targetSocket = findSocketByDeviceId(message.target_device);
    
    if (targetSocket) {
      // Forward encrypted payload directly
      if (message.payload instanceof ArrayBuffer) {
        targetSocket.send(message.payload);
      } else {
        targetSocket.send(JSON.stringify({
          type: 'sync',
          from_device: state.deviceId,
          payload: message.payload,
          timestamp: Date.now(),
        }));
      }
      
      socket.send(JSON.stringify({
        type: 'ack',
        message_id: message.message_id,
        status: 'delivered',
      }));
    } else {
      // Store for later delivery
      await storePendingMessage(message.target_device, {
        type: 'sync',
        from_device: state.deviceId,
        payload: message.payload,
        timestamp: Date.now(),
      }, env);
      
      socket.send(JSON.stringify({
        type: 'ack',
        message_id: message.message_id,
        status: 'stored',
      }));
    }
  } else {
    // Broadcast to all paired devices
    const pairedSockets = findSocketsByDeviceIds(state.pairedDevices);
    
    for (const [deviceId, targetSocket] of pairedSockets) {
      if (targetSocket && targetSocket !== socket) {
        if (message.payload instanceof ArrayBuffer) {
          targetSocket.send(message.payload);
        } else {
          targetSocket.send(JSON.stringify({
            type: 'sync',
            from_device: state.deviceId,
            payload: message.payload,
            timestamp: Date.now(),
          }));
        }
      }
    }
    
    socket.send(JSON.stringify({
      type: 'ack',
      message_id: message.message_id,
      status: 'broadcast',
      recipients: pairedSockets.length,
    }));
  }
}

/**
 * Store a pending message for offline device
 * SPEC: Store-and-forward for 30 days
 */
async function storePendingMessage(
  deviceId: string,
  message: Record<string, unknown>,
  env: Env
): Promise<void> {
  const messageId = crypto.randomUUID();
  const key = `pending:${deviceId}:${messageId}`;
  
  // Store in KV with 30-day expiration
  const retentionDays = parseInt(env.DATA_RETENTION_DAYS || '30');
  const expirationTtl = retentionDays * 24 * 60 * 60;
  
  await env.WEBSOCKET_STATE.put(
    key,
    JSON.stringify(message),
    { expirationTtl }
  );
  
  // Update pending message list for device
  const pendingKey = `pending_list:${deviceId}`;
  const pendingList = await env.WEBSOCKET_STATE.get(pendingKey);
  const messages = pendingList ? JSON.parse(pendingList) : [];
  messages.push(messageId);
  
  await env.WEBSOCKET_STATE.put(
    pendingKey,
    JSON.stringify(messages),
    { expirationTtl }
  );
}

/**
 * Retrieve pending messages for a device
 */
export async function retrievePendingMessages(
  deviceId: string,
  env: Env
): Promise<Record<string, unknown>[]> {
  const pendingKey = `pending_list:${deviceId}`;
  const pendingList = await env.WEBSOCKET_STATE.get(pendingKey);
  
  if (!pendingList) {
    return [];
  }
  
  const messageIds: string[] = JSON.parse(pendingList);
  const messages: Record<string, unknown>[] = [];
  
  for (const messageId of messageIds) {
    const key = `pending:${deviceId}:${messageId}`;
    const message = await env.WEBSOCKET_STATE.get(key);
    if (message) {
      messages.push(JSON.parse(message));
      await env.WEBSOCKET_STATE.delete(key);
    }
  }
  
  // Clear pending list
  await env.WEBSOCKET_STATE.delete(pendingKey);
  
  return messages;
}

/**
 * Find WebSocket by device ID
 */
function findSocketByDeviceId(deviceId: string): WebSocket | null {
  for (const [socket, state] of activeConnections) {
    if (state.deviceId === deviceId) {
      return socket;
    }
  }
  return null;
}

/**
 * Find WebSockets by multiple device IDs
 */
function findSocketsByDeviceIds(deviceIds: string[]): Map<string, WebSocket | null> {
  const result = new Map<string, WebSocket | null>();
  
  for (const deviceId of deviceIds) {
    result.set(deviceId, findSocketByDeviceId(deviceId));
  }
  
  return result;
}

/**
 * Get active connection stats
 */
export function getConnectionStats(): {
  total_connections: number;
  authenticated_connections: number;
} {
  let authenticated = 0;
  
  for (const state of activeConnections.values()) {
    if (state.isAuthenticated) {
      authenticated++;
    }
  }
  
  return {
    total_connections: activeConnections.size,
    authenticated_connections: authenticated,
  };
}
