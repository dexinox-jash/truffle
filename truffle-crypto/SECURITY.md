# Security Policy

## Zero-Knowledge Guarantee

Project Truffle maintains a **mathematical inability** to decrypt user content. This is not a policy choice—it is a cryptographic guarantee enforced by the design of our system.

### What This Means

- **We cannot read your data**: Even with full access to our servers, we cannot decrypt your wiki content, screenshots, or metadata.
- **We cannot comply with decryption requests**: Law enforcement, hackers, or internal threats cannot force us to decrypt data we mathematically cannot access.
- **Your keys, your control**: All encryption keys are generated and stored on your devices. We never have access to them.

### Technical Basis

Our zero-knowledge architecture relies on:

1. **Client-side encryption**: All data is encrypted on your device before transmission
2. **X3DH key exchange**: Devices establish shared secrets without server involvement
3. **No key escrow**: We have no mechanism to recover or access your keys
4. **Open source**: Our cryptographic implementation is auditable (AGPL-3.0)

## Reporting Security Vulnerabilities

We take security seriously. If you discover a vulnerability, please:

1. **Do not** open a public issue
2. Email security@truffle.io with details
3. Allow 90 days for remediation before public disclosure
4. We will acknowledge receipt within 24 hours

### What to Include

- Description of the vulnerability
- Steps to reproduce
- Potential impact
- Suggested fix (if any)

### Bug Bounty

We operate a bug bounty program through HackerOne. Critical vulnerabilities in our cryptographic implementation may be eligible for rewards.

## Security Checklist

### For Users

- [ ] Keep your devices physically secure
- [ ] Use strong device passwords/biometrics
- [ ] Verify SAS codes during device pairing
- [ ] Regularly export your data (5-minute export)
- [ ] Keep the app updated

### For Developers

- [ ] All crypto operations use constant-time comparisons
- [ ] All keys implement `ZeroizeOnDrop`
- [ ] No `unsafe` code in crypto paths
- [ ] All nonces are randomly generated (never reused)
- [ ] All ciphertext is authenticated (AEAD)
- [ ] Dependencies are regularly audited

## Cryptographic Primitives

### Symmetric Encryption

| Algorithm | Use Case | Security Level |
|-----------|----------|----------------|
| AES-256-GCM | Desktop (hardware accelerated) | 256-bit |
| ChaCha20-Poly1305 | Mobile (constant-time) | 256-bit |

### Asymmetric Encryption

| Algorithm | Purpose | Security Level |
|-----------|---------|----------------|
| X25519 | ECDH key exchange | 128-bit |
| Ed25519 | Digital signatures | 128-bit |
| Kyber-768 | Post-quantum KEM | 192-bit (NIST Level 3) |

### Hash Functions

| Algorithm | Purpose |
|-----------|---------|
| SHA3-256 | Content addressing |
| SHA2-256 | General hashing |
| HKDF-SHA256 | Key derivation |
| HMAC-SHA256 | Message authentication |

## Threat Model

### Assets

1. **User Content**: Screenshots, wiki nodes, metadata
2. **Encryption Keys**: Sync keys, identity keys
3. **Sync Data**: Encrypted CRDT updates

### Threats

#### High Severity

| Threat | Likelihood | Impact | Mitigation |
|--------|------------|--------|------------|
| Server compromise | Low | Critical | Zero-knowledge architecture |
| Man-in-the-middle | Low | High | X3DH + SAS verification |
| Device theft | Medium | High | Device encryption + remote wipe |

#### Medium Severity

| Threat | Likelihood | Impact | Mitigation |
|--------|------------|--------|------------|
| Replay attacks | Low | Medium | Unique nonces + timestamps |
| Key extraction | Low | Medium | Secure Enclave/TPM storage |
| Side-channel | Low | Medium | Constant-time operations |

#### Low Severity

| Threat | Likelihood | Impact | Mitigation |
|--------|------------|--------|------------|
| Cryptanalysis | Very Low | Critical | Post-quantum hybrid |
| Implementation bugs | Low | Medium | Extensive testing + audit |

## Incident Response

### Severity 1: Key Compromise (Theoretical)

1. Confirm zero-knowledge status (verify no keys compromised)
2. Rotate relay server certificates (precautionary)
3. Notify users via in-app push (no email required, no PII exposed)
4. Publish transparency report within 24h

### Severity 2: Ransomware on User Device

1. Out of scope (user device compromise)
2. Provide recovery guide: restore from second device or backup

### Severity 3: Service Outage

1. Fail-open to local mode (sync pauses, local functionality continues)
2. Status page update (status.truffle.io)

## Security Audit History

| Date | Auditor | Scope | Results |
|------|---------|-------|---------|
| TBD | TBD | Full crypto audit | Pending |

## Contact

- Security Team: security@truffle.io
- GPG Key: [Download](https://truffle.io/security.gpg)
- Bug Bounty: [HackerOne](https://hackerone.com/truffle)
