#!/bin/bash
# Media Gateway Local Development Setup Script
# Run: chmod +x scripts/dev-setup.sh && ./scripts/dev-setup.sh

set -e

echo "=== Media Gateway Development Setup ==="

# Check dependencies
command -v docker >/dev/null 2>&1 || { echo "Docker is required but not installed. Aborting."; exit 1; }
command -v docker-compose >/dev/null 2>&1 || { echo "Docker Compose is required but not installed. Aborting."; exit 1; }

# Create .env if it doesn't exist
if [ ! -f .env ]; then
    echo "Creating .env from .env.example..."
    cp .env.example .env
    echo "Please update .env with your API keys and configuration."
fi

# Start services
echo "Starting Docker services..."
docker-compose up -d

# Wait for services to be healthy
echo "Waiting for services to be healthy..."

# Wait for PostgreSQL
echo -n "Waiting for PostgreSQL..."
until docker-compose exec -T postgres pg_isready -U mediagateway -d media_gateway >/dev/null 2>&1; do
    echo -n "."
    sleep 2
done
echo " Ready!"

# Wait for Redis
echo -n "Waiting for Redis..."
until docker-compose exec -T redis redis-cli ping >/dev/null 2>&1; do
    echo -n "."
    sleep 2
done
echo " Ready!"

# Wait for Qdrant
echo -n "Waiting for Qdrant..."
until curl -sf http://localhost:6333/health >/dev/null 2>&1; do
    echo -n "."
    sleep 2
done
echo " Ready!"

echo ""
echo "=== All services are running! ==="
echo ""
echo "Service endpoints:"
echo "  PostgreSQL: localhost:5432"
echo "  Redis:      localhost:6379"
echo "  Qdrant:     localhost:6333 (REST), localhost:6334 (gRPC)"
echo ""
echo "To stop services: docker-compose down"
echo "To view logs:     docker-compose logs -f"
