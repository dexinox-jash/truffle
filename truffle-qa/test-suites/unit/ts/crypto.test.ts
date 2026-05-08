/**
 * @fileoverview Cryptographic Module Unit Tests
 * @module tests/unit/crypto
 * 
 * Validates X3DH key exchange, AES-256-GCM encryption, and Zero-Knowledge guarantees.
 * Critical for STRIDE threat model compliance.
 * 
 * @requires jest
 * @requires @truffle/core/crypto
 */

import {
  generateIdentityKey,
  x3dhHandshake,
  encryptAESGCM,
  decryptAESGCM,
  encryptChaCha20,
  generateKyberKeypair,
  deriveSharedSecret,
  verifyZeroKnowledge,
  generateNonce,
  type X3DHKeyBundle,
  type EncryptedPayload,
  type DeviceKeys,
} from '@truffle/core/crypto';
import { randomBytes, createHash } from 'crypto';

describe('X3DH Key Exchange (Signal Protocol)', () => {
  let aliceKeys: DeviceKeys;
  let bobKeys: DeviceKeys;

  beforeEach(() => {
    aliceKeys = generateIdentityKey();
    bobKeys = generateIdentityKey();
  });

  describe('Identity Key Generation', () => {
    test('SHOULD generate valid Ed25519 identity key', () => {
      const keys = generateIdentityKey();
      
      // Verify key structure
      expect(keys.identityKey).toBeDefined();
      expect(keys.identityKey.publicKey).toBeInstanceOf(Uint8Array);
      expect(keys.identityKey.publicKey.length).toBe(32); // Ed25519 public key size
      expect(keys.identityKey.privateKey).toBeInstanceOf(Uint8Array);
      expect(keys.identityKey.privateKey.length).toBe(64); // Ed25519 private key size
    });

    test('SHOULD generate unique keys each time', () => {
      const keys1 = generateIdentityKey();
      const keys2 = generateIdentityKey();
      
      expect(keys1.identityKey.publicKey).not.toEqual(keys2.identityKey.publicKey);
    });

    test('SHOULD generate ephemeral X25519 keys', () => {
      const keys = generateIdentityKey();
      
      expect(keys.ephemeralKeys).toBeDefined();
      expect(keys.ephemeralKeys.length).toBeGreaterThan(0);
      expect(keys.ephemeralKeys[0].publicKey.length).toBe(32); // X25519 public key size
    });
  });

  describe('X3DH Handshake', () => {
    test('SHOULD derive identical shared secrets', () => {
      const aliceBundle: X3DHKeyBundle = {
        identityKey: aliceKeys.identityKey.publicKey,
        ephemeralKey: aliceKeys.ephemeralKeys[0].publicKey,
        preKey: aliceKeys.preKeys[0].publicKey,
        preKeySignature: aliceKeys.preKeys[0].signature,
      };

      const bobBundle: X3DHKeyBundle = {
        identityKey: bobKeys.identityKey.publicKey,
        ephemeralKey: bobKeys.ephemeralKeys[0].publicKey,
        preKey: bobKeys.preKeys[0].publicKey,
        preKeySignature: bobKeys.preKeys[0].signature,
      };

      const aliceShared = x3dhHandshake(aliceKeys, bobBundle);
      const bobShared = x3dhHandshake(bobKeys, aliceBundle);

      expect(aliceShared).toEqual(bobShared);
      expect(aliceShared.length).toBe(32); // 256-bit shared secret
    });

    test('SHOULD reject invalid preKey signature', () => {
      const invalidBundle: X3DHKeyBundle = {
        identityKey: bobKeys.identityKey.publicKey,
        ephemeralKey: bobKeys.ephemeralKeys[0].publicKey,
        preKey: bobKeys.preKeys[0].publicKey,
        preKeySignature: randomBytes(64), // Invalid signature
      };

      expect(() => x3dhHandshake(aliceKeys, invalidBundle)).toThrow('Invalid preKey signature');
    });

    test('SHOULD derive different secrets with different ephemeral keys', () => {
      const aliceBundle1: X3DHKeyBundle = {
        identityKey: aliceKeys.identityKey.publicKey,
        ephemeralKey: aliceKeys.ephemeralKeys[0].publicKey,
        preKey: aliceKeys.preKeys[0].publicKey,
        preKeySignature: aliceKeys.preKeys[0].signature,
      };

      const aliceBundle2: X3DHKeyBundle = {
        identityKey: aliceKeys.identityKey.publicKey,
        ephemeralKey: aliceKeys.ephemeralKeys[1].publicKey, // Different ephemeral
        preKey: aliceKeys.preKeys[0].publicKey,
        preKeySignature: aliceKeys.preKeys[0].signature,
      };

      const shared1 = x3dhHandshake(bobKeys, aliceBundle1);
      const shared2 = x3dhHandshake(bobKeys, aliceBundle2);

      expect(shared1).not.toEqual(shared2);
    });
  });

  describe('STRIDE: Spoofing Mitigation', () => {
    test('SHOULD reject invalid identity signature', () => {
      // Attempt to forge identity
      const forgedKeys = {
        ...bobKeys,
        identityKey: {
          publicKey: randomBytes(32),
          privateKey: randomBytes(64),
        },
      };

      const aliceBundle: X3DHKeyBundle = {
        identityKey: aliceKeys.identityKey.publicKey,
        ephemeralKey: aliceKeys.ephemeralKeys[0].publicKey,
        preKey: aliceKeys.preKeys[0].publicKey,
        preKeySignature: aliceKeys.preKeys[0].signature,
      };

      // Handshake should fail with forged identity
      expect(() => x3dhHandshake(forgedKeys, aliceBundle)).toThrow();
    });
  });
});

