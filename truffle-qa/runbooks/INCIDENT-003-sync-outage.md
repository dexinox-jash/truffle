# Incident Response Runbook: INCIDENT-003
## Sync Service Outage

**Classification:** SEV-2 (High)  
**Response Time:** Within 4 hours  
**Last Updated:** 2024  

---

## 1. Incident Overview

### 1.1 Description

A sync service outage occurs when:
- Relay servers become unavailable
- Cloudflare Workers fail
- R2 storage inaccessible
- Network partition affects sync
- Rate limiting triggered
- Certificate expiration

### 1.2 Impact Assessment

| Impact Area | Severity | Description |
|-------------|----------|-------------|
| Sync Functionality | HIGH | Multi-device sync unavailable |
| User Experience | MEDIUM | Manual sync required |
| Data Integrity | LOW | Local data unaffected |
| Compliance | LOW | No data loss |

### 1.3 Key Point

> **Local-first architecture means sync outage does NOT affect core functionality.**
> Users can continue capturing, compiling, and using their wiki 100% offline.

### 1.4 Triggers

- Sync success rate <95%
- Relay server health check failures
- Cloudflare status page alerts
- User reports of sync issues
- Error rate spike in sync operations

---

## 2. Immediate Response (0-4 Hours)

### 2.1 Alert and Assessment

```bash
# Page on-call engineer
pagerduty trigger "SEV-2: Sync outage detected"

# Check service status
./scripts/check-sync-health.sh

# Create incident channel
slack create-channel "incident-003-sync-$(date +%Y%m%d)"
```

### 2.2 Health Check Commands

```bash
# Check relay server health
curl -f https://relay.truffle.io/health || echo "Relay unhealthy"

# Check Cloudflare Workers
wrangler tail | grep -i error

# Check R2 storage
aws s3 ls s3://truffle-sync-bucket/ --endpoint-url $R2_ENDPOINT

# Check sync success rate
./scripts/metrics/sync-success-rate.sh --last-hour

# Check error logs
./scripts/logs/sync-errors.sh --tail 100
```

### 2.3 Fail-Open Response

```typescript
// Local-first: App continues working
class SyncManager {
  async handleOutage(): Promise<void> {
    // 1. Notify user (non-blocking)
    this.notifyUser({
      type: 'sync_unavailable',
      message: 'Sync temporarily unavailable. Your data is safe on your device.',
      severity: 'info',
    });
    
    // 2. Queue sync operations for later
    this.queueForRetry();
    
    // 3. Continue local operations
    this.localMode = true;
    
    // 4. Retry sync periodically
    this.startRetryLoop({
      interval: 60000, // 1 minute
      maxRetries: null, // Keep trying
    });
  }
  
  private queueForRetry(): void {
    const pending = this.getPendingSyncs();
    for (const sync of pending) {
      retryQueue.push({
        ...sync,
        queuedAt: Date.now(),
        retryAfter: Date.now() + 60000,
      });
    }
  }
}
```

---

## 3. Investigation (1-8 Hours)

### 3.1 Root Cause Categories

| Category | Symptoms | Investigation |
|----------|----------|---------------|
| Infrastructure | All users affected | Cloudflare status, R2 health |
| Regional | Geographic pattern | CDN edge status, routing |
| Rate Limiting | 429 errors | Request volume, quota usage |
| Certificate | TLS errors | Cert expiry, renewal |
| Code | Specific errors | Deployment correlation |

### 3.2 Investigation Commands

```bash
# Check Cloudflare status
curl https://www.cloudflarestatus.com/api/v2/status.json | jq '.status'

# Analyze error patterns
./scripts/analyze-sync-errors.sh --time-range "4h"

# Check rate limit status
wrangler kv:key get "rate-limit-status" --namespace-id $NAMESPACE_ID

# Verify certificate expiry
echo | openssl s_client -servername relay.truffle.io -connect relay.truffle.io:443 2>/dev/null | openssl x509 -noout -dates

# Check deployment correlation
./scripts/check-deployment-correlation.sh --incident-start "$START_TIME"
```

### 3.3 Log Analysis

```typescript
// Analyze sync failure patterns
async function analyzeSyncFailures(
  timeRange: [Date, Date]
): Promise<FailureAnalysis> {
  const logs = await getSyncLogs(timeRange);
  
  const analysis = {
    totalAttempts: logs.length,
    failures: logs.filter(l => !l.success),
    errorTypes: {},
    affectedDevices: new Set(),
  };
  
  for (const log of logs) {
    if (!log.success) {
      analysis.errorTypes[log.error] = (analysis.errorTypes[log.error] || 0) + 1;
      analysis.affectedDevices.add(log.deviceId);
    }
  }
  
  return analysis;
}
```

---

## 4. Remediation (4-24 Hours)

### 4.1 Infrastructure Issues

```bash
# Restart relay workers
wrangler publish --env production

# Purge CDN cache if needed
cloudflare cache purge --zone-id $ZONE_ID

# Scale up if needed (Cloudflare auto-scales)
# No manual action needed

# Failover to backup region
./scripts/failover-to-backup.sh --region "us-east"
```

### 4.2 Rate Limiting Issues

```typescript
// Adjust rate limits temporarily
async function adjustRateLimits(): Promise<void> {
  // Increase limits for paid users
  await rateLimiter.updateConfig({
    tiers: {
      free: { daily: 100 },
      mycelium: { daily: 10000 }, // Increased from 10000
      forest: { daily: 100000 },
      enterprise: { daily: Infinity },
    },
  });
  
  // Notify affected users
  await notifyRateLimitedUsers({
    message: 'Sync rate limits temporarily increased due to service issues.',
  });
}
```

