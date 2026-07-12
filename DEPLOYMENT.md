# Deployment Guide

## Production Deployment

### Prerequisites

- Docker and Docker Compose
- PostgreSQL 16+
- Redis 7+
- Access to a Soroban RPC endpoint

### Docker Compose (Production)

```bash
# Set environment variables
cp .env.example .env
# Edit .env with production values

# Start services
docker compose up -d

# Verify health
curl http://localhost:8080/v1/health
```

### Manual Deployment

```bash
# Build the binary
cargo build --release --features db-postgres,cache-redis,telemetry

# Set environment variables
export DATABASE_URL="postgres://..."
export REDIS_URL="redis://..."
export APP_SECRET_KEY="..."
export ENCRYPTION_KEY="..."

# Run migrations
./target/release/astro-orbit-backend migrate

# Start server
./target/release/astro-orbit-backend
```

### Environment Variables

See [.env.example](.env.example) for the complete list of configuration
variables with descriptions.

### Health Checks

The application exposes:
- `GET /v1/health` — Application health (database, redis, stellar)
- `GET /v1/version` — Build version and commit
- `GET /metrics` — Prometheus metrics

### Monitoring

- **Metrics**: Prometheus scraping at `/metrics` (port 8080 or configured METRICS_PORT)
- **Tracing**: OpenTelemetry OTLP export
- **Logs**: Structured JSON to stdout

### Kubernetes (Future)

Helm charts and K8s manifests will be added in a future release.
