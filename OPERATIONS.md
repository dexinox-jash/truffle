# Ruflo Platform Operations Guide

## Table of Contents
- [Deployment](#deployment)
- [Monitoring](#monitoring)
- [Troubleshooting](#troubleshooting)
- [Backup & Recovery](#backup--recovery)
- [Scaling](#scaling)

## Deployment

### Prerequisites
- Kubernetes cluster (v1.28+)
- kubectl configured
- Kustomize installed
- Docker registry access

### Deploy to Staging

```bash
# Deploy infrastructure
kubectl apply -k k8s/infra

# Deploy application
kubectl apply -k k8s/overlays/staging

# Verify deployment
kubectl get pods -n ruflo-staging
kubectl logs -n ruflo-staging -l app=ruflo-gateway
```

### Deploy to Production

```bash
# Update image tags in production overlay
cd k8s/overlays/production
kustomize edit set image ruflo/gateway=ghcr.io/ruflo-ai/gateway:v1.2.3

# Apply deployment
kubectl apply -k .

# Verify rollout
kubectl rollout status deployment/ruflo-gateway -n ruflo
```

### Database Migrations

```bash
# Run migrations
./scripts/db-migrate.sh migrate

# Check status
./scripts/db-migrate.sh status

# Rollback last migration
./scripts/db-migrate.sh rollback
```

## Monitoring

### Access Dashboards

| Service | URL | Credentials |
|---------|-----|-------------|
| Grafana | http://grafana.ruflo.io | admin / ruflo-admin |
| Prometheus | http://prometheus.ruflo.io | - |
| Kibana | http://kibana.ruflo.io | - |

### Key Metrics

- **Request Rate**: `rate(http_requests_total[5m])`
- **Error Rate**: `rate(http_requests_total{status=~"5.."}[5m])`
- **Latency P95**: `histogram_quantile(0.95, rate(http_request_duration_seconds_bucket[5m]))`
- **CPU Usage**: `rate(container_cpu_usage_seconds_total[5m])`
- **Memory Usage**: `container_memory_usage_bytes`

### Alerts

| Alert | Severity | Condition |
|-------|----------|-----------|
| HighErrorRate | Critical | Error rate > 5% for 5m |
| ServiceDown | Critical | Service unavailable for 1m |
| HighLatency | Warning | P95 latency > 1s for 5m |
| HighCPU | Warning | CPU > 80% for 10m |
| HighMemory | Warning | Memory > 85% for 10m |

## Troubleshooting

### Service Unavailable

```bash
# Check pod status
kubectl get pods -n ruflo

# Check logs
kubectl logs -n ruflo -l app=ruflo-gateway --tail=100

# Check events
kubectl get events -n ruflo --sort-by='.lastTimestamp'

# Check resource usage
kubectl top pods -n ruflo
```

### Database Issues

```bash
# Connect to PostgreSQL
kubectl exec -it -n ruflo deployment/postgres -- psql -U postgres -d ruflo

# Check connections
SELECT * FROM pg_stat_activity;

# Check slow queries
SELECT * FROM pg_stat_statements ORDER BY mean_time DESC LIMIT 10;
```

### Performance Issues

1. **High Latency**
   - Check database query performance
   - Verify Redis cache hit rate
   - Scale up affected services

2. **High Error Rate**
   - Check service logs for errors
   - Verify downstream service health
   - Check rate limiting status

3. **Out of Memory**
   - Increase memory limits
   - Check for memory leaks
   - Add more nodes to cluster

## Backup & Recovery

### PostgreSQL Backup

```bash
# Automated backup (daily at 2 AM)
kubectl create cronjob -n ruflo postgres-backup \
  --image=postgres:16-alpine \
  --schedule="0 2 * * *" \
  -- pg_dump -h postgres -U postgres ruflo > /backups/ruflo-$(date +%Y%m%d).sql

# Manual backup
kubectl exec -it -n ruflo deployment/postgres -- \
  pg_dump -U postgres ruflo > backup.sql

# Restore from backup
kubectl exec -i -n ruflo deployment/postgres -- \
  psql -U postgres -d ruflo < backup.sql
```

### Persistent Volume Backups

```bash
# Create snapshot
kubectl apply -f - <<EOF
apiVersion: snapshot.storage.k8s.io/v1
kind: VolumeSnapshot
metadata:
  name: postgres-snapshot
  namespace: ruflo
spec:
  volumeSnapshotClassName: standard
  source:
    persistentVolumeClaimName: postgres-storage-postgres-0
EOF
```

## Scaling

### Horizontal Pod Autoscaling

```bash
# Enable HPA for gateway
kubectl autoscale deployment ruflo-gateway \
  --cpu-percent=70 \
  --min=3 \
  --max=10 \
  -n ruflo

# View HPA status
kubectl get hpa -n ruflo
```

### Vertical Scaling

```bash
# Update resource limits
kubectl patch deployment ruflo-asr -n ruflo -p '{"spec":{"template":{"spec":{"containers":[{"name":"asr","resources":{"limits":{"memory":"4Gi","cpu":"2000m"}}}]}}}}'
```

### Cluster Scaling

```bash
# Add node to cluster (EKS example)
eksctl scale nodegroup --cluster=ruflo --name=workers --nodes=5

# Verify nodes
kubectl get nodes
```

## Maintenance Windows

### Scheduled Maintenance

1. Announce maintenance window (24h notice)
2. Scale down non-critical services
3. Apply updates
4. Verify health checks
5. Scale up services
6. Monitor for issues

### Emergency Procedures

1. **Rollback Deployment**
   ```bash
   kubectl rollout undo deployment/ruflo-gateway -n ruflo
   ```

2. **Evacuate Node**
   ```bash
   kubectl drain <node-name> --ignore-daemonsets
   ```

3. **Restart Service**
   ```bash
   kubectl rollout restart deployment/ruflo-gateway -n ruflo
   ```

## Contact

- **On-call**: +1-XXX-XXX-XXXX
- **Slack**: #ruflo-ops
- **Email**: ops@ruflo.io
