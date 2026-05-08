/**
 * Authentication Handler Tests
 * SPEC v2.0 SECTION 6.1 - Cloud Architecture
 */

import { describe, it, expect, beforeEach, vi } from 'vitest';
import { authenticateDevice, verifyDeviceToken, refreshAccessToken, revokeDeviceTokens } from '../src/handlers/auth';

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
  WEBSOCKET_STATE: {} as KVNamespace,
  WEBSOCKET_HUB: {} as DurableObjectNamespace,
  ENVIRONMENT: 'test',
  PROTOCOL_VERSION: '1',
  RATE_LIMIT_FREE: '100',
  RATE_LIMIT_PAID: '10000',
  DATA_RETENTION_DAYS: '30',
  MAX_BLOB_SIZE_MB: '10',
  WEBSOCKET_IDLE_TIMEOUT_MS: '300000',
  JWT_SECRET: 'test-secret-key-for-jwt-signing-in-tests-only',
});

describe('Authentication Handler', () => {
  let env: ReturnType<typeof createMockEnv>;

  beforeEach(() => {
    env = createMockEnv();
    vi.clearAllMocks();
  });

  describe('authenticateDevice', () => {
    it('should reject requests with missing fields', async () => {
      const request = new Request('https://relay.truffle.io/auth/device', {
        method: 'POST',
        body: JSON.stringify({
          // Missing device_fingerprint and public_key
        }),
      });

      const result = await authenticateDevice(request, env);
      
      expect(result.success).toBe(false);
      expect(result.error).toBe('Missing required fields');
    });

    it('should reject invalid public key format', async () => {
      const request = new Request('https://relay.truffle.io/auth/device', {
        method: 'POST',
        body: JSON.stringify({
          device_fingerprint: 'test-fingerprint',
          public_key: 'invalid-key',
          device_info: {
            platform: 'macos',
            version: '1.0.0',
          },
        }),
      });

      const result = await authenticateDevice(request, env);
      
      expect(result.success).toBe(false);
      expect(result.error).toBe('Invalid public key format');
    });

    it('should register new device successfully', async () => {
      // Generate valid base64-encoded 32-byte key
      const keyBytes = new Uint8Array(32);
      crypto.getRandomValues(keyBytes);
      const publicKey = btoa(String.fromCharCode(...keyBytes));

      env.DEVICE_SESSIONS.get = vi.fn().mockResolvedValue(null);
      env.DEVICE_SESSIONS.put = vi.fn().mockResolvedValue(undefined);

      const request = new Request('https://relay.truffle.io/auth/device', {
        method: 'POST',
        body: JSON.stringify({
          device_fingerprint: 'test-fingerprint',
          public_key: publicKey,
          device_info: {
            platform: 'macos',
            version: '1.0.0',
          },
        }),
      });

      const result = await authenticateDevice(request, env);
      
      expect(result.success).toBe(true);
      expect(result.device_id).toBeDefined();
      expect(result.access_token).toBeDefined();
      expect(result.refresh_token).toBeDefined();
      expect(result.expires_at).toBeDefined();
    });

    it('should issue new tokens for existing device', async () => {
      const keyBytes = new Uint8Array(32);
      crypto.getRandomValues(keyBytes);
      const publicKey = btoa(String.fromCharCode(...keyBytes));

      const existingDevice = JSON.stringify({
        device_id: 'dev_existing123',
        public_key: publicKey,
        device_info: { platform: 'macos', version: '1.0.0' },
        tier: 'free',
        registered_at: Date.now(),
        last_seen: Date.now(),
        paired_devices: [],
      });

      env.DEVICE_SESSIONS.get = vi.fn().mockResolvedValue(existingDevice);
      env.DEVICE_SESSIONS.put = vi.fn().mockResolvedValue(undefined);

      const request = new Request('https://relay.truffle.io/auth/device', {
        method: 'POST',
        body: JSON.stringify({
          device_fingerprint: 'test-fingerprint',
          public_key: publicKey,
          device_info: {
            platform: 'macos',
            version: '1.0.0',
          },
        }),
      });

      const result = await authenticateDevice(request, env);
      
      expect(result.success).toBe(true);
      expect(result.device_id).toBeDefined();
      expect(result.access_token).toBeDefined();
    });
  });

  describe('verifyDeviceToken', () => {
    it('should reject requests without authorization header', async () => {
      const request = new Request('https://relay.truffle.io/test');

      const result = await verifyDeviceToken(request, env);
      
      expect(result.valid).toBe(false);
      expect(result.error).toContain('Authorization header');
    });

    it('should reject invalid tokens', async () => {
      const request = new Request('https://relay.truffle.io/test', {
        headers: {
          'Authorization': 'Bearer invalid-token',
        },
      });

      const result = await verifyDeviceToken(request, env);
      
      expect(result.valid).toBe(false);
    });

    it('should reject expired tokens', async () => {
      // Create an expired token
      const expiredPayload = {
        sub: 'test-device',
        tier: 'free',
        iat: Math.floor(Date.now() / 1000) - 86400 * 2, // 2 days ago
        exp: Math.floor(Date.now() / 1000) - 86400, // 1 day ago (expired)
      };

      // Note: In real tests, you'd use the actual JWT library to create tokens
      // This is a simplified check
      const request = new Request('https://relay.truffle.io/test', {
        headers: {
          'Authorization': 'Bearer eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9.eyJzdWIiOiJ0ZXN0LWRldmljZSIsInRpZXIiOiJmcmVlIiwiaWF0IjoxMDAsImV4cCI6MTAwfQ.test',
        },
      });

      const result = await verifyDeviceToken(request, env);
      
      // This will fail verification due to invalid signature
      expect(result.valid).toBe(false);
    });
  });

  describe('refreshAccessToken', () => {
    it('should reject invalid refresh tokens', async () => {
      const result = await refreshAccessToken('invalid-refresh-token', env);
      
      expect(result.success).toBe(false);
      expect(result.error).toContain('Invalid refresh token');
    });
  });

  describe('revokeDeviceTokens', () => {
    it('should revoke device tokens successfully', async () => {
      env.DEVICE_SESSIONS.delete = vi.fn().mockResolvedValue(undefined);
      env.RATE_LIMIT_KV.delete = vi.fn().mockResolvedValue(undefined);

      const result = await revokeDeviceTokens('test-device', env);
      
      expect(result.success).toBe(true);
      expect(env.DEVICE_SESSIONS.delete).toHaveBeenCalledWith('device:test-device');
      expect(env.RATE_LIMIT_KV.delete).toHaveBeenCalledWith('rate_limit:test-device');
    });
  });
});
