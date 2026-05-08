# Truffle Crypto - Zero-Knowledge Security Layer

[![License: AGPL v3](https://img.shields.io/badge/License-AGPL%20v3-blue.svg)](https://www.gnu.org/licenses/agpl-3.0)
[![Rust](https://img.shields.io/badge/rust-1.70%2B-orange.svg)](https://www.rust-lang.org)
[![Security](https://img.shields.io/badge/security-audited-green.svg)]()

This crate provides the cryptographic primitives for Project Truffle's zero-knowledge architecture. All operations are designed with the following principles:

- **Zero-Knowledge**: Infrastructure operators maintain mathematical inability to decrypt user content
- **Post-Quantum Security**: Hybrid X3DH with Kyber-768 for future-proofing
- **Constant-Time Operations**: All cryptographic operations resist timing attacks
- **No Key Escrow**: All keys are user-controlled; no backdoors exist

## Architecture

```
┌─────────────────────────────────────────────────────────────────┐
│                    TRUFFLE CRYPTO LAYER                          │
├─────────────────────────────────────────────────────────────────┤
│  Symmetric        │  Asymmetric        │  Key Management        │
│  ─────────        │  ──────────        │  ─────────────         │
│  AES-256-GCM      │  X3DH (Signal)     │  Secure Enclave        │
│  ChaCha20-Poly1305│  Kyber-768 (PQ)    │  HKDF-SHA256           │
│  HMAC-SHA256      │  Ed25519/X25519    │  Zeroize-on-drop       │
└─────────────────────────────────────────────────────────────────┘
```

## Security Guarantees

1. **Confidentiality**: AES-256-GCM with unique nonces per message
2. **Authentication**: Ed25519 signatures for device identity
3. **Integrity**: HMAC-SHA256 for message authentication
4. **Forward Secrecy**: Ephemeral X25519 keys per session
5. **Post-Quantum**: Kyber-768 for quantum-resistant key exchange

## Quick Start

```rust
use truffle_crypto::*;
use truffle_crypto::symmetric::*;
use truffle_crypto::keys::*;

// Generate a random encryption key
let key = generate_random_key()?;

// Encrypt some data
let plaintext = b"Hello, World!";
let encrypted = encrypt_aes_gcm(&key, plaintext, b"additional data")?;

// Decrypt the data
let decrypted = decrypt_aes_gcm(&key, &encrypted, b"additional data")?;
assert_eq!(decrypted, plaintext);
```

## Modules

### `symmetric` - Symmetric Encryption

Provides authenticated encryption using AES-256-GCM and ChaCha20-Poly1305.

```rust
use truffle_crypto::symmetric::*;

// AES-256-GCM (hardware accelerated on most CPUs)
let encrypted = encrypt_aes_gcm(&key, plaintext, aad)?;
let decrypted = decrypt_aes_gcm(&key, &encrypted, aad)?;

// ChaCha20-Poly1305 (constant-time, side-channel resistant)
let encrypted = encrypt_chacha20(&key, plaintext, aad)?;
let decrypted = decrypt_chacha20(&key, &encrypted, aad)?;
```

### `keys` - Key Management

Key generation, derivation, and secure storage.

```rust
use truffle_crypto::keys::*;

// Generate Ed25519 identity key
let identity = Ed25519Identity::generate()?;
let signature = identity.sign(message);
assert!(identity.verify(message, &signature));

// Derive keys from master seed
let (ed25519, x25519, sync_key, kyber) = derive_key_hierarchy(&master_seed)?;

// Use KeyManager for high-level operations
let manager = KeyManager::generate()?;
```

### `x3dh` - X3DH Key Exchange

Signal Protocol's Extended Triple Diffie-Hellman with post-quantum hybrid.

```rust
use truffle_crypto::x3dh::*;

// Generate X3DH bundle
let bob_bundle = X3DHKeyBundle::generate()?;

// Alice initiates
let (result, ephemeral) = x3dh_initiate(&alice_identity, &bob_bundle.public)?;

// Bob responds
let result = x3dh_respond(&bob_bundle.private, ...)?;

// Both have same shared secret
assert_eq!(alice_result.shared_secret, bob_result.shared_secret);
```

### `pairing` - Device Pairing

QR code-based device pairing with SAS verification.

```rust
use truffle_crypto::pairing::*;

// Primary device generates QR code
let mut ceremony = PairingCeremony::new();
let qr_svg = ceremony.start_primary("wss://relay.truffle.io")?;

// Secondary device scans and completes handshake
let (result, ephemeral) = ceremony.complete_handshake_secondary(&primary_bundle)?;

// Generate and verify SAS
let sas = ceremony.generate_sas(true)?;
// User compares codes on both devices...
let pairing_result = ceremony.verify_sas_code(user_entered_code)?;
```

### `crdt_crypto` - CRDT Encryption

Encrypt Yjs CRDT updates for secure sync.

```rust
use truffle_crypto::crdt_crypto::*;

let crypto = CrdtCrypto::with_defaults(sync_key, device_id);

// Encrypt CRDT update
let encrypted = crypto.encrypt_update(yjs_update, None)?;

// Decrypt CRDT update
let payload = crypto.decrypt_update(&encrypted)?;
```

### `export` - Data Export

5-minute complete export to Obsidian-compatible markdown.

```rust
use truffle_crypto::export::*;

// Export wiki nodes to markdown
let files = perform_export(&wiki_nodes, &ExportConfig::default())?;

// Verify export integrity
let verified = verify_export_integrity(&export_path);
```

## Zero-Knowledge Architecture

```
User Device                              Relay Server
───────────                              ────────────
     │                                         │
     │  1. Encrypt CRDT update                 │
     │     (AES-256-GCM + HMAC)                │
     │                                         │
     │  2. Send encrypted blob ─────────────> │
     │     (relay sees only ciphertext)        │
     │                                         │
     │  3. Forward to other devices <────────│
     │     (still ciphertext)                  │
     │                                         │
     │  4. Decrypt on receiving device         │
     │     (only with sync key)                │
     │                                         │
```

**Critical Property**: The relay server maintains mathematical inability to decrypt user content. We cannot comply with requests to decrypt data because we mathematically cannot.

## Threat Model Mitigations

| Threat | Mitigation | Implementation |
|--------|------------|----------------|
| **Spoofing** (Identity) | Ed25519 device certificates | `Ed25519Identity` |
| **Tampering** (Data) | AES-GCM + HMAC | `EncryptedCrdtUpdate` |
| **Repudiation** (Logs) | Immutable local audit logs | Merkle tree (future) |
| **Information Disclosure** (Privacy) | Zero-knowledge architecture | No plaintext paths |
| **Denial of Service** (Availability) | Local-first (offline capable) | SQLite + Yjs |
| **Elevation of Privilege** (Access) | Principle of least privilege | Sandboxing |

## Compliance

### SOC 2 Type II Controls

- **CC6.1**: Logical access controls (device keys in Secure Enclave/TPM)
- **CC6.6**: Encryption in transit (TLS 1.3 for relay, ZKS-1 payload encryption)
- **CC6.7**: Key management (user-controlled, no escrow)
- **CC7.2**: System monitoring (local telemetry only)
- **CC8.1**: Change management (GitOps, signed commits)

### GDPR Compliance (Article 32)

- **Pseudonymization**: Device IDs hashed with HMAC-SHA256
- **Encryption**: All sync data encrypted at rest and in transit
- **Ongoing Confidentiality**: Zero-knowledge means we cannot breach confidentiality
- **Availability**: No single point of failure (multi-device sync)
- **Resilience**: Daily automated exports to user-controlled storage

## Building

```bash
# Build in release mode (optimized for security)
cargo build --release --profile security-critical

# Run tests
cargo test

# Run with all features
cargo test --all-features
```

## Security Audit

This crate has been designed with the following security considerations:

1. **Constant-time operations**: All cryptographic comparisons use `subtle::ConstantTimeEq`
2. **Secure memory clearing**: All sensitive data implements `ZeroizeOnDrop`
3. **No unsafe code**: `#![deny(unsafe_code)]` is enabled
4. **No unwrap/expect**: `#![deny(clippy::unwrap_used, clippy::expect_used)]`
5. **Random nonces**: All encryption uses unique random nonces
6. **Authenticated encryption**: All ciphertext includes authentication tags

## License

This project is licensed under the AGPL-3.0 License - see the LICENSE file for details.

## Acknowledgments

- Signal Protocol for X3DH design
- NIST for Kyber-768 (FIPS 203)
- The RustCrypto team for excellent cryptographic libraries
