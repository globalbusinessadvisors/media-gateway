# BATCH_013_TASKS.md

**Generated**: 2025-12-07
**Status**: COMPLETE
**Completed**: 2025-12-07
**Prerequisites**: BATCH_012 complete (test infrastructure, production hardening verified)
**Focus**: Production Deployment Readiness, API Documentation, Security Hardening, Performance Validation
**Analysis Sources**: 9-Agent Claude-Flow Swarm Analysis of SPARC Master Documents, Batches 001-012, All Crate Source Code

---

## Overview

BATCH_013 focuses on production deployment readiness as defined in SPARC Phase 5 (Completion). Tasks are derived from gap analysis comparing current implementation against SPARC requirements, with emphasis on items blocking production launch.

**Priority Legend**:
- P0: Critical/Blocking (Must complete before production)
- P1: High Priority (Required within 30 days post-launch)
- P2: Medium Priority (Required within 90 days post-launch)

---

## TASK-001: Implement OpenAPI 3.1 Documentation with Utoipa

**Priority**: P0-BLOCKING
**Estimated Effort**: 8 hours
**Dependencies**: None

### Description
Add OpenAPI 3.1 specification generation using utoipa crate. All API Gateway routes currently lack machine-readable documentation, preventing SDK generation and interactive API exploration.

### Files to Modify
- `/workspaces/media-gateway/crates/api/Cargo.toml` - Add utoipa, utoipa-swagger-ui dependencies
- `/workspaces/media-gateway/crates/api/src/routes/user.rs` - Add OpenAPI annotations
- `/workspaces/media-gateway/crates/api/src/routes/content.rs` - Add OpenAPI annotations
- `/workspaces/media-gateway/crates/api/src/routes/search.rs` - Add OpenAPI annotations
- `/workspaces/media-gateway/crates/api/src/routes/discover.rs` - Add OpenAPI annotations
- `/workspaces/media-gateway/crates/api/src/routes/playback.rs` - Add OpenAPI annotations
- `/workspaces/media-gateway/crates/api/src/routes/sona.rs` - Add OpenAPI annotations
- `/workspaces/media-gateway/crates/api/src/routes/sync.rs` - Add OpenAPI annotations
- `/workspaces/media-gateway/crates/api/src/server.rs` - Mount Swagger UI at /swagger-ui

### Implementation Pattern
```rust
// Cargo.toml additions
utoipa = { version = "4", features = ["actix_extras"] }
utoipa-swagger-ui = { version = "6", features = ["actix-web"] }

// Route annotation example
#[utoipa::path(
    get,
    path = "/api/v1/user/profile",
    responses(
        (status = 200, description = "User profile retrieved", body = UserProfile),
        (status = 401, description = "Unauthorized")
    ),
    security(("bearer_auth" = []))
)]
async fn get_profile(...) -> impl Responder { ... }
```

### Acceptance Criteria
- [x] `utoipa` and `utoipa-swagger-ui` added to Cargo.toml
- [x] All 35+ API endpoints have OpenAPI annotations
- [x] Swagger UI accessible at `/swagger-ui`
- [x] OpenAPI JSON exportable at `/api-docs/openapi.json`
- [x] Request/response schemas documented for all endpoints

---

## TASK-002: Implement Security Headers Middleware

**Priority**: P0-BLOCKING
**Estimated Effort**: 4 hours
**Dependencies**: None

### Description
Create security headers middleware to add HSTS, CSP, X-Frame-Options, and other security headers. Current API exposes endpoints without essential browser security protections (OWASP A05:2021 - Security Misconfiguration).

### Files to Create
- `/workspaces/media-gateway/crates/api/src/middleware/security_headers.rs` - Security headers middleware

### Files to Modify
- `/workspaces/media-gateway/crates/api/src/middleware/mod.rs` - Export security_headers module
- `/workspaces/media-gateway/crates/api/src/server.rs` - Apply SecurityHeaders middleware

### Implementation
```rust
pub struct SecurityHeaders;

impl<S, B> Transform<S, ServiceRequest> for SecurityHeaders
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error>,
{
    fn new_transform(&self, service: S) -> Self::Future {
        // Add headers: Strict-Transport-Security, Content-Security-Policy,
        // X-Frame-Options, X-Content-Type-Options, Referrer-Policy, Permissions-Policy
    }
}
```

