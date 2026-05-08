/**
 * Blob Handler - Encrypted Blob Storage and Retrieval
 * SPEC v2.0 SECTION 2.3.4 - Relay Server
 * 
 * CRITICAL: Zero-access infrastructure. All blobs are encrypted client-side.
 * We CANNOT decrypt any content. We only store and forward encrypted data.
 */

import type { Env } from '../index';
import { storeInR2, retrieveFromR2, deleteFromR2, listBlobsInR2 } from '../storage/r2';

// Blob metadata interface
interface BlobMetadata {
  blob_id: string;
  device_id: string;
  size_bytes: number;
  content_hash: string; // SHA-256 of encrypted content
  uploaded_at: number;
  expires_at: number;
  protocol_version: string;
  // CRITICAL: No encryption keys, no plaintext metadata
}

// Store blob result
interface StoreResult {
  success: boolean;
  blob_id?: string;
  error?: string;
  metadata?: BlobMetadata;
}

// Retrieve blob result
interface RetrieveResult {
  success: boolean;
  data?: ArrayBuffer;
  error?: string;
  metadata?: BlobMetadata;
}

// Delete blob result
interface DeleteResult {
  success: boolean;
  error?: string;
}

// Maximum blob size (10MB default)
const DEFAULT_MAX_BLOB_SIZE = 10 * 1024 * 1024;

/**
 * Store encrypted blob in R2
 * CRITICAL: Blob must be encrypted client-side. We verify it's not plaintext.
 */
export async function storeBlob(
  request: Request,
  env: Env,
  deviceId: string
): Promise<StoreResult> {
  try {
    // Get max blob size from env
    const maxBlobSize = parseInt(env.MAX_BLOB_SIZE_MB || '10') * 1024 * 1024;
    
    // Get content length
    const contentLength = parseInt(request.headers.get('Content-Length') || '0');
    if (contentLength === 0) {
      return { success: false, error: 'Empty blob' };
    }
    if (contentLength > maxBlobSize) {
      return { 
        success: false, 
        error: `Blob too large. Max size: ${maxBlobSize / 1024 / 1024}MB` 
      };
    }
    
    // Read blob data
    const blobData = await request.arrayBuffer();
    
    // Verify blob is not plaintext (zero-access check)
    const isEncrypted = await verifyEncryptedBlob(blobData);
    if (!isEncrypted) {
      return { 
        success: false, 
        error: 'Blob appears to be unencrypted. All blobs must be client-side encrypted.' 
      };
    }
    
    // Generate blob ID (content-addressable)
    const contentHash = await computeSHA256(blobData);
    const blobId = `blob_${contentHash.slice(0, 32)}`;
    
    // Calculate expiration
    const retentionDays = parseInt(env.DATA_RETENTION_DAYS || '30');
    const uploadedAt = Date.now();
    const expiresAt = uploadedAt + (retentionDays * 24 * 60 * 60 * 1000);
    
    // Create metadata
    const metadata: BlobMetadata = {
      blob_id: blobId,
      device_id: deviceId,
      size_bytes: blobData.byteLength,
      content_hash: contentHash,
      uploaded_at: uploadedAt,
      expires_at: expiresAt,
      protocol_version: env.PROTOCOL_VERSION || '1',
    };
    
    // Store blob in R2
    const blobKey = `${deviceId}/${blobId}`;
    const stored = await storeInR2(
      env.SYNC_BLOBS,
      blobKey,
      blobData,
      {
        'Content-Type': 'application/octet-stream',
        'X-Device-ID': deviceId,
        'X-Blob-ID': blobId,
        'X-Content-Hash': contentHash,
        'X-Uploaded-At': uploadedAt.toString(),
        'X-Expires-At': expiresAt.toString(),
        'X-Protocol-Version': env.PROTOCOL_VERSION || '1',
        // CRITICAL: No encryption keys stored
      }
    );
    
    if (!stored) {
      return { success: false, error: 'Failed to store blob' };
    }
    
    // Store metadata separately for quick lookup
    const metadataKey = `${deviceId}/${blobId}.meta`;
    await storeInR2(
      env.SYNC_BLOBS,
      metadataKey,
      JSON.stringify(metadata),
      {
        'Content-Type': 'application/json',
      }
    );
    
    return {
      success: true,
      blob_id: blobId,
      metadata,
    };
  } catch (err) {
    console.error('Store blob error:', err);
    return { 
      success: false, 
      error: err instanceof Error ? err.message : 'Unknown error' 
    };
  }
}

/**
 * Retrieve encrypted blob from R2
 * CRITICAL: We return the encrypted blob as-is. We NEVER decrypt.
 */
export async function retrieveBlob(
  blobId: string,
  env: Env,
  deviceId: string
): Promise<RetrieveResult> {
  try {
    // Validate blob ID format
    if (!isValidBlobId(blobId)) {
      return { success: false, error: 'Invalid blob ID format' };
    }
    
    // Construct blob key
    const blobKey = `${deviceId}/${blobId}`;
    
    // Retrieve blob from R2
    const blobData = await retrieveFromR2(env.SYNC_BLOBS, blobKey);
    
    if (!blobData) {
      return { success: false, error: 'Blob not found' };
    }
    
    // Retrieve metadata
    const metadataKey = `${deviceId}/${blobId}.meta`;
    const metadataData = await retrieveFromR2(env.SYNC_BLOBS, metadataKey);
    let metadata: BlobMetadata | undefined;
    
    if (metadataData) {
      const metadataText = new TextDecoder().decode(metadataData);
      metadata = JSON.parse(metadataText);
    }
    
    // Verify content hash if metadata exists
    if (metadata) {
      const contentHash = await computeSHA256(blobData);
      if (contentHash !== metadata.content_hash) {
        return { success: false, error: 'Blob integrity check failed' };
      }
    }
    
    return {
      success: true,
      data: blobData,
      metadata,
    };
  } catch (err) {
    console.error('Retrieve blob error:', err);
    return { 
      success: false, 
      error: err instanceof Error ? err.message : 'Unknown error' 
    };
  }
}

