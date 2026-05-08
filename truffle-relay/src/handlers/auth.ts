/**
 * Authentication Handler - Device Authentication
 * SPEC v2.0 SECTION 6.1 - Cloud Architecture
 * SPEC v2.0 SECTION 2.3.2 - Device Pairing Ceremony
 * 
 * Uses Clerk.com for device management only - NO user data access.
 * We only authenticate that a device is registered and valid.
 */

import type { Env } from '../index';
import jwt from '@tsndr/cloudflare-worker-jwt';

// Device authentication result
interface AuthResult {
  valid: boolean;
  deviceId?: string;
  tier?: 'free' | 'paid' | 'enterprise';
  error?: string;
}

// Device registration request
interface DeviceRegistrationRequest {
  device_fingerprint: string;
  public_key: string; // Ed25519 public key (for verification only, not stored)
  device_info: {
    platform: 'macos' | 'windows' | 'linux' | 'ios' | 'android';
    version: string;
    model?: string;
  };
  pairing_token?: string; // For secondary device pairing
}

// Device registration response
interface DeviceRegistrationResponse {
  success: boolean;
  device_id?: string;
  access_token?: string;
  refresh_token?: string;
  expires_at?: number;
  error?: string;
}

// Clerk session verification result
interface ClerkSession {
  sub: string; // Device ID
  tier: 'free' | 'paid' | 'enterprise';
  iat: number;
  exp: number;
}

// JWT secret (should be set via wrangler secret put)
const JWT_ALGORITHM = 'HS256';
const TOKEN_EXPIRY_SECONDS = 86400; // 24 hours
const REFRESH_TOKEN_EXPIRY_SECONDS = 604800; // 7 days

/**
 * Authenticate device and issue tokens
 * SPEC: Device registration and token issuance
 */
export async function authenticateDevice(
  request: Request,
  env: Env
): Promise<DeviceRegistrationResponse> {
  try {
    // Parse request body
    const body: DeviceRegistrationRequest = await request.json();
    
    // Validate required fields
    if (!body.device_fingerprint || !body.public_key) {
      return { success: false, error: 'Missing required fields' };
    }
    
    // Validate public key format (Ed25519 keys are 32 bytes, base64 encoded = 44 chars)
    if (!isValidPublicKey(body.public_key)) {
      return { success: false, error: 'Invalid public key format' };
    }
    
    // Generate device ID from fingerprint
    const deviceId = await generateDeviceId(body.device_fingerprint);
    
    // Check if device already exists
    const existingDevice = await env.DEVICE_SESSIONS.get(`device:${deviceId}`);
    
    if (existingDevice) {
      // Device exists - issue new tokens
      const tokens = await generateTokens(deviceId, env);
      
      // Update last seen
      const deviceData = JSON.parse(existingDevice);
      deviceData.last_seen = Date.now();
      await env.DEVICE_SESSIONS.put(
        `device:${deviceId}`,
        JSON.stringify(deviceData),
        { expirationTtl: 90 * 24 * 60 * 60 } // 90 days
      );
      
      return {
        success: true,
        device_id: deviceId,
        ...tokens,
      };
    }
    
    // New device registration
    // Determine tier (default to free)
    let tier: 'free' | 'paid' | 'enterprise' = 'free';
    
    // If pairing token provided, validate and inherit tier from primary device
    if (body.pairing_token) {
      const primaryDevice = await validatePairingToken(body.pairing_token, env);
      if (primaryDevice) {
        tier = primaryDevice.tier;
      }
    }
    
    // Store device info (NO private keys, NO user data)
    const deviceData = {
      device_id: deviceId,
      public_key: body.public_key, // For signature verification only
      device_info: body.device_info,
      tier,
      registered_at: Date.now(),
      last_seen: Date.now(),
      paired_devices: [],
    };
    
    await env.DEVICE_SESSIONS.put(
      `device:${deviceId}`,
      JSON.stringify(deviceData),
      { expirationTtl: 90 * 24 * 60 * 60 } // 90 days
    );
    
    // Generate tokens
    const tokens = await generateTokens(deviceId, env, tier);
    
    return {
      success: true,
      device_id: deviceId,
      ...tokens,
    };
  } catch (err) {
    console.error('Device authentication error:', err);
    return { 
      success: false, 
      error: err instanceof Error ? err.message : 'Authentication failed' 
    };
  }
}