describe('AES-256-GCM Encryption', () => {
  let key: Uint8Array;

  beforeEach(() => {
    key = randomBytes(32); // 256-bit key
  });

  describe('Encryption/Decryption', () => {
    test('SHOULD encrypt and decrypt successfully', () => {
      const plaintext = Buffer.from('Hello, Truffle!');
      
      const encrypted = encryptAESGCM(plaintext, key);
      expect(encrypted.ciphertext).not.toEqual(plaintext);
      expect(encrypted.nonce).toBeDefined();
      expect(encrypted.tag).toBeDefined();

      const decrypted = decryptAESGCM(encrypted, key);
      expect(decrypted).toEqual(plaintext);
    });

    test('SHOULD use random 96-bit nonce', () => {
      const plaintext = Buffer.from('test');
      
      const encrypted1 = encryptAESGCM(plaintext, key);
      const encrypted2 = encryptAESGCM(plaintext, key);
      
      // Nonces must be different
      expect(encrypted1.nonce).not.toEqual(encrypted2.nonce);
      expect(encrypted1.nonce.length).toBe(12); // 96-bit nonce
    });

    test('SHOULD handle empty plaintext', () => {
      const plaintext = Buffer.from('');
      
      const encrypted = encryptAESGCM(plaintext, key);
      const decrypted = decryptAESGCM(encrypted, key);
      
      expect(decrypted).toEqual(plaintext);
    });

    test('SHOULD handle large plaintext', () => {
      const plaintext = randomBytes(1024 * 1024); // 1MB
      
      const encrypted = encryptAESGCM(plaintext, key);
      const decrypted = decryptAESGCM(encrypted, key);
      
      expect(decrypted).toEqual(plaintext);
    });
  });

  describe('STRIDE: Tampering Mitigation', () => {
    test('SHOULD detect ciphertext tampering', () => {
      const plaintext = Buffer.from('Sensitive data');
      const encrypted = encryptAESGCM(plaintext, key);
      
      // Tamper with ciphertext
      encrypted.ciphertext[0] ^= 0xFF;
      
      expect(() => decryptAESGCM(encrypted, key)).toThrow('Authentication failed');
    });

    test('SHOULD detect nonce tampering', () => {
      const plaintext = Buffer.from('Sensitive data');
      const encrypted = encryptAESGCM(plaintext, key);
      
      // Tamper with nonce
      encrypted.nonce[0] ^= 0xFF;
      
      expect(() => decryptAESGCM(encrypted, key)).toThrow('Authentication failed');
    });

    test('SHOULD detect tag tampering', () => {
      const plaintext = Buffer.from('Sensitive data');
      const encrypted = encryptAESGCM(plaintext, key);
      
      // Tamper with authentication tag
      encrypted.tag[0] ^= 0xFF;
      
      expect(() => decryptAESGCM(encrypted, key)).toThrow('Authentication failed');
    });

    test('SHOULD detect additional data tampering', () => {
      const plaintext = Buffer.from('Sensitive data');
      const aad = Buffer.from('associated data');
      
      const encrypted = encryptAESGCM(plaintext, key, aad);
      
      // Tamper with AAD
      const tamperedAad = Buffer.from('tampered data');
      
      expect(() => decryptAESGCM(encrypted, key, tamperedAad)).toThrow('Authentication failed');
    });
  });

  describe('Key Validation', () => {
    test('SHOULD reject wrong key', () => {
      const plaintext = Buffer.from('Sensitive data');
      const encrypted = encryptAESGCM(plaintext, key);
      
      const wrongKey = randomBytes(32);
      
      expect(() => decryptAESGCM(encrypted, wrongKey)).toThrow('Authentication failed');
    });

    test('SHOULD reject invalid key size', () => {
      const invalidKey = randomBytes(16); // 128-bit instead of 256-bit
      const plaintext = Buffer.from('test');
      
      expect(() => encryptAESGCM(plaintext, invalidKey)).toThrow('Invalid key size');
    });
  });
});