### Acceptance Criteria
- [x] Strict-Transport-Security header with max-age=31536000; includeSubDomains
- [x] X-Frame-Options: DENY
- [x] X-Content-Type-Options: nosniff
- [x] Content-Security-Policy configured for API responses
- [x] All responses include security headers (verified via curl)

---

## TASK-003: Fix Overly Permissive CORS Configuration

**Priority**: P0-BLOCKING
**Estimated Effort**: 2 hours
**Dependencies**: None

### Description
Replace `allow_any_origin()` CORS configuration with environment-based allowlist. Current configuration allows any website to make credentialed requests, enabling CSRF and data exfiltration attacks.

### Files to Modify
- `/workspaces/media-gateway/crates/api/src/server.rs:70-81` - Replace CORS configuration

### Current Code (INSECURE)
```rust
let cors = Cors::default()
    .allow_any_origin()
    .allow_any_method()
    .allow_any_header();
```

### Target Code (SECURE)
```rust
let allowed_origins = std::env::var("ALLOWED_ORIGINS")
    .unwrap_or_else(|_| "https://app.mediagateway.io".to_string());

let cors = Cors::default()
    .allowed_origin_fn(move |origin, _req| {
        allowed_origins.split(',').any(|allowed| {
            origin.to_str().map(|o| o == allowed).unwrap_or(false)
        })
    })
    .allowed_methods(vec!["GET", "POST", "PUT", "DELETE", "OPTIONS"])
    .allowed_headers(vec![header::AUTHORIZATION, header::CONTENT_TYPE])
    .supports_credentials()
    .max_age(3600);
```

### Acceptance Criteria
- [x] CORS rejects requests from non-allowlisted origins
- [x] `ALLOWED_ORIGINS` environment variable documented
- [x] Kubernetes manifests updated with ALLOWED_ORIGINS
- [x] Integration test validates CORS enforcement

---

## TASK-004: Implement Graceful Shutdown Handlers

**Priority**: P0-BLOCKING
**Estimated Effort**: 6 hours
**Dependencies**: None

### Description
Add SIGTERM/SIGINT signal handlers with graceful shutdown to all 8 microservices. Current services terminate immediately on pod termination, causing in-flight request failures during deployments.

### Files to Modify
- `/workspaces/media-gateway/crates/api/src/main.rs`
- `/workspaces/media-gateway/crates/discovery/src/main.rs`
- `/workspaces/media-gateway/crates/sona/src/main.rs`
- `/workspaces/media-gateway/crates/auth/src/main.rs`
- `/workspaces/media-gateway/crates/sync/src/main.rs`
- `/workspaces/media-gateway/crates/ingestion/src/main.rs`
- `/workspaces/media-gateway/crates/playback/src/main.rs`
- `/workspaces/media-gateway/crates/mcp-server/src/main.rs`

### Implementation Pattern
```rust
use tokio::signal;
use std::time::Duration;

let server = HttpServer::new(|| { ... })
    .bind(("0.0.0.0", 8080))?
    .shutdown_timeout(30)
    .run();

let shutdown_signal = async {
    signal::unix::signal(signal::unix::SignalKind::terminate())
        .expect("Failed to install SIGTERM handler")
        .recv()
        .await;
    tracing::info!("SIGTERM received, initiating graceful shutdown");
};

tokio::select! {
    result = server => result?,
    _ = shutdown_signal => {
        tracing::info!("Shutdown complete");
    }
}
```

### Acceptance Criteria
- [x] All 8 services handle SIGTERM gracefully
- [x] 30-second shutdown timeout configured
- [x] In-flight requests complete before termination
- [x] Database connections closed cleanly
- [x] Shutdown logged with structured tracing

---

## TASK-005: Complete Health Check Endpoints for All Services

**Priority**: P0-BLOCKING
**Estimated Effort**: 6 hours
**Dependencies**: None

### Description
Implement missing `/ready` and `/liveness` endpoints across all services. Kubernetes probes are configured but endpoints don't exist, causing orchestration failures.

### Files to Modify
- `/workspaces/media-gateway/crates/api/src/routes/health.rs` - Add /ready, /liveness
- `/workspaces/media-gateway/crates/sona/src/health.rs` - Add /ready, /liveness
- `/workspaces/media-gateway/crates/sync/src/health.rs` - Add /ready, /liveness
- `/workspaces/media-gateway/crates/ingestion/src/health.rs` - Add /ready, /liveness

### Files to Create
- `/workspaces/media-gateway/crates/auth/src/health.rs` - Complete health module (currently missing)

