# OpenTelemetry OTLP Quick Start Guide

This guide helps you get started with the complete OpenTelemetry OTLP integration in under 5 minutes.

## Quick Start (3 Steps)

### 1. Start Jaeger with OTLP Support

```bash
docker run -d --name jaeger \
  -e COLLECTOR_OTLP_ENABLED=true \
  -p 4317:4317 \
  -p 16686:16686 \
  jaegertracing/all-in-one:1.51
```

### 2. Set Environment Variables

```bash
export OTEL_EXPORTER_OTLP_ENDPOINT=http://localhost:4317
export SERVICE_NAME=my-service
export RUST_ENV=development
```

### 3. Run Your Application

```rust
use media_gateway_core::telemetry::{TracingConfig, init_tracing};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let config = TracingConfig::from_env();
    init_tracing(config).await?;

    // Your code here
    tracing::info!("Service started with OTLP tracing");

    Ok(())
}
```

**View traces**: http://localhost:16686

---

## Full Example with Actix Web

### Start All Services

```bash
cd examples
docker-compose -f docker-compose.telemetry.yml up -d
```

This starts:
- ✅ Jaeger (traces) - http://localhost:16686
- ✅ PostgreSQL (demo database)
- ✅ Redis (demo cache)
- ✅ Prometheus (metrics) - http://localhost:9090
- ✅ Grafana (dashboards) - http://localhost:3000

### Run Example Application

```bash
# Set environment
export OTEL_EXPORTER_OTLP_ENDPOINT=http://localhost:4317
export SERVICE_NAME=example-service
export RUST_ENV=development

# Run example
cargo run --example telemetry_otlp_example
```

### Generate Sample Traces

```bash
# Health check
curl http://localhost:8080/health

# User endpoint (triggers DB + external API spans)
curl http://localhost:8080/api/users/123

# Session endpoint (triggers Redis span)
curl http://localhost:8080/api/sessions/abc123

# Checkout endpoint (triggers complex nested spans)
curl -X POST http://localhost:8080/api/checkout/cart_456
```

### View Results

1. **Jaeger UI**: http://localhost:16686
   - Service: `example-service`
   - Operation: `http.request`
   - Look for nested spans showing DB, Redis, and API calls

2. **Grafana**: http://localhost:3000
   - Default credentials: `admin` / `admin`
   - Explore traces and metrics

---

## Environment Variables Reference

| Variable | Required | Default | Description |
|----------|----------|---------|-------------|
| `OTEL_EXPORTER_OTLP_ENDPOINT` | ✅ Yes | `http://localhost:4317` | OTLP gRPC endpoint |
| `SERVICE_NAME` | ⚠️ Recommended | `media-gateway` | Service identifier |
| `RUST_ENV` | ❌ No | `development` | Environment (`production` = 10% sampling) |
| `OTEL_SAMPLING_RATE` | ❌ No | `1.0` (dev) / `0.1` (prod) | Override sampling (0.0-1.0) |
| `OTEL_CONSOLE_ENABLED` | ❌ No | `true` | Enable console logging |

---

## Production Deployment

### Kubernetes with Jaeger Operator

```yaml
apiVersion: v1
kind: ConfigMap
metadata:
  name: app-config
data:
  OTEL_EXPORTER_OTLP_ENDPOINT: "http://jaeger-collector:4317"
  SERVICE_NAME: "api-gateway"
  RUST_ENV: "production"
  OTEL_SAMPLING_RATE: "0.1"  # 10% sampling
---
apiVersion: apps/v1
kind: Deployment
metadata:
  name: api-gateway
spec:
  replicas: 3
  template:
    spec:
      containers:
      - name: api
        image: media-gateway-api:latest
        envFrom:
        - configMapRef:
            name: app-config
        ports:
        - containerPort: 8080
```

### AWS ECS with X-Ray

```bash
# Use AWS Distro for OpenTelemetry (ADOT) Collector
export OTEL_EXPORTER_OTLP_ENDPOINT=http://localhost:4317
export OTEL_PROPAGATORS=tracecontext,xray
export OTEL_RESOURCE_ATTRIBUTES="service.name=api-gateway,deployment.environment=production"
```

### Google Cloud Run

