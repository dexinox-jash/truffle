/**
 * Rate Limit Handler Tests
 * SPEC v2.0 SECTION 2.3.4 - Relay Server
 */

import { describe, it, expect, beforeEach, vi } from 'vitest';
import { checkRateLimit, getRateLimitStatus, resetRateLimit } from '../src/handlers/rate-limit';

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
    get: vi.fn().mockResolvedValue(JSON.stringify({ tier: 'free' })),
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
  JWT_SECRET: 'test-secret',
});

describe('Rate Limit Handler', () => {
  let env: ReturnType<typeof createMockEnv>;

  beforeEach(() => {
    env = createMockEnv();
    vi.clearAllMocks();
  });

  describe('checkRateLimit', () => {
    it('should allow requests under the limit', async () => {
      env.RATE_LIMIT_KV.get = vi.fn().mockResolvedValue(null);

      const result = await checkRateLimit('test-device', env, 'blob');
      
      expect(result.allowed).toBe(true);
      expect(result.limit).toBe(100);
      expect(result.remaining).toBeGreaterThan(0);
    });

    it('should track request count correctly', async () => {
      env.RATE_LIMIT_KV.get = vi.fn().mockResolvedValue(JSON.stringify({
        window_start: Date.now(),
        request_count: 50,
      }));

      const result = await checkRateLimit('test-device', env, 'blob');
      
      expect(result.allowed).toBe(true);
      expect(env.RATE_LIMIT_KV.put).toHaveBeenCalled();
    });

    it('should block requests over the limit', async () => {
      env.RATE_LIMIT_KV.get = vi.fn().mockResolvedValue(JSON.stringify({
        window_start: Date.now(),
        request_count: 100,
      }));

      const result = await checkRateLimit('test-device', env, 'blob');
      
      expect(result.allowed).toBe(false);
      expect(result.remaining).toBe(0);
    });

    it('should reset window after 24 hours', async () => {
      const oldTimestamp = Date.now() - (25 * 60 * 60 * 1000); // 25 hours ago
      
      env.RATE_LIMIT_KV.get = vi.fn().mockResolvedValue(JSON.stringify({
        window_start: oldTimestamp,
        request_count: 1000, // Would be over limit if not reset
      }));

      const result = await checkRateLimit('test-device', env, 'blob');
      
      expect(result.allowed).toBe(true);
      expect(result.remaining).toBeGreaterThan(0);
    });

    it('should apply different costs for different operation types', async () => {
      env.RATE_LIMIT_KV.get = vi.fn().mockResolvedValue(null);

      const blobResult = await checkRateLimit('test-device', env, 'blob');
      const syncResult = await checkRateLimit('test-device', env, 'sync');
      const wsResult = await checkRateLimit('test-device', env, 'websocket');
      
      // All should be allowed for new window
      expect(blobResult.allowed).toBe(true);
      expect(syncResult.allowed).toBe(true);
      expect(wsResult.allowed).toBe(true);
    });

    it('should use paid tier limits for paid devices', async () => {
      env.DEVICE_SESSIONS.get = vi.fn().mockResolvedValue(JSON.stringify({ tier: 'paid' }));
      env.RATE_LIMIT_KV.get = vi.fn().mockResolvedValue(null);

      const result = await checkRateLimit('paid-device', env, 'blob');
      
      expect(result.limit).toBe(10000);
    });
  });

  describe('getRateLimitStatus', () => {
    it('should return status for new device', async () => {
      env.RATE_LIMIT_KV.get = vi.fn().mockResolvedValue(null);

      const result = await getRateLimitStatus('test-device', env);
      
      expect(result.device_id).toBe('test-device');
      expect(result.tier).toBe('free');
      expect(result.daily_limit).toBe(100);
      expect(result.used_today).toBe(0);
      expect(result.remaining_today).toBe(100);
    });

    it('should return current usage for existing device', async () => {
      env.RATE_LIMIT_KV.get = vi.fn().mockResolvedValue(JSON.stringify({
        window_start: Date.now(),
        request_count: 42,
      }));

      const result = await getRateLimitStatus('test-device', env);
      
      expect(result.used_today).toBe(42);
      expect(result.remaining_today).toBe(58);
    });
  });

  describe('resetRateLimit', () => {
    it('should reset rate limit for device', async () => {
      env.RATE_LIMIT_KV.delete = vi.fn().mockResolvedValue(undefined);

      const result = await resetRateLimit('test-device', env);
      
      expect(result.success).toBe(true);
      expect(env.RATE_LIMIT_KV.delete).toHaveBeenCalledWith('rate_limit:test-device');
    });
  });
});
