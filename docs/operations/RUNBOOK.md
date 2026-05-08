# Ruflo AI Platform - Operational Runbook

> **Classification:** INTERNAL - Operations Team  
> **Version:** 14.0  
> **Last Updated:** April 10, 2026

---

## Table of Contents

1. [On-Call Procedures](#on-call-procedures)
2. [Service Health Monitoring](#service-health-monitoring)
3. [Incident Response](#incident-response)
4. [Deployment Procedures](#deployment-procedures)
5. [Backup & Recovery](#backup--recovery)
6. [Performance Tuning](#performance-tuning)
7. [Security Operations](#security-operations)
8. [Common Issues](#common-issues)

---

## On-Call Procedures

### Escalation Matrix

| Severity | Response Time | Escalation Path |
|----------|---------------|-----------------|
| P0 (Critical) | 5 minutes | On-call → Lead → VP Eng → CTO |
| P1 (High) | 15 minutes | On-call → Lead |
| P2 (Medium) | 1 hour | On-call |
| P3 (Low) | 4 hours | Ticket queue |

### P0 Criteria

- Complete platform outage
- Data loss or corruption
- Security breach
- Authentication system failure
- Payment processing down

### On-Call Checklist

```bash
# 1. Verify alert authenticity
kubectl get pods -A | grep -v Running

# 2. Check overall health
curl https://api.ruflo.io/health

# 3. Review recent deployments
kubectl rollout history deployment/ruflo-gateway

# 4. Check error rates
grafana-cli dashboard get-truffle-error-rate

# 5. Notify team
# Post in #incidents Slack channel
```

---

## Service Health Monitoring

### Key Metrics Dashboard

**Grafana:** https://grafana.ruflo.io/d/truffle-health

#### Critical Metrics

| Metric | Target | Warning | Critical |
|--------|--------|---------|----------|
| API Error Rate | <0.1% | 0.5% | 1% |
| P95 Latency | <200ms | 500ms | 1000ms |
| Transcription Queue | <100 | 500 | 1000 |
| Active Streams | >1000 | <500 | <100 |
| DB Connections | <80% | 90% | 95% |
| Memory Usage | <70% | 85% | 95% |

### Health Check Commands

```bash
# Gateway health
curl https://api.ruflo.io/health

# Individual service health
kubectl exec -it deployment/ruflo-asr -- curl localhost:8080/health

# Database connectivity
kubectl exec -it deployment/ruflo-storage -- pg_isready

# Redis connectivity
kubectl exec -it deployment/ruflo-realtime -- redis-cli ping

# NATS connectivity
kubectl exec -it deployment/vortex-infra -- nats server check
```

### Log Aggregation

**Kibana:** https://kibana.ruflo.io

```bash
# Search for errors
kubectl logs -l app=ruflo-gateway --tail=100 | grep ERROR

# Follow real-time logs
kubectl logs -l app=ruflo-asr -f

# Search by trace ID
curl -X POST https://kibana.ruflo.io/api/search \
  -d '{"query": {"trace.id": "trace-uuid"}}'
```

---

## Incident Response

### Incident Response Playbook

#### Phase 1: Detection (0-5 min)

1. **Acknowledge Alert**
   ```bash
   # In PagerDuty, acknowledge the alert
   # Post in #incidents: "Investigating [alert name]"
   ```

2. **Assess Impact**
   ```bash
   # Check error rates
   curl https://grafana.ruflo.io/api/datasources/proxy/1/query \
     -d 'query=sum(rate(http_requests_total{status=~"5.."}[5m]))'
   
   # Check affected services
   kubectl get pods -A --field-selector status.phase!=Running
   ```

#### Phase 2: Containment (5-15 min)

**If service is degraded:**
```bash
# Scale up replicas
kubectl scale deployment ruflo-asr --replicas=10

# Enable circuit breaker
kubectl patch configmap ruflo-gateway-config \
  --patch '{"data": {"circuit_breaker": "enabled"}}'
```

**If database is overloaded:**
```bash
# Enable read replicas
kubectl apply -f k8s/postgres-read-replica.yaml

# Enable connection pooling
kubectl patch configmap ruflo-storage-config \
  --patch '{"data": {"pool_size": "100"}}'
```

#### Phase 3: Resolution (15-60 min)

```bash
# Rollback if needed
kubectl rollout undo deployment/ruflo-gateway

# Or fix forward
kubectl apply -f k8s/fix-deployment.yaml
kubectl rollout status deployment/ruflo-gateway
```

#### Phase 4: Post-Incident

1. Document timeline in incident ticket
2. Schedule post-mortem within 24 hours
3. Create action items
4. Update runbook if needed

### Common Incident Scenarios

#### Scenario 1: High Error Rate

**Symptoms:**
- Error rate > 1%
- Multiple 5xx responses
- Latency spikes

**Response:**
```bash
# 1. Identify failing service
kubectl get pods -A | grep -v Running

# 2. Check logs
kubectl logs -l app=ruflo-gateway --tail=1000 | grep ERROR

# 3. If deployment issue, rollback
kubectl rollout undo deployment/ruflo-gateway

# 4. Verify recovery
curl https://api.ruflo.io/health
```

#### Scenario 2: Transcription Queue Backup

**Symptoms:**
- Queue depth > 1000
- Processing latency > 5 min
- User complaints

**Response:**
```bash
# 1. Scale ASR workers
kubectl scale deployment ruflo-asr --replicas=20

# 2. Check GPU utilization
kubectl top nodes --show-capacity | grep gpu

# 3. If GPU bottleneck, enable CPU fallback
kubectl patch configmap ruflo-asr-config \
  --patch '{"data": {"fallback_to_cpu": "true"}}'

# 4. Monitor queue
kubectl exec -it deployment/ruflo-asr -- redis-cli LLEN transcription:queue
```

#### Scenario 3: Database Connection Exhaustion

**Symptoms:**
- Connection pool errors
- Timeout errors
- "too many connections" in logs

**Response:**
```bash
# 1. Check current connections
kubectl exec -it deployment/ruflo-storage -- psql -c "SELECT count(*) FROM pg_stat_activity;"

# 2. Kill idle connections
kubectl exec -it deployment/ruflo-storage -- psql -c "SELECT pg_terminate_backend(pid) FROM pg_stat_activity WHERE state = 'idle' AND state_change < NOW() - INTERVAL '10 minutes';"

# 3. Increase pool size temporarily
kubectl patch configmap ruflo-storage-config \
  --patch '{"data": {"max_connections": "500"}}'

# 4. Restart affected pods
kubectl rollout restart deployment/ruflo-storage
```

#### Scenario 4: Security Incident

**Symptoms:**
- Unusual API traffic patterns
- Failed authentication spikes
- Suspicious IP addresses

**Response:**
```bash
# 1. Enable WAF blocking
kubectl patch configmap ruflo-gateway-config \
  --patch '{"data": {"waf_mode": "block"}}'

# 2. Block suspicious IPs
kubectl exec -it deployment/ruflo-gateway -- iptables -A INPUT -s {IP} -j DROP

# 3. Force token rotation
kubectl delete secret jwt-signing-key
kubectl apply -f k8s/secrets/jwt-signing-key.yaml

# 4. Enable enhanced logging
kubectl patch configmap ruflo-audit-config \
  --patch '{"data": {"log_level": "debug"}}'

# 5. Notify security team
# Page security-oncall immediately
```

---

## Deployment Procedures

### Pre-Deployment Checklist

```bash
# 1. Verify tests pass
cargo test --workspace
npm test

# 2. Check security scan
cargo audit
npm audit

# 3. Verify staging deployment
kubectl config use-context staging
curl https://api-staging.ruflo.io/health

# 4. Review diff
git diff HEAD~1 --stat

# 5. Check canary metrics
# Verify staging error rates < 0.1%
```

### Production Deployment

#### Blue-Green Deployment

```bash
# 1. Deploy to green environment
kubectl apply -f k8s/production/green/

# 2. Wait for green to be ready
kubectl rollout status deployment/ruflo-gateway-green

# 3. Verify green health
./scripts/verify_deployment.sh green

# 4. Switch traffic to green
kubectl patch service ruflo-gateway \
  --patch '{"spec": {"selector": {"version": "green"}}}'

# 5. Monitor for 30 minutes
# Watch error rates, latency

# 6. If issues, rollback
curl -X POST https://api.ruflo.io/admin/rollback \
  -H "Authorization: Bearer $ADMIN_TOKEN"
```

#### Canary Deployment

```bash
# 1. Deploy canary (10% traffic)
kubectl apply -f k8s/canary/

# 2. Monitor metrics for 15 minutes
# Check error rate < 0.1%
# Check p95 latency < 200ms

# 3. If healthy, increase to 50%
kubectl patch virtualservice ruflo-gateway \
  --patch '{"spec": {"http": [{"route": [{"weight": 50}]}]}}'

# 4. Monitor for 15 more minutes

# 5. Full rollout
kubectl patch virtualservice ruflo-gateway \
  --patch '{"spec": {"http": [{"route": [{"weight": 100}]}]}}'

# 6. Cleanup canary
kubectl delete -f k8s/canary/
```

### Rollback Procedures

```bash
# Quick rollback
kubectl rollout undo deployment/ruflo-gateway

# Rollback to specific revision
kubectl rollout undo deployment/ruflo-gateway --to-revision=3

# Database rollback (if migration)
kubectl exec -it deployment/ruflo-storage -- \
  psql -c "BEGIN; ROLLBACK;" # Or apply down migration

# Full platform rollback
./scripts/rollback-platform.sh --version=v13.9
```

---

## Backup & Recovery

### Backup Schedule

| Data | Frequency | Retention | Location |
|------|-----------|-----------|----------|
| PostgreSQL | Hourly | 30 days | S3 (us-east-1, eu-west-1) |
| Redis | Daily | 7 days | S3 |
| User Files | Real-time | 2555 days | S3 + Glacier |
| Configurations | On change | Forever | Git + S3 |

### Manual Backup

```bash
# PostgreSQL backup
kubectl exec -it deployment/postgres -- \
  pg_dump -Fc ruflo_production > backup_$(date +%Y%m%d_%H%M%S).sql

# Redis backup
kubectl exec -it deployment/redis -- \
  redis-cli BGSAVE
kubectl cp redis:/data/dump.rdb ./redis_backup.rdb

# Upload to S3
aws s3 cp backup.sql s3://ruflo-backups/postgres/
aws s3 cp redis_backup.rdb s3://ruflo-backups/redis/
```

### Point-in-Time Recovery

```bash
# 1. Stop affected services
kubectl scale deployment ruflo-storage --replicas=0

# 2. Restore from backup
aws s3 cp s3://ruflo-backups/postgres/backup_20260410_120000.sql ./
kubectl exec -it deployment/postgres -- \
  pg_restore -d ruflo_production backup_20260410_120000.sql

# 3. Replay WAL to target time
kubectl exec -it deployment/postgres -- \
  pg_waldump --timeline=1 --start=0/01000000

# 4. Restart services
kubectl scale deployment ruflo-storage --replicas=3

# 5. Verify data integrity
./scripts/verify_data_integrity.sh
```

### Disaster Recovery

**Region Failure (Failover to us-west-2):**

```bash
# 1. Promote read replica to primary
kubectl exec -it deployment/postgres-west -- \
  pg_ctl promote

# 2. Update DNS
aws route53 change-resource-record-sets \
  --hosted-zone-id Z123456789 \
  --change-batch file://failover.json

# 3. Scale up services in new region
kubectl config use-context production-west
kubectl scale deployment --all --replicas=3

# 4. Verify health
curl https://api.ruflo.io/health

# 5. Notify users of reduced capacity
```

---

## Performance Tuning

### Database Optimization

```sql
-- Analyze query performance
SELECT query, mean_exec_time, calls 
FROM pg_stat_statements 
ORDER BY mean_exec_time DESC 
LIMIT 10;

-- Add missing indexes
CREATE INDEX CONCURRENTLY idx_meetings_created_at 
ON meetings(created_at) 
WHERE deleted_at IS NULL;

-- Vacuum and analyze
VACUUM ANALYZE meetings;

-- Update statistics
ANALYZE VERBOSE;
```

### Cache Optimization

```bash
# Check cache hit ratio
kubectl exec -it deployment/redis -- \
  redis-cli INFO stats | grep keyspace

# Preload hot data
kubectl exec -it deployment/ruflo-storage -- \
  psql -c "SELECT pg_prewarm('meetings');"

# Clear stale cache
kubectl exec -it deployment/redis -- \
  redis-cli --eval purge_stale_keys.lua
```

### ASR Performance

```bash
# Check GPU utilization
nvidia-smi

# Optimize batch size
kubectl patch configmap ruflo-asr-config \
  --patch '{"data": {"batch_size": "32"}}'

# Enable model quantization
kubectl patch configmap ruflo-asr-config \
  --patch '{"data": {"quantization": "int8"}}'
```

---

## Security Operations

### Security Monitoring

**SIEM Dashboard:** https://siem.ruflo.io

#### Daily Security Checks

```bash
# Check failed logins
kubectl logs -l app=ruflo-auth | grep "Failed login" | wc -l

# Check for suspicious IPs
kubectl logs -l app=ruflo-gateway | grep "rate_limit" | awk '{print $5}' | sort | uniq -c | sort -rn | head -10

# Verify certificate expiry
openssl x509 -in /etc/ssl/ruflo.crt -noout -dates

# Check for new CVEs
cargo audit
npm audit
```

### Certificate Rotation

```bash
# Generate new certificate
certbot renew --force-renewal

# Update Kubernetes secret
kubectl create secret tls ruflo-tls \
  --cert=/etc/letsencrypt/live/api.ruflo.io/fullchain.pem \
  --key=/etc/letsencrypt/live/api.ruflo.io/privkey.pem \
  --dry-run=client -o yaml | kubectl apply -f -

# Rolling restart
kubectl rollout restart deployment/ruflo-gateway
```

### Secret Rotation

```bash
# 1. Generate new secrets
openssl rand -base64 32 > new_jwt_secret
openssl rand -base64 32 > new_encryption_key

# 2. Update secrets
kubectl create secret generic jwt-secret \
  --from-file=jwt-secret=./new_jwt_secret \
  --dry-run=client -o yaml | kubectl apply -f -

# 3. Rolling restart
kubectl rollout restart deployment/ruflo-auth

# 4. Verify
kubectl exec -it deployment/ruflo-auth -- cat /secrets/jwt-secret
```

---

## Common Issues

### Issue: Transcription Quality Degraded

**Symptoms:** Low accuracy, many [unintelligible] markers

**Diagnosis:**
```bash
# Check model version
kubectl exec -it deployment/ruflo-asr -- whisper --version

# Check audio quality metrics
kubectl logs -l app=ruflo-asr | grep "audio_quality"

# Verify GPU memory
nvidia-smi
```

**Resolution:**
```bash
# Restart ASR pods
kubectl rollout restart deployment/ruflo-asr

# If GPU OOM, reduce batch size
kubectl patch configmap ruflo-asr-config \
  --patch '{"data": {"batch_size": "16"}}'
```

### Issue: Sync Delays

**Symptoms:** Devices not syncing, stale data

**Diagnosis:**
```bash
# Check NATS queue depth
kubectl exec -it deployment/nats -- nats stream info SYNC

# Check sync service logs
kubectl logs -l app=ruflo-sync | grep "queue_depth"

# Verify WebSocket connections
kubectl exec -it deployment/ruflo-realtime -- netstat -an | grep :8080 | wc -l
```

**Resolution:**
```bash
# Scale sync workers
kubectl scale deployment ruflo-sync --replicas=10

# Clear stuck messages
kubectl exec -it deployment/nats -- nats stream purge SYNC
```

### Issue: Search Slow

**Symptoms:** Search queries taking > 500ms

**Diagnosis:**
```bash
# Check index size
kubectl exec -it deployment/ruflo-search -- curl localhost:9200/_cat/indices

# Check query logs
kubectl logs -l app=ruflo-search | grep "took"
```

**Resolution:**
```bash
# Force merge segments
kubectl exec -it deployment/ruflo-search -- \
  curl -X POST localhost:9200/meetings/_forcemerge?max_num_segments=1

# Add more replicas
kubectl patch statefulset ruflo-search \
  --patch '{"spec": {"replicas": 5}}'
```

---

## Emergency Contacts

| Role | Name | Phone | Slack |
|------|------|-------|-------|
| VP Engineering | Emergency | +1-555-0100 | @vp-eng |
| CTO | Emergency | +1-555-0101 | @cto |
| Security Lead | Emergency | +1-555-0102 | @security |
| SRE Lead | On-Call | +1-555-0103 | @sre-oncall |
| Database Admin | On-Call | +1-555-0104 | @dba-oncall |

---

**Document Owner:** SRE Team  
**Review Cycle:** Monthly  
**Next Review:** May 10, 2026

---
