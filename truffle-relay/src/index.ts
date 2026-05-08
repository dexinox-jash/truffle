/**
 * Project Truffle - Zero-Knowledge Relay Server
 * Main Worker Entry Point
 * 
 * SPEC v2.0 SECTION 2.3.4 - Relay Server
 * SPEC v2.0 SECTION 6.1 - Cloud Architecture
 * 
 * CRITICAL: This is ZERO-ACCESS infrastructure.
 * We CANNOT and MUST NOT decrypt any user content.
 * Store-and-forward encrypted blobs ONLY.
 */

import { Router, error, json } from 'itty-router';
import { handleWebSocketUpgrade, handleWebSocketMessage } from './handlers/websocket';
import { storeBlob, retrieveBlob, deleteBlob } from './handlers/blob';
import { authenticateDevice, verifyDeviceToken } from './handlers/auth';
import { checkRateLimit, getRateLimitStatus } from './handlers/rate-limit';
import { logAccess } from './crypto/audit';
import { verifyZeroAccess } from './crypto/verify';

// Environment bindings interface
export interface Env {
  // R2 Buckets
  SYNC_BLOBS: R2Bucket;
  AUDIT_LOGS: R2Bucket;
  
  // KV Namespaces
  RATE_LIMIT_KV: KVNamespace;
  DEVICE_SESSIONS: KVNamespace;
  WEBSOCKET_STATE: KVNamespace;
  
  // Durable Objects
  WEBSOCKET_HUB: DurableObjectNamespace;
  
  // Analytics
  RELAY_METRICS?: AnalyticsEngineDataset;
  
  // Secrets (set via wrangler secret put)
  CLERK_SECRET_KEY?: string;
  CLERK_PUBLISHABLE_KEY?: string;
  JWT_SECRET?: string;
  
  // Environment variables
  ENVIRONMENT: string;
  PROTOCOL_VERSION: string;
  RATE_LIMIT_FREE: string;
  RATE_LIMIT_PAID: string;
  DATA_RETENTION_DAYS: string;
  MAX_BLOB_SIZE_MB: string;
  WEBSOCKET_IDLE_TIMEOUT_MS: string;
}

// CORS headers for API responses
const corsHeaders = {
  'Access-Control-Allow-Origin': '*',
  'Access-Control-Allow-Methods': 'GET, POST, PUT, DELETE, OPTIONS',
  'Access-Control-Allow-Headers': 'Content-Type, Authorization, X-Device-ID, X-Protocol-Version',
  'Access-Control-Max-Age': '86400',
};

// Create router
const router = Router({ base: '/' });

// Health check endpoint
router.get('/health', async (request: Request, env: Env) => {
  return json({
    status: 'healthy',
    service: 'truffle-relay',
    version: '1.0.0',
    protocol_version: env.PROTOCOL_VERSION,
    environment: env.ENVIRONMENT,
    timestamp: Date.now(),
    zero_access: true,
  }, { headers: corsHeaders });
});

// Zero-access verification endpoint
router.get('/verify-zero-access', async (request: Request, env: Env) => {
  const verification = await verifyZeroAccess(env);
  return json(verification, { headers: corsHeaders });
});

// Device authentication endpoint
router.post('/auth/device', async (request: Request, env: Env) => {
  const result = await authenticateDevice(request, env);
  await logAccess(request, env, 'auth', result.success);
  return json(result, { 
    status: result.success ? 200 : 401,
    headers: corsHeaders 
  });
});

// Rate limit status endpoint
router.get('/rate-limit', async (request: Request, env: Env) => {
  const deviceId = request.headers.get('X-Device-ID');
  if (!deviceId) {
    return json({ error: 'X-Device-ID header required' }, { status: 400, headers: corsHeaders });
  }
  
  const status = await getRateLimitStatus(deviceId, env);
  return json(status, { headers: corsHeaders });
});

// Blob storage endpoints - SPEC: Store-and-forward encrypted blobs
router.post('/blob', async (request: Request, env: Env) => {
  const startTime = Date.now();
  
  // Authenticate device
  const auth = await verifyDeviceToken(request, env);
  if (!auth.valid) {
    await logAccess(request, env, 'blob_store', false, auth.error);
    return json({ error: auth.error || 'Unauthorized' }, { status: 401, headers: corsHeaders });
  }
  
  // Check rate limit
  const rateLimit = await checkRateLimit(auth.deviceId!, env, 'blob');
  if (!rateLimit.allowed) {
    await logAccess(request, env, 'blob_store', false, 'rate_limit_exceeded');
    return json({ 
      error: 'Rate limit exceeded',
      limit: rateLimit.limit,
      remaining: rateLimit.remaining,
      reset_at: rateLimit.resetAt,
    }, { status: 429, headers: corsHeaders });
  }
  
  // Store blob
  const result = await storeBlob(request, env, auth.deviceId!);
  await logAccess(request, env, 'blob_store', result.success, undefined, Date.now() - startTime);
  
  return json(result, { 
    status: result.success ? 201 : 400,
    headers: corsHeaders 
  });
});