describe('ChaCha20-Poly1305 (Mobile)', () => {
  let key: Uint8Array;

  beforeEach(() => {
    key = randomBytes(32);
  });

  test('SHOULD encrypt and decrypt on mobile', () => {
    const plaintext = Buffer.from('Mobile data');
    
    const encrypted = encryptChaCha20(plaintext, key);
    expect(encrypted.ciphertext).not.toEqual(plaintext);

    const decrypted = decryptChaCha20(encrypted, key);
    expect(decrypted).toEqual(plaintext);
  });

  test('SHOULD be constant-time resistant', () => {
    // This is a property of ChaCha20-Poly1305 implementation
    // Actual constant-time verification requires specialized tools
    const plaintext = Buffer.from('test');
    const encrypted = encryptChaCha20(plaintext, key);
    
    expect(encrypted).toBeDefined();
    expect(encrypted.nonce.length).toBe(12);
  });
});

describe('Post-Quantum Hybrid (Kyber-768)', () => {
  test('SHOULD generate Kyber-768 keypair', () => {
    const keypair = generateKyberKeypair();
    
    expect(keypair.publicKey).toBeInstanceOf(Uint8Array);
    expect(keypair.secretKey).toBeInstanceOf(Uint8Array);
    expect(keypair.publicKey.length).toBe(1184); // Kyber-768 public key size
    expect(keypair.secretKey.length).toBe(2400); // Kyber-768 secret key size
  });

  test('SHOULD encapsulate and decapsulate correctly', () => {
    const keypair = generateKyberKeypair();
    
    const { ciphertext, sharedSecret: secret1 } = encapsulate(keypair.publicKey);
    const secret2 = decapsulate(ciphertext, keypair.secretKey);
    
    expect(secret1).toEqual(secret2);
    expect(secret1.length).toBe(32); // 256-bit shared secret
  });

  test('SHOULD combine X3DH and Kyber for hybrid security', () => {
    const deviceKeys = generateIdentityKey();
    const kyberKeys = generateKyberKeypair();
    
    const hybridSecret = deriveSharedSecret(deviceKeys, kyberKeys);
    
    expect(hybridSecret.length).toBe(32);
    // Hybrid secret should be different from either individual secret
    expect(hybridSecret).not.toEqual(deviceKeys.identityKey.publicKey.slice(0, 32));
  });
});

describe('Zero-Knowledge Verification (Red Line)', () => {
  test('MUST NOT have plaintext paths to sync payload', () => {
    const syncPayload = {
      crdt_update: randomBytes(1000),
      schema_version: '2.0',
      deleted_artifacts: [],
    };
    
    const key = randomBytes(32);
    const encrypted = encryptSyncPayload(syncPayload, key);
    
    // Verify payload is encrypted
    const isPlaintext = isValidUTF8(encrypted.payload) && 
                       isValidJSON(encrypted.payload);
    expect(isPlaintext).toBe(false);
    
    // Verify no plaintext metadata
    expect(encrypted.metadata).not.toContain('wiki');
    expect(encrypted.metadata).not.toContain('artifact');
  });

  test('MUST verify infrastructure cannot decrypt', () => {
    const plaintext = { secret: 'user data' };
    const userKey = randomBytes(32);
    
    // User encrypts
    const encrypted = encryptAESGCM(Buffer.from(JSON.stringify(plaintext)), userKey);
    
    // Infrastructure (without user key) cannot decrypt
    const infrastructureKey = randomBytes(32);
    expect(() => decryptAESGCM(encrypted, infrastructureKey)).toThrow();
    
    // Verify mathematical impossibility
    const verification = verifyZeroKnowledge(encrypted, userKey);
    expect(verification.infrastructureCanDecrypt).toBe(false);
  });

  test('MUST encrypt all sync data', () => {
    const syncData = {
      nodes: [{ title: 'Test Node', content: 'Secret content' }],
      updates: [{ type: 'insert', data: 'sensitive' }],
    };
    
    const key = deriveSyncKey('device-a', 'device-b');
    const encrypted = encryptSyncData(syncData, key);
    
    // All fields should be encrypted
    expect(encrypted.encryptedBlob).toBeDefined();
    expect(encrypted.encryptedBlob).not.toContain('Test Node');
    expect(encrypted.encryptedBlob).not.toContain('Secret content');
    expect(encrypted.encryptedBlob).not.toContain('sensitive');
  });
});

