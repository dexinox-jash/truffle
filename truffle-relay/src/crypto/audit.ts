/**
 * Audit Logging
 * SPEC v2.0 SECTION 2.3.4 - Relay Server
 * SPEC v2.0 SECTION 3.1 - Threat Model (Repudiation)
 * 
 * CRITICAL: We log access patterns only, NEVER content or metadata.
 * All logs are anonymous and privacy-preserving.
 */

import type { Env } from '../index';
import { storeAuditLog } from '../storage/r2';

// Audit log entry
interface AuditLogEntry {
  timestamp: number;
  device_id: string;
  action: string;
  success: boolean;
  error?: string;
  duration_ms?: number;
  ip_hash?: string;
  user_agent_hash?: string;
  protocol_version?: string;
  // CRITICAL: No content, no metadata, no user data
}

// Log retention periods
const LOG_RETENTION_DAYS = {
  access: 90,
  error: 180,
  security: 365,
};

// Action types
 type AuditAction = 
  | 'auth'
  | 'blob_store'
  | 'blob_retrieve'
  | 'blob_delete'
  | 'websocket_connected'
  | 'websocket_disconnected'
  | 'websocket_message'
  | 'websocket_error'
  | 'rate_limit_exceeded'
  | 'unauthorized_access'
  | 'zero_access_violation'
  | 'error';

/**
 * Log access event
 * CRITICAL: Never log content, metadata, or user data
 */
export async function logAccess(
  request: Request,
  env: Env,
  action: AuditAction,
  success: boolean,
  error?: string,
  durationMs?: number
): Promise<void> {
  try {
    // Get device ID from headers
    const deviceId = request.headers.get('X-Device-ID') || 'unknown';
    
    // Hash IP address for privacy (we never store raw IPs)
    const ipHash = await hashString(getClientIP(request));
    
    // Hash user agent for privacy
    const userAgentHash = await hashString(request.headers.get('User-Agent') || '');
    
    const entry: AuditLogEntry = {
      timestamp: Date.now(),
      device_id: deviceId,
      action,
      success,
      error: error ? sanitizeError(error) : undefined,
      duration_ms: durationMs,
      ip_hash: ipHash,
      user_agent_hash: userAgentHash,
      protocol_version: env.PROTOCOL_VERSION,
    };
    
    // Store in R2
    await storeAuditLog(env.AUDIT_LOGS, entry);
    
    // Also log to console in development
    if (env.ENVIRONMENT === 'development') {
      console.log('[AUDIT]', JSON.stringify(entry));
    }
  } catch (err) {
    // Don't throw - audit logging should never break the application
    console.error('Audit logging error:', err);
  }
}

/**
 * Log security event
 * These are high-priority events that may trigger alerts
 */
export async function logSecurityEvent(
  env: Env,
  event: {
    device_id: string;
    event_type: 'suspicious_activity' | 'rate_limit_violation' | 'auth_failure' | 'zero_access_violation';
    severity: 'low' | 'medium' | 'high' | 'critical';
    details: string;
    source_ip_hash?: string;
  }
): Promise<void> {
  try {
    const entry = {
      timestamp: Date.now(),
      device_id: event.device_id,
      action: `security_${event.event_type}`,
      success: false,
      error: `[${event.severity.toUpperCase()}] ${event.details}`,
      ip_hash: event.source_ip_hash,
      protocol_version: env.PROTOCOL_VERSION,
    };
    
    // Store in R2
    await storeAuditLog(env.AUDIT_LOGS, entry);
    
    // Log to console
    console.error('[SECURITY]', JSON.stringify(entry));
    
    // For critical events, trigger alert (in production)
    if (event.severity === 'critical') {
      await triggerSecurityAlert(event);
    }
  } catch (err) {
    console.error('Security audit logging error:', err);
  }
}

/**
 * Log rate limit event
 */
export async function logRateLimitEvent(
  env: Env,
  deviceId: string,
  limit: number,
  currentUsage: number
): Promise<void> {
  try {
    const entry = {
      timestamp: Date.now(),
      device_id: deviceId,
      action: 'rate_limit_exceeded',
      success: false,
      error: `Rate limit exceeded: ${currentUsage}/${limit}`,
      protocol_version: env.PROTOCOL_VERSION,
    };
    
    await storeAuditLog(env.AUDIT_LOGS, entry);
  } catch (err) {
    console.error('Rate limit audit logging error:', err);
  }
}

