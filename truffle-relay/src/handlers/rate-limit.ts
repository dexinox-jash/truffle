/**
 * Rate Limiting Handler
 * SPEC v2.0 SECTION 2.3.4 - Relay Server
 * 
 * Rate limits:
 * - Free tier: 100 messages/day/device
 * - Paid tier: 10,000 messages/day/device
 * 
 * CRITICAL: Rate limits are per-device, not per-user. We have no user data.
 */

import type { Env } from '../index';
import { getDeviceTier } from './auth';

// Rate limit result
interface RateLimitResult {
  allowed: boolean;
  limit: number;
  remaining: number;
  resetAt: number;
  windowStart: number;
}

// Rate limit status
interface RateLimitStatus {
  device_id: string;
  tier: 'free' | 'paid' | 'enterprise';
  daily_limit: number;
  used_today: number;
  remaining_today: number;
  resets_at: number;
  window_start: number;
}

// Rate limit window (24 hours in milliseconds)
const RATE_LIMIT_WINDOW_MS = 24 * 60 * 60 * 1000;

// Default rate limits
const DEFAULT_FREE_LIMIT = 100;
const DEFAULT_PAID_LIMIT = 10000;
const DEFAULT_ENTERPRISE_LIMIT = 100000;

// Rate limit types
 type RateLimitType = 'blob' | 'sync' | 'websocket' | 'auth';

// Rate limit multipliers by type
const RATE_LIMIT_MULTIPLIERS: Record<RateLimitType, number> = {
  blob: 1,
  sync: 0.5, // Sync messages count as half
  websocket: 0.1, // WebSocket messages count as 0.1
  auth: 0, // Auth requests don't count
};

/**
 * Check rate limit for a device
 */
export async function checkRateLimit(
  deviceId: string,
  env: Env,
  type: RateLimitType = 'blob'
): Promise<RateLimitResult> {
  // Get device tier
  const tier = await getDeviceTier(deviceId, env);
  
  // Get rate limit for tier
  const dailyLimit = getRateLimitForTier(tier, env);
  
  // Get current rate limit data
  const rateLimitKey = `rate_limit:${deviceId}`;
  const rateLimitData = await env.RATE_LIMIT_KV.get(rateLimitKey);
  
  const now = Date.now();
  let windowStart: number;
  let requestCount: number;
  
  if (rateLimitData) {
    const data = JSON.parse(rateLimitData);
    windowStart = data.window_start;
    requestCount = data.request_count;
    
    // Check if window has reset
    if (now - windowStart >= RATE_LIMIT_WINDOW_MS) {
      // Reset window
      windowStart = now;
      requestCount = 0;
    }
  } else {
    // New window
    windowStart = now;
    requestCount = 0;
  }
  
  // Calculate cost for this request
  const cost = RATE_LIMIT_MULTIPLIERS[type];
  
  // Check if request is allowed
  const newCount = requestCount + cost;
  const allowed = newCount <= dailyLimit;
  
  if (allowed) {
    // Update rate limit data
    await env.RATE_LIMIT_KV.put(
      rateLimitKey,
      JSON.stringify({
        window_start: windowStart,
        request_count: newCount,
        last_request: now,
      }),
      { expirationTtl: Math.ceil(RATE_LIMIT_WINDOW_MS / 1000) }
    );
  }
  
  return {
    allowed,
    limit: dailyLimit,
    remaining: Math.max(0, dailyLimit - newCount),
    resetAt: windowStart + RATE_LIMIT_WINDOW_MS,
    windowStart,
  };
}

/**
 * Get rate limit status for a device
 */
export async function getRateLimitStatus(
  deviceId: string,
  env: Env
): Promise<RateLimitStatus> {
  // Get device tier
  const tier = await getDeviceTier(deviceId, env);
  
  // Get rate limit for tier
  const dailyLimit = getRateLimitForTier(tier, env);
  
  // Get current rate limit data
  const rateLimitKey = `rate_limit:${deviceId}`;
  const rateLimitData = await env.RATE_LIMIT_KV.get(rateLimitKey);
  
  const now = Date.now();
  
  if (rateLimitData) {
    const data = JSON.parse(rateLimitData);
    const windowStart = data.window_start;
    const requestCount = data.request_count;
    
    // Check if window has reset
    if (now - windowStart >= RATE_LIMIT_WINDOW_MS) {
      return {
        device_id: deviceId,
        tier,
        daily_limit: dailyLimit,
        used_today: 0,
        remaining_today: dailyLimit,
        resets_at: now + RATE_LIMIT_WINDOW_MS,
        window_start: now,
      };
    }
    
    return {
      device_id: deviceId,
      tier,
      daily_limit: dailyLimit,
      used_today: Math.floor(requestCount),
      remaining_today: Math.max(0, dailyLimit - Math.floor(requestCount)),
      resets_at: windowStart + RATE_LIMIT_WINDOW_MS,
      window_start: windowStart,
    };
  }
  
  // No rate limit data yet
  return {
    device_id: deviceId,
    tier,
    daily_limit: dailyLimit,
    used_today: 0,
    remaining_today: dailyLimit,
    resets_at: now + RATE_LIMIT_WINDOW_MS,
    window_start: now,
  };
}

/**
 * Reset rate limit for a device
 */
export async function resetRateLimit(
  deviceId: string,
  env: Env
): Promise<{ success: boolean }> {
  const rateLimitKey = `rate_limit:${deviceId}`;
  await env.RATE_LIMIT_KV.delete(rateLimitKey);
  return { success: true };
}