```bash
gcloud run deploy api-gateway \
  --image gcr.io/project/api-gateway:latest \
  --set-env-vars="OTEL_EXPORTER_OTLP_ENDPOINT=http://otel-collector:4317" \
  --set-env-vars="SERVICE_NAME=api-gateway" \
  --set-env-vars="RUST_ENV=production"
```

---

## Sampling Strategies

### Development (100% sampling)
```bash
export RUST_ENV=development
# All traces captured
```

### Staging (50% sampling)
```bash
export RUST_ENV=staging
export OTEL_SAMPLING_RATE=0.5
# 50% of traces captured
```

### Production (10% sampling)
```bash
export RUST_ENV=production
# Automatic 10% sampling
# Override with OTEL_SAMPLING_RATE if needed
```

---

## Troubleshooting

### No traces appearing in Jaeger

**Check 1**: Verify OTLP endpoint is accessible
```bash
telnet localhost 4317
# Should connect successfully
```

**Check 2**: Enable console logging
```bash
export OTEL_CONSOLE_ENABLED=true
cargo run --example telemetry_otlp_example
# Should see trace logs in console
```

**Check 3**: Force 100% sampling
```bash
export OTEL_SAMPLING_RATE=1.0
```

**Check 4**: Check Jaeger logs
```bash
docker logs jaeger
# Look for "OTLP gRPC receiver started"
```

### Connection refused errors

```
Error: Failed to initialize OTLP exporter: connection refused
```

**Solution**: Ensure Jaeger is running
```bash
docker ps | grep jaeger
# Should show running container

# Restart if needed
docker restart jaeger
```

### Missing trace context across services

**Solution**: Ensure TracingMiddleware is added
```rust
App::new()
    .wrap(TracingMiddleware)  // Must be first wrapper
    .wrap(other_middleware)
```

### High memory usage

**Solution**: Reduce batch size
```rust
// In tracing.rs, adjust BatchConfig:
.with_max_queue_size(1024)    // Default: 2048
.with_max_export_batch_size(256)  // Default: 512
```

---

## Performance Tips

### 1. Use Appropriate Sampling

- **Dev/Test**: 100% (`OTEL_SAMPLING_RATE=1.0`)
- **Staging**: 25-50% (`OTEL_SAMPLING_RATE=0.25`)
- **Production**: 5-10% (`OTEL_SAMPLING_RATE=0.1`)

### 2. Limit Span Attributes

```rust
// ❌ Too many attributes
let _span = create_span("op", &[
    ("attr1", "val1"),
    ("attr2", "val2"),
    // ... 20 more attributes
]);

// ✅ Key attributes only
let _span = create_span("op", &[
    ("user_id", user_id),
    ("transaction_id", tx_id),
]);
```

### 3. Truncate Large Values

```rust
// Already implemented in db_query_span
// Queries > 200 chars are truncated
```

### 4. Use Batch Export

```rust
// Already configured in init_tracing
// Spans batched every 5s or 512 spans
```

---

## Next Steps

1. ✅ **Basic Setup**: Follow Quick Start above
2. 📊 **Add Custom Spans**: Use `create_span()` for business operations
3. 🔍 **Database Tracing**: Use `db_query_span()` for SQL queries
4. 🚀 **Redis Tracing**: Use `redis_op_span()` for cache operations
5. 🌐 **API Tracing**: Use `external_api_span()` for HTTP calls
6. 📈 **Production**: Deploy with appropriate sampling rate

---

## Resources

- 📖 [Full Documentation](../docs/telemetry-otlp-integration.md)
- 🔬 [Example Code](./telemetry_otlp_example.rs)
- 🐳 [Docker Compose](./docker-compose.telemetry.yml)
- 🌐 [OpenTelemetry Docs](https://opentelemetry.io/docs/)
- 📊 [Jaeger Docs](https://www.jaegertracing.io/docs/)

---

## Quick Commands Cheatsheet

```bash
# Start Jaeger
docker run -d -e COLLECTOR_OTLP_ENABLED=true -p 4317:4317 -p 16686:16686 jaegertracing/all-in-one:1.51

# Set environment
export OTEL_EXPORTER_OTLP_ENDPOINT=http://localhost:4317
export SERVICE_NAME=my-service

# Run example
cargo run --example telemetry_otlp_example

# View traces
open http://localhost:16686

# Stop Jaeger
docker stop jaeger && docker rm jaeger
```

**Happy Tracing! 🎉**
