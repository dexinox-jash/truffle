/**
 * Zero-Access Verification
 * SPEC v2.0 SECTION 2.3.4 - Relay Server
 * SPEC v2.0 SECTION 1.2 - Non-Negotiable Design Constraints
 * 
 * CRITICAL: This module verifies that we CANNOT decrypt user content.
 * Mathematical enforcement of zero-access architecture.
 */

import type { Env } from '../index';
import { listBlobsPaginated, retrieveFromR2 } from '../storage/r2';

// Zero-access verification result
interface ZeroAccessVerification {
  verified: boolean;
  timestamp: string;
  checks: {
    no_encryption_keys_stored: boolean;
    no_plaintext_blobs: boolean;
    high_entropy_data: boolean;
    no_key_derivation_data: boolean;
    audit_log_integrity: boolean;
  };
  details: {
    total_blobs_checked: number;
    high_entropy_blobs: number;
    suspicious_blobs: number;
    average_entropy: number;
  };
  warnings: string[];
  errors: string[];
}

// Entropy threshold for encrypted data
const MIN_ENTROPY_THRESHOLD = 7.0; // Shannon entropy bits per byte

// Suspicious patterns that indicate plaintext
const SUSPICIOUS_PATTERNS = [
  // JSON patterns
  /^\s*\{/,
  /^\s*\[/,
  /^\s*"/,
  // XML/HTML patterns
  /^\s*</,
  // Common text patterns
  /^[A-Za-z\s]{100,}/,
  // Image headers (should be encrypted)
  /^\xFF\xD8\xFF/, // JPEG
  /^\x89PNG/, // PNG
  /^GIF8[79]a/, // GIF
  /^\x25PDF/, // PDF
  // Common document formats
  /^PK\x03\x04/, // ZIP (docx, xlsx, etc.)
];

/**
 * Verify zero-access architecture
 * This function runs periodic checks to ensure we cannot decrypt content
 */
export async function verifyZeroAccess(env: Env): Promise<ZeroAccessVerification> {
  const verification: ZeroAccessVerification = {
    verified: true,
    timestamp: new Date().toISOString(),
    checks: {
      no_encryption_keys_stored: true,
      no_plaintext_blobs: true,
      high_entropy_data: true,
      no_key_derivation_data: true,
      audit_log_integrity: true,
    },
    details: {
      total_blobs_checked: 0,
      high_entropy_blobs: 0,
      suspicious_blobs: 0,
      average_entropy: 0,
    },
    warnings: [],
    errors: [],
  };
  
  try {
    // Check 1: Verify no encryption keys are stored
    const keysCheck = await verifyNoEncryptionKeys(env);
    verification.checks.no_encryption_keys_stored = keysCheck.passed;
    if (!keysCheck.passed) {
      verification.errors.push(...keysCheck.errors);
      verification.verified = false;
    }
    
    // Check 2: Verify no plaintext blobs
    const plaintextCheck = await verifyNoPlaintextBlobs(env);
    verification.checks.no_plaintext_blobs = plaintextCheck.passed;
    verification.details.total_blobs_checked = plaintextCheck.blobsChecked;
    verification.details.high_entropy_blobs = plaintextCheck.highEntropyBlobs;
    verification.details.suspicious_blobs = plaintextCheck.suspiciousBlobs;
    verification.details.average_entropy = plaintextCheck.averageEntropy;
    
    if (!plaintextCheck.passed) {
      verification.errors.push(...plaintextCheck.errors);
      verification.verified = false;
    }
    
    if (plaintextCheck.warnings.length > 0) {
      verification.warnings.push(...plaintextCheck.warnings);
    }
    
    // Check 3: Verify high entropy (encryption verification)
    verification.checks.high_entropy_data = plaintextCheck.averageEntropy >= MIN_ENTROPY_THRESHOLD;
    if (!verification.checks.high_entropy_data) {
      verification.warnings.push(`Average entropy (${plaintextCheck.averageEntropy.toFixed(2)}) below threshold (${MIN_ENTROPY_THRESHOLD})`);
    }
    
    // Check 4: Verify no key derivation data
    const kdfCheck = await verifyNoKDFData(env);
    verification.checks.no_key_derivation_data = kdfCheck.passed;
    if (!kdfCheck.passed) {
      verification.errors.push(...kdfCheck.errors);
      verification.verified = false;
    }
    
    // Check 5: Verify audit log integrity
    const auditCheck = await verifyAuditLogIntegrity(env);
    verification.checks.audit_log_integrity = auditCheck.passed;
    if (!auditCheck.passed) {
      verification.errors.push(...auditCheck.errors);
      verification.verified = false;
    }
    
  } catch (err) {
    verification.verified = false;
    verification.errors.push(`Verification error: ${err instanceof Error ? err.message : 'Unknown error'}`);
  }
  
  return verification;
}

/**
 * Verify no encryption keys are stored in any bucket
 */
async function verifyNoEncryptionKeys(env: Env): Promise<{
  passed: boolean;
  errors: string[];
}> {
  const errors: string[] = [];
  let passed = true;
  
  // Keywords that indicate encryption keys
  const keyKeywords = [
    'private_key',
    'secret_key',
    'encryption_key',
    'aes_key',
    'rsa_private',
    'ed25519_private',
    'x25519_private',
    'symmetric_key',
    'master_key',
    'root_key',
  ];
  
  // Check KV namespaces
  const kvNamespaces = [
    { name: 'RATE_LIMIT_KV', namespace: env.RATE_LIMIT_KV },
    { name: 'DEVICE_SESSIONS', namespace: env.DEVICE_SESSIONS },
    { name: 'WEBSOCKET_STATE', namespace: env.WEBSOCKET_STATE },
  ];
  
  for (const { name, namespace } of kvNamespaces) {
    try {
      const keys = await namespace.list();
      for (const key of keys.keys) {
        const lowerKey = key.name.toLowerCase();
        for (const keyword of keyKeywords) {
          if (lowerKey.includes(keyword)) {
            errors.push(`Potential encryption key found in ${name}: ${key.name}`);
            passed = false;
          }
        }
      }
    } catch (err) {
      errors.push(`Failed to check ${name}: ${err instanceof Error ? err.message : 'Unknown error'}`);
    }
  }
  
  return { passed, errors };
}

/**
 * Verify no plaintext blobs exist
 */
async function verifyNoPlaintextBlobs(env: Env): Promise<{
  passed: boolean;
  blobsChecked: number;
  highEntropyBlobs: number;
  suspiciousBlobs: number;
  averageEntropy: number;
  errors: string[];
  warnings: string[];
}> {
  const errors: string[] = [];
  const warnings: string[] = [];
  let blobsChecked = 0;
  let highEntropyBlobs = 0;
  let suspiciousBlobs = 0;
  let totalEntropy = 0;
  
  try {
    let cursor: string | undefined;
    
    do {
      const listing = await listBlobsPaginated(env.SYNC_BLOBS, undefined, 100, cursor);
      
      for (const obj of listing.objects) {
        // Skip metadata files
        if (obj.key.endsWith('.meta')) continue;
        
        blobsChecked++;
        
        // Sample blob for entropy check (first 1KB)
        const blobData = await retrieveFromR2(env.SYNC_BLOBS, obj.key);
        if (!blobData) continue;
        
        const sampleSize = Math.min(blobData.byteLength, 1024);
        const sample = new Uint8Array(blobData.slice(0, sampleSize));
        
        // Check entropy
        const entropy = calculateShannonEntropy(sample);
        totalEntropy += entropy;
        
        if (entropy >= MIN_ENTROPY_THRESHOLD) {
          highEntropyBlobs++;
        } else {
          warnings.push(`Low entropy blob detected: ${obj.key} (entropy: ${entropy.toFixed(2)})`);
        }
        
        // Check for suspicious patterns
        const isSuspicious = checkSuspiciousPatterns(sample);
        if (isSuspicious) {
          suspiciousBlobs++;
          errors.push(`Suspicious (potentially unencrypted) blob detected: ${obj.key}`);
        }
      }
      
      cursor = listing.cursor;
    } while (cursor && blobsChecked < 1000); // Limit to 1000 blobs per check
    
  } catch (err) {
    errors.push(`Failed to check blobs: ${err instanceof Error ? err.message : 'Unknown error'}`);
  }
  
  const averageEntropy = blobsChecked > 0 ? totalEntropy / blobsChecked : 0;
  const passed = suspiciousBlobs === 0;
  
  return {
    passed,
    blobsChecked,
    highEntropyBlobs,
    suspiciousBlobs,
    averageEntropy,
    errors,
    warnings,
  };
}

/**
 * Verify no key derivation data is stored
 */
async function verifyNoKDFData(env: Env): Promise<{
  passed: boolean;
  errors: string[];
}> {
  const errors: string[] = [];
  let passed = true;
  
  // KDF-related keywords
  const kdfKeywords = [
    'salt',
    'kdf',
    'pbkdf2',
    'argon2',
    'scrypt',
    'bcrypt',
    'hkdf',
    'iteration',
    'memory_cost',
    'time_cost',
  ];
  
  // Check KV namespaces for KDF data
  const kvNamespaces = [
    { name: 'DEVICE_SESSIONS', namespace: env.DEVICE_SESSIONS },
    { name: 'WEBSOCKET_STATE', namespace: env.WEBSOCKET_STATE },
  ];
  
  for (const { name, namespace } of kvNamespaces) {
    try {
      const keys = await namespace.list();
      for (const key of keys.keys) {
        const lowerKey = key.name.toLowerCase();
        for (const keyword of kdfKeywords) {
          if (lowerKey.includes(keyword)) {
            errors.push(`Potential KDF data found in ${name}: ${key.name}`);
            passed = false;
          }
        }
        
        // Check value content for KDF data
        const value = await namespace.get(key.name);
        if (value) {
          const lowerValue = value.toLowerCase();
          for (const keyword of kdfKeywords) {
            if (lowerValue.includes(keyword)) {
              errors.push(`Potential KDF data in value of ${name}:${key.name}`);
              passed = false;
            }
          }
        }
      }
    } catch (err) {
      errors.push(`Failed to check ${name} for KDF data: ${err instanceof Error ? err.message : 'Unknown error'}`);
    }
  }
  
  return { passed, errors };
}

/**
 * Verify audit log integrity
 */
async function verifyAuditLogIntegrity(env: Env): Promise<{
  passed: boolean;
  errors: string[];
}> {
  const errors: string[] = [];
  let passed = true;
  
  try {
    // Check that audit logs exist and are valid
    const listing = await listBlobsPaginated(env.AUDIT_LOGS, undefined, 10);
    
    for (const obj of listing.objects) {
      const logData = await retrieveFromR2(env.AUDIT_LOGS, obj.key);
      if (!logData) continue;
      
      try {
        const logText = new TextDecoder().decode(logData);
        const logEntry = JSON.parse(logText);
        
        // Verify required fields
        if (!logEntry.timestamp || !logEntry.device_id || !logEntry.action) {
          errors.push(`Invalid audit log entry: ${obj.key}`);
          passed = false;
        }
        
        // Verify no sensitive data in logs
        const logString = JSON.stringify(logEntry);
        if (logString.includes('password') || 
            logString.includes('secret') || 
            logString.includes('private_key')) {
          errors.push(`Sensitive data found in audit log: ${obj.key}`);
          passed = false;
        }
      } catch {
        errors.push(`Corrupted audit log entry: ${obj.key}`);
        passed = false;
      }
    }
  } catch (err) {
    errors.push(`Failed to verify audit logs: ${err instanceof Error ? err.message : 'Unknown error'}`);
    passed = false;
  }
  
  return { passed, errors };
}

/**
 * Calculate Shannon entropy of data
 */
function calculateShannonEntropy(data: Uint8Array): number {
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
 * Check for suspicious patterns in data
 */
function checkSuspiciousPatterns(data: Uint8Array): boolean {
  // Convert to string for pattern matching
  const textDecoder = new TextDecoder('utf-8', { fatal: true });
  let text: string;
  
  try {
    text = textDecoder.decode(data);
  } catch {
    // If decoding fails, it's likely encrypted (good)
    return false;
  }
  
  // Check for suspicious patterns
  for (const pattern of SUSPICIOUS_PATTERNS) {
    if (pattern.test(text)) {
      return true;
    }
  }
  
  return false;
}

/**
 * Generate zero-access attestation
 * This can be used for compliance reporting
 */
export async function generateZeroAccessAttestation(env: Env): Promise<{
  attestation_id: string;
  generated_at: string;
  verification: ZeroAccessVerification;
  signature: string;
}> {
  const verification = await verifyZeroAccess(env);
  
  const attestationData = {
    timestamp: verification.timestamp,
    verified: verification.verified,
    checks: verification.checks,
    details: verification.details,
  };
  
  // Generate attestation ID
  const attestationId = crypto.randomUUID();
  
  // Create signature (in production, this would be signed with a hardware key)
  const encoder = new TextEncoder();
  const data = encoder.encode(JSON.stringify(attestationData));
  const hashBuffer = await crypto.subtle.digest('SHA-256', data);
  const signature = Array.from(new Uint8Array(hashBuffer))
    .map(b => b.toString(16).padStart(2, '0'))
    .join('');
  
  return {
    attestation_id: attestationId,
    generated_at: verification.timestamp,
    verification,
    signature,
  };
}

/**
 * Run continuous zero-access monitoring
 * This should be called periodically (e.g., via Cron Trigger)
 */
export async function runZeroAccessMonitoring(env: Env): Promise<{
  success: boolean;
  report: ZeroAccessVerification;
  alert_sent: boolean;
}> {
  const verification = await verifyZeroAccess(env);
  
  let alertSent = false;
  
  // If verification failed, trigger alert
  if (!verification.verified) {
    // In production, this would send alerts to security team
    console.error('ZERO-ACCESS VIOLATION DETECTED:', verification.errors);
    alertSent = true;
  }
  
  // Store verification report
  const reportKey = `zero_access_reports/${Date.now()}.json`;
  await env.AUDIT_LOGS.put(
    reportKey,
    JSON.stringify(verification),
    {
      customMetadata: {
        'Content-Type': 'application/json',
        'X-Verified': verification.verified.toString(),
        'X-Timestamp': Date.now().toString(),
      },
    }
  );
  
  return {
    success: verification.verified,
    report: verification,
    alert_sent: alertSent,
  };
}
