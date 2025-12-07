# Media Gateway Rollback Procedures

## Quick Rollback (< 5 minutes)

### Kubernetes Rollback
```bash
# Rollback to previous deployment
kubectl rollout undo deployment/api-gateway -n media-gateway
kubectl rollout undo deployment/auth -n media-gateway
kubectl rollout undo deployment/discovery -n media-gateway

# Verify rollback
kubectl rollout status deployment/api-gateway -n media-gateway
```

### Database Rollback
```bash
# If migration needs reverting
sqlx migrate revert

# If full restore needed
psql -h $DB_HOST -U $DB_USER media_gateway < backup_YYYYMMDD.sql
```

## Detailed Rollback Procedure

### Step 1: Assess Impact
- Check error rates in Grafana
- Review recent alerts in Alertmanager
- Check logs in Loki for root cause

### Step 2: Communicate
- Post in #incidents Slack channel
- Page on-call if P0/P1 severity
- Update status page if user-facing

### Step 3: Execute Rollback
```bash
# Record current revision
kubectl rollout history deployment/api-gateway -n media-gateway

# Rollback to specific revision if needed
kubectl rollout undo deployment/api-gateway -n media-gateway --to-revision=N
```

### Step 4: Verify Recovery
- Check health endpoints return 200
- Verify error rate dropping
- Confirm user transactions succeeding

### Step 5: Post-Incident
- Create incident report
- Schedule post-mortem within 48 hours
- Update runbooks with learnings