/**
 * Verify device token from request
 */
export async function verifyDeviceToken(
  request: Request,
  env: Env
): Promise<AuthResult> {
  try {
    // Get token from Authorization header
    const authHeader = request.headers.get('Authorization');
    if (!authHeader || !authHeader.startsWith('Bearer ')) {
      return { valid: false, error: 'Missing or invalid Authorization header' };
    }
    
    const token = authHeader.slice(7);
    
    // Verify JWT
    if (!env.JWT_SECRET) {
      console.error('JWT_SECRET not configured');
      return { valid: false, error: 'Server configuration error' };
    }
    
    const isValid = await jwt.verify(token, env.JWT_SECRET, JWT_ALGORITHM);
    if (!isValid) {
      return { valid: false, error: 'Invalid token' };
    }
    
    // Decode and validate claims
    const decoded = jwt.decode(token) as { payload?: ClerkSession };
    if (!decoded.payload) {
      return { valid: false, error: 'Invalid token payload' };
    }
    
    const payload = decoded.payload;
    
    // Check expiration
    if (payload.exp < Math.floor(Date.now() / 1000)) {
      return { valid: false, error: 'Token expired' };
    }
    
    // Verify device exists
    const deviceData = await env.DEVICE_SESSIONS.get(`device:${payload.sub}`);
    if (!deviceData) {
      return { valid: false, error: 'Device not found' };
    }
    
    return {
      valid: true,
      deviceId: payload.sub,
      tier: payload.tier,
    };
  } catch (err) {
    console.error('Token verification error:', err);
    return { valid: false, error: 'Token verification failed' };
  }
}

/**
 * Refresh access token
 */
export async function refreshAccessToken(
  refreshToken: string,
  env: Env
): Promise<{ success: boolean; access_token?: string; expires_at?: number; error?: string }> {
  try {
    // Verify refresh token
    if (!env.JWT_SECRET) {
      return { success: false, error: 'Server configuration error' };
    }
    
    const isValid = await jwt.verify(refreshToken, env.JWT_SECRET, JWT_ALGORITHM);
    if (!isValid) {
      return { success: false, error: 'Invalid refresh token' };
    }
    
    const decoded = jwt.decode(refreshToken) as { payload?: { sub: string; type: string } };
    if (!decoded.payload || decoded.payload.type !== 'refresh') {
      return { success: false, error: 'Invalid refresh token' };
    }
    
    const deviceId = decoded.payload.sub;
    
    // Verify device exists
    const deviceData = await env.DEVICE_SESSIONS.get(`device:${deviceId}`);
    if (!deviceData) {
      return { success: false, error: 'Device not found' };
    }
    
    const device = JSON.parse(deviceData);
    
    // Generate new access token
    const accessToken = await jwt.sign(
      {
        sub: deviceId,
        tier: device.tier,
        iat: Math.floor(Date.now() / 1000),
        exp: Math.floor(Date.now() / 1000) + TOKEN_EXPIRY_SECONDS,
      },
      env.JWT_SECRET,
      JWT_ALGORITHM
    );
    
    return {
      success: true,
      access_token: accessToken,
      expires_at: Math.floor(Date.now() / 1000) + TOKEN_EXPIRY_SECONDS,
    };
  } catch (err) {
    console.error('Token refresh error:', err);
    return { success: false, error: 'Token refresh failed' };
  }
}

/**
 * Revoke device tokens
 */
export async function revokeDeviceTokens(
  deviceId: string,
  env: Env
): Promise<{ success: boolean; error?: string }> {
  try {
    // Delete device session
    await env.DEVICE_SESSIONS.delete(`device:${deviceId}`);
    
    // Also delete any pending rate limit data
    await env.RATE_LIMIT_KV.delete(`rate_limit:${deviceId}`);
    
    return { success: true };
  } catch (err) {
    console.error('Token revocation error:', err);
    return { success: false, error: 'Token revocation failed' };
  }
}

/**
 * Generate device ID from fingerprint
 */
async function generateDeviceId(fingerprint: string): Promise<string> {
  // Hash fingerprint with SHA-256
  const encoder = new TextEncoder();
  const data = encoder.encode(fingerprint);
  const hashBuffer = await crypto.subtle.digest('SHA-256', data);
  const hashArray = Array.from(new Uint8Array(hashBuffer));
  const hashHex = hashArray.map(b => b.toString(16).padStart(2, '0')).join('');
  
  // Return first 32 chars as device ID
  return `dev_${hashHex.slice(0, 32)}`;
}

