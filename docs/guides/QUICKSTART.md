# Ruflo AI Platform - Quickstart Guide

> **Get up and running with Ruflo AI in 15 minutes**

---

## Prerequisites

- Docker Desktop or Kubernetes cluster
- Git
- 8GB RAM minimum (16GB recommended)
- API key from [ruflo.io](https://ruflo.io)

---

## Option 1: Docker Compose (Fastest)

### 1. Clone Repository

```bash
git clone https://github.com/ruflo/platform.git
cd platform
```

### 2. Configure Environment

```bash
cp .env.example .env
# Edit .env with your API keys
nano .env
```

Required variables:
```env
RUFO_API_KEY=your_api_key_here
RUFO_ENVIRONMENT=development
POSTGRES_PASSWORD=secure_password
REDIS_PASSWORD=secure_password
```

### 3. Start Services

```bash
docker-compose up -d

# Wait for services to start
./scripts/wait-for-healthy.sh

# Expected output:
# ✓ postgres: healthy
# ✓ redis: healthy  
# ✓ ruflo-gateway: healthy
# ✓ ruflo-asr: healthy
```

### 4. Verify Installation

```bash
# Check health
curl http://localhost:8080/health

# Expected: {"status":"healthy","version":"14.0"}

# Upload test audio
curl -X POST http://localhost:8080/v1/audio/upload \
  -H "Authorization: Bearer $RUFO_API_KEY" \
  -F "file=@tests/fixtures/sample_meeting.wav"

# Check status
curl http://localhost:8080/v1/audio/{audio_id}/status
```

---

## Option 2: Kubernetes (Production-like)

### 1. Prerequisites

```bash
# Install kubectl, helm, and k3d
brew install kubectl helm k3d

# Create local cluster
k3d cluster create ruflo \
  --servers 1 \
  --agents 3 \
  --port "8080:80@loadbalancer"
```

### 2. Deploy Platform

```bash
# Add Helm repo
helm repo add ruflo https://charts.ruflo.io
helm repo update

# Install with default values
helm install ruflo ruflo/platform \
  --set apiKey=your_api_key \
  --set environment=development

# Wait for deployment
kubectl wait --for=condition=ready pod -l app=ruflo-gateway --timeout=300s
```

### 3. Access Platform

```bash
# Port forward for local access
kubectl port-forward svc/ruflo-gateway 8080:80

# Or get LoadBalancer IP
kubectl get svc ruflo-gateway
```

---

## Your First Meeting

### Using cURL

```bash
# 1. Upload meeting recording
curl -X POST http://localhost:8080/v1/audio/upload \
  -H "Authorization: Bearer $RUFO_API_KEY" \
  -F "file=@quarterly_review.wav" \
  -F "language=en-US"

# Response: {"audio_id": "audio-123", "status": "processing"}

# 2. Check processing status
curl http://localhost:8080/v1/audio/audio-123/status \
  -H "Authorization: Bearer $RUFO_API_KEY"

# 3. Get transcript
curl http://localhost:8080/v1/asr/transcript/transcript-456 \
  -H "Authorization: Bearer $RUFO_API_KEY" | jq

# 4. Generate summary
curl -X POST http://localhost:8080/v1/ai/summarize \
  -H "Authorization: Bearer $RUFO_API_KEY" \
  -H "Content-Type: application/json" \
  -d '{"transcript_id": "transcript-456", "summary_type": "executive"}' | jq

# 5. Search across meetings
curl -X POST http://localhost:8080/v1/search \
  -H "Authorization: Bearer $RUFO_API_KEY" \
  -H "Content-Type: application/json" \
  -d '{
    "query": "quarterly revenue",
    "query_type": "hybrid",
    "limit": 5
  }' | jq
```

### Using Python SDK

```bash
pip install ruflo-sdk
```

```python
from ruflo import RufloClient

# Initialize client
client = RufloClient(
    api_key="your_api_key",
    base_url="http://localhost:8080"
)

# Upload and transcribe
result = client.transcribe(
    audio_path="quarterly_review.wav",
    speaker_diarization=True,
    language="en-US"
)

print(f"Transcript ID: {result.transcript_id}")

# Get full transcript
transcript = client.get_transcript(result.transcript_id)
for segment in transcript.segments:
    print(f"[{segment.speaker}] {segment.text}")

# Generate summary
summary = client.summarize(
    transcript_id=result.transcript_id,
    summary_type="executive"
)
print(f"\nSummary:\n{summary.summary}")

# Extract action items
for item in summary.action_items:
    print(f"- {item.task} (assigned to: {item.assignee})")

# Search across all meetings
results = client.search(
    query="revenue projections",
    filters={"date_range": {"from": "2026-01-01"}}
)

for result in results:
    print(f"{result.meeting_title}: {result.segment.text}")
```

### Using TypeScript SDK

```bash
npm install @ruflo/sdk
```

```typescript
import { RufloClient } from '@ruflo/sdk';

const client = new RufloClient({
  apiKey: 'your_api_key',
  baseUrl: 'http://localhost:8080'
});

async function processMeeting() {
  // Upload audio
  const upload = await client.audio.upload({
    file: './quarterly_review.wav',
    language: 'en-US'
  });
  
  console.log(`Uploaded: ${upload.audio_id}`);
  
  // Wait for transcription
  const transcript = await client.waitForTranscript(upload.transcript_id);
  
  // Print segments
  transcript.segments.forEach(seg => {
    console.log(`[${seg.speaker}] ${seg.text}`);
  });
  
  // Get AI summary
  const summary = await client.ai.summarize({
    transcriptId: transcript.id,
    summaryType: 'executive'
  });
  
  console.log('\nSummary:', summary.summary);
  
  // Real-time updates via WebSocket
  const ws = client.realtime.connect({
    meetingId: 'meeting-123'
  });
  
  ws.on('transcript.final', (data) => {
    console.log(`[${data.speaker}] ${data.text}`);
  });
}

processMeeting();
```

---

## Real-time Streaming

### WebSocket Example

```javascript
const ws = new WebSocket('ws://localhost:8080/v1/stream');

ws.onopen = () => {
  // Authenticate
  ws.send(JSON.stringify({
    type: 'auth',
    token: 'your_api_key'
  }));
  
  // Join meeting
  ws.send(JSON.stringify({
    type: 'join',
    meeting_id: 'meeting-123'
  }));
};

ws.onmessage = (event) => {
  const data = JSON.parse(event.data);
  
  switch(data.type) {
    case 'transcript.final':
      console.log(`[${data.speaker}] ${data.text}`);
      break;
    case 'sentiment.shift':
      console.log(`Sentiment changed to ${data.current}`);
      break;
    case 'action_item.detected':
      console.log(`Action item: ${data.task}`);
      break;
  }
};

// Send audio (from microphone)
navigator.mediaDevices.getUserMedia({ audio: true })
  .then(stream => {
    const mediaRecorder = new MediaRecorder(stream);
    
    mediaRecorder.ondataavailable = (e) => {
      const reader = new FileReader();
      reader.onloadend = () => {
        ws.send(JSON.stringify({
          type: 'audio',
          data: reader.result.split(',')[1]  // base64
        }));
      };
      reader.readAsDataURL(e.data);
    };
    
    mediaRecorder.start(100);  // 100ms chunks
  });
```

---

## Next Steps

### Explore Features

```bash
# Voice biometrics
curl -X POST http://localhost:8080/v1/voice/enroll \
  -F "user_id=user@example.com" \
  -F "audio=@voice_sample.wav"

# Predict meeting outcomes
curl -X POST http://localhost:8080/v1/predict/outcome \
  -H "Content-Type: application/json" \
  -d '{
    "meeting_context": {
      "title": "Q2 Planning",
      "participants": ["user1", "user2"],
      "duration_minutes": 60
    }
  }'

# Cultural adaptation
curl -X POST http://localhost:8080/v1/cultural/adapt \
  -H "Content-Type: application/json" \
  -d '{
    "meeting_id": "meeting-123",
    "target_culture": "ja-JP"
  }'
```

### Read Full Documentation

- [API Reference](../api/API_REFERENCE.md) - Complete API documentation
- [Operations Runbook](../operations/RUNBOOK.md) - Production operations
- [Architecture](../../ARCHITECTURE.md) - System architecture
- [Deployment Guide](../../DEPLOYMENT_GUIDE.md) - Production deployment

### Join Community

- Discord: [discord.gg/ruflo](https://discord.gg/ruflo)
- GitHub: [github.com/ruflo/platform](https://github.com/ruflo/platform)
- Support: support@ruflo.io

---

## Troubleshooting

### Port Already in Use

```bash
# Find process using port 8080
lsof -i :8080

# Kill process or use different port
docker-compose -f docker-compose.yml -f docker-compose.override.yml up
```

### Container Won't Start

```bash
# Check logs
docker-compose logs ruflo-asr

# Reset everything
docker-compose down -v
docker-compose up -d
```

### API Key Issues

```bash
# Verify key
curl http://localhost:8080/v1/auth/verify \
  -H "Authorization: Bearer $RUFO_API_KEY"

# Check key permissions
curl http://localhost:8080/v1/auth/permissions \
  -H "Authorization: Bearer $RUFO_API_KEY"
```

### Memory Issues

```bash
# Increase Docker memory limit
# Docker Desktop → Settings → Resources → Memory: 16GB

# Or use external services
docker-compose -f docker-compose.external.yml up
```

---

**Questions?** Reach out at support@ruflo.io

---
