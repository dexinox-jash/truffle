# Project Truffle - Zero-Knowledge Relay Server

[![Zero-Access](https://img.shields.io/badge/Zero--Access-Verified-success)](https://docs.truffle.io/zero-access)
[![Cloudflare Workers](https://img.shields.io/badge/Cloudflare-Workers-f38020)](https://workers.cloudflare.com)
[![License](https://img.shields.io/badge/License-AGPL--3.0-blue.svg)](LICENSE)

> **CRITICAL**: This is ZERO-ACCESS infrastructure. We CANNOT decrypt user content. Store-and-forward encrypted blobs ONLY.

## Overview

The Truffle Relay Server is a zero-knowledge, edge-deployed infrastructure component that enables secure device synchronization while maintaining mathematical inability to access user data.

### Architecture

```
┌─────────────┐     ┌─────────────────┐     ┌─────────────┐
│  Device A   │◄───►│  Cloudflare     │◄───►│  Device B   │
│  (Client)   │     │  Workers + R2   │     │  (Client)   │
└─────────────┘     └─────────────────┘     └─────────────┘
       │                     │                     │
       ▼                     ▼                     ▼
  ┌─────────┐          ┌──────────┐          ┌─────────┐
  │ Encrypt │          │  Route   │          │ Decrypt │
  │  + Sign │          │  + Store │          │ + Verify│
  └─────────┘          └──────────┘          └─────────┘
```

## Features

- **Zero-Access Architecture**: Mathematical inability to decrypt content
- **Edge Deployment**: Global low-latency via Cloudflare Workers
- **WebSocket Support**: Real-time sync with store-and-forward
- **Rate Limiting**: 100/day (free), 10,000/day (paid)
- **30-Day Auto-Delete**: Automatic data lifecycle management
- **Device Authentication**: Clerk.com integration for device-only auth
- **Audit Logging**: Privacy-preserving access logs

## Quick Start

### Prerequisites

- Node.js 18+
- Cloudflare account
- Wrangler CLI: `npm install -g wrangler`

### Installation

```bash
# Clone the repository
git clone https://github.com/truffle/relay.git
cd truffle-relay

# Install dependencies
npm install

# Configure wrangler
wrangler login

# Set secrets
wrangler secret put JWT_SECRET
wrangler secret put CLERK_SECRET_KEY
```

### Development

```bash
# Start local development server
npm run dev

# Run tests
npm test

# Type check
npm run typecheck

# Lint
npm run lint
```

### Deployment

```bash
# Deploy to staging
./deploy.sh staging

# Deploy to production
./deploy.sh production
```

## API Endpoints

### Health Check
```http
GET /health
```

### Device Authentication
```http
POST /auth/device
Content-Type: application/json

{
  "device_fingerprint": "sha256_hash",
  "public_key": "base64_ed25519_public_key",
  "device_info": {
    "platform": "macos",
    "version": "1.0.0"
  }
}
```

### Store Blob
```http
POST /blob
Authorization: Bearer <token>
X-Device-ID: <device_id>
Content-Type: application/octet-stream

<encrypted_blob_data>
```

### Retrieve Blob
```http
GET /blob/<blob_id>
Authorization: Bearer <token>
X-Device-ID: <device_id>
```

### WebSocket Connection
```http
GET /ws
Authorization: Bearer <token>
Upgrade: websocket
```

## Configuration

### Environment Variables

| Variable | Description | Default |
|----------|-------------|---------|
| `ENVIRONMENT` | Deployment environment | `development` |
| `PROTOCOL_VERSION` | Sync protocol version | `1` |
| `RATE_LIMIT_FREE` | Free tier daily limit | `100` |
| `RATE_LIMIT_PAID` | Paid tier daily limit | `10000` |
| `DATA_RETENTION_DAYS` | Blob retention period | `30` |
| `MAX_BLOB_SIZE_MB` | Maximum blob size | `10` |

### Secrets (via `wrangler secret put`)

| Secret | Description |
|--------|-------------|
| `JWT_SECRET` | JWT signing key |
| `CLERK_SECRET_KEY` | Clerk.com API key |

## Zero-Access Verification

The relay includes continuous zero-access verification:

```bash
curl https://relay.truffle.io/verify-zero-access
```

This endpoint verifies:
- No encryption keys are stored
- All blobs have high entropy (encrypted)
- No plaintext patterns detected
- Audit log integrity

## Rate Limits

| Tier | Daily Limit | Burst |
|------|-------------|-------|
| Free | 100 | 10/min |
| Paid | 10,000 | 100/min |
| Enterprise | 100,000 | 1000/min |

## Data Retention

- **Sync Blobs**: 30 days (auto-delete via lifecycle rules)
- **Audit Logs**: 90 days
- **Pending Messages**: 30 days

## Security

### Threat Model (STRIDE)

| Threat | Mitigation |
|--------|------------|
| Spoofing | Ed25519 device certificates |
| Tampering | AES-256-GCM + HMAC |
| Repudiation | Immutable audit logs |
| Information Disclosure | Zero-knowledge architecture |
| Denial of Service | Rate limiting + local-first |
| Elevation of Privilege | Principle of least privilege |

### Compliance

- SOC 2 Type II (CC6.1, CC6.6, CC6.7)
- GDPR Article 32
- ISO 27001 ready

## Infrastructure (Terraform)

```bash
cd terraform

# Initialize
terraform init

# Plan
terraform plan -var="environment=staging"

# Apply
terraform apply -var="environment=staging"
```

## Testing

```bash
# Unit tests
npm test

# Coverage
npm run test:coverage

# Watch mode
npm run test:watch
```

## Monitoring

- **Health**: `GET /health`
- **Rate Limit Status**: `GET /rate-limit`
- **Zero-Access Verification**: `GET /verify-zero-access`

## License

AGPL-3.0 - See [LICENSE](LICENSE) for details.

## Contributing

See [CONTRIBUTING.md](CONTRIBUTING.md) for guidelines.

## Support

- Documentation: https://docs.truffle.io
- Issues: https://github.com/truffle/relay/issues
- Security: security@truffle.io

---

**Project Truffle** - Your Visual Memory, Compiled. 🔒