### Implementation Requirements
- `/health` - Basic health check (already exists)
- `/ready` - Readiness probe checking all dependencies (DB, Redis, Qdrant)
- `/liveness` - Liveness probe (lightweight, no dependency checks)

### Acceptance Criteria
- [x] All 8 services expose `/health`, `/ready`, `/liveness` endpoints
- [x] Readiness checks verify database connectivity
- [x] Readiness checks verify Redis connectivity
- [x] Liveness returns 200 within 100ms
- [x] Kubernetes probes pass for all services

---

## TASK-006: Execute k6 Performance Tests and Establish Baselines

**Priority**: P0-BLOCKING
**Estimated Effort**: 8 hours
**Dependencies**: Docker Compose services running

### Description
Execute existing k6 performance test scripts and document baseline metrics. Scripts exist but have never been run - no performance baseline data exists to validate SPARC latency requirements.

### Files to Execute
- `/workspaces/media-gateway/tests/performance/k6/baseline.js` - 10K VUs, 1000 RPS, 30 min
- `/workspaces/media-gateway/tests/performance/k6/stress.js` - 20K VUs, 3500 RPS peak
- `/workspaces/media-gateway/tests/performance/k6/spike.js` - 100K VUs sudden load

### Files to Create
- `/workspaces/media-gateway/tests/performance/results/baseline-YYYYMMDD.json` - Baseline results
- `/workspaces/media-gateway/tests/performance/BASELINE_REPORT.md` - Performance baseline documentation

### Performance Targets (from SPARC)
| Service | p95 Target | p99 Target |
|---------|-----------|-----------|
| Search API | < 500ms | < 1s |
| SONA Recommendations | < 5ms | < 10ms |
| Sync Operations | < 100ms | < 200ms |
| Auth Operations | < 50ms | < 100ms |

### Acceptance Criteria
- [x] k6 baseline test completes successfully
- [x] Performance metrics exported to JSON
- [x] Baseline report documents all p50/p95/p99 latencies
- [x] Results compared against SPARC targets
- [x] CI integration for performance regression detection

---

## TASK-007: Implement SLO/SLI Recording Rules and Error Budget Tracking

**Priority**: P1-HIGH
**Estimated Effort**: 6 hours
**Dependencies**: TASK-006 (baseline metrics needed)

### Description
Create Prometheus recording rules for SLI calculation and error budget tracking. SPARC requires 99.9% availability SLO with 43.2 minutes/month error budget.

### Files to Create
- `/workspaces/media-gateway/config/prometheus/recording_rules.yml` - SLI recording rules
- `/workspaces/media-gateway/config/grafana/dashboards/slo-error-budget.json` - SLO dashboard

### Files to Modify
- `/workspaces/media-gateway/config/prometheus.yml` - Include recording_rules.yml

### Recording Rules
```yaml
groups:
  - name: sli_recording_rules
    interval: 30s
    rules:
      - record: sli:availability:ratio_rate5m
        expr: sum(rate(http_requests_total{status!~"5.."}[5m])) / sum(rate(http_requests_total[5m]))

      - record: sli:api_latency:p95_5m
        expr: histogram_quantile(0.95, rate(http_request_duration_seconds_bucket{job="api-gateway"}[5m]))

      - record: slo:error_budget:remaining
        expr: 1 - ((1 - sli:availability:ratio_rate5m) / (1 - 0.999))
```

### Acceptance Criteria
- [x] Recording rules calculate availability SLI
- [x] Recording rules calculate latency SLIs for all services
- [x] Error budget remaining visible in Grafana
- [x] Alert fires when error budget < 25%
- [x] Dashboard shows 30-day rolling SLO compliance

---

## TASK-008: Complete OpenTelemetry Distributed Tracing Integration

**Priority**: P1-HIGH
**Estimated Effort**: 8 hours
**Dependencies**: None

### Description
Implement actual OTLP exporter in tracing initialization. Current code is stubbed for testing - production deployment will have no distributed tracing capability.

### Files to Modify
- `/workspaces/media-gateway/crates/core/Cargo.toml` - Add opentelemetry-otlp dependency
- `/workspaces/media-gateway/crates/core/src/telemetry/tracing.rs` - Implement OTLP exporter
- `/workspaces/media-gateway/docker/docker-compose.yml` - Add OTEL_EXPORTER_OTLP_ENDPOINT env vars