/**
 * Delete blob from R2
 */
export async function deleteBlob(
  blobId: string,
  env: Env,
  deviceId: string
): Promise<DeleteResult> {
  try {
    // Validate blob ID format
    if (!isValidBlobId(blobId)) {
      return { success: false, error: 'Invalid blob ID format' };
    }
    
    // Construct keys
    const blobKey = `${deviceId}/${blobId}`;
    const metadataKey = `${deviceId}/${blobId}.meta`;
    
    // Delete blob and metadata
    await deleteFromR2(env.SYNC_BLOBS, blobKey);
    await deleteFromR2(env.SYNC_BLOBS, metadataKey);
    
    return { success: true };
  } catch (err) {
    console.error('Delete blob error:', err);
    return { 
      success: false, 
      error: err instanceof Error ? err.message : 'Unknown error' 
    };
  }
}

/**
 * List blobs for a device
 */
export async function listBlobs(
  env: Env,
  deviceId: string,
  prefix?: string,
  limit?: number
): Promise<{ success: boolean; blobs?: BlobMetadata[]; error?: string }> {
  try {
    const listPrefix = prefix ? `${deviceId}/${prefix}` : `${deviceId}/`;
    const objects = await listBlobsInR2(env.SYNC_BLOBS, listPrefix, limit);
    
    const blobs: BlobMetadata[] = [];
    
    for (const obj of objects) {
      // Only include metadata files
      if (obj.key.endsWith('.meta')) {
        const metadataData = await retrieveFromR2(env.SYNC_BLOBS, obj.key);
        if (metadataData) {
          const metadataText = new TextDecoder().decode(metadataData);
          blobs.push(JSON.parse(metadataText));
        }
      }
    }
    
    return { success: true, blobs };
  } catch (err) {
    console.error('List blobs error:', err);
    return { 
      success: false, 
      error: err instanceof Error ? err.message : 'Unknown error' 
    };
  }
}

/**
 * Verify that a blob is encrypted (not plaintext)
 * CRITICAL: Zero-access verification - we must not store plaintext
 */
async function verifyEncryptedBlob(data: ArrayBuffer): Promise<boolean> {
  // Check minimum size (encrypted data should have IV + ciphertext + tag)
  if (data.byteLength < 32) {
    return false;
  }
  
  // Read first bytes to check for encryption indicators
  const firstBytes = new Uint8Array(data.slice(0, 16));
  
  // Check for common plaintext patterns that should NOT be present
  const textDecoder = new TextDecoder('utf-8', { fatal: true });
  
  try {
    // Try to decode first bytes as UTF-8
    const decoded = textDecoder.decode(firstBytes);
    
    // Check for plaintext JSON patterns
    if (decoded.trim().startsWith('{') || 
        decoded.trim().startsWith('[') ||
        decoded.trim().startsWith('"') ||
        decoded.trim().startsWith('<')) {
      // This looks like plaintext - reject it
      return false;
    }
    
    // Check for common image headers (should be encrypted)
    const jpgHeader = [0xFF, 0xD8, 0xFF];
    const pngHeader = [0x89, 0x50, 0x4E, 0x47];
    const gifHeader = [0x47, 0x49, 0x46, 0x38];
    
    const bytesArray = Array.from(firstBytes.slice(0, 4));
    
    if (jpgHeader.every((b, i) => bytesArray[i] === b) ||
        pngHeader.every((b, i) => bytesArray[i] === b) ||
        gifHeader.every((b, i) => bytesArray[i] === b)) {
      // This looks like an image file - reject it
      return false;
    }
  } catch {
    // If decoding fails, it's likely encrypted (good)
    return true;
  }
  
  // Additional entropy check for encrypted data
  const entropy = calculateEntropy(new Uint8Array(data.slice(0, 256)));
  if (entropy < 7.0) {
    // Low entropy suggests plaintext or weak encryption
    return false;
  }
  
  return true;
}

/**
 * Calculate Shannon entropy of data
 */
function calculateEntropy(data: Uint8Array): number {
  const frequencies = new Map<number, number>();
  
  for (const byte of data) {
    frequencies.set(byte, (frequencies.get(byte) || 0) + 1);
  }
  
  let entropy = 0;
  const len = data.length;
  
  for (const count of frequencies.values()) {
    const probability = count / len;
    entropy -= probability * Math.log2(probability);
  }
  
  return entropy;
}

/**
 * Compute SHA-256 hash of data
 */
async function computeSHA256(data: ArrayBuffer): Promise<string> {
  const hashBuffer = await crypto.subtle.digest('SHA-256', data);
  const hashArray = Array.from(new Uint8Array(hashBuffer));
  return hashArray.map(b => b.toString(16).padStart(2, '0')).join('');
}

/**
 * Validate blob ID format
 */
function isValidBlobId(blobId: string): boolean {
  // Blob IDs should be: blob_<32 hex chars>
  const pattern = /^blob_[a-f0-9]{32}$/;
  return pattern.test(blobId);
}