/**
 * Generate access and refresh tokens
 */
async function generateTokens(
  deviceId: string,
  env: Env,
  tier: 'free' | 'paid' | 'enterprise' = 'free'
): Promise<{ access_token: string; refresh_token: string; expires_at: number }> {
  if (!env.JWT_SECRET) {
    throw new Error('JWT_SECRET not configured');
  }
  
  const now = Math.floor(Date.now() / 1000);
  
  // Access token
  const accessToken = await jwt.sign(
    {
      sub: deviceId,
      tier,
      iat: now,
      exp: now + TOKEN_EXPIRY_SECONDS,
    },
    env.JWT_SECRET,
    JWT_ALGORITHM
  );
  
  // Refresh token
  const refreshToken = await jwt.sign(
    {
      sub: deviceId,
      type: 'refresh',
      iat: now,
      exp: now + REFRESH_TOKEN_EXPIRY_SECONDS,
    },
    env.JWT_SECRET,
    JWT_ALGORITHM
  );
  
  return {
    access_token: accessToken,
    refresh_token: refreshToken,
    expires_at: now + TOKEN_EXPIRY_SECONDS,
  };
}

/**
 * Validate pairing token for secondary device
 */
async function validatePairingToken(
  pairingToken: string,
  env: Env
): Promise<{ deviceId: string; tier: 'free' | 'paid' | 'enterprise' } | null> {
  try {
    // Pairing tokens are short-lived (5 minutes)
    const pairingData = await env.DEVICE_SESSIONS.get(`pairing:${pairingToken}`);
    if (!pairingData) {
      return null;
    }
    
    const pairing = JSON.parse(pairingData);
    
    // Check expiration
    if (pairing.expires_at < Date.now()) {
      await env.DEVICE_SESSIONS.delete(`pairing:${pairingToken}`);
      return null;
    }
    
    // Get primary device info
    const primaryDeviceData = await env.DEVICE_SESSIONS.get(`device:${pairing.primary_device}`);
    if (!primaryDeviceData) {
      return null;
    }
    
    const primaryDevice = JSON.parse(primaryDeviceData);
    
    // Delete used pairing token
    await env.DEVICE_SESSIONS.delete(`pairing:${pairingToken}`);
    
    return {
      deviceId: pairing.primary_device,
      tier: primaryDevice.tier,
    };
  } catch {
    return null;
  }
}

/**
 * Create pairing token for secondary device
 */
export async function createPairingToken(
  deviceId: string,
  env: Env
): Promise<{ success: boolean; pairing_token?: string; expires_at?: number; error?: string }> {
  try {
    // Generate random pairing token
    const tokenBytes = new Uint8Array(32);
    crypto.getRandomValues(tokenBytes);
    const pairingToken = Array.from(tokenBytes)
      .map(b => b.toString(16).padStart(2, '0'))
      .join('');
    
    const expiresAt = Date.now() + 5 * 60 * 1000; // 5 minutes
    
    // Store pairing token
    await env.DEVICE_SESSIONS.put(
      `pairing:${pairingToken}`,
      JSON.stringify({
        primary_device: deviceId,
        created_at: Date.now(),
        expires_at: expiresAt,
      }),
      { expirationTtl: 300 } // 5 minutes
    );
    
    return {
      success: true,
      pairing_token: pairingToken,
      expires_at: expiresAt,
    };
  } catch (err) {
    console.error('Pairing token creation error:', err);
    return { success: false, error: 'Failed to create pairing token' };
  }
}

/**
 * Validate public key format
 */
function isValidPublicKey(publicKey: string): boolean {
  // Ed25519 public keys are 32 bytes, base64 encoded = 44 chars
  // Allow some flexibility for different encodings
  try {
    // Check if it's valid base64
    const decoded = atob(publicKey);
    return decoded.length === 32;
  } catch {
    return false;
  }
}

/**
 * Get device tier
 */
export async function getDeviceTier(
  deviceId: string,
  env: Env
): Promise<'free' | 'paid' | 'enterprise'> {
  const deviceData = await env.DEVICE_SESSIONS.get(`device:${deviceId}`);
  if (!deviceData) {
    return 'free';
  }
  
  const device = JSON.parse(deviceData);
  return device.tier || 'free';
}
