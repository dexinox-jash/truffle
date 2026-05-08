/**
 * R2 Storage Integration
 * SPEC v2.0 SECTION 2.3.4 - Relay Server
 * SPEC v2.0 SECTION 6.1 - Cloud Architecture
 * 
 * Cloudflare R2: S3-compatible, zero egress fees
 * Used for: Encrypted blob storage, audit logs
 * 
 * CRITICAL: Zero-access infrastructure. We only store encrypted data.
 */

// R2 operation result
interface R2Result<T> {
  success: boolean;
  data?: T;
  error?: string;
}

// R2 object metadata
interface R2ObjectMetadata {
  key: string;
  size: number;
  etag: string;
  uploaded: Date;
  httpEtag: string;
  version?: string;
  customMetadata?: Record<string, string>;
}

// R2 list result
interface R2ListResult {
  objects: R2ObjectMetadata[];
  truncated: boolean;
  cursor?: string;
}

/**
 * Store object in R2 bucket
 */
export async function storeInR2(
  bucket: R2Bucket,
  key: string,
  data: ArrayBuffer | string,
  metadata?: Record<string, string>
): Promise<boolean> {
  try {
    const object = await bucket.put(key, data, {
      customMetadata: metadata,
      httpMetadata: {
        contentType: metadata?.['Content-Type'] || 'application/octet-stream',
      },
    });
    
    return object !== null;
  } catch (err) {
    console.error(`R2 store error for key ${key}:`, err);
    return false;
  }
}

/**
 * Retrieve object from R2 bucket
 */
export async function retrieveFromR2(
  bucket: R2Bucket,
  key: string
): Promise<ArrayBuffer | null> {
  try {
    const object = await bucket.get(key);
    
    if (!object) {
      return null;
    }
    
    return await object.arrayBuffer();
  } catch (err) {
    console.error(`R2 retrieve error for key ${key}:`, err);
    return null;
  }
}

/**
 * Retrieve object with metadata from R2 bucket
 */
export async function retrieveFromR2WithMetadata(
  bucket: R2Bucket,
  key: string
): Promise<{ data: ArrayBuffer; metadata: Record<string, string> } | null> {
  try {
    const object = await bucket.get(key);
    
    if (!object) {
      return null;
    }
    
    const data = await object.arrayBuffer();
    const metadata = object.customMetadata || {};
    
    return { data, metadata };
  } catch (err) {
    console.error(`R2 retrieve with metadata error for key ${key}:`, err);
    return null;
  }
}

/**
 * Delete object from R2 bucket
 */
export async function deleteFromR2(
  bucket: R2Bucket,
  key: string
): Promise<boolean> {
  try {
    await bucket.delete(key);
    return true;
  } catch (err) {
    console.error(`R2 delete error for key ${key}:`, err);
    return false;
  }
}

/**
 * Check if object exists in R2 bucket
 */
export async function existsInR2(
  bucket: R2Bucket,
  key: string
): Promise<boolean> {
  try {
    const object = await bucket.head(key);
    return object !== null;
  } catch {
    return false;
  }
}

/**
 * Get object metadata from R2 bucket
 */
export async function getMetadataFromR2(
  bucket: R2Bucket,
  key: string
): Promise<R2ObjectMetadata | null> {
  try {
    const object = await bucket.head(key);
    
    if (!object) {
      return null;
    }
    
    return {
      key: object.key,
      size: object.size,
      etag: object.etag,
      uploaded: object.uploaded,
      httpEtag: object.httpEtag,
      version: object.version,
      customMetadata: object.customMetadata,
    };
  } catch (err) {
    console.error(`R2 get metadata error for key ${key}:`, err);
    return null;
  }
}

/**
 * List objects in R2 bucket
 */
export async function listBlobsInR2(
  bucket: R2Bucket,
  prefix?: string,
  limit?: number,
  cursor?: string
): Promise<R2ObjectMetadata[]> {
  try {
    const options: R2ListOptions = {
      limit: limit || 1000,
    };
    
    if (prefix) {
      options.prefix = prefix;
    }
    
    if (cursor) {
      options.cursor = cursor;
    }
    
    const listing = await bucket.list(options);
    
    return listing.objects.map(obj => ({
      key: obj.key,
      size: obj.size,
      etag: obj.etag,
      uploaded: obj.uploaded,
      httpEtag: obj.httpEtag,
      version: obj.version,
      customMetadata: obj.customMetadata,
    }));
  } catch (err) {
    console.error('R2 list error:', err);
    return [];
  }
}

/**
 * List objects with pagination
 */