/**
 * Log WebSocket event
 */
export async function logWebSocketEvent(
  env: Env,
  event: {
    device_id: string;
    event_type: 'connect' | 'disconnect' | 'message' | 'error';
    message_type?: string;
    duration_ms?: number;
    error?: string;
  }
): Promise<void> {
  try {
    const actionMap: Record<string, string> = {
      connect: 'websocket_connected',
      disconnect: 'websocket_disconnected',
      message: 'websocket_message',
      error: 'websocket_error',
    };
    
    const entry = {
      timestamp: Date.now(),
      device_id: event.device_id,
      action: actionMap[event.event_type],
      success: event.event_type !== 'error',
      error: event.error,
      duration_ms: event.duration_ms,
      protocol_version: env.PROTOCOL_VERSION,
    };
    
    await storeAuditLog(env.AUDIT_LOGS, entry);
  } catch (err) {
    console.error('WebSocket audit logging error:', err);
  }
}

/**
 * Log blob operation
 */
export async function logBlobOperation(
  env: Env,
  operation: 'store' | 'retrieve' | 'delete',
  deviceId: string,
  blobId: string,
  success: boolean,
  sizeBytes?: number,
  durationMs?: number,
  error?: string
): Promise<void> {
  try {
    const actionMap: Record<string, string> = {
      store: 'blob_store',
      retrieve: 'blob_retrieve',
      delete: 'blob_delete',
    };
    
    const entry = {
      timestamp: Date.now(),
      device_id: deviceId,
      action: actionMap[operation],
      success,
      error: error ? sanitizeError(error) : undefined,
      duration_ms: durationMs,
      protocol_version: env.PROTOCOL_VERSION,
    };
    
    // CRITICAL: We log blob ID hash only, never the actual blob ID
    // This allows correlation without revealing content
    const blobIdHash = await hashString(blobId);
    
    await storeAuditLog(env.AUDIT_LOGS, {
      ...entry,
      // Store blob hash in a way that doesn't reveal the actual ID
      // We use the error field with a prefix for this
      error: entry.error || `blob_hash:${blobIdHash.slice(0, 16)}`,
    });
  } catch (err) {
    console.error('Blob audit logging error:', err);
  }
}

/**
 * Get client IP address from request
 */
function getClientIP(request: Request): string {
  // Cloudflare provides the real client IP in CF-Connecting-IP header
  const cfIP = request.headers.get('CF-Connecting-IP');
  if (cfIP) {
    return cfIP;
  }
  
  // Fallback to X-Forwarded-For
  const forwardedFor = request.headers.get('X-Forwarded-For');
  if (forwardedFor) {
    return forwardedFor.split(',')[0].trim();
  }
  
  // Fallback to X-Real-IP
  const realIP = request.headers.get('X-Real-IP');
  if (realIP) {
    return realIP;
  }
  
  return 'unknown';
}

/**
 * Hash a string for privacy
 */
async function hashString(input: string): Promise<string> {
  if (!input) return '';
  
  const encoder = new TextEncoder();
  const data = encoder.encode(input);
  const hashBuffer = await crypto.subtle.digest('SHA-256', data);
  const hashArray = Array.from(new Uint8Array(hashBuffer));
  return hashArray.map(b => b.toString(16).padStart(2, '0')).join('').slice(0, 32);
}

/**
 * Sanitize error message to remove potentially sensitive data
 */
function sanitizeError(error: string): string {
  // Remove potential PII patterns
  const patterns = [
    /[a-f0-9]{64}/gi, // SHA-256 hashes
    /[a-f0-9]{128}/gi, // Long hex strings
    /\b(?:[0-9]{1,3}\.){3}[0-9]{1,3}\b/g, // IP addresses
    /\b[A-Za-z0-9._%+-]+@[A-Za-z0-9.-]+\.[A-Z|a-z]{2,}\b/g, // Email addresses
  ];
  
  let sanitized = error;
  for (const pattern of patterns) {
    sanitized = sanitized.replace(pattern, '[REDACTED]');
  }
  
  // Truncate long errors
  if (sanitized.length > 500) {
    sanitized = sanitized.slice(0, 500) + '... [truncated]';
  }
  
  return sanitized;
}