router.get('/blob/:blobId', async (request: Request, env: Env) => {
  const startTime = Date.now();
  const { blobId } = request.params as { blobId: string };
  
  // Authenticate device
  const auth = await verifyDeviceToken(request, env);
  if (!auth.valid) {
    await logAccess(request, env, 'blob_retrieve', false, auth.error);
    return json({ error: auth.error || 'Unauthorized' }, { status: 401, headers: corsHeaders });
  }
  
  // Check rate limit
  const rateLimit = await checkRateLimit(auth.deviceId!, env, 'blob');
  if (!rateLimit.allowed) {
    await logAccess(request, env, 'blob_retrieve', false, 'rate_limit_exceeded');
    return json({ 
      error: 'Rate limit exceeded',
      limit: rateLimit.limit,
      remaining: rateLimit.remaining,
      reset_at: rateLimit.resetAt,
    }, { status: 429, headers: corsHeaders });
  }
  
  // Retrieve blob
  const result = await retrieveBlob(blobId, env, auth.deviceId!);
  await logAccess(request, env, 'blob_retrieve', result.success, undefined, Date.now() - startTime);
  
  if (!result.success) {
    return json({ error: result.error }, { status: 404, headers: corsHeaders });
  }
  
  // Return blob with appropriate headers
  return new Response(result.data, {
    status: 200,
    headers: {
      ...corsHeaders,
      'Content-Type': 'application/octet-stream',
      'X-Blob-ID': blobId,
      'X-Device-ID': auth.deviceId!,
      'Cache-Control': 'no-store, no-cache, must-revalidate',
    },
  });
});

router.delete('/blob/:blobId', async (request: Request, env: Env) => {
  const startTime = Date.now();
  const { blobId } = request.params as { blobId: string };
  
  // Authenticate device
  const auth = await verifyDeviceToken(request, env);
  if (!auth.valid) {
    await logAccess(request, env, 'blob_delete', false, auth.error);
    return json({ error: auth.error || 'Unauthorized' }, { status: 401, headers: corsHeaders });
  }
  
  // Delete blob
  const result = await deleteBlob(blobId, env, auth.deviceId!);
  await logAccess(request, env, 'blob_delete', result.success, undefined, Date.now() - startTime);
  
  return json(result, { 
    status: result.success ? 200 : 404,
    headers: corsHeaders 
  });
});

// WebSocket upgrade endpoint - SPEC: Real-time sync
router.get('/ws', async (request: Request, env: Env) => {
  const upgradeHeader = request.headers.get('Upgrade');
  if (upgradeHeader !== 'websocket') {
    return json({ error: 'Expected Upgrade: websocket' }, { status: 400, headers: corsHeaders });
  }
  
  return handleWebSocketUpgrade(request, env);
});

// Options handler for CORS preflight
router.options('*', () => {
  return new Response(null, { 
    status: 204, 
    headers: corsHeaders 
  });
});

// 404 handler
router.all('*', () => {
  return json({ 
    error: 'Not Found',
    service: 'truffle-relay',
    documentation: 'https://docs.truffle.io/relay-api',
  }, { status: 404, headers: corsHeaders });
});

// Main fetch handler
export default {
  async fetch(request: Request, env: Env, ctx: ExecutionContext): Promise<Response> {
    // Add security headers to all responses
    const securityHeaders = {
      'X-Content-Type-Options': 'nosniff',
      'X-Frame-Options': 'DENY',
      'X-XSS-Protection': '1; mode=block',
      'Referrer-Policy': 'strict-origin-when-cross-origin',
      'Strict-Transport-Security': 'max-age=31536000; includeSubDomains; preload',
      'Content-Security-Policy': "default-src 'none'; frame-ancestors 'none';",
      'X-Truffle-Relay': 'zero-access',
    };
    
    try {
      const response = await router.handle(request, env, ctx);
      
      // Add security headers to response
      Object.entries(securityHeaders).forEach(([key, value]) => {
        response.headers.set(key, value);
      });
      
      return response;
    } catch (err) {
      console.error('Relay error:', err);
      
      // Log error
      await logAccess(request, env, 'error', false, err instanceof Error ? err.message : 'Unknown error');
      
      return json({ 
        error: 'Internal Server Error',
        request_id: crypto.randomUUID(),
      }, { 
        status: 500, 
        headers: { ...corsHeaders, ...securityHeaders }
      });
    }
  },
};