export async function listBlobsPaginated(
  bucket: R2Bucket,
  prefix?: string,
  limit?: number,
  cursor?: string
): Promise<R2ListResult> {
  try {
    const options: R2ListOptions = {
      limit: limit || 1000,
    };
    
    if (prefix) {
      options.prefix = prefix;
    }
    
    if (cursor) {
      options.cursor = cursor;
    }
    
    const listing = await bucket.list(options);
    
    return {
      objects: listing.objects.map(obj => ({
        key: obj.key,
        size: obj.size,
        etag: obj.etag,
        uploaded: obj.uploaded,
        httpEtag: obj.httpEtag,
        version: obj.version,
        customMetadata: obj.customMetadata,
      })),
      truncated: listing.truncated,
      cursor: listing.cursor,
    };
  } catch (err) {
    console.error('R2 list paginated error:', err);
    return {
      objects: [],
      truncated: false,
    };
  }
}

/**
 * Copy object within R2 bucket
 */
export async function copyInR2(
  bucket: R2Bucket,
  sourceKey: string,
  destinationKey: string
): Promise<boolean> {
  try {
    const sourceObject = await bucket.get(sourceKey);
    
    if (!sourceObject) {
      return false;
    }
    
    const data = await sourceObject.arrayBuffer();
    
    await bucket.put(destinationKey, data, {
      customMetadata: sourceObject.customMetadata,
      httpMetadata: sourceObject.httpMetadata,
    });
    
    return true;
  } catch (err) {
    console.error(`R2 copy error from ${sourceKey} to ${destinationKey}:`, err);
    return false;
  }
}

/**
 * Move object within R2 bucket
 */
export async function moveInR2(
  bucket: R2Bucket,
  sourceKey: string,
  destinationKey: string
): Promise<boolean> {
  const copied = await copyInR2(bucket, sourceKey, destinationKey);
  
  if (!copied) {
    return false;
  }
  
  await deleteFromR2(bucket, sourceKey);
  return true;
}

/**
 * Get total storage size for a prefix
 */
export async function getStorageSize(
  bucket: R2Bucket,
  prefix?: string
): Promise<number> {
  let totalSize = 0;
  let cursor: string | undefined;
  
  do {
    const result = await listBlobsPaginated(bucket, prefix, 1000, cursor);
    
    for (const obj of result.objects) {
      totalSize += obj.size;
    }
    
    cursor = result.cursor;
  } while (cursor);
  
  return totalSize;
}

/**
 * Delete all objects with a prefix
 */
export async function deletePrefix(
  bucket: R2Bucket,
  prefix: string
): Promise<number> {
  let deletedCount = 0;
  let cursor: string | undefined;
  
  do {
    const result = await listBlobsPaginated(bucket, prefix, 1000, cursor);
    
    for (const obj of result.objects) {
      await deleteFromR2(bucket, obj.key);
      deletedCount++;
    }
    
    cursor = result.cursor;
  } while (cursor);
  
  return deletedCount;
}

/**
 * Generate presigned URL for object (not supported in R2, but documented)
 * Note: R2 doesn't support presigned URLs like S3. Use worker endpoints instead.
 */
export function generateBlobUrl(
  baseUrl: string,
  deviceId: string,
  blobId: string
): string {
  return `${baseUrl}/blob/${blobId}`;
}

/**
 * Store audit log entry
 */
export async function storeAuditLog(
  bucket: R2Bucket,
  entry: {
    timestamp: number;
    device_id: string;
    action: string;
    success: boolean;
    error?: string;
    duration_ms?: number;
    ip_hash?: string;
  }
): Promise<boolean> {
  const date = new Date(entry.timestamp);
  const datePrefix = `${date.getUTCFullYear()}/${String(date.getUTCMonth() + 1).padStart(2, '0')}/${String(date.getUTCDate()).padStart(2, '0')}`;
  const logKey = `${datePrefix}/${entry.device_id}_${entry.timestamp}_${crypto.randomUUID()}.json`;
  
  return storeInR2(bucket, logKey, JSON.stringify(entry), {
    'Content-Type': 'application/json',
    'X-Device-ID': entry.device_id,
    'X-Action': entry.action,
    'X-Timestamp': entry.timestamp.toString(),
  });
}

/**
 * Get storage statistics
 */
export async function getStorageStats(
  bucket: R2Bucket,
  prefix?: string
): Promise<{
  total_objects: number;
  total_bytes: number;
  oldest_object?: Date;
  newest_object?: Date;
}> {
  let totalObjects = 0;
  let totalBytes = 0;
  let oldestObject: Date | undefined;
  let newestObject: Date | undefined;
  let cursor: string | undefined;
  
  do {
    const result = await listBlobsPaginated(bucket, prefix, 1000, cursor);
    
    for (const obj of result.objects) {
      totalObjects++;
      totalBytes += obj.size;
      
      if (!oldestObject || obj.uploaded < oldestObject) {
        oldestObject = obj.uploaded;
      }
      
      if (!newestObject || obj.uploaded > newestObject) {
        newestObject = obj.uploaded;
      }
    }
    
    cursor = result.cursor;
  } while (cursor);
  
  return {
    total_objects: totalObjects,
    total_bytes: totalBytes,
    oldest_object: oldestObject,
    newest_object: newestObject,
  };
}