### 4.3 Certificate Issues

```bash
# Emergency certificate renewal
certbot renew --force-renewal --cert-name relay.truffle.io

# Update Cloudflare
cloudflare certificate upload \
  --certificate /etc/letsencrypt/live/relay.truffle.io/fullchain.pem \
  --private-key /etc/letsencrypt/live/relay.truffle.io/privkey.pem

# Verify renewal
echo | openssl s_client -connect relay.truffle.io:443 2>/dev/null | openssl x509 -noout -text
```

### 4.4 Code Issues

```bash
# Rollback if deployment caused issue
wrangler rollback --version $PREVIOUS_VERSION

# Or deploy hotfix
git checkout hotfix/sync-fix
wrangler publish --env production
```

---

## 5. Communication

### 5.1 Status Page Update

```markdown
## Sync Service Disruption

**Status:** Investigating  
**Impact:** Sync functionality temporarily unavailable  
**Local functionality:** Unaffected  

### Update Timeline

- [TIME] - Issue detected
- [TIME] - Investigation started
- [TIME] - Root cause identified: [CAUSE]
- [TIME] - Remediation in progress
- [TIME] - Service restored

### User Impact
- Your data remains safe on your device
- Core functionality (capture, compile, search) works normally
- Sync will resume automatically when service is restored
- No action required from users
```

### 5.2 In-App Notification

```typescript
const notification = {
  title: 'Sync Temporarily Unavailable',
  body: 'We\'re experiencing sync issues. Your data is safe on your device and all other features work normally.',
  actions: [
    { label: 'Check Status', action: 'open_status_page' },
  ],
  priority: 'low',
  dismissible: true,
};
```

---

## 6. Recovery Validation

### 6.1 Service Restoration Checks

```bash
# Verify relay health
curl -f https://relay.truffle.io/health

# Test sync operation
./scripts/test-sync.sh --device-id "test-device"

# Check success rate
./scripts/metrics/sync-success-rate.sh --last-hour

# Verify queue processing
./scripts/metrics/sync-queue-depth.sh
```

### 6.2 Post-Recovery Testing

```typescript
// Verify full recovery
describe('Post-Outage Validation', () => {
  test('relay server is healthy', async () => {
    const health = await checkRelayHealth();
    expect(health.status).toBe('healthy');
  });

  test('sync operations succeed', async () => {
    const result = await testSync(deviceA, deviceB);
    expect(result.success).toBe(true);
    expect(result.latency).toBeLessThan(5000);
  });

  test('queued syncs are processed', async () => {
    const queueDepth = await getQueueDepth();
    expect(queueDepth).toBe(0);
  });

  test('success rate above 99%', async () => {
    const rate = await getSyncSuccessRate('1h');
    expect(rate).toBeGreaterThan(0.99);
  });
});
```

---

## 7. Prevention Measures

### 7.1 Enhanced Monitoring

```typescript
// Proactive sync monitoring
const syncMonitoring = {
  // Alert thresholds
  alerts: {
    successRate: { threshold: 0.95, severity: 'warning' },
    successRate: { threshold: 0.90, severity: 'critical' },
    latency: { threshold: 5000, severity: 'warning' },
    errorRate: { threshold: 0.05, severity: 'warning' },
  },
  
  // Auto-remediation
  autoRemediate: {
    enabled: true,
    actions: [
      { condition: 'successRate < 0.90', action: 'page_oncall' },
      { condition: 'errorRate > 0.10', action: 'restart_workers' },
    ],
  },
};
```

### 7.2 Multi-Region Deployment

```yaml
# Cloudflare Workers multi-region
name: truffle-relay
main: src/index.ts
routes:
  - pattern: relay.truffle.io
    custom_domain: true

# Deploy to multiple regions
deploy:
  regions:
    - eu-west
    - us-east
    - us-west
    - ap-south
  
  failover:
    enabled: true
    health_check_interval: 30s
```

### 7.3 Certificate Management

```bash
# Automated certificate renewal
certbot renew --quiet --deploy-hook "./scripts/update-cloudflare-cert.sh"

# Certificate expiry monitoring
./scripts/monitor-cert-expiry.sh --days-before-expiry 30
```

---

## 8. Appendices

### Appendix A: Quick Commands

```bash
# Check sync health
./scripts/check-sync-health.sh

# View sync logs
wrangler tail

# Check error rate
./scripts/metrics/sync-error-rate.sh

# Test sync manually
./scripts/test-sync.sh --device-id "your-device"

# Update status page
./scripts/update-status-page.sh --status "investigating" --message "..."

# Failover to backup
./scripts/failover-to-backup.sh
```

### Appendix B: Escalation Path

| Duration | Action | Owner |
|----------|--------|-------|
| 0-1 hour | Initial response, assessment | On-call engineer |
| 1-4 hours | Active remediation | Engineering lead |
| 4-8 hours | Escalate to VP Engineering | Engineering lead |
| 8+ hours | Executive notification | VP Engineering |

### Appendix C: Related Runbooks

- [INCIDENT-001: Key Compromise](./INCIDENT-001-key-compromise.md)
- [INCIDENT-002: Model Poisoning](./INCIDENT-002-model-poisoning.md)
- [Infrastructure Playbook](../../docs/ops/infrastructure.md)

---

*Classification: Internal*  
*Review Cycle: Quarterly*  
*Owner: Infrastructure Team*
