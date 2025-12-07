# Media Gateway Incident Response Procedures

## Severity Definitions

| Severity | Definition | Example | Response SLA |
|----------|------------|---------|--------------|
| P0 | Complete service outage | All APIs returning 5xx | 15 minutes |
| P1 | Major degradation | >10% error rate | 1 hour |
| P2 | Minor degradation | Single service issues | 4 hours |
| P3 | Minimal impact | Non-critical bugs | 24 hours |

## Incident Commander Responsibilities

1. **Assess** the situation and assign severity
2. **Communicate** to stakeholders
3. **Coordinate** response team
4. **Document** timeline and actions
5. **Resolve** and verify fix
6. **Review** in post-mortem

## Response Procedure

### Phase 1: Detection (0-5 min)
- [ ] Alert received via PagerDuty/Slack
- [ ] Acknowledge alert
- [ ] Initial assessment of impact
- [ ] Assign severity level

### Phase 2: Triage (5-15 min)
- [ ] Identify affected components
- [ ] Check recent deployments
- [ ] Review error logs
- [ ] Check dependency health

### Phase 3: Mitigation (15-60 min)
- [ ] Implement immediate fix or rollback
- [ ] Communicate status to stakeholders
- [ ] Monitor recovery metrics
- [ ] Verify fix is working

### Phase 4: Resolution (1-4 hours)
- [ ] Confirm full service restoration
- [ ] Update status page
- [ ] Notify stakeholders of resolution
- [ ] Document incident timeline

### Phase 5: Post-Incident (24-48 hours)
- [ ] Create incident report
- [ ] Schedule post-mortem meeting
- [ ] Identify action items
- [ ] Update runbooks

## Communication Templates

### Initial Alert
```
🚨 INCIDENT: [Title]
Severity: P[0-3]
Status: Investigating
Impact: [Description of user impact]
Start Time: [HH:MM UTC]
Incident Commander: [Name]
```

### Status Update
```
📢 UPDATE: [Title]
Status: [Investigating/Identified/Monitoring/Resolved]
Current Impact: [Description]
Actions Taken: [What we've done]
Next Steps: [What we're doing next]
ETA: [If known]
```

### Resolution Notice
```
✅ RESOLVED: [Title]
Duration: [X hours Y minutes]
Root Cause: [Brief description]
Resolution: [What fixed it]
Post-mortem: [Scheduled for DATE]
```

## On-Call Rotation

Week rotation schedule managed in PagerDuty.

Primary: First responder
Secondary: Backup if primary unavailable
Escalation: Platform team lead

## Useful Commands During Incidents

```bash
# Quick status check
kubectl get pods -n media-gateway
kubectl top pods -n media-gateway

# Recent logs
kubectl logs -n media-gateway deploy/api-gateway --since=5m -f

# Restart problematic service
kubectl rollout restart deployment/api-gateway -n media-gateway

# Scale up quickly
kubectl scale deployment/api-gateway -n media-gateway --replicas=5

# Check all health endpoints
./scripts/health-check-all.sh
```
