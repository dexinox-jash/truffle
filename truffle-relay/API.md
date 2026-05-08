# Truffle Relay API Documentation

## Overview

The Truffle Relay Server provides a zero-knowledge infrastructure for encrypted device synchronization. All content is client-side encrypted; the relay only stores and forwards encrypted blobs.

**Base URL**: `https://relay.truffle.io`

**Protocol Version**: `1`

## Authentication

All API requests (except health check) require authentication via Bearer token.

```http
Authorization: Bearer <access_token>
X-Device-ID: <device_id>
```

### Device Registration

Before making authenticated requests, devices must register to obtain an access token.

```http
POST /auth/device
Content-Type: application/json

{
  "device_fingerprint": "sha256_hash_of_device",
  "public_key": "base64_encoded_ed25519_public_key",
  "device_info": {
    "platform": "macos|windows|linux|ios|android",
    "version": "1.0.0",
    "model": "optional_model_info"
  },
  "pairing_token": "optional_pairing_token_for_secondary_devices"
}
```

**Response**:
```json
{
  "success": true,
  "device_id": "dev_abc123...",
  "access_token": "eyJhbGciOiJIUzI1NiIs...",
  "refresh_token": "eyJhbGciOiJIUzI1NiIs...",
  "expires_at": 1704067200
}
```

## Endpoints

### Health Check

Check relay server status.

```http
GET /health
```

**Response**:
```json
{
  "status": "healthy",
  "service": "truffle-relay",
  "version": "1.0.0",
  "protocol_version": "1",
  "environment": "production",
  "timestamp": 1704067200000,
  "zero_access": true
}
```

### Zero-Access Verification

Verify that the relay maintains zero-access architecture.

```http
GET /verify-zero-access
```

**Response**:
```json
{
  "verified": true,
  "timestamp": "2024-01-01T00:00:00.000Z",
  "checks": {
    "no_encryption_keys_stored": true,
    "no_plaintext_blobs": true,
    "high_entropy_data": true,
    "no_key_derivation_data": true,
    "audit_log_integrity": true
  },
  "details": {
    "total_blobs_checked": 1000,
    "high_entropy_blobs": 1000,
    "suspicious_blobs": 0,
    "average_entropy": 7.95
  },
  "warnings": [],
  "errors": []
}
```

### Rate Limit Status

Check current rate limit status.

```http
GET /rate-limit
Authorization: Bearer <token>
X-Device-ID: <device_id>
```

**Response**:
```json
{
  "device_id": "dev_abc123...",
  "tier": "free",
  "daily_limit": 100,
  "used_today": 42,
  "remaining_today": 58,
  "resets_at": 1704153600000,
  "window_start": 1704067200000
}
```

### Store Blob

Store an encrypted blob.

```http
POST /blob
Authorization: Bearer <token>
X-Device-ID: <device_id>
Content-Type: application/octet-stream
Content-Length: <size>

<encrypted_blob_data>
```

**Constraints**:
- Maximum size: 10MB
- Must be encrypted (high entropy)
- Plaintext will be rejected

**Response**:
```json
{
  "success": true,
  "blob_id": "blob_abc123...",
  "metadata": {
    "blob_id": "blob_abc123...",
    "device_id": "dev_abc123...",
    "size_bytes": 1024,
    "content_hash": "sha256_hash",
    "uploaded_at": 1704067200000,
    "expires_at": 1706659200000,
    "protocol_version": "1"
  }
}
```

### Retrieve Blob

Retrieve an encrypted blob.

```http
GET /blob/<blob_id>
Authorization: Bearer <token>
X-Device-ID: <device_id>
```

**Response**:
```
HTTP/1.1 200 OK
Content-Type: application/octet-stream
X-Blob-ID: blob_abc123...
X-Device-ID: dev_abc123...
Cache-Control: no-store, no-cache, must-revalidate

<encrypted_blob_data>
```

### Delete Blob

Delete a blob.

```http
DELETE /blob/<blob_id>
Authorization: Bearer <token>
X-Device-ID: <device_id>
```

