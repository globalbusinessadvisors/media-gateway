# Media Gateway Production Deployment Runbook

## Overview
This runbook covers the deployment procedure for Media Gateway microservices to production Kubernetes clusters.

## Pre-Deployment Checklist

### Code Verification
- [ ] All tests pass: `SQLX_OFFLINE=true cargo test --workspace`
- [ ] No clippy warnings: `cargo clippy --workspace -- -D warnings`
- [ ] Code formatted: `cargo fmt --all -- --check`
- [ ] Security scan passed: `cargo audit`
- [ ] Docker images built: `docker-compose build`

### Environment Verification
- [ ] `JWT_SECRET` is set (minimum 256-bit key)
- [ ] `DATABASE_URL` points to production database
- [ ] `REDIS_URL` points to production Redis cluster
- [ ] `ALLOWED_ORIGINS` contains only production domains
- [ ] `RUST_ENV=production` is set
- [ ] All secrets are in Kubernetes secrets (not ConfigMaps)

### Database Preparation
- [ ] Database migrations are ready: `ls migrations/`
- [ ] Backup created: `pg_dump -h $DB_HOST -U $DB_USER media_gateway > backup_$(date +%Y%m%d).sql`
- [ ] Migration tested in staging

## Deployment Procedure

### Step 1: Apply Database Migrations
```bash
# From bastion host or CI/CD pipeline
export DATABASE_URL="postgresql://user:pass@prod-db:5432/media_gateway"
sqlx migrate run
```

### Step 2: Deploy Services (Rolling Update)
```bash
# Apply Kubernetes manifests in order
kubectl apply -f k8s/namespace.yaml
kubectl apply -f k8s/configmaps/
kubectl apply -f k8s/secrets/
kubectl apply -f k8s/services/

# Deploy in dependency order
kubectl rollout restart deployment/postgres -n media-gateway
kubectl rollout status deployment/postgres -n media-gateway

kubectl rollout restart deployment/redis -n media-gateway
kubectl rollout status deployment/redis -n media-gateway

kubectl rollout restart deployment/auth -n media-gateway
kubectl rollout status deployment/auth -n media-gateway

kubectl rollout restart deployment/api-gateway -n media-gateway
kubectl rollout status deployment/api-gateway -n media-gateway
```

### Step 3: Health Verification
```bash
# Verify all pods are running
kubectl get pods -n media-gateway

# Check health endpoints
for svc in api-gateway discovery sona auth sync ingestion playback; do
  echo "Checking $svc..."
  kubectl exec -n media-gateway deploy/api-gateway -- \
    curl -s http://$svc:8080/health | jq '.status'
done

# Verify readiness
kubectl get pods -n media-gateway -o jsonpath='{.items[*].status.conditions[?(@.type=="Ready")].status}'
```

### Step 4: Smoke Tests
```bash
# Run critical path tests
./scripts/smoke-tests.sh production

# Manual verification
curl -H "Authorization: Bearer $TEST_TOKEN" https://api.mediagateway.io/api/v1/status
curl https://api.mediagateway.io/health
```

## Post-Deployment Verification

### Metrics Verification
- [ ] Prometheus scraping all targets: http://prometheus:9090/targets
- [ ] Grafana dashboards loading: http://grafana:3000
- [ ] No alerts firing: http://alertmanager:9093

### Log Verification
- [ ] Logs flowing to Loki: http://grafana:3000/explore
- [ ] No ERROR level logs in last 5 minutes
- [ ] Request tracing visible in Jaeger

## Emergency Contacts

| Role | Name | Contact |
|------|------|---------|
| On-Call Engineer | Rotating | PagerDuty |
| Platform Lead | TBD | Slack #platform |
| Database Admin | TBD | Slack #data |

## Version History

| Version | Date | Author | Changes |
|---------|------|--------|---------|
| 1.0 | 2025-12-07 | Claude | Initial version |
