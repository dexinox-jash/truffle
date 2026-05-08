/**
 * WebSocket Handler Tests
 * SPEC v2.0 SECTION 2.3.4 - Relay Server
 */

import { describe, it, expect, beforeEach, vi } from 'vitest';
import { handleWebSocketUpgrade, retrievePendingMessages } from '../src/handlers/websocket';

// Mock environment
const createMockEnv = () => ({
  SYNC_BLOBS: {} as R2Bucket,
  AUDIT_LOGS: {} as R2Bucket,
  RATE_LIMIT_KV: {
    get: vi.fn(),
    put: vi.fn(),
    delete: vi.fn(),
    list: vi.fn(),
  } as unknown as KVNamespace,
  DEVICE_SESSIONS: {
    get: vi.fn(),
    put: vi.fn(),
    delete: vi.fn(),
    list: vi.fn(),
  } as unknown as KVNamespace,
  WEBSOCKET_STATE: {
    get: vi.fn(),
    put: vi.fn(),
    delete: vi.fn(),
    list: vi.fn(),
  } as unknown as KVNamespace,
  WEBSOCKET_HUB: {} as DurableObjectNamespace,
  ENVIRONMENT: 'test',
  PROTOCOL_VERSION: '1',
  RATE_LIMIT_FREE: '100',
  RATE_LIMIT_PAID: '10000',
  DATA_RETENTION_DAYS: '30',
  MAX_BLOB_SIZE_MB: '10',
  WEBSOCKET_IDLE_TIMEOUT_MS: '300000',
  JWT_SECRET: 'test-secret',
});

describe('WebSocket Handler', () => {
  let env: ReturnType<typeof createMockEnv>;

  beforeEach(() => {
    env = createMockEnv();
    vi.clearAllMocks();
  });

  describe('handleWebSocketUpgrade', () => {
    it('should reject requests without Upgrade header', async () => {
      const request = new Request('https://relay.truffle.io/ws', {
        headers: {
          'Authorization': 'Bearer valid-token',
          'X-Device-ID': 'test-device',
        },
      });

      const response = await handleWebSocketUpgrade(request, env);
      
      expect(response.status).toBe(400);
      const body = await response.json();
      expect(body.error).toContain('Upgrade: websocket');
    });

    it('should reject requests without authorization', async () => {
      const request = new Request('https://relay.truffle.io/ws', {
        headers: {
          'Upgrade': 'websocket',
          'X-Device-ID': 'test-device',
        },
      });

      const response = await handleWebSocketUpgrade(request, env);
      
      expect(response.status).toBe(401);
    });

    it('should reject rate-limited devices', async () => {
      // Mock rate limit exceeded
      env.RATE_LIMIT_KV.get = vi.fn().mockResolvedValue(JSON.stringify({
        window_start: Date.now(),
        request_count: 1000,
      }));

      const request = new Request('https://relay.truffle.io/ws', {
        headers: {
          'Upgrade': 'websocket',
          'Authorization': 'Bearer valid-token',
          'X-Device-ID': 'test-device',
        },
      });

      const response = await handleWebSocketUpgrade(request, env);
      
      expect(response.status).toBe(429);
    });
  });

  describe('retrievePendingMessages', () => {
    it('should return empty array when no pending messages', async () => {
      env.WEBSOCKET_STATE.get = vi.fn().mockResolvedValue(null);

      const messages = await retrievePendingMessages('test-device', env);
      
      expect(messages).toEqual([]);
    });

    it('should retrieve and delete pending messages', async () => {
      const pendingList = JSON.stringify(['msg1', 'msg2']);
      const message1 = JSON.stringify({ type: 'sync', data: 'test1' });
      const message2 = JSON.stringify({ type: 'sync', data: 'test2' });

      env.WEBSOCKET_STATE.get = vi.fn()
        .mockResolvedValueOnce(pendingList)
        .mockResolvedValueOnce(message1)
        .mockResolvedValueOnce(message2);
      
      env.WEBSOCKET_STATE.delete = vi.fn().mockResolvedValue(undefined);

      const messages = await retrievePendingMessages('test-device', env);
      
      expect(messages).toHaveLength(2);
      expect(env.WEBSOCKET_STATE.delete).toHaveBeenCalledWith('pending_list:test-device');
      expect(env.WEBSOCKET_STATE.delete).toHaveBeenCalledWith('pending:test-device:msg1');
      expect(env.WEBSOCKET_STATE.delete).toHaveBeenCalledWith('pending:test-device:msg2');
    });
  });
});
