/**
 * Lifecycle Management - 30-Day Auto-Delete
 * SPEC v2.0 SECTION 2.3.4 - Relay Server
 * 
 * Data Retention: 30 days (auto-delete via R2 lifecycle rules)
 * Long-term sync requires device online within 30 days.
 * 
 * CRITICAL: All data is encrypted - we cannot read it anyway.
 */

import type { Env } from '../index';
import { listBlobsPaginated, deleteFromR2, getStorageStats } from './r2';

// Default retention period (30 days)
const DEFAULT_RETENTION_DAYS = 30;

// Lifecycle rule configuration
interface LifecycleRule {
  id: string;
  enabled: boolean;
  filter?: {
    prefix?: string;
  };
  expiration?: {
    days: number;
  };
  abortIncompleteMultipartUpload?: {
    daysAfterInitiation: number;
  };
}

// Lifecycle configuration
interface LifecycleConfiguration {
  rules: LifecycleRule[];
}

// Cleanup result
interface CleanupResult {
  success: boolean;
  deleted_count: number;
  failed_count: number;
  bytes_freed: number;
  errors: string[];
}

// Expiration check result
interface ExpirationCheck {
  key: string;
  uploaded_at: Date;
  expires_at: Date;
  is_expired: boolean;
  days_until_expiration: number;
}

/**
 * Get default lifecycle configuration for R2 bucket
 * This should be applied via Terraform or R2 API
 */
export function getDefaultLifecycleConfiguration(
  retentionDays: number = DEFAULT_RETENTION_DAYS
): LifecycleConfiguration {
  return {
    rules: [
      {
        id: 'sync-blobs-expiration',
        enabled: true,
        filter: {
          prefix: '', // Apply to all objects
        },
        expiration: {
          days: retentionDays,
        },
        abortIncompleteMultipartUpload: {
          daysAfterInitiation: 1,
        },
      },
      {
        id: 'audit-logs-expiration',
        enabled: true,
        filter: {
          prefix: '', // Apply to all objects
        },
        expiration: {
          days: retentionDays * 3, // Keep audit logs 3x longer (90 days)
        },
      },
    ],
  };
}

/**
 * Check if an object has expired based on retention policy
 */
export function checkObjectExpiration(
  uploadedAt: Date,
  retentionDays: number = DEFAULT_RETENTION_DAYS
): ExpirationCheck {
  const now = new Date();
  const expiresAt = new Date(uploadedAt);
  expiresAt.setDate(expiresAt.getDate() + retentionDays);
  
  const isExpired = now > expiresAt;
  const daysUntilExpiration = Math.ceil(
    (expiresAt.getTime() - now.getTime()) / (1000 * 60 * 60 * 24)
  );
  
  return {
    key: '', // Will be set by caller
    uploaded_at: uploadedAt,
    expires_at: expiresAt,
    is_expired: isExpired,
    days_until_expiration: Math.max(0, daysUntilExpiration),
  };
}

/**
 * Manually clean up expired objects
 * This is a fallback in case lifecycle rules fail
 */
export async function cleanupExpiredObjects(
  env: Env,
  retentionDays: number = DEFAULT_RETENTION_DAYS,
  dryRun: boolean = false
): Promise<CleanupResult> {
  const result: CleanupResult = {
    success: true,
    deleted_count: 0,
    failed_count: 0,
    bytes_freed: 0,
    errors: [],
  };
  
  try {
    // Clean up sync blobs
    const syncResult = await cleanupBucket(
      env.SYNC_BLOBS,
      retentionDays,
      dryRun
    );
    
    result.deleted_count += syncResult.deleted_count;
    result.failed_count += syncResult.failed_count;
    result.bytes_freed += syncResult.bytes_freed;
    result.errors.push(...syncResult.errors);
    
    // Clean up audit logs (3x retention)
    const auditResult = await cleanupBucket(
      env.AUDIT_LOGS,
      retentionDays * 3,
      dryRun
    );
    
    result.deleted_count += auditResult.deleted_count;
    result.failed_count += auditResult.failed_count;
    result.bytes_freed += auditResult.bytes_freed;
    result.errors.push(...auditResult.errors);
    
    return result;
  } catch (err) {
    result.success = false;
    result.errors.push(err instanceof Error ? err.message : 'Unknown error');
    return result;
  }
}

