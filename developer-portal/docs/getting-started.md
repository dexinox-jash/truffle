# Getting Started with Ruflo API

Welcome to the Ruflo AI Meeting Notes Platform Developer Portal. This guide will help you integrate with our powerful meeting intelligence APIs.

## Quick Start

### 1. Get Your API Keys

Sign up at [developer.ruflo.io](https://developer.ruflo.io) and create an application to get your API credentials.

```bash
export RUFLO_API_KEY="your-api-key"
export RUFLO_API_SECRET="your-api-secret"
```

### 2. Make Your First Request

```bash
curl -X GET https://api.ruflo.io/v1/meetings \
  -H "Authorization: Bearer $RUFLO_API_KEY"
```

### 3. Install an SDK

**TypeScript/JavaScript:**
```bash
npm install @ruflo/sdk
```

**Python:**
```bash
pip install ruflo-sdk
```

## Authentication

Ruflo uses OAuth 2.0 for authentication. Include your access token in the Authorization header:

```
Authorization: Bearer {access_token}
```

### Getting an Access Token

```bash
curl -X POST https://api.ruflo.io/v1/auth/token \
  -H "Content-Type: application/json" \
  -d '{
    "grant_type": "client_credentials",
    "client_id": "your-client-id",
    "client_secret": "your-client-secret"
  }'
```

## Base URLs

| Environment | URL |
|-------------|-----|
| Production | `https://api.ruflo.io` |
| Sandbox | `https://sandbox-api.ruflo.io` |

## API Versioning

The current API version is `v1`. Include it in the URL path:

```
https://api.ruflo.io/v1/{resource}
```

## Rate Limits

- **Free Tier**: 100 requests/minute
- **Pro Tier**: 1,000 requests/minute
- **Enterprise**: Custom limits

Rate limit headers are included in all responses:

```
X-RateLimit-Limit: 1000
X-RateLimit-Remaining: 999
X-RateLimit-Reset: 1640995200
```

## Webhooks

Subscribe to real-time events using webhooks:

```bash
curl -X POST https://api.ruflo.io/v1/webhooks \
  -H "Authorization: Bearer $TOKEN" \
  -H "Content-Type: application/json" \
  -d '{
    "url": "https://your-app.com/webhooks/ruflo",
    "events": ["meeting.created", "meeting.ended", "transcription.completed"]
  }'
```

## Next Steps

- [API Reference](./api-reference.md)
- [SDK Documentation](./sdks.md)
- [Webhook Events](./webhooks.md)
- [Error Codes](./errors.md)