### Dependencies to Add
```toml
opentelemetry = "0.21"
opentelemetry-otlp = { version = "0.14", features = ["tonic"] }
opentelemetry-semantic-conventions = "0.13"
tracing-opentelemetry = "0.22"
```

### Implementation
Replace stubbed `init_tracing()` with actual OTLP exporter:
```rust
let otlp_exporter = opentelemetry_otlp::new_exporter()
    .tonic()
    .with_endpoint(&config.otlp_endpoint)
    .build_span_exporter()?;

let tracer_provider = TracerProvider::builder()
    .with_batch_exporter(otlp_exporter, runtime::Tokio)
    .build();
```

### Acceptance Criteria
- [x] OpenTelemetry crates added to core Cargo.toml
- [x] OTLP exporter sends traces to Jaeger
- [x] All services export traces with service.name attribute
- [x] Traces visible in Jaeger UI
- [x] 10% sampling in production, 100% in development

---

## TASK-009: Create Production Deployment Runbook

**Priority**: P1-HIGH
**Estimated Effort**: 8 hours
**Dependencies**: TASK-004, TASK-005

### Description
Create comprehensive production deployment runbook with step-by-step procedures, rollback instructions, and troubleshooting guide. No deployment documentation currently exists.

### Files to Create
- `/workspaces/media-gateway/docs/runbooks/DEPLOYMENT.md` - Main deployment runbook
- `/workspaces/media-gateway/docs/runbooks/ROLLBACK.md` - Rollback procedures
- `/workspaces/media-gateway/docs/runbooks/TROUBLESHOOTING.md` - Common issues and solutions
- `/workspaces/media-gateway/docs/runbooks/INCIDENT_RESPONSE.md` - On-call procedures

### Runbook Contents
1. **Pre-deployment checklist** - Tests pass, migrations ready, feature flags set
2. **Deployment procedure** - Step-by-step kubectl/ArgoCD commands
3. **Health verification** - How to verify deployment success
4. **Rollback procedure** - How to revert to previous version
5. **Smoke tests** - Critical path verification
6. **Escalation contacts** - On-call rotation and escalation

### Acceptance Criteria
- [x] Deployment runbook covers all 8 microservices
- [x] Rollback procedure tested in staging
- [x] Troubleshooting covers top 10 failure scenarios
- [x] Incident response defines severity levels and SLAs
- [x] Runbook reviewed by ops team

---

## TASK-010: Implement Input Sanitization Layer

**Priority**: P1-HIGH
**Estimated Effort**: 5 hours
**Dependencies**: None

### Description
Add HTML sanitization for user-generated content to prevent XSS attacks. No sanitization exists - search queries and user input rendered without encoding.

### Files to Create
- `/workspaces/media-gateway/crates/core/src/sanitization.rs` - Sanitization utilities

### Files to Modify
- `/workspaces/media-gateway/crates/core/Cargo.toml` - Add ammonia dependency
- `/workspaces/media-gateway/crates/core/src/lib.rs` - Export sanitization module
- `/workspaces/media-gateway/crates/discovery/src/search.rs` - Sanitize search queries
- `/workspaces/media-gateway/crates/api/src/routes/user.rs` - Sanitize user profile updates

### Implementation
```rust
// Cargo.toml
ammonia = "3.3"

// sanitization.rs
pub fn sanitize_html(input: &str) -> String {
    ammonia::clean(input)
}

pub fn sanitize_search_query(input: &str) -> String {
    input
        .chars()
        .filter(|c| c.is_alphanumeric() || c.is_whitespace() || *c == '-' || *c == '_')
        .take(256)
        .collect()
}
```

### Acceptance Criteria
- [x] `ammonia` crate added to core dependencies
- [x] Search queries sanitized before processing
- [x] User profile updates sanitized
- [x] XSS payloads in tests are neutralized
- [x] No raw HTML rendered in API responses

---

## TASK-011: Deploy Loki for Centralized Log Aggregation

**Priority**: P1-HIGH
**Estimated Effort**: 5 hours
**Dependencies**: None

### Description
Deploy Loki and Promtail for centralized log aggregation. Currently no log aggregation exists - debugging requires manual container inspection across 8 services.

### Files to Create
- `/workspaces/media-gateway/config/loki/loki-config.yml` - Loki configuration
- `/workspaces/media-gateway/config/promtail/promtail-config.yml` - Log collection config
- `/workspaces/media-gateway/config/grafana/dashboards/logs-explorer.json` - Log dashboard