**Response**:
```json
{
  "success": true
}
```

## WebSocket API

### Connection

```javascript
const ws = new WebSocket('wss://relay.truffle.io/ws', [], {
  headers: {
    'Authorization': 'Bearer <token>',
    'X-Device-ID': '<device_id>'
  }
});
```

### Message Types

#### Ping/Pong

```json
// Client -> Server
{ "type": "ping" }

// Server -> Client
{ "type": "pong", "timestamp": 1704067200000 }
```

#### Sync Message

```json
// Client -> Server (broadcast to paired devices)
{
  "type": "sync",
  "payload": "base64_or_binary_encrypted_data",
  "message_id": "uuid"
}

// Server -> Client (acknowledgment)
{
  "type": "ack",
  "message_id": "uuid",
  "status": "delivered|stored|broadcast"
}
```

#### Targeted Sync

```json
// Client -> Server (to specific device)
{
  "type": "sync",
  "target_device": "dev_target123...",
  "payload": "base64_or_binary_encrypted_data",
  "message_id": "uuid"
}
```

#### Pairing Message

```json
// Client -> Server (for X3DH key exchange)
{
  "type": "pairing",
  "target_device": "dev_primary123...",
  "payload": "base64_encoded_key_material",
  "message_id": "uuid"
}
```

## Rate Limits

| Operation | Free Tier | Paid Tier | Enterprise |
|-----------|-----------|-----------|------------|
| Blob Store | 100/day | 10,000/day | 100,000/day |
| Blob Retrieve | 100/day | 10,000/day | 100,000/day |
| Sync Messages | 100/day | 10,000/day | 100,000/day |
| WebSocket Messages | 100/day | 10,000/day | 100,000/day |

Rate limit headers are included in all responses:

```http
X-RateLimit-Limit: 100
X-RateLimit-Remaining: 58
X-RateLimit-Reset: 1704153600
X-RateLimit-Window: 86400
```

## Error Responses

### 400 Bad Request

```json
{
  "error": "Invalid request",
  "details": "Description of what went wrong"
}
```

### 401 Unauthorized

```json
{
  "error": "Invalid or expired token"
}
```

### 403 Forbidden

```json
{
  "error": "Access denied"
}
```

### 404 Not Found

```json
{
  "error": "Blob not found"
}
```

### 429 Too Many Requests

```json
{
  "error": "Rate limit exceeded",
  "limit": 100,
  "remaining": 0,
  "reset_at": 1704153600000
}
```

### 500 Internal Server Error

```json
{
  "error": "Internal Server Error",
  "request_id": "uuid-for-debugging"
}
```

## Data Retention

- **Sync Blobs**: 30 days (auto-deleted)
- **Pending Messages**: 30 days
- **Audit Logs**: 90 days
- **Device Sessions**: 90 days of inactivity

## Security Headers

All responses include security headers:

```http
X-Content-Type-Options: nosniff
X-Frame-Options: DENY
X-XSS-Protection: 1; mode=block
Referrer-Policy: strict-origin-when-cross-origin
Strict-Transport-Security: max-age=31536000; includeSubDomains; preload
Content-Security-Policy: default-src 'none'; frame-ancestors 'none';
X-Truffle-Relay: zero-access
```

## Client SDK Example

```typescript
import { TruffleRelayClient } from '@truffle/relay-client';

const client = new TruffleRelayClient({
  baseUrl: 'https://relay.truffle.io',
  deviceFingerprint: 'your-device-fingerprint',
  publicKey: 'your-ed25519-public-key',
});

// Register device
const auth = await client.authenticate({
  platform: 'macos',
  version: '1.0.0',
});

// Store encrypted blob
const blobId = await client.storeBlob(encryptedData);

// Retrieve encrypted blob
const data = await client.retrieveBlob(blobId);

// Connect WebSocket
const ws = await client.connectWebSocket();
ws.onSync = (encryptedUpdate) => {
  // Decrypt and apply update
};
```

## Changelog

### v1.0.0
- Initial release
- Zero-access architecture
- WebSocket support
- Rate limiting
- 30-day data retention