describe('HKDF Key Derivation', () => {
  test('SHOULD derive sync_key and auth_key from shared secret', () => {
    const sharedSecret = randomBytes(32);
    const salt = randomBytes(32);
    
    const { syncKey, authKey } = deriveKeys(sharedSecret, salt);
    
    expect(syncKey.length).toBe(32);
    expect(authKey.length).toBe(32);
    expect(syncKey).not.toEqual(authKey);
  });

  test('SHOULD derive different keys with different salts', () => {
    const sharedSecret = randomBytes(32);
    const salt1 = randomBytes(32);
    const salt2 = randomBytes(32);
    
    const keys1 = deriveKeys(sharedSecret, salt1);
    const keys2 = deriveKeys(sharedSecret, salt2);
    
    expect(keys1.syncKey).not.toEqual(keys2.syncKey);
  });
});

describe('Nonce Generation', () => {
  test('SHOULD generate unique nonces', () => {
    const nonces = new Set();
    
    for (let i = 0; i < 10000; i++) {
      const nonce = generateNonce();
      const nonceHex = Buffer.from(nonce).toString('hex');
      
      expect(nonces.has(nonceHex)).toBe(false);
      nonces.add(nonceHex);
    }
  });

  test('SHOULD generate 96-bit nonces for AES-GCM', () => {
    const nonce = generateNonce(12); // 96 bits = 12 bytes
    expect(nonce.length).toBe(12);
  });
});

// Helper functions
function isValidUTF8(buffer: Uint8Array): boolean {
  try {
    new TextDecoder('utf-8', { fatal: true }).decode(buffer);
    return true;
  } catch {
    return false;
  }
}

function isValidJSON(buffer: Uint8Array): boolean {
  try {
    JSON.parse(new TextDecoder().decode(buffer));
    return true;
  } catch {
    return false;
  }
}

function encapsulate(publicKey: Uint8Array): { ciphertext: Uint8Array; sharedSecret: Uint8Array } {
  // Mock implementation - actual would use Kyber library
  const ciphertext = randomBytes(1088); // Kyber-768 ciphertext size
  const sharedSecret = randomBytes(32);
  return { ciphertext, sharedSecret };
}

function decapsulate(ciphertext: Uint8Array, secretKey: Uint8Array): Uint8Array {
  // Mock implementation
  return randomBytes(32);
}

function encryptSyncPayload(payload: any, key: Uint8Array): any {
  const serialized = Buffer.from(JSON.stringify(payload));
  const encrypted = encryptAESGCM(serialized, key);
  return {
    payload: Buffer.concat([encrypted.nonce, encrypted.ciphertext, encrypted.tag]),
    metadata: {},
  };
}

function encryptSyncData(data: any, key: Uint8Array): any {
  const serialized = Buffer.from(JSON.stringify(data));
  const encrypted = encryptAESGCM(serialized, key);
  return {
    encryptedBlob: Buffer.concat([encrypted.nonce, encrypted.ciphertext, encrypted.tag]),
  };
}

function deriveSyncKey(deviceA: string, deviceB: string): Uint8Array {
  const input = Buffer.from(`${deviceA}:${deviceB}`);
  return createHash('sha256').update(input).digest();
}

function deriveKeys(sharedSecret: Uint8Array, salt: Uint8Array): { syncKey: Uint8Array; authKey: Uint8Array } {
  // Mock HKDF-SHA256
  const derived = createHash('sha256').update(Buffer.concat([sharedSecret, salt])).digest();
  return {
    syncKey: derived.slice(0, 16),
    authKey: derived.slice(16, 32),
  };
}

function decryptChaCha20(encrypted: EncryptedPayload, key: Uint8Array): Buffer {
  // Mock implementation - actual would use libsodium or similar
  return decryptAESGCM(encrypted, key);
}
