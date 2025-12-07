# Media Gateway Troubleshooting Guide

## Common Issues

### 1. High Latency (>500ms p95)

**Symptoms:**
- Slow API responses
- SLO error budget burning fast
- User complaints

**Diagnosis:**
```bash
# Check slow queries
kubectl exec -n media-gateway deploy/postgres -- \
  psql -c "SELECT query, calls, mean_time FROM pg_stat_statements ORDER BY mean_time DESC LIMIT 10;"

# Check Redis latency
kubectl exec -n media-gateway deploy/redis -- redis-cli --latency

# Check connection pool exhaustion
curl http://api-gateway:8080/metrics | grep pool
```

**Resolution:**
- Add database indexes for slow queries
- Scale up replicas if CPU bound
- Increase connection pool size
- Enable query caching

### 2. 5xx Errors Spike

**Symptoms:**
- Error rate > 0.1%
- Alerts firing
- User-facing errors

**Diagnosis:**
```bash
# Check pod status
kubectl get pods -n media-gateway

# Check recent logs
kubectl logs -n media-gateway deploy/api-gateway --since=5m | grep -i error

# Check dependent services
kubectl exec -n media-gateway deploy/api-gateway -- curl -s http://auth:8083/health
```

**Resolution:**
- Restart unhealthy pods
- Check circuit breaker status
- Verify external dependencies (DB, Redis, Qdrant)
- Scale up if overloaded

### 3. Database Connection Errors

**Symptoms:**
- "connection refused" or "too many connections"
- Pods failing health checks

**Diagnosis:**
```bash
# Check connection count
kubectl exec -n media-gateway deploy/postgres -- \
  psql -c "SELECT count(*) FROM pg_stat_activity;"

# Check max connections
kubectl exec -n media-gateway deploy/postgres -- \
  psql -c "SHOW max_connections;"
```

**Resolution:**
- Increase max_connections in PostgreSQL
- Reduce pool size per service
- Add PgBouncer for connection pooling

### 4. Memory Pressure / OOMKilled

**Symptoms:**
- Pods restarting with OOMKilled
- High memory usage in metrics

**Diagnosis:**
```bash
# Check memory usage
kubectl top pods -n media-gateway

# Check resource limits
kubectl describe pod -n media-gateway api-gateway-xxx | grep -A 5 Limits
```

**Resolution:**
- Increase memory limits
- Check for memory leaks in recent changes
- Enable memory profiling

### 5. Rate Limiting Too Aggressive

**Symptoms:**
- 429 responses for legitimate traffic
- User complaints about being blocked

**Diagnosis:**
```bash
# Check rate limit metrics
curl http://api-gateway:8080/metrics | grep rate_limit

# Check Redis rate limit keys
kubectl exec -n media-gateway deploy/redis -- redis-cli KEYS "rate_limit:*" | wc -l
```

**Resolution:**
- Adjust rate limits in configuration
- Add specific user/IP to allowlist
- Scale rate limit window

## Escalation Matrix

| Severity | Response Time | Escalation Path |
|----------|--------------|-----------------|
| P0 - Critical | 15 min | On-call → Platform Lead → Engineering Director |
| P1 - High | 1 hour | On-call → Platform Lead |
| P2 - Medium | 4 hours | On-call |
| P3 - Low | 24 hours | Ticket queue |