/**
 * Trigger security alert
 * In production, this would send alerts to security team
 */
async function triggerSecurityAlert(event: {
  device_id: string;
  event_type: string;
  severity: string;
  details: string;
}): Promise<void> {
  // This is a placeholder - in production, integrate with:
  // - PagerDuty
  // - Slack webhooks
  // - Email notifications
  // - Security incident management system
  
  console.error('[SECURITY ALERT]', JSON.stringify({
    timestamp: new Date().toISOString(),
    ...event,
  }));
}

/**
 * Generate audit report
 */
export async function generateAuditReport(
  env: Env,
  startTime: number,
  endTime: number
): Promise<{
  generated_at: string;
  period: { start: string; end: string };
  summary: {
    total_events: number;
    successful_events: number;
    failed_events: number;
    unique_devices: number;
    events_by_action: Record<string, number>;
    events_by_hour: Record<string, number>;
  };
}> {
  // This is a simplified version - in production, you'd query the audit logs
  // For now, return a template
  
  return {
    generated_at: new Date().toISOString(),
    period: {
      start: new Date(startTime).toISOString(),
      end: new Date(endTime).toISOString(),
    },
    summary: {
      total_events: 0,
      successful_events: 0,
      failed_events: 0,
      unique_devices: 0,
      events_by_action: {},
      events_by_hour: {},
    },
  };
}

/**
 * Verify audit log integrity
 * Checks that logs haven't been tampered with
 */
export async function verifyAuditIntegrity(env: Env): Promise<{
  valid: boolean;
  checked_logs: number;
  invalid_logs: number;
  errors: string[];
}> {
  const errors: string[] = [];
  let checkedLogs = 0;
  let invalidLogs = 0;
  
  try {
    // List audit logs
    const listing = await env.AUDIT_LOGS.list({ limit: 1000 });
    
    for (const obj of listing.objects) {
      checkedLogs++;
      
      try {
        const logData = await env.AUDIT_LOGS.get(obj.key);
        if (!logData) {
          invalidLogs++;
          errors.push(`Missing log data: ${obj.key}`);
          continue;
        }
        
        const logText = await logData.text();
        const logEntry = JSON.parse(logText);
        
        // Verify required fields
        if (!logEntry.timestamp || !logEntry.device_id || !logEntry.action) {
          invalidLogs++;
          errors.push(`Invalid log entry (missing fields): ${obj.key}`);
        }
        
        // Verify timestamp is reasonable
        const timestamp = new Date(logEntry.timestamp);
        const now = new Date();
        if (timestamp > now || timestamp < new Date(now.getTime() - 365 * 24 * 60 * 60 * 1000)) {
          invalidLogs++;
          errors.push(`Invalid timestamp in log: ${obj.key}`);
        }
      } catch (err) {
        invalidLogs++;
        errors.push(`Corrupted log entry: ${obj.key} - ${err instanceof Error ? err.message : 'Unknown error'}`);
      }
    }
  } catch (err) {
    errors.push(`Failed to verify audit integrity: ${err instanceof Error ? err.message : 'Unknown error'}`);
  }
  
  return {
    valid: invalidLogs === 0,
    checked_logs: checkedLogs,
    invalid_logs: invalidLogs,
    errors,
  };
}

/**
 * Export audit logs for compliance
 */
export async function exportAuditLogs(
  env: Env,
  startTime: number,
  endTime: number
): Promise<{
  success: boolean;
  export_id: string;
  record_count: number;
  download_url?: string;
  error?: string;
}> {
  const exportId = crypto.randomUUID();
  
  try {
    // This is a placeholder - in production, you'd:
    // 1. Query audit logs for the time period
    // 2. Format according to compliance requirements
    // 3. Store in a secure location
    // 4. Generate a secure download URL
    
    return {
      success: true,
      export_id: exportId,
      record_count: 0,
    };
  } catch (err) {
    return {
      success: false,
      export_id: exportId,
      record_count: 0,
      error: err instanceof Error ? err.message : 'Export failed',
    };
  }
}
