import http from 'k6/http';
import { check, sleep, group } from 'k6';
import { Rate, Trend, Counter } from 'k6/metrics';

// Custom metrics
const errorRate = new Rate('errors');
const apiLatency = new Trend('api_latency');
const successfulRequests = new Counter('successful_requests');

// Test configuration
export const options = {
  stages: [
    { duration: '2m', target: 100 },   // Ramp up to 100 users
    { duration: '5m', target: 100 },   // Stay at 100 users
    { duration: '2m', target: 200 },   // Ramp up to 200 users
    { duration: '5m', target: 200 },   // Stay at 200 users
    { duration: '2m', target: 400 },   // Ramp up to 400 users
    { duration: '5m', target: 400 },   // Stay at 400 users
    { duration: '5m', target: 0 },     // Ramp down
  ],
  thresholds: {
    http_req_duration: ['p(95)<500'],   // 95% of requests under 500ms
    http_req_failed: ['rate<0.01'],      // Error rate under 1%
    errors: ['rate<0.05'],
  },
};

const BASE_URL = __ENV.BASE_URL || 'https://api.staging.ruflo.ai';
const AUTH_TOKEN = __ENV.AUTH_TOKEN || '';

export function setup() {
  // Login and get auth token
  const loginRes = http.post(`${BASE_URL}/api/v1/auth/login`, JSON.stringify({
    email: 'load-test@ruflo.ai',
    password: 'test-password-123',
  }), {
    headers: { 'Content-Type': 'application/json' },
  });

  check(loginRes, {
    'login successful': (r) => r.status === 200,
  });

  return { token: loginRes.json('access_token') };
}

export default function (data) {
  const headers = {
    'Content-Type': 'application/json',
    'Authorization': `Bearer ${data.token}`,
  };

  group('Health Check', () => {
    const res = http.get(`${BASE_URL}/health`);
    const success = check(res, {
      'health status is 200': (r) => r.status === 200,
      'health response valid': (r) => r.json('status') === 'healthy',
    });
    errorRate.add(!success);
    apiLatency.add(res.timings.duration);
  });

  group('API Endpoints', () => {
    // Get meetings list
    const meetingsRes = http.get(`${BASE_URL}/api/v1/meetings?limit=20`, { headers });
    const meetingsSuccess = check(meetingsRes, {
      'meetings list status is 200': (r) => r.status === 200,
      'meetings response is array': (r) => Array.isArray(r.json()),
    });
    errorRate.add(!meetingsSuccess);
    apiLatency.add(meetingsRes.timings.duration);

    if (meetingsSuccess) {
      successfulRequests.add(1);
    }

    // Create meeting
    const createRes = http.post(`${BASE_URL}/api/v1/meetings`, JSON.stringify({
      title: `Load Test Meeting ${Date.now()}`,
      description: 'Created during load test',
      scheduled_at: new Date(Date.now() + 3600000).toISOString(),
      duration_minutes: 30,
    }), { headers });

    const createSuccess = check(createRes, {
      'create meeting status is 201': (r) => r.status === 201,
      'create meeting has id': (r) => r.json('id') !== undefined,
    });
    errorRate.add(!createSuccess);
    apiLatency.add(createRes.timings.duration);

    if (createSuccess) {
      successfulRequests.add(1);
      const meetingId = createRes.json('id');

      // Get meeting details
      const getRes = http.get(`${BASE_URL}/api/v1/meetings/${meetingId}`, { headers });
      check(getRes, {
        'get meeting status is 200': (r) => r.status === 200,
      });
      apiLatency.add(getRes.timings.duration);

      // Update meeting
      const updateRes = http.patch(`${BASE_URL}/api/v1/meetings/${meetingId}`, JSON.stringify({
        title: 'Updated Load Test Meeting',
      }), { headers });
      check(updateRes, {
        'update meeting status is 200': (r) => r.status === 200,
      });
      apiLatency.add(updateRes.timings.duration);

      // Delete meeting
      const deleteRes = http.del(`${BASE_URL}/api/v1/meetings/${meetingId}`, null, { headers });
      check(deleteRes, {
        'delete meeting status is 204': (r) => r.status === 204,
      });
      apiLatency.add(deleteRes.timings.duration);
    }
  });

  group('WebSocket Connection', () => {
    // WebSocket load testing would use k6/ws
    // This is a placeholder for WebSocket load tests
    const wsRes = http.get(`${BASE_URL}/ready`);
    check(wsRes, {
      'websocket service ready': (r) => r.status === 200,
    });
  });

  sleep(1);
}

export function handleSummary(data) {
  return {
    'load-test-results.json': JSON.stringify(data),
    stdout: textSummary(data, { indent: ' ', enableColors: true }),
  };
}

function textSummary(data, options) {
  // Simple text summary
  return `
Load Test Results
=================
Duration: ${data.state.testRunDuration}
Requests: ${data.metrics.http_reqs.values.count}
Failed: ${data.metrics.http_req_failed.values.rate * 100}%
Avg Latency: ${data.metrics.http_req_duration.values.avg}ms
P95 Latency: ${data.metrics.http_req_duration.values['p(95)']}ms
`;
}