/**
 * Clean up expired objects from a single bucket
 */
async function cleanupBucket(
  bucket: R2Bucket,
  retentionDays: number,
  dryRun: boolean
): Promise<CleanupResult> {
  const result: CleanupResult = {
    success: true,
    deleted_count: 0,
    failed_count: 0,
    bytes_freed: 0,
    errors: [],
  };
  
  const now = Date.now();
  const retentionMs = retentionDays * 24 * 60 * 60 * 1000;
  const cutoffDate = new Date(now - retentionMs);
  
  let cursor: string | undefined;
  
  do {
    const listing = await listBlobsPaginated(bucket, undefined, 1000, cursor);
    
    for (const obj of listing.objects) {
      // Check if object has expired
      if (obj.uploaded < cutoffDate) {
        if (!dryRun) {
          const deleted = await deleteFromR2(bucket, obj.key);
          
          if (deleted) {
            result.deleted_count++;
            result.bytes_freed += obj.size;
          } else {
            result.failed_count++;
            result.errors.push(`Failed to delete ${obj.key}`);
          }
        } else {
          // Dry run - just count
          result.deleted_count++;
          result.bytes_freed += obj.size;
        }
      }
    }
    
    cursor = listing.cursor;
  } while (cursor);
  
  return result;
}

/**
 * Get expiration statistics for a bucket
 */
export async function getExpirationStats(
  bucket: R2Bucket,
  retentionDays: number = DEFAULT_RETENTION_DAYS
): Promise<{
  total_objects: number;
  expired_objects: number;
  expiring_soon: number; // Within 7 days
  total_bytes: number;
  expired_bytes: number;
}> {
  const now = Date.now();
  const retentionMs = retentionDays * 24 * 60 * 60 * 1000;
  const cutoffDate = new Date(now - retentionMs);
  const soonDate = new Date(now + 7 * 24 * 60 * 60 * 1000); // 7 days from now
  
  let totalObjects = 0;
  let expiredObjects = 0;
  let expiringSoon = 0;
  let totalBytes = 0;
  let expiredBytes = 0;
  
  let cursor: string | undefined;
  
  do {
    const listing = await listBlobsPaginated(bucket, undefined, 1000, cursor);
    
    for (const obj of listing.objects) {
      totalObjects++;
      totalBytes += obj.size;
      
      if (obj.uploaded < cutoffDate) {
        expiredObjects++;
        expiredBytes += obj.size;
      } else if (obj.uploaded < soonDate) {
        expiringSoon++;
      }
    }
    
    cursor = listing.cursor;
  } while (cursor);
  
  return {
    total_objects: totalObjects,
    expired_objects: expiredObjects,
    expiring_soon: expiringSoon,
    total_bytes: totalBytes,
    expired_bytes: expiredBytes,
  };
}

/**
 * Schedule object deletion at a specific time
 * Uses KV to track scheduled deletions
 */
export async function scheduleDeletion(
  env: Env,
  key: string,
  deleteAt: Date
): Promise<boolean> {
  try {
    const scheduleKey = `deletion_schedule:${key}`;
    await env.WEBSOCKET_STATE.put(
      scheduleKey,
      JSON.stringify({
        key,
        delete_at: deleteAt.toISOString(),
        scheduled_at: new Date().toISOString(),
      }),
      { expirationTtl: Math.ceil((deleteAt.getTime() - Date.now()) / 1000) + 3600 }
    );
    
    return true;
  } catch (err) {
    console.error('Schedule deletion error:', err);
    return false;
  }
}

/**
 * Process scheduled deletions
 */
export async function processScheduledDeletions(env: Env): Promise<{
  processed: number;
  deleted: number;
  failed: number;
}> {
  let processed = 0;
  let deleted = 0;
  let failed = 0;
  
  // List all scheduled deletions
  const scheduled = await env.WEBSOCKET_STATE.list({ prefix: 'deletion_schedule:' });
  
  for (const key of scheduled.keys) {
    processed++;
    
    const scheduleData = await env.WEBSOCKET_STATE.get(key.name);
    if (!scheduleData) continue;
    
    const schedule = JSON.parse(scheduleData);
    const deleteAt = new Date(schedule.delete_at);
    
    // Check if it's time to delete
    if (deleteAt <= new Date()) {
      // Extract device ID and blob ID from key
      const keyParts = schedule.key.split('/');
      if (keyParts.length >= 2) {
        const deleted_success = await deleteFromR2(env.SYNC_BLOBS, schedule.key);
        
        if (deleted_success) {
          deleted++;
        } else {
          failed++;
        }
      }
      
      // Remove from schedule
      await env.WEBSOCKET_STATE.delete(key.name);
    }
  }
  
  return { processed, deleted, failed };
}