### Files to Modify
- `/workspaces/media-gateway/docker/docker-compose.yml` - Add Loki, Promtail services
- `/workspaces/media-gateway/config/grafana/provisioning/datasources/prometheus.yml` - Add Loki datasource

### Docker Compose Addition
```yaml
loki:
  image: grafana/loki:2.9.3
  ports:
    - "3100:3100"
  volumes:
    - ./config/loki/loki-config.yml:/etc/loki/local-config.yaml
  command: -config.file=/etc/loki/local-config.yaml

promtail:
  image: grafana/promtail:2.9.3
  volumes:
    - /var/lib/docker/containers:/var/lib/docker/containers:ro
    - ./config/promtail/promtail-config.yml:/etc/promtail/config.yml
```

### Acceptance Criteria
- [x] Loki accessible on port 3100
- [x] Promtail collecting logs from all containers
- [x] Logs queryable in Grafana Explore
- [x] Log labels include service name and level
- [x] 7-day log retention configured

---

## TASK-012: Complete Rate Limiting Coverage for All Routes

**Priority**: P1-HIGH
**Estimated Effort**: 3 hours
**Dependencies**: None

### Description
Apply rate limiting middleware to currently unprotected routes (playback, sona, sync). Only user, content, search, discover routes have rate limiting - remaining routes can be abused.

### Files to Modify
- `/workspaces/media-gateway/crates/api/src/routes/playback.rs` - Add rate limiting
- `/workspaces/media-gateway/crates/api/src/routes/sona.rs` - Add rate limiting
- `/workspaces/media-gateway/crates/api/src/routes/sync.rs` - Add rate limiting
- `/workspaces/media-gateway/crates/api/src/routes/health.rs` - Exempt from rate limiting

### Rate Limits by Route
| Route Group | Requests/min | Burst |
|-------------|--------------|-------|
| /api/v1/playback/* | 60 | 10 |
| /api/v1/sona/* | 30 | 5 |
| /api/v1/sync/* | 120 | 20 |
| /health, /ready, /liveness | Exempt | - |

### Acceptance Criteria
- [x] Playback routes rate limited to 60 req/min
- [x] SONA routes rate limited to 30 req/min
- [x] Sync routes rate limited to 120 req/min
- [x] Health endpoints exempt from rate limiting
- [x] Rate limit headers returned (X-RateLimit-Remaining)

---

## Verification Checklist

After completing all BATCH_013 tasks:

```bash
# 1. API Documentation
curl http://localhost:8080/swagger-ui | grep "Swagger UI"
curl http://localhost:8080/api-docs/openapi.json | jq '.paths | keys | length'

# 2. Security Headers
curl -I http://localhost:8080/health | grep -E "Strict-Transport|X-Frame-Options|X-Content-Type"

# 3. CORS Verification
curl -H "Origin: https://evil.com" -I http://localhost:8080/api/v1/user/profile

# 4. Health Endpoints
for port in 8080 8081 8082 8083 8084 8085 8086 8087; do
  curl -s http://localhost:$port/ready | jq '.status'
done

# 5. Performance Baseline
cd tests/performance && k6 run k6/baseline.js --out json=results/baseline.json

# 6. Distributed Tracing
curl http://localhost:16686/api/services | jq '.data[]'

# 7. Log Aggregation
curl http://localhost:3100/ready
```

---

## Success Metrics

| Metric | Target | Measurement |
|--------|--------|-------------|
| OpenAPI endpoints documented | 35+ | `openapi.json` paths count |
| Security headers present | 5 headers | Response header check |
| Services with graceful shutdown | 8/8 | SIGTERM handling test |
| Health endpoints complete | 24 (8×3) | Endpoint availability |
| Performance baseline | Documented | k6 results JSON |
| Traces in Jaeger | All services | Jaeger service list |
| Logs in Loki | All services | Grafana query |

---

## Next Batch Preview (BATCH_014)

Based on remaining SPARC Completion Phase requirements:
- Service mesh deployment (Istio mTLS, traffic management)
- Disaster recovery testing and multi-region failover
- Chaos engineering tests (Toxiproxy integration)
- Contract testing framework (Pact)
- Security penetration testing execution
- Helm chart migration from Kustomize

---

**Document Version**: 1.0
**Generated By**: 9-Agent Claude-Flow Swarm Analysis
**Analysis Sources**: SPARC Master Documents (5), Batch Task Files (001-012), All Crate Source Code, Infrastructure Configuration
