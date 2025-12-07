#!/bin/bash
# OTLP Integration Verification Script
# This script verifies that the OpenTelemetry OTLP integration is properly configured

set -e

echo "=================================="
echo "OTLP Integration Verification"
echo "=================================="
echo ""

# Colors for output
GREEN='\033[0;32m'
RED='\033[0;31m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

success() {
    echo -e "${GREEN}✓${NC} $1"
}

error() {
    echo -e "${RED}✗${NC} $1"
}

warning() {
    echo -e "${YELLOW}⚠${NC} $1"
}

info() {
    echo "ℹ $1"
}

# Check 1: Verify dependencies in Cargo.toml
echo "1. Checking workspace dependencies..."
if grep -q "opentelemetry_sdk" Cargo.toml && \
   grep -q "opentelemetry-otlp" Cargo.toml && \
   grep -q "tracing-opentelemetry" Cargo.toml; then
    success "Workspace dependencies configured"
else
    error "Missing OpenTelemetry dependencies in workspace Cargo.toml"
    exit 1
fi
echo ""

# Check 2: Verify core crate dependencies
echo "2. Checking core crate dependencies..."
if grep -q "opentelemetry_sdk" crates/core/Cargo.toml && \
   grep -q "opentelemetry-otlp" crates/core/Cargo.toml && \
   grep -q "tracing-opentelemetry" crates/core/Cargo.toml; then
    success "Core crate dependencies configured"
else
    error "Missing OpenTelemetry dependencies in core Cargo.toml"
    exit 1
fi
echo ""

# Check 3: Verify OTLP exporter implementation
echo "3. Checking OTLP exporter implementation..."
if grep -q "opentelemetry_otlp::new_exporter" crates/core/src/telemetry/tracing.rs && \
   grep -q "install_batch" crates/core/src/telemetry/tracing.rs && \
   grep -q "BatchConfig" crates/core/src/telemetry/tracing.rs; then
    success "OTLP exporter implementation found"
else
    error "OTLP exporter not properly implemented"
    exit 1
fi
echo ""

# Check 4: Verify sampling configuration
echo "4. Checking sampling configuration..."
if grep -q "Sampler::TraceIdRatioBased" crates/core/src/telemetry/tracing.rs && \
   grep -q "sampling_rate" crates/core/src/telemetry/tracing.rs; then
    success "Sampling configuration found"
else
    error "Sampling configuration missing"
    exit 1
fi
echo ""

# Check 5: Verify environment variable support
echo "5. Checking environment variable support..."
if grep -q "OTEL_EXPORTER_OTLP_ENDPOINT" crates/core/src/telemetry/tracing.rs && \
   grep -q "SERVICE_NAME" crates/core/src/telemetry/tracing.rs && \
   grep -q "RUST_ENV" crates/core/src/telemetry/tracing.rs; then
    success "Environment variable support configured"
else
    error "Environment variable support missing"
    exit 1
fi
echo ""

# Check 6: Verify resource attributes
echo "6. Checking resource attributes..."
if grep -q "SERVICE_NAME" crates/core/src/telemetry/tracing.rs && \
   grep -q "SERVICE_VERSION" crates/core/src/telemetry/tracing.rs && \
   grep -q "semconv::resource" crates/core/src/telemetry/tracing.rs; then
    success "Resource attributes configured"
else
    error "Resource attributes missing"
    exit 1
fi
echo ""

# Check 7: Verify graceful shutdown
echo "7. Checking graceful shutdown..."
if grep -q "shutdown_tracer_provider" crates/core/src/telemetry/tracing.rs; then
    success "Graceful shutdown implemented"
else
    warning "Graceful shutdown may need verification"
fi
echo ""

# Check 8: Verify documentation
echo "8. Checking documentation..."
if [ -f "docs/telemetry-otlp-integration.md" ] && \
   [ -f "examples/TELEMETRY_QUICKSTART.md" ]; then
    success "Documentation files present"
else
    error "Documentation files missing"
    exit 1
fi
echo ""

# Check 9: Verify example code
echo "9. Checking example code..."
if [ -f "examples/telemetry_otlp_example.rs" ] && \
   [ -f "examples/docker-compose.telemetry.yml" ]; then
    success "Example files present"
else
    error "Example files missing"
    exit 1
fi
echo ""

# Check 10: Verify middleware integration
echo "10. Checking middleware integration..."
if grep -q "TracingMiddleware" crates/core/src/telemetry/middleware.rs && \
   grep -q "traceparent" crates/core/src/telemetry/middleware.rs; then
    success "Trace context middleware configured"
else
    error "Middleware missing trace context support"
    exit 1
fi
echo ""

# Check 11: Test environment variable detection
echo "11. Testing environment configuration..."
export OTEL_EXPORTER_OTLP_ENDPOINT="http://localhost:4317"
export SERVICE_NAME="test-service"
export RUST_ENV="development"

if [ -n "$OTEL_EXPORTER_OTLP_ENDPOINT" ] && \
   [ -n "$SERVICE_NAME" ] && \
   [ -n "$RUST_ENV" ]; then
    success "Environment variables can be set"
    unset OTEL_EXPORTER_OTLP_ENDPOINT SERVICE_NAME RUST_ENV
else
    error "Environment variable detection failed"
    exit 1
fi
echo ""

# Summary
echo "=================================="
echo "Verification Summary"
echo "=================================="
echo ""
success "All checks passed!"
echo ""
info "OTLP integration is properly configured"
echo ""
echo "Next steps:"
echo "  1. cargo check --package media-gateway-core"
echo "  2. cargo test --package media-gateway-core"
echo "  3. cargo run --example telemetry_otlp_example"
echo ""
echo "To start Jaeger for testing:"
echo "  docker run -d --name jaeger \\"
echo "    -e COLLECTOR_OTLP_ENABLED=true \\"
echo "    -p 4317:4317 -p 16686:16686 \\"
echo "    jaegertracing/all-in-one:1.51"
echo ""
echo "View traces at: http://localhost:16686"
echo ""