/**
 * Extend object expiration (for paid users)
 */
export async function extendExpiration(
  env: Env,
  key: string,
  additionalDays: number
): Promise<boolean> {
  try {
    // Get current metadata
    const object = await env.SYNC_BLOBS.head(key);
    if (!object) {
      return false;
    }
    
    // Calculate new expiration
    const currentExpiresAt = object.customMetadata?.['X-Expires-At'];
    const currentDate = currentExpiresAt ? new Date(parseInt(currentExpiresAt)) : object.uploaded;
    const newExpiresAt = new Date(currentDate);
    newExpiresAt.setDate(newExpiresAt.getDate() + additionalDays);
    
    // Update metadata (R2 doesn't support metadata-only updates, so we need to copy)
    const data = await env.SYNC_BLOBS.get(key);
    if (!data) {
      return false;
    }
    
    const arrayBuffer = await data.arrayBuffer();
    
    await env.SYNC_BLOBS.put(key, arrayBuffer, {
      customMetadata: {
        ...object.customMetadata,
        'X-Expires-At': newExpiresAt.getTime().toString(),
        'X-Extended-At': Date.now().toString(),
      },
      httpMetadata: object.httpMetadata,
    });
    
    return true;
  } catch (err) {
    console.error('Extend expiration error:', err);
    return false;
  }
}

/**
 * Get lifecycle policy for Terraform
 * Returns the lifecycle rules as JSON for Terraform configuration
 */
export function getTerraformLifecyclePolicy(
  retentionDays: number = DEFAULT_RETENTION_DAYS
): string {
  const config = getDefaultLifecycleConfiguration(retentionDays);
  return JSON.stringify(config, null, 2);
}

/**
 * Validate lifecycle configuration
 */
export function validateLifecycleConfiguration(
  config: LifecycleConfiguration
): { valid: boolean; errors: string[] } {
  const errors: string[] = [];
  
  if (!config.rules || config.rules.length === 0) {
    errors.push('At least one lifecycle rule is required');
  }
  
  for (const rule of config.rules) {
    if (!rule.id) {
      errors.push('Lifecycle rule must have an ID');
    }
    
    if (rule.expiration && rule.expiration.days < 1) {
      errors.push(`Rule ${rule.id}: Expiration days must be at least 1`);
    }
    
    if (rule.abortIncompleteMultipartUpload && rule.abortIncompleteMultipartUpload.daysAfterInitiation < 1) {
      errors.push(`Rule ${rule.id}: Abort incomplete multipart upload days must be at least 1`);
    }
  }
  
  return {
    valid: errors.length === 0,
    errors,
  };
}

/**
 * Generate lifecycle report
 */
export async function generateLifecycleReport(env: Env): Promise<{
  generated_at: string;
  retention_days: number;
  sync_blobs: {
    stats: Awaited<ReturnType<typeof getStorageStats>>;
    expiration: Awaited<ReturnType<typeof getExpirationStats>>;
  };
  audit_logs: {
    stats: Awaited<ReturnType<typeof getStorageStats>>;
    expiration: Awaited<ReturnType<typeof getExpirationStats>>;
  };
}> {
  const retentionDays = parseInt(env.DATA_RETENTION_DAYS || DEFAULT_RETENTION_DAYS.toString());
  
  const [syncStats, syncExpiration, auditStats, auditExpiration] = await Promise.all([
    getStorageStats(env.SYNC_BLOBS),
    getExpirationStats(env.SYNC_BLOBS, retentionDays),
    getStorageStats(env.AUDIT_LOGS),
    getExpirationStats(env.AUDIT_LOGS, retentionDays * 3),
  ]);
  
  return {
    generated_at: new Date().toISOString(),
    retention_days: retentionDays,
    sync_blobs: {
      stats: syncStats,
      expiration: syncExpiration,
    },
    audit_logs: {
      stats: auditStats,
      expiration: auditExpiration,
    },
  };
}
