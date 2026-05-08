/**
 * Blob Handler Tests
 * SPEC v2.0 SECTION 2.3.4 - Relay Server
 */

import { describe, it, expect, beforeEach, vi } from 'vitest';
import { storeBlob, retrieveBlob, deleteBlob } from '../src/handlers/blob';

// Mock R2 bucket
const createMockR2Bucket = () => ({
  put: vi.fn(),
  get: vi.fn(),
  head: vi.fn(),
  delete: vi.fn(),
  list: vi.fn(),
});

// Mock environment
const createMockEnv = () => ({
  SYNC_BLOBS: createMockR2Bucket() as unknown as R2Bucket,
  AUDIT_LOGS: createMockR2Bucket() as unknown as R2Bucket,
  RATE_LIMIT_KV: {} as KVNamespace,
  DEVICE_SESSIONS: {} as KVNamespace,
  WEBSOCKET_STATE: {} as KVNamespace,
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

describe('Blob Handler', () => {
  let env: ReturnType<typeof createMockEnv>;

  beforeEach(() => {
    env = createMockEnv();
    vi.clearAllMocks();
  });

  describe('storeBlob', () => {
    it('should reject empty blobs', async () => {
      const request = new Request('https://relay.truffle.io/blob', {
        method: 'POST',
        headers: {
          'Content-Length': '0',
        },
      });

      const result = await storeBlob(request, env, 'test-device');
      
      expect(result.success).toBe(false);
      expect(result.error).toBe('Empty blob');
    });

    it('should reject blobs that are too large', async () => {
      const request = new Request('https://relay.truffle.io/blob', {
        method: 'POST',
        headers: {
          'Content-Length': (20 * 1024 * 1024).toString(), // 20MB
        },
        body: new ArrayBuffer(20 * 1024 * 1024),
      });

      const result = await storeBlob(request, env, 'test-device');
      
      expect(result.success).toBe(false);
      expect(result.error).toContain('too large');
    });

    it('should reject unencrypted blobs', async () => {
      // Create a plaintext JSON blob
      const plaintext = JSON.stringify({ message: 'This is plaintext' });
      const encoder = new TextEncoder();
      const data = encoder.encode(plaintext);

      const request = new Request('https://relay.truffle.io/blob', {
        method: 'POST',
        headers: {
          'Content-Length': data.length.toString(),
        },
        body: data,
      });

      const result = await storeBlob(request, env, 'test-device');
      
      expect(result.success).toBe(false);
      expect(result.error).toContain('unencrypted');
    });

    it('should store encrypted blobs successfully', async () => {
      // Create encrypted-like data (high entropy)
      const encryptedData = new Uint8Array(1024);
      crypto.getRandomValues(encryptedData);

      const request = new Request('https://relay.truffle.io/blob', {
        method: 'POST',
        headers: {
          'Content-Length': encryptedData.length.toString(),
        },
        body: encryptedData,
      });

      // Mock successful R2 put
      (env.SYNC_BLOBS.put as ReturnType<typeof vi.fn>).mockResolvedValue({
        key: 'test-device/blob_test',
      });

      const result = await storeBlob(request, env, 'test-device');
      
      expect(result.success).toBe(true);
      expect(result.blob_id).toBeDefined();
      expect(result.metadata).toBeDefined();
      expect(result.metadata?.device_id).toBe('test-device');
    });
  });

  describe('retrieveBlob', () => {
    it('should return error for invalid blob ID format', async () => {
      const result = await retrieveBlob('invalid-id', env, 'test-device');
      
      expect(result.success).toBe(false);
      expect(result.error).toBe('Invalid blob ID format');
    });

    it('should return error for non-existent blob', async () => {
      (env.SYNC_BLOBS.get as ReturnType<typeof vi.fn>).mockResolvedValue(null);

      const result = await retrieveBlob('blob_1234567890abcdef1234567890abcdef', env, 'test-device');
      
      expect(result.success).toBe(false);
      expect(result.error).toBe('Blob not found');
    });

    it('should retrieve blob successfully', async () => {
      // Create encrypted-like data
      const encryptedData = new Uint8Array(1024);
      crypto.getRandomValues(encryptedData);

      const mockObject = {
        arrayBuffer: vi.fn().mockResolvedValue(encryptedData.buffer),
        customMetadata: {
          'X-Content-Hash': 'test-hash',
        },
      };

      (env.SYNC_BLOBS.get as ReturnType<typeof vi.fn>).mockResolvedValue(mockObject);

      const result = await retrieveBlob('blob_1234567890abcdef1234567890abcdef', env, 'test-device');
      
      expect(result.success).toBe(true);
      expect(result.data).toBeDefined();
    });
  });

  describe('deleteBlob', () => {
    it('should return error for invalid blob ID format', async () => {
      const result = await deleteBlob('invalid-id', env, 'test-device');
      
      expect(result.success).toBe(false);
      expect(result.error).toBe('Invalid blob ID format');
    });

    it('should delete blob successfully', async () => {
      (env.SYNC_BLOBS.delete as ReturnType<typeof vi.fn>).mockResolvedValue(undefined);

      const result = await deleteBlob('blob_1234567890abcdef1234567890abcdef', env, 'test-device');
      
      expect(result.success).toBe(true);
    });
  });
});
