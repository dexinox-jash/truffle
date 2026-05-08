# Ruflo AI Platform - Complete API Reference

> **Version:** 14.0  
> **Last Updated:** April 10, 2026  
> **Classification:** Public API Documentation

---

## Table of Contents

1. [Authentication](#authentication)
2. [Core Services](#core-services)
3. [AI Services](#ai-services)
4. [Real-time Services](#real-time-services)
5. [Advanced Services](#advanced-services)
6. [WebSocket Events](#websocket-events)
7. [Error Codes](#error-codes)
8. [Rate Limits](#rate-limits)

---

## Authentication

All API requests require authentication via JWT Bearer tokens obtained from the `ruflo-auth` service.

### POST /auth/login
Authenticate and obtain access token.

**Request:**
```json
{
  "email": "user@example.com",
  "password": "secure_password",
  "device_id": "uuid-device-identifier"
}
```

**Response:**
```json
{
  "access_token": "eyJhbGciOiJSUzI1NiIs...",
  "refresh_token": "eyJhbGciOiJSUzI1NiIs...",
  "expires_in": 3600,
  "token_type": "Bearer"
}
```

### POST /auth/refresh
Refresh an expiring access token.

**Request:**
```json
{
  "refresh_token": "eyJhbGciOiJSUzI1NiIs..."
}
```

**Headers:**
```
Authorization: Bearer {access_token}
X-API-Version: 2024-01
```

---

## Core Services

### ruflo-audio Service

#### POST /audio/upload
Upload audio file for processing.

**Request:**
```bash
curl -X POST https://api.ruflo.io/v1/audio/upload \
  -H "Authorization: Bearer $TOKEN" \
  -H "Content-Type: multipart/form-data" \
  -F "file=@meeting_recording.wav" \
  -F "meeting_id=uuid-meeting-id" \
  -F "language=en-US"
```

**Response:**
```json
{
  "audio_id": "uuid-audio-id",
  "status": "processing",
  "duration_seconds": 3600,
  "estimated_completion": "2026-04-10T15:30:00Z",
  "webhook_url": "https://api.ruflo.io/v1/webhooks/audio/uuid-audio-id"
}
```

#### GET /audio/{audio_id}/status
Check processing status.

**Response:**
```json
{
  "audio_id": "uuid-audio-id",
  "status": "completed",
  "progress": 100,
  "transcript_id": "uuid-transcript-id",
  "processing_time_ms": 45000
}
```

---

### ruflo-asr Service

#### POST /asr/transcribe
Submit audio for transcription.

**Request:**
```json
{
  "audio_url": "https://storage.ruflo.io/audio/file.wav",
  "model": "whisper-large-v3",
  "language": "auto",
  "speaker_diarization": true,
  "punctuation": true,
  "timestamps": "word"
}
```

**Response:**
```json
{
  "transcript_id": "uuid-transcript-id",
  "status": "processing",
  "estimated_duration": "2m"
}
```

#### GET /asr/transcript/{transcript_id}
Retrieve completed transcription.

**Response:**
```json
{
  "transcript_id": "uuid-transcript-id",
  "status": "completed",
  "language": "en",
  "duration_seconds": 3600,
  "segments": [
    {
      "id": 1,
      "speaker": "SPEAKER_01",
      "start_time": 0.0,
      "end_time": 5.3,
      "text": "Welcome everyone to our quarterly planning meeting.",
      "confidence": 0.98,
      "words": [
        {"word": "Welcome", "start": 0.0, "end": 0.5, "confidence": 0.99},
        {"word": "everyone", "start": 0.5, "end": 0.9, "confidence": 0.98}
      ]
    }
  ],
  "speakers": [
    {
      "id": "SPEAKER_01",
      "name": "Unknown",
      "confidence": 0.95,
      "speech_duration": 1200
    }
  ]
}
```

---

### ruflo-storage Service

#### POST /storage/buckets
Create a new storage bucket.

**Request:**
```json
{
  "name": "meeting-recordings-q2-2026",
  "region": "us-east-1",
  "encryption": "AES-256-GCM",
  "versioning": true,
  "retention_days": 2555
}
```

**Response:**
```json
{
  "bucket_id": "uuid-bucket-id",
  "name": "meeting-recordings-q2-2026",
  "created_at": "2026-04-10T12:00:00Z",
  "endpoint": "https://s3.ruflo.io/meeting-recordings-q2-2026"
}
```

#### PUT /storage/objects/{bucket_id}
Upload object to bucket.

**Request:**
```bash
curl -X PUT https://api.ruflo.io/v1/storage/objects/uuid-bucket-id \
  -H "Authorization: Bearer $TOKEN" \
  -H "Content-Type: application/octet-stream" \
  -H "X-Encryption-Key: {encrypted_key}" \
  --data-binary @file.dat
```

---

### ruflo-search Service

#### POST /search
Full-text and semantic search across meetings.

**Request:**
```json
{
  "query": "quarterly revenue projections",
  "query_type": "hybrid",
  "filters": {
    "date_range": {
      "from": "2026-01-01",
      "to": "2026-04-10"
    },
    "participants": ["user@example.com"],
    "meeting_types": ["planning", "review"]
  },
  "semantic_weight": 0.7,
  "limit": 20,
  "offset": 0
}
```

**Response:**
```json
{
  "total": 47,
  "results": [
    {
      "score": 0.94,
      "type": "transcript_segment",
      "meeting_id": "uuid-meeting-id",
      "meeting_title": "Q1 2026 Revenue Review",
      "timestamp": "2026-03-15T14:30:00Z",
      "segment": {
        "speaker": "John Smith",
        "text": "Our quarterly revenue projections show a 15% increase...",
        "start_time": 420,
        "confidence": 0.98
      },
      "semantic_similarity": 0.91,
      "text_match_score": 0.97
    }
  ],
  "facets": {
    "dates": {
      "2026-03": 12,
      "2026-02": 18,
      "2026-01": 17
    },
    "meeting_types": {
      "planning": 25,
      "review": 15,
      "standup": 7
    }
  }
}
```

#### POST /search/semantic
Pure semantic search using vector similarity.

**Request:**
```json
{
  "query": "discussions about budget constraints and cost cutting",
  "embedding_model": "all-MiniLM-L6-v2",
  "threshold": 0.75,
  "top_k": 10
}
```

---

### ruflo-analysis Service

#### POST /analysis/sentiment
Analyze sentiment of transcript or text.

**Request:**
```json
{
  "transcript_id": "uuid-transcript-id",
  "granularity": "segment",
  "model": "roberta-sentiment-v2"
}
```

**Response:**
```json
{
  "overall_sentiment": {
    "label": "positive",
    "score": 0.72,
    "confidence": 0.89
  },
  "timeline": [
    {
      "timestamp": 0,
      "sentiment": "neutral",
      "score": 0.5
    },
    {
      "timestamp": 600,
      "sentiment": "positive",
      "score": 0.78
    }
  ],
  "by_speaker": {
    "SPEAKER_01": {
      "dominant_sentiment": "positive",
      "score": 0.81
    }
  }
}
```

#### POST /analysis/entities
Extract named entities from transcript.

**Response:**
```json
{
  "entities": [
    {
      "text": "Acme Corporation",
      "type": "ORGANIZATION",
      "start_char": 45,
      "end_char": 61,
      "confidence": 0.96
    },
    {
      "text": "$5 million",
      "type": "MONEY",
      "start_char": 120,
      "end_char": 130,
      "normalized": {
        "amount": 5000000,
        "currency": "USD"
      }
    }
  ],
  "relationships": [
    {
      "subject": "Acme Corporation",
      "predicate": "acquired",
      "object": "TechStart Inc",
      "confidence": 0.88
    }
  ]
}
```

#### POST /analysis/topics
Extract key topics and themes.

**Response:**
```json
{
  "topics": [
    {
      "id": "topic-1",
      "keywords": ["revenue", "growth", "projections", "Q2"],
      "coherence_score": 0.87,
      "segment_ids": [1, 5, 12, 23]
    }
  ],
  "summary": "Primary discussion focused on Q2 revenue projections and growth strategies"
}
```

---

## AI Services

### ruflo-ai-ml Service

#### POST /ai/summarize
Generate intelligent meeting summary.

**Request:**
```json
{
  "transcript_id": "uuid-transcript-id",
  "summary_type": "comprehensive",
  "max_length": 500,
  "include_action_items": true,
  "include_decisions": true,
  "tone": "professional"
}
```

**Response:**
```json
{
  "summary_id": "uuid-summary-id",
  "summary": "Q1 2026 review meeting covered revenue growth of 15%...",
  "key_points": [
    "Revenue increased 15% quarter-over-quarter",
    "Three new enterprise clients signed",
    "Budget approved for hiring 10 engineers"
  ],
  "action_items": [
    {
      "task": "Prepare Q2 hiring plan",
      "assignee": "Sarah Johnson",
      "due_date": "2026-04-20",
      "priority": "high"
    }
  ],
  "decisions": [
    {
      "decision": "Approve $2M marketing budget",
      "decided_by": "Executive team",
      "context": "For Q2 product launch campaign"
    }
  ]
}
```

#### POST /ai/generate
Generate content from meeting context.

**Request:**
```json
{
  "transcript_id": "uuid-transcript-id",
  "generation_type": "follow_up_email",
  "recipient_context": "team_members",
  "tone": "professional",
  "include_attachments": ["summary", "action_items"]
}
```

**Response:**
```json
{
  "generated_content": "Subject: Action Items from Q1 Review Meeting\n\nTeam,\n\nThank you for a productive Q1 review...",
  "metadata": {
    "tokens_used": 450,
    "model": "gpt-4-turbo",
    "generation_time_ms": 1200
  }
}
```

---

### ruflo-predict Service

#### POST /predict/outcome
Predict meeting outcome before it occurs.

**Request:**
```json
{
  "meeting_context": {
    "title": "Q2 Budget Approval",
    "participants": ["user1@example.com", "user2@example.com"],
    "historical_success_rate": 0.72,
    "agenda_items": 5,
    "duration_minutes": 60,
    "time_of_day": "morning"
  },
  "prediction_type": "success_probability"
}
```

**Response:**
```json
{
  "prediction_id": "uuid-prediction-id",
  "outcomes": {
    "success_probability": 0.84,
    "decision_likelihood": 0.91,
    "conflict_risk": 0.12,
    "engagement_forecast": 0.88
  },
  "recommendations": [
    "Schedule for morning hours for higher engagement",
    "Send pre-read materials 24h in advance",
    "Limit to 5 agenda items for optimal outcomes"
  ],
  "confidence": 0.87
}
```

#### GET /predict/conflict/{meeting_id}
Real-time conflict detection.

**Response:**
```json
{
  "alert_level": "warning",
  "confidence": 0.76,
  "indicators": [
    {
      "type": "sentiment_shift",
      "severity": "medium",
      "timestamp": 1800,
      "description": "Sudden negative sentiment detected"
    }
  ],
  "suggested_intervention": "Consider taking a 5-minute break",
  "estimated_time_to_escalation": 240
}
```

---

### ruflo-voice Service

#### POST /voice/enroll
Enroll speaker voice for identification.

**Request:**
```bash
curl -X POST https://api.ruflo.io/v1/voice/enroll \
  -H "Authorization: Bearer $TOKEN" \
  -F "user_id=user@example.com" \
  -F "audio=@voice_sample.wav" \
  -F "samples_required=3"
```

**Response:**
```json
{
  "enrollment_id": "uuid-enrollment-id",
  "status": "active",
  "voiceprint_id": "uuid-voiceprint-id",
  "quality_score": 0.94,
  "samples_received": 3,
  "samples_needed": 0
}
```

#### POST /voice/identify
Identify speakers in audio.

**Request:**
```json
{
  "audio_id": "uuid-audio-id",
  "candidate_speakers": ["user1@example.com", "user2@example.com"],
  "min_confidence": 0.85
}
```

**Response:**
```json
{
  "identification_id": "uuid-id-id",
  "speakers": [
    {
      "segment_id": 1,
      "speaker_id": "user1@example.com",
      "confidence": 0.96,
      "start_time": 0,
      "end_time": 125
    }
  ],
  "unidentified_segments": [
    {
      "segment_id": 3,
      "start_time": 450,
      "end_time": 520,
      "reason": "insufficient_audio"
    }
  ]
}
```

---

## Real-time Services

### ruflo-realtime Service

#### WebSocket Connection
Connect to real-time transcription stream.

**Connection URL:**
```
wss://realtime.ruflo.io/v1/stream?token={jwt_token}&meeting_id={uuid}
```

**Send Audio:**
```json
{
  "type": "audio",
  "data": "base64_encoded_pcm_audio",
  "timestamp": 1712750400000
}
```

**Receive Transcript:**
```json
{
  "type": "transcript",
  "is_final": false,
  "text": "This is a partial transcript",
  "speaker": "SPEAKER_01",
  "confidence": 0.92,
  "timestamp": 1712750401000
}
```

**Final Transcript:**
```json
{
  "type": "transcript",
  "is_final": true,
  "text": "This is the final corrected transcript",
  "speaker": "SPEAKER_01",
  "words": [
    {"word": "This", "start": 0.0, "end": 0.2},
    {"word": "is", "start": 0.2, "end": 0.3}
  ],
  "timestamp": 1712750402000
}
```

---

### ruflo-collab Service

#### POST /collab/sessions
Create collaborative editing session.

**Request:**
```json
{
  "meeting_id": "uuid-meeting-id",
  "document_type": "meeting_notes",
  "participants": ["user1@example.com", "user2@example.com"],
  "permissions": {
    "user1@example.com": "write",
    "user2@example.com": "write"
  }
}
```

**Response:**
```json
{
  "session_id": "uuid-session-id",
  "websocket_url": "wss://collab.ruflo.io/v1/session/uuid-session-id",
  "yjs_document_id": "doc-uuid",
  "created_at": "2026-04-10T12:00:00Z"
}
```

---

## Advanced Services

### ruflo-facilitator Service

#### POST /facilitator/autonomous
Start autonomous AI meeting facilitation.

**Request:**
```json
{
  "meeting_id": "uuid-meeting-id",
  "facilitation_mode": "collaborative",
  "objectives": [
    "Reach consensus on Q2 priorities",
    "Assign action items with deadlines"
  ],
  "intervention_triggers": {
    "sentiment_threshold": 0.3,
    "silence_timeout_seconds": 30,
    "off_topic_alert": true
  }
}
```

**Response:**
```json
{
  "facilitation_id": "uuid-facilitation-id",
  "status": "active",
  "ai_facilitator": {
    "persona": "professional_moderator",
    "voice_enabled": true,
    "intervention_style": "gentle"
  },
  "interventions": [
    {
      "type": "agenda_reminder",
      "triggered_at": 600,
      "message": "We've been discussing this topic for 10 minutes. Shall we move to the next agenda item?"
    }
  ]
}
```

---

### ruflo-cultural Service

#### GET /cultural/contexts
Get available cultural contexts.

**Response:**
```json
{
  "contexts": [
    {
      "code": "en-US",
      "name": "American English",
      "communication_style": "direct",
      "formality_level": "medium",
      "decision_making": "individual"
    },
    {
      "code": "ja-JP",
      "name": "Japanese",
      "communication_style": "indirect",
      "formality_level": "high",
      "decision_making": "consensus"
    }
  ]
}
```

#### POST /cultural/adapt
Adapt meeting insights for cultural context.

**Request:**
```json
{
  "meeting_id": "uuid-meeting-id",
  "target_culture": "ja-JP",
  "content_type": "summary",
  "adaptation_level": "full"
}
```

**Response:**
```json
{
  "adapted_content": "（敬語を使用した要約）...",
  "adaptations_applied": [
    "Converted to keigo (honorific language)",
    "Emphasized group consensus over individual decisions",
    "Added appropriate hierarchical acknowledgments"
  ]
}
```

---

### ruflo-quantum Service

#### POST /quantum/optimize
Use quantum algorithms for meeting optimization.

**Request:**
```json
{
  "optimization_type": "schedule",
  "constraints": {
    "participants": ["user1", "user2", "user3"],
    "duration_minutes": 60,
    "priority": "high",
    "required_resources": ["conference_room_a", "projector"]
  },
  "objective": "minimize_conflicts"
}
```

**Response:**
```json
{
  "optimization_id": "uuid-optimization-id",
  "algorithm": "QAOA",
  "qubits_used": 16,
  "result": {
    "optimal_time": "2026-04-15T10:00:00Z",
    "confidence": 0.91,
    "alternative_slots": [
      "2026-04-15T14:00:00Z",
      "2026-04-16T09:00:00Z"
    ]
  },
  "execution_time_ms": 450
}
```

---

### ruflo-temporal Service

#### POST /temporal/predict
Predict future outcomes from current meeting.

**Request:**
```json
{
  "meeting_id": "uuid-meeting-id",
  "prediction_horizon": "7_days",
  "outcome_types": ["decision_follow_through", "action_completion", "satisfaction"],
  "confidence_threshold": 0.7
}
```

**Response:**
```json
{
  "prediction_id": "uuid-prediction-id",
  "temporal_analysis": {
    "current_trajectory": "positive",
    "momentum_score": 0.78
  },
  "predicted_outcomes": [
    {
      "outcome_type": "decision_follow_through",
      "probability": 0.85,
      "confidence": 0.82,
      "timeline": "3_days"
    }
  ],
  "intervention_recommendations": [
    "Send reminder 24h before action deadlines",
    "Schedule follow-up for complex decisions"
  ]
}
```

---

## WebSocket Events

### Event Types

| Event | Description | Payload |
|-------|-------------|---------|
| `transcript.partial` | Interim transcription | `{text, speaker, confidence}` |
| `transcript.final` | Final transcription | `{text, speaker, words, timestamp}` |
| `sentiment.shift` | Sentiment change detected | `{previous, current, timestamp}` |
| `conflict.warning` | Potential conflict alert | `{level, confidence, indicators}` |
| `action_item.detected` | Action item identified | `{task, assignee, confidence}` |
| `participant.joined` | New participant | `{user_id, timestamp}` |
| `participant.left` | Participant left | `{user_id, duration}` |
| `ai.intervention` | AI facilitator message | `{message, type, urgency}` |
| `sync.completed` | Sync operation done | `{device_id, timestamp}` |

### Connection Management

```javascript
const ws = new WebSocket('wss://api.ruflo.io/v1/events');

ws.onopen = () => {
  ws.send(JSON.stringify({
    type: 'auth',
    token: 'jwt_token'
  }));
};

ws.onmessage = (event) => {
  const data = JSON.parse(event.data);
  switch(data.type) {
    case 'transcript.final':
      handleFinalTranscript(data);
      break;
    case 'conflict.warning':
      handleConflictWarning(data);
      break;
  }
};
```

---

## Error Codes

### HTTP Status Codes

| Code | Meaning | Description |
|------|---------|-------------|
| 200 | OK | Request successful |
| 201 | Created | Resource created |
| 400 | Bad Request | Invalid request parameters |
| 401 | Unauthorized | Authentication required |
| 403 | Forbidden | Insufficient permissions |
| 404 | Not Found | Resource not found |
| 409 | Conflict | Resource conflict |
| 422 | Unprocessable | Validation error |
| 429 | Rate Limited | Too many requests |
| 500 | Server Error | Internal server error |
| 503 | Unavailable | Service temporarily unavailable |

### Error Response Format

```json
{
  "error": {
    "code": "TRANSCRIPT_NOT_FOUND",
    "message": "The requested transcript does not exist",
    "details": {
      "transcript_id": "uuid-transcript-id"
    },
    "request_id": "req-uuid",
    "timestamp": "2026-04-10T12:00:00Z"
  }
}
```

### Common Error Codes

| Code | Description | Resolution |
|------|-------------|------------|
| `AUTH_TOKEN_EXPIRED` | JWT token expired | Refresh token |
| `RATE_LIMIT_EXCEEDED` | API rate limit hit | Wait and retry |
| `AUDIO_FORMAT_INVALID` | Unsupported audio format | Convert to WAV/MP3 |
| `MODEL_OVERLOADED` | AI model at capacity | Retry with backoff |
| `ENCRYPTION_KEY_INVALID` | Wrong encryption key | Verify key |

---

## Rate Limits

### Limits by Tier

| Tier | Requests/Min | Concurrent Streams | Storage |
|------|--------------|-------------------|---------|
| Free | 60 | 1 | 5GB |
| Pro | 600 | 5 | 100GB |
| Enterprise | 6000 | Unlimited | Unlimited |

### Rate Limit Headers

```
X-RateLimit-Limit: 600
X-RateLimit-Remaining: 599
X-RateLimit-Reset: 1712750460
```

### WebSocket Limits

- Max connections per token: 5
- Message size limit: 64KB
- Idle timeout: 5 minutes
- Reconnection window: 30 seconds

---

## SDKs

### TypeScript/JavaScript

```bash
npm install @ruflo/sdk
```

```typescript
import { RufloClient } from '@ruflo/sdk';

const client = new RufloClient({
  apiKey: 'your-api-key',
  environment: 'production'
});

const transcript = await client.transcribe({
  audioUrl: 'https://example.com/audio.wav',
  speakerDiarization: true
});
```

### Python

```bash
pip install ruflo-sdk
```

```python
from ruflo import RufloClient

client = RufloClient(api_key="your-api-key")

result = client.transcribe(
    audio_url="https://example.com/audio.wav",
    speaker_diarization=True
)
```

---

**Document Version:** 14.0  
**API Version:** 2024-01  
**Support:** api-support@ruflo.io

---
