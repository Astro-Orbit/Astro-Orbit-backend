# Astro Orbit Backend

**Open-source developer platform for the Stellar and Soroban ecosystem.**

[![CI](https://github.com/astro-orbit/astro-orbit-backend/actions/workflows/ci.yml/badge.svg)](https://github.com/astro-orbit/astro-orbit-backend/actions/workflows/ci.yml)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](LICENSE)
[![Rust](https://img.shields.io/badge/rust-1.84%2B-blue)](https://www.rust-lang.org)

---

Astro Orbit provides everything a developer needs to build, deploy, manage,
monitor, and secure Soroban applications from one place. Think of it as
**Vercel + Supabase + GitHub + Foundry**, tailored to Stellar developers.

## Architecture

```
HTTP (Axum Router)
  │
Middleware (Auth, Rate Limit, Logging, Request ID)
  │
Handlers (Parse input, call service, format response)
  │
Services (Business logic, orchestration, transactions)
  │
Repositories (Data access via SQLx traits)
  │
Database (PostgreSQL)
```

See [ARCHITECTURE.md](ARCHITECTURE.md) for the complete engineering handbook.

## Quick Start

### Prerequisites

- Rust 1.84+
- PostgreSQL 16+
- Redis 7+
- Docker (optional, for containerized development)

### Setup

```bash
# Clone the repository
git clone https://github.com/astro-orbit/astro-orbit-backend.git
cd astro-orbit-backend

# Copy environment configuration
cp .env.example .env

# Start infrastructure services
docker compose -f docker-compose.dev.yml up -d db redis

# Run database migrations
./scripts/init_db.sh

# Start the development server
cargo run
```

The server starts at `http://localhost:8080`. Health check: `GET /v1/health`.

## Project Status

Astro Orbit is in **Phase 0 (Foundation)**. See [ROADMAP.md](ROADMAP.md) for
the complete development plan.

## Documentation

| Document | Description |
|----------|-------------|
| [ARCHITECTURE.md](ARCHITECTURE.md) | Engineering handbook and architecture reference |
| [CONTRIBUTING.md](CONTRIBUTING.md) | Contribution workflow and conventions |
| [DEPLOYMENT.md](DEPLOYMENT.md) | Deployment and operations guide |
| [SECURITY.md](SECURITY.md) | Security model and vulnerability reporting |
| [ROADMAP.md](ROADMAP.md) | Development milestones and priorities |
| [CHANGELOG.md](CHANGELOG.md) | Release history |

## License

This project is licensed under the MIT License — see [LICENSE](LICENSE).