/**
 * Get rate limit for tier
 */
function getRateLimitForTier(tier: 'free' | 'paid' | 'enterprise', env: Env): number {
  switch (tier) {
    case 'free':
      return parseInt(env.RATE_LIMIT_FREE || DEFAULT_FREE_LIMIT.toString());
    case 'paid':
      return parseInt(env.RATE_LIMIT_PAID || DEFAULT_PAID_LIMIT.toString());
    case 'enterprise':
      return DEFAULT_ENTERPRISE_LIMIT;
    default:
      return DEFAULT_FREE_LIMIT;
  }
}

/**
 * Check bulk rate limit for multiple operations
 */
export async function checkBulkRateLimit(
  deviceId: string,
  env: Env,
  operationCount: number,
  type: RateLimitType = 'blob'
): Promise<RateLimitResult> {
  // Get device tier
  const tier = await getDeviceTier(deviceId, env);
  
  // Get rate limit for tier
  const dailyLimit = getRateLimitForTier(tier, env);
  
  // Get current rate limit data
  const rateLimitKey = `rate_limit:${deviceId}`;
  const rateLimitData = await env.RATE_LIMIT_KV.get(rateLimitKey);
  
  const now = Date.now();
  let windowStart: number;
  let requestCount: number;
  
  if (rateLimitData) {
    const data = JSON.parse(rateLimitData);
    windowStart = data.window_start;
    requestCount = data.request_count;
    
    // Check if window has reset
    if (now - windowStart >= RATE_LIMIT_WINDOW_MS) {
      windowStart = now;
      requestCount = 0;
    }
  } else {
    windowStart = now;
    requestCount = 0;
  }
  
  // Calculate cost for bulk operations
  const cost = operationCount * RATE_LIMIT_MULTIPLIERS[type];
  
  // Check if operations are allowed
  const newCount = requestCount + cost;
  const allowed = newCount <= dailyLimit;
  
  if (allowed) {
    // Update rate limit data
    await env.RATE_LIMIT_KV.put(
      rateLimitKey,
      JSON.stringify({
        window_start: windowStart,
        request_count: newCount,
        last_request: now,
      }),
      { expirationTtl: Math.ceil(RATE_LIMIT_WINDOW_MS / 1000) }
    );
  }
  
  return {
    allowed,
    limit: dailyLimit,
    remaining: Math.max(0, dailyLimit - newCount),
    resetAt: windowStart + RATE_LIMIT_WINDOW_MS,
    windowStart,
  };
}

/**
 * Get rate limit headers for response
 */
export function getRateLimitHeaders(result: RateLimitResult): Record<string, string> {
  return {
    'X-RateLimit-Limit': result.limit.toString(),
    'X-RateLimit-Remaining': Math.floor(result.remaining).toString(),
    'X-RateLimit-Reset': Math.floor(result.resetAt / 1000).toString(),
    'X-RateLimit-Window': '86400',
  };
}

/**
 * Increment rate limit counter (for tracking without blocking)
 */
export async function incrementRateLimit(
  deviceId: string,
  env: Env,
  type: RateLimitType = 'blob'
): Promise<void> {
  // Get current rate limit data
  const rateLimitKey = `rate_limit:${deviceId}`;
  const rateLimitData = await env.RATE_LIMIT_KV.get(rateLimitKey);
  
  const now = Date.now();
  let windowStart: number;
  let requestCount: number;
  
  if (rateLimitData) {
    const data = JSON.parse(rateLimitData);
    windowStart = data.window_start;
    requestCount = data.request_count;
    
    // Check if window has reset
    if (now - windowStart >= RATE_LIMIT_WINDOW_MS) {
      windowStart = now;
      requestCount = 0;
    }
  } else {
    windowStart = now;
    requestCount = 0;
  }
  
  // Increment counter
  const cost = RATE_LIMIT_MULTIPLIERS[type];
  const newCount = requestCount + cost;
  
  // Update rate limit data
  await env.RATE_LIMIT_KV.put(
    rateLimitKey,
    JSON.stringify({
      window_start: windowStart,
      request_count: newCount,
      last_request: now,
    }),
    { expirationTtl: Math.ceil(RATE_LIMIT_WINDOW_MS / 1000) }
  );
}

/**
 * Get global rate limit statistics
 */
export async function getGlobalRateLimitStats(env: Env): Promise<{
  total_devices: number;
  devices_at_limit: number;
  average_usage: number;
}> {
  // List all rate limit keys
  const keys = await env.RATE_LIMIT_KV.list({ prefix: 'rate_limit:' });
  
  let totalDevices = 0;
  let devicesAtLimit = 0;
  let totalUsage = 0;
  
  for (const key of keys.keys) {
    const data = await env.RATE_LIMIT_KV.get(key.name);
    if (data) {
      const rateLimit = JSON.parse(data);
      totalDevices++;
      totalUsage += rateLimit.request_count;
      
      // Check if at limit (assume free tier for simplicity)
      if (rateLimit.request_count >= DEFAULT_FREE_LIMIT) {
        devicesAtLimit++;
      }
    }
  }
  
  return {
    total_devices: totalDevices,
    devices_at_limit: devicesAtLimit,
    average_usage: totalDevices > 0 ? totalUsage / totalDevices : 0,
  };
}
