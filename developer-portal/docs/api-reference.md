# Ruflo API Reference

Complete reference for all Ruflo API endpoints.

## Meetings API

### List Meetings

```http
GET /v1/meetings
```

**Parameters:**
- `org_id` (string, required) - Organization ID
- `status` (string, optional) - Filter by status: `scheduled`, `live`, `completed`
- `from` (ISO 8601, optional) - Start date filter
- `to` (ISO 8601, optional) - End date filter
- `limit` (integer, optional) - Max results (default: 20, max: 100)
- `offset` (integer, optional) - Pagination offset

**Response:**
```json
{
  "meetings": [
    {
      "id": "mtg_123",
      "title": "Weekly Standup",
      "status": "completed",
      "started_at": "2024-01-15T09:00:00Z",
      "ended_at": "2024-01-15T09:30:00Z",
      "duration_seconds": 1800,
      "participants": [
        {
          "id": "usr_456",
          "name": "Alice Smith",
          "email": "alice@example.com"
        }
      ]
    }
  ],
  "total": 150,
  "limit": 20,
  "offset": 0
}
```

### Get Meeting

```http
GET /v1/meetings/{meeting_id}
```

### Create Meeting

```http
POST /v1/meetings
```

**Request Body:**
```json
{
  "title": "Product Planning",
  "scheduled_at": "2024-01-20T14:00:00Z",
  "duration_minutes": 60,
  "participants": [
    {"email": "bob@example.com"},
    {"email": "carol@example.com"}
  ],
  "settings": {
    "record": true,
    "transcribe": true,
    "language": "en"
  }
}
```

## Transcriptions API

### Get Transcription

```http
GET /v1/meetings/{meeting_id}/transcription
```

**Response:**
```json
{
  "meeting_id": "mtg_123",
  "language": "en",
  "segments": [
    {
      "id": "seg_001",
      "speaker": "Alice Smith",
      "text": "Let's discuss the roadmap for Q2.",
      "start_time": 0.0,
      "end_time": 3.5,
      "confidence": 0.98
    }
  ],
  "full_text": "Let's discuss the roadmap for Q2..."
}
```

### Search Transcriptions

```http
POST /v1/transcriptions/search
```

**Request Body:**
```json
{
  "query": "roadmap Q2",
  "org_id": "org_123",
  "filters": {
    "from_date": "2024-01-01",
    "to_date": "2024-01-31"
  }
}
```

## AI Analysis API

### Get Meeting Analysis

```http
GET /v1/meetings/{meeting_id}/analysis
```

**Response:**
```json
{
  "meeting_id": "mtg_123",
  "sentiment": {
    "overall": 0.75,
    "timeline": [...]
  },
  "action_items": [
    {
      "id": "ai_001",
      "text": "Prepare Q2 roadmap presentation",
      "assignee": "Bob Jones",
      "priority": "high"
    }
  ],
  "decisions": [...],
  "topics": [...],
  "speakers": [...]
}
```

### Generate Summary

```http
POST /v1/meetings/{meeting_id}/summarize
```

**Request Body:**
```json
{
  "style": "detailed",  // brief, detailed, bullet_points
  "max_length": 500
}
```

## Predictions API

### Predict Meeting Outcome

```http
POST /v1/predict/outcome
```

**Request Body:**
```json
{
  "meeting_type": "planning",
  "scheduled_duration": 60,
  "participants": ["usr_1", "usr_2", "usr_3"],
  "agenda_items": ["Review Q1", "Plan Q2"]
}
```

**Response:**
```json
{
  "success_probability": 0.82,
  "confidence": 0.75,
  "factors": [
    {
      "name": "Has agenda",
      "impact": 0.15,
      "direction": "positive"
    }
  ],
  "recommendations": [...]
}
```

### Optimize Schedule

```http
POST /v1/optimize/schedule
```

## Voice Biometrics API

### Enroll Voice

```http
POST /v1/voice/enroll
```

**Request Body:**
```json
{
  "user_id": "usr_123",
  "voice_samples": [
    {
      "audio_data": "base64-encoded-audio",
      "format": "wav",
      "duration_seconds": 15.5
    }
  ]
}
```

### Verify Voice

```http
POST /v1/voice/verify
```

### Identify Speaker

```http
POST /v1/voice/identify
```

## Translation API

### Translate Text

```http
POST /v1/translate
```

**Request Body:**
```json
{
  "text": "Hello, how are you?",
  "source_language": "en",
  "target_language": "es"
}
```

**Response:**
```json
{
  "translated_text": "Hola, ¿cómo estás?",
  "source_language": "en",
  "target_language": "es",
  "confidence": 0.98
}
```

### Detect Language

```http
POST /v1/detect-language
```

## Workflows API

### Create Workflow

```http
POST /v1/workflows
```

**Request Body:**
```json
{
  "name": "Post-meeting Slack Summary",
  "trigger": {
    "type": "meeting.ended",
    "config": {}
  },
  "actions": [
    {
      "type": "slack.send_message",
      "config": {
        "channel": "#meetings",
        "message": "{{meeting.summary}}"
      }
    }
  ]
}
```

### List Workflows

```http
GET /v1/workflows
```

### Execute Workflow

```http
POST /v1/workflows/{workflow_id}/execute
```

## Plugins API

### List Plugins

```http
GET /v1/plugins
```

### Publish Plugin

```http
POST /v1/plugins
```

### Execute Plugin

```http
POST /v1/plugins/{plugin_id}/execute
```

## Error Responses

All errors follow this format:

```json
{
  "error": true,
  "code": "MEETING_NOT_FOUND",
  "message": "Meeting with ID mtg_999 not found",
  "details": {
    "meeting_id": "mtg_999"
  }
}
```

### Common Error Codes

| Code | HTTP Status | Description |
|------|-------------|-------------|
| `UNAUTHORIZED` | 401 | Invalid or missing authentication |
| `FORBIDDEN` | 403 | Insufficient permissions |
| `NOT_FOUND` | 404 | Resource not found |
| `RATE_LIMITED` | 429 | Too many requests |
| `INTERNAL_ERROR` | 500 | Server error |
