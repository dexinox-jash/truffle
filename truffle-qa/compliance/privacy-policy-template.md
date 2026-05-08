# Privacy Policy
## Project Truffle - Your Visual Memory, Compiled

**Effective Date:** [DATE]  
**Last Updated:** [DATE]  
**Version:** 1.0.0  

---

## 1. Introduction

Project Truffle ("we", "us", "our") is committed to protecting your privacy. This Privacy Policy explains how we collect, use, disclose, and safeguard your information when you use our application.

**Our Core Principle: Zero-Knowledge Architecture**

We have designed Truffle with a zero-knowledge architecture, which means:
- Your screenshots and compiled data never leave your device unencrypted
- We cannot access, read, or decrypt your content
- Even if compelled by legal order, we cannot provide your data to third parties

---

## 2. Information We Collect

### 2.1 Information You Provide

We do not require account creation or personal information to use Truffle.

### 2.2 Information Collected Automatically

| Information | Purpose | Storage |
|-------------|---------|---------|
| Device fingerprint (hashed) | Sync identification | Encrypted, pseudonymized |
| App version | Compatibility | Aggregated, anonymous |
| Sync success/failure rates | Service improvement | Aggregated, anonymous |
| Compilation latency metrics | Performance optimization | Aggregated, anonymous |

### 2.3 What We Do NOT Collect

We explicitly do NOT collect:
- Screenshot content or images
- OCR-extracted text
- Wiki node titles or content
- Entity names or relationships
- Your IP address
- Your precise location
- Any content from your device

---

## 3. How We Use Your Information

### 3.1 Purpose Limitation

| Data | Use | Legal Basis |
|------|-----|-------------|
| Hashed device ID | Device sync | Legitimate interest |
| App version | Compatibility | Legitimate interest |
| Aggregated metrics | Service improvement | Legitimate interest |

### 3.2 No Content Processing

Your screenshots and wiki content are:
- Processed entirely on your device
- Never transmitted to our servers
- Never accessible by us
- Under your complete control

---

## 4. Data Storage and Security

### 4.1 Local-First Architecture

All your data is stored locally on your device:
- Screenshots: Your device's file system
- Wiki database: Local SQLite database
- Embeddings: Local vector index
- Keys: Secure Enclave (iOS) or TPM (Windows/Linux)

### 4.2 Encryption

| Data Type | Encryption | Key Location |
|-----------|------------|--------------|
| Wiki database | AES-256-GCM | Device only |
| Sync data | AES-256-GCM | Device only |
| Device keys | Hardware-backed | Secure Enclave/TPM |

### 4.3 Zero-Knowledge Sync

When you sync between devices:
- Data is encrypted on your device before transmission
- We relay only encrypted data (we cannot decrypt it)
- Keys are exchanged directly between your devices
- We have zero knowledge of your content

---

## 5. Data Sharing and Disclosure

### 5.1 We Do Not Sell Your Data

We do not sell, rent, or trade your personal information to third parties.

### 5.2 Legal Requests

Due to our zero-knowledge architecture:
- We cannot access your encrypted data
- We cannot provide your content in response to legal requests
- We will notify you of any legal requests we receive

### 5.3 Service Providers

We use the following service providers:

| Provider | Service | Data Shared |
|----------|---------|-------------|
| Cloudflare | CDN, Relay | Encrypted blobs only (we cannot decrypt) |
| Stripe | Payment processing | Payment information (if applicable) |

---

## 6. Your Rights (GDPR)

Under the General Data Protection Regulation (GDPR), you have the following rights:

### 6.1 Right to Access

You can export all your data at any time:
- Go to Settings → Export
- Choose format (Markdown, Obsidian)
- Complete export in under 5 minutes

### 6.2 Right to Rectification

You can edit any wiki node directly in the application.

### 6.3 Right to Erasure (Right to be Forgotten)

To delete your data:
1. Delete the application (removes all local data)
2. Use the "Delete Account" feature to sync deletion to all devices
3. Contact us for verification

### 6.4 Right to Data Portability

Export your data in standard formats:
- Markdown (compatible with Obsidian, Notion)
- Git repository with full history

### 6.5 Right to Object

Opt-out of optional telemetry:
- Go to Settings → Privacy
- Toggle "Share Anonymous Metrics"

### 6.6 Right to Restrict Processing

Pause sync at any time:
- Go to Settings → Sync
- Toggle "Pause Sync"

### 6.7 Exercising Your Rights

To exercise any of these rights, contact us at:
- Email: privacy@truffle.io
- Response time: Within 30 days

---

## 7. International Data Transfers

### 7.1 Data Residency

| Region | Relay Server Location |
|--------|----------------------|
| EU | Cloudflare EU region |
| US | Cloudflare US region |
| Enterprise | Customer-specified (self-hosted option) |

### 7.2 Transfer Safeguards

All sync data is:
- Encrypted with AES-256-GCM before transmission
- Keys never leave your device
- We cannot decrypt even if data crosses borders

---

## 8. Children's Privacy

Truffle is not intended for children under 16. We do not knowingly collect information from children under 16.

---

## 9. Changes to This Privacy Policy

We may update this Privacy Policy from time to time. We will notify you of any changes by:
- In-app notification
- Email (if provided)
- Updated effective date

---

## 10. Contact Us

If you have questions about this Privacy Policy, contact us:

**Email:** privacy@truffle.io  
**Address:** [COMPANY ADDRESS]  
**DPO:** [DATA PROTECTION OFFICER NAME]  

---

## 11. Data Processing Agreement (Enterprise)

For enterprise customers, we provide a Data Processing Agreement (DPA) that:
- Defines our role as data processor
- Specifies security measures (Article 32)
- Outlines subprocessor governance
- Details breach notification procedures

Contact us at enterprise@truffle.io for the full DPA.

---

## 12. Technical Summary

### 12.1 Security Measures

| Measure | Implementation |
|---------|----------------|
| Encryption | AES-256-GCM |
| Key Exchange | X3DH (Signal Protocol) |
| Post-Quantum | Kyber-768 hybrid |
| Authentication | Ed25519 |
| Integrity | HMAC-SHA256 |

### 12.2 Certifications

- SOC 2 Type II (in progress)
- ISO 27001 (planned)
- GDPR compliant

---

*This Privacy Policy is effective as of [DATE].*
