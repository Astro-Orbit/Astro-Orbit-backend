# Astro Orbit

[![CI](https://github.com/astro-orbit/astro-orbit-backend/actions/workflows/ci.yml/badge.svg)](https://github.com/astro-orbit/astro-orbit-backend/actions/workflows/ci.yml)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](LICENSE)
[![Rust](https://img.shields.io/badge/rust-1.84%2B-blue)](https://www.rust-lang.org)

**Backend — REST API, Soroban RPC, job queue, and event bus for the Astro Orbit platform.**

Open-source developer platform for the Stellar and Soroban ecosystem.

## Overview

Astro Orbit provides everything a developer needs to build, deploy, manage, monitor, and secure Soroban applications from one place. Think of it as **Vercel + Supabase + GitHub + Foundry**, tailored to Stellar developers.

The backend is the orchestration layer: it serves the REST API consumed by the frontend, manages deployment pipelines via a state machine, queues background work with retries and DLQ, emits and consumes domain events, and communicates with Soroban contracts through a typed JSON-RPC client.

## Features

- **REST API**: Full CRUD for organizations, projects, contracts, deployments, repositories, users, and API keys
- **Soroban RPC Client**: Typed JSON-RPC transport — `getLatestLedger`, `getContractData`, `simulateTransaction`, `sendTransaction`, event subscriptions
- **Registry Client**: Cached on-chain contract lookups via the Registry Soroban contract
- **Deployment Pipeline**: State machine — `pending → building → deploying → completed | failed`, with rollback and cancellation
- **Deployment Validator**: Pre-flight validation — network checks, WASM binary verification, gas estimation
- **Event Bus**: In-process pub-sub — services emit typed events, registered handlers react asynchronously
- **Job Queue**: PostgreSQL-backed priority queue (PgJobQueue) with scheduled execution, retry policies, max attempts, and dead-letter routing
- **Background Workers**: Long-running processes that poll the queue, execute deployment steps, index Soroban events, and dispatch notifications
- **Stellar Error Types**: Structured domain errors for failed Stellar operations (insufficient fee, bad sequence, contract error, etc.)
- **SEP-10 Authentication**: Challenge-response flow with JWT issuance, refresh, and revocation
- **Rate Limiting**: Configurable per-endpoint and per-IP rate limits with Redis-backed counters
- **OpenTelemetry**: Distributed tracing (OTLP), metrics endpoint, structured JSON logging

## Architecture

```mermaid
graph TB
    subgraph Astro_Orbit_Platform
        FE["Astro-Orbit Frontend<br/>Next.js 16 · React 19"]
        BE["Astro-Orbit Backend"]
        CON["Astro-Orbit Contracts<br/>Soroban SDK 27"]
    end

    subgraph Backend_Layers
        HTTP[Axum HTTP · Port 8080]
        MIDDLEWARE[Auth · CORS · Rate Limit · Tracing · Request ID]
        HANDLERS["Handlers<br/>Parse · Validate · Respond"]
        SERVICES["Services<br/>Business Logic · Orchestration"]
        REPOS["Repositories<br/>SQLx · PostgreSQL"]
        RPC[Soroban RPC Client · Typed JSON-RPC]
        REG[Registry Client · On-Chain Lookup Cache]
        DEPLOY[Deployment Pipeline · State Machine]
        EVENTS[Event Bus · Broadcast · Handler Dispatch]
        JOBQ[Job Queue · PgJobQueue · DLQ]
        WORKER[Background Workers · Poll · Execute · Retry]
    end

    subgraph Storage_
        DB[("PostgreSQL 16+")]
        REDIS[("Redis 7+")]
    end

    FE -->|HTTP REST| HTTP
    HTTP --> MIDDLEWARE
    MIDDLEWARE --> HANDLERS
    HANDLERS --> SERVICES
    SERVICES --> REPOS
    REPOS --> DB
    SERVICES --> REDIS

    SERVICES --> DEPLOY
    SERVICES --> RPC
    RPC -->|JSON-RPC| CON

    REG -->|query| RPC
    RPC -->|getLatestLedger| CON
    RPC -->|getContractData| CON
    RPC -->|simulateTransaction| CON
    RPC -->|sendTransaction| CON

    SERVICES -->|emit| EVENTS
    EVENTS -->|dispatch| SERVICES
    EVENTS -->|enqueue| JOBQ

    SERVICES -->|schedule| JOBQ
    JOBQ --> DB
    WORKER -->|poll| JOBQ
    WORKER -->|execute| SERVICES
    WORKER -->|retry / DLQ| JOBQ

    subgraph Telemetry
        METRICS["Metrics · Port 9090"]
        TRACES["OTLP Exporter"]
        LOGS["Structured JSON Logs"]
    end

    HTTP --> METRICS
    SERVICES --> TRACES
    HANDLERS --> LOGS
```

### Core Components

- **HTTP Layer (Axum)**: Router with typed path parameters, query extractors, JSON body deserialization. Middleware stack: auth (JWT), CORS, rate limiting, request tracing, correlation ID injection.
- **Handlers**: Thin layer — parse request, validate input, call service, format response. No business logic.
- **Services**: Business logic, orchestration, transaction boundaries. Coordinate between handlers, repositories, RPC client, and event bus.
- **Repositories**: Data access via SQLx traits. Each domain (orgs, projects, contracts, deployments, users) has its own repository with dedicated query methods.
- **Soroban RPC Client**: Typed JSON-RPC client wrapping `reqwest`. Methods for ledger queries, contract data retrieval, transaction simulation, and submission. Handles retry, timeout, and error mapping to typed Stellar errors.
- **Registry Client**: Cached wrapper around the Registry Soroban contract. Stores contract address → metadata mappings in Redis with configurable TTL.
- **Deployment Pipeline**: State machine with validated transitions. Each stage has a dedicated handler — build (WASM compilation), scan (security), deploy (Soroban tx), verify (on-chain check). Failed stages can be retried; the pipeline supports rollback to a previous successful deployment.
- **Event Bus**: In-process `tokio::broadcast`-based pub-sub. Services emit strongly-typed events (e.g., `DeploymentStateChanged { id, from, to }`). Handlers register for event types and run asynchronously.
- **Job Queue**: `PgJobQueue` backed by PostgreSQL. Jobs have priority, scheduled execution time, retry policy (max attempts, backoff strategy), and a dead-letter queue for permanently failed jobs.
- **Background Workers**: Tokio tasks that poll the job queue, execute the assigned job handler, and update job status. Handlers exist for: `DeployContract`, `VerifyContract`, `IndexSorobanEvents`, `NotifyDeploymentComplete`.

## REST API Endpoints

### Health

| Method | Path | Auth | Description |
|---|---|---|---|
| `GET` | `/v1/health` | — | Liveness check — returns Stellar RPC status |
| `GET` | `/v1/ready` | — | Readiness probe — checks DB, Redis, Stellar |

### Auth

| Method | Path | Auth | Description |
|---|---|---|---|
| `GET` | `/v1/auth/challenge` | — | Get SEP-10 challenge XDR |
| `POST` | `/v1/auth/login` | — | Submit signed challenge, receive JWT |
| `POST` | `/v1/auth/refresh` | Bearer | Refresh access token |
| `POST` | `/v1/auth/logout` | Bearer | Revoke refresh token |

### Users

| Method | Path | Auth | Description |
|---|---|---|---|
| `GET` | `/v1/users/me` | Bearer | Current user profile |
| `PATCH` | `/v1/users/me` | Bearer | Update profile |
| `GET` | `/v1/users/:id` | Bearer | Get user by ID |

### Organizations

| Method | Path | Auth | Description |
|---|---|---|---|
| `POST` | `/v1/organizations` | Bearer | Create organization |
| `GET` | `/v1/organizations` | Bearer | List user's organizations |
| `GET` | `/v1/organizations/:id` | Bearer | Get organization details |
| `PATCH` | `/v1/organizations/:id` | Bearer (admin) | Update organization |
| `DELETE` | `/v1/organizations/:id` | Bearer (owner) | Delete organization |
| `GET` | `/v1/organizations/:id/members` | Bearer | List members |
| `POST` | `/v1/organizations/:id/invite` | Bearer (admin) | Invite member |

### Projects

| Method | Path | Auth | Description |
|---|---|---|---|
| `POST` | `/v1/orgs/:orgId/projects` | Bearer (admin) | Create project |
| `GET` | `/v1/orgs/:orgId/projects` | Bearer | List projects |
| `GET` | `/v1/projects/:id` | Bearer | Get project details |
| `PATCH` | `/v1/projects/:id` | Bearer (admin) | Update project |
| `DELETE` | `/v1/projects/:id` | Bearer (admin) | Delete project |

### Contracts

| Method | Path | Auth | Description |
|---|---|---|---|
| `POST` | `/v1/projects/:projectId/contracts` | Bearer (admin) | Register contract |
| `GET` | `/v1/projects/:projectId/contracts` | Bearer | List contracts |
| `GET` | `/v1/contracts/:id` | Bearer | Get contract details |
| `GET` | `/v1/contracts/:id/versions` | Bearer | List contract versions |

### Deployments

| Method | Path | Auth | Description |
|---|---|---|---|
| `POST` | `/v1/projects/:projectId/deployments` | Bearer (admin) | Start deployment |
| `GET` | `/v1/projects/:projectId/deployments` | Bearer | List deployments |
| `GET` | `/v1/deployments/:id` | Bearer | Get deployment details |
| `POST` | `/v1/deployments/:id/cancel` | Bearer (admin) | Cancel deployment |
| `POST` | `/v1/deployments/:id/rollback` | Bearer (admin) | Rollback deployment |
| `GET` | `/v1/deployments/:id/logs` | Bearer | Get deployment logs |

### Repositories

| Method | Path | Auth | Description |
|---|---|---|---|
| `POST` | `/v1/projects/:projectId/repositories` | Bearer (admin) | Link repository |
| `GET` | `/v1/projects/:projectId/repositories` | Bearer | List repositories |
| `GET` | `/v1/repositories/:id` | Bearer | Get repository details |
| `POST` | `/v1/repositories/:id/sync` | Bearer (admin) | Trigger sync |
| `DELETE` | `/v1/repositories/:id` | Bearer (admin) | Unlink repository |

### Analytics

| Method | Path | Auth | Description |
|---|---|---|---|
| `GET` | `/v1/orgs/:orgId/analytics/overview` | Bearer | Overview stats |
| `GET` | `/v1/orgs/:orgId/dashboard/stats` | Bearer | Dashboard stats |
| `GET` | `/v1/dashboard/activity` | Bearer | Activity feed |

### API Keys

| Method | Path | Auth | Description |
|---|---|---|---|
| `GET` | `/v1/api-keys` | Bearer | List API keys |
| `POST` | `/v1/api-keys` | Bearer | Create API key |
| `DELETE` | `/v1/api-keys/:id` | Bearer | Revoke API key |

## Tech Stack

| Layer | Technology | Purpose |
|---|---|---|
| Language | Rust 1.84+ | Performance, safety, zero-cost abstractions |
| Web Framework | Axum | Async HTTP router with tower middleware |
| ORM / Query | SQLx | Compile-time checked SQL queries |
| Database | PostgreSQL 16+ | Primary data store |
| Cache | Redis 7+ | Session cache, rate limiting, registry cache |
| RPC Client | reqwest + serde_json | JSON-RPC transport to Soroban RPC |
| Job Queue | PgJobQueue (custom) | PostgreSQL-backed priority queue |
| Event Bus | tokio::broadcast | In-process pub-sub |
| Auth | SEP-10 + JWT (jsonwebtoken) | Stellar wallet authentication |
| Serialization | serde / serde_json | JSON serialization |
| Telemetry | OpenTelemetry + tracing | Distributed tracing, metrics, logging |
| Containerization | Docker (multi-stage) | Production deployment |

## Deployment Pipeline

```
┌──────────┐     ┌──────────┐     ┌──────────┐     ┌──────────┐
│  Pending  │────▶│ Building  │────▶│Deploying │────▶│Completed │
└──────────┘     └──────────┘     └──────────┘     └──────────┘
       │               │               │               │
       ├───────────────┴───────────────┴───────────────┤
       │                    Failed                      │
       └────────────────────────────────────────────────┘
       │               │               │               │
       └───────────────┴───────────────┴───────────────┘
                        Cancelled
```

### Valid State Transitions

| From | To | Trigger | Validation |
|---|---|---|---|
| Pending | Building | Pipeline starts | Org/project exists, contract registered |
| Building | Deploying | WASM build succeeds | Binary hash verified, gas estimated |
| Deploying | Completed | Soroban tx confirmed | Ledger sequence matches, contract deployed |
| Any | Failed | Error at any stage | Error code and message captured |
| Any | Cancelled | User cancels | Only before completed |

### Pipeline Stages

1. **Build**: Compile Rust contract to WASM, run `cargo build --target wasm32-unknown-unknown --release`
2. **Scan**: Security scan of WASM binary (basic entropy check, known pattern detection)
3. **Deploy**: Submit `upload_wasm` + `create_contract` Soroban transactions
4. **Verify**: Confirm contract exists on-chain via `getContractData`

## Job Queue Architecture

```
┌──────────────┐
│   Producer    │  Services enqueue jobs with priority and schedule
└──────┬───────┘
       │
       ▼
┌───────────────────────────────────────┐
│          PostgreSQL Queue             │
│  ┌─────────┐  ┌─────────┐  ┌──────┐  │
│  │ Pending  │  │ Running │  │ DLQ  │  │
│  │ (sorted) │  │ (locked)│  │(dead)│  │
│  └─────────┘  └─────────┘  └──────┘  │
└───────────────────────────────────────┘
       │
       ▼
┌──────────────┐
│   Consumer    │  Background workers poll, execute, update
└──────┬───────┘
       │
       ▼
┌──────────────────┐
│  Completed / Fail │  Success → remove; Fail → retry or DLQ
└──────────────────┘
```

### Job Types

| Job Type | Handler | Max Retries | Backoff |
|---|---|---|---|
| `DeployContract` | `deploy_handler` | 3 | exponential (5s, 25s, 125s) |
| `VerifyContract` | `verify_handler` | 2 | exponential (10s, 100s) |
| `IndexSorobanEvents` | `indexer_handler` | 5 | exponential (1s, 5s, 25s, 125s, 625s) |
| `NotifyDeploymentComplete` | `notify_handler` | 3 | exponential (5s, 25s, 125s) |

## Event Bus

The event bus uses `tokio::broadcast` with a configurable buffer. Services emit typed events; registered handlers run asynchronously.

### Event Types

| Event | Payload | Emitted By | Consumed By |
|---|---|---|---|
| `OrganizationCreated` | `{ id, name, owner_id }` | Org Service | Indexer, Notifier |
| `ProjectCreated` | `{ id, org_id, name }` | Project Service | Indexer, Notifier |
| `ContractDeployed` | `{ id, project_id, contract_id }` | Deploy Service | Registry, Indexer |
| `ContractVerified` | `{ id, source_hash }` | Verify Service | Contract Service |
| `DeploymentStarted` | `{ id, project_id }` | Deploy Service | Monitor |
| `DeploymentStateChanged` | `{ id, from, to }` | Pipeline Handler | Notifier, Logger |
| `DeploymentCompleted` | `{ id, contract_id }` | Pipeline Handler | Registry, Notifier |
| `DeploymentFailed` | `{ id, error }` | Pipeline Handler | Notifier, Logger |

## Error Codes

| Code | Error | Description | Resolution |
|---|---|---|---|
| 1001 | OrgNotFound | Organization ID does not exist | Verify org ID |
| 1002 | ProjectNotFound | Project ID does not exist | Verify project ID |
| 1003 | ContractNotFound | Contract ID does not exist | Verify contract ID |
| 1004 | DeploymentNotFound | Deployment ID does not exist | Verify deployment ID |
| 1005 | RepositoryNotFound | Repository ID does not exist | Verify repository ID |
| 1006 | UserNotFound | User ID does not exist | Verify user ID |
| 2001 | Unauthorized | Missing or invalid JWT | Re-authenticate |
| 2002 | Forbidden | Insufficient role permissions | Request role upgrade |
| 2003 | TokenExpired | JWT has expired | Refresh token |
| 2004 | InvalidChallenge | SEP-10 challenge verification failed | Check wallet signature |
| 3001 | InvalidTransition | Deployment state transition not valid | Follow state machine |
| 3002 | BuildFailed | WASM compilation error | Check build logs |
| 3003 | DeployFailed | Soroban transaction failed | Check transaction hash |
| 3004 | RollbackFailed | Rollback to previous version failed | Check deployment history |
| 4001 | SorobanRpcError | Underlying RPC call failed | Retry or check Stellar network |
| 4002 | ContractNotDeployed | Contract not found on-chain | Deploy contract first |
| 4003 | InsufficientFee | Transaction fee too low | Increase fee |
| 4004 | BadSequence | Account sequence number mismatch | Retry with correct sequence |
| 5001 | ValidationError | Request body validation failed | Check error details in response |
| 5002 | RateLimited | Too many requests | Retry after rate limit window |
| 5003 | InternalError | Unexpected server error | Contact support |

## Environment Variables

### Required

| Variable | Description |
|---|---|
| `DATABASE_URL` | PostgreSQL connection string |
| `APP_SECRET_KEY` | 64-byte hex string for JWT signing |
| `ENCRYPTION_KEY` | 32-byte hex string for data encryption |
| `STELLAR_RPC_URL` | Soroban RPC endpoint |
| `STELLAR_NETWORK_PASSPHRASE` | Stellar network passphrase |
| `JWT_ACCESS_TOKEN_TTL_MINUTES` | Access token lifetime |

### Optional

| Variable | Default | Description |
|---|---|---|
| `APP_PORT` | `8080` | HTTP server port |
| `APP_HOST` | `0.0.0.0` | HTTP server bind address |
| `APP_ENV` | `development` | Environment (development, staging, production) |
| `APP_DEBUG` | `true` | Enable debug output |
| `REDIS_URL` | `redis://localhost:6379` | Redis connection string |
| `LOG_LEVEL` | `info` | Log level (debug, info, warn, error) |
| `LOG_FORMAT` | `json` | Log output format |
| `RATE_LIMIT_REQUESTS_PER_MINUTE` | `100` | Global rate limit |
| `RATE_LIMIT_AUTH_REQUESTS_PER_MINUTE` | `10` | Auth endpoint rate limit |
| `CORS_ALLOWED_ORIGINS` | `http://localhost:3000` | CORS origin whitelist |
| `OTEL_EXPORTER_OTLP_ENDPOINT` | — | OpenTelemetry collector endpoint |
| `METRICS_PORT` | `9090` | Prometheus metrics port |

## Getting Started

### Prerequisites

- Rust 1.84+
- PostgreSQL 16+
- Redis 7+
- Docker (optional)

### Setup

```bash
# Clone the repository
git clone https://github.com/Astro-Orbit/Astro-Orbit-backend.git
cd Astro-Orbit-backend

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

### Available Scripts

| Command | Description |
|---|---|
| `cargo run` | Start development server |
| `cargo build --release` | Production build |
| `cargo test` | Run all tests |
| `cargo fmt` | Format code |
| `cargo clippy` | Lint check |
| `cargo doc --open` | Generate and open docs |
| `./scripts/init_db.sh` | Initialize database |
| `./scripts/migrate.sh` | Run migrations |

## Docker

### Build

```bash
docker build -t astro-orbit-backend .
```

Multi-stage build — planner stage caches dependencies, builder stage compiles, runner stage uses `distroless/cc-debian12` for minimal footprint.

### Run

```bash
docker run -p 8080:8080 -p 9090:9090 \
  -e DATABASE_URL=postgres://user:pass@host:5432/astro_orbit \
  -e APP_SECRET_KEY=your-64-byte-hex \
  -e STELLAR_RPC_URL=https://soroban-testnet.stellar.org \
  astro-orbit-backend
```

### Docker Compose

```bash
docker compose -f docker-compose.dev.yml up -d
```

Starts backend, PostgreSQL, and Redis. The backend is configured via environment variables in `docker-compose.dev.yml`.

## Testing

```bash
# Run all tests
cargo test

# Run with output
cargo test -- --nocapture

# Run specific test
cargo test test_deployment_pipeline

# Run tests with database (requires PostgreSQL)
DATABASE_URL=postgres://... cargo test --features integration-tests

# Check code coverage (requires tarpaulin)
cargo tarpaulin --ignore-tests
```

### Test Structure

```
tests/
├── handlers/        # Handler integration tests
├── services/        # Service unit tests
├── repositories/    # Repository integration tests (DB required)
├── stellar/         # Soroban RPC client tests (mocked)
├── deployment/      # Pipeline state machine tests
├── events/          # Event bus tests
├── jobs/            # Job queue tests
└── common/          # Test helpers and fixtures
```

## Cross-Repository Links

| Repository | Description |
|---|---|
| [Astro-Orbit Frontend](https://github.com/Astro-Orbit/Astro-Orbit-frontend) | Web dashboard & developer portal |
| [Astro-Orbit Contracts](https://github.com/Astro-Orbit/Astro-Orbit-contracts) | Soroban smart contracts (5 contracts, 30 tests) |

## Roadmap

- [x] Organization and project management
- [x] Contract registration and versioning
- [x] Deployment state machine and pipeline
- [x] Soroban RPC client with typed JSON-RPC
- [x] Event bus and job queue infrastructure
- [x] Background workers for deployment and indexing
- [x] Dashboard aggregation endpoints
- [ ] Smart contract verification service
- [ ] Webhook notifications for deployment events
- [ ] Multi-network support (testnet, mainnet, custom)
- [ ] Blue/green deployment strategies

## Project Status

Astro Orbit is in active development. See [ROADMAP.md](ROADMAP.md) for the complete development plan.

## Documentation

| Document | Description |
|---|---|
| [ARCHITECTURE.md](ARCHITECTURE.md) | Engineering handbook and architecture reference |
| [CONTRIBUTING.md](CONTRIBUTING.md) | Contribution workflow and conventions |
| [DEPLOYMENT.md](DEPLOYMENT.md) | Deployment and operations guide |
| [SECURITY.md](SECURITY.md) | Security model and vulnerability reporting |
| [ROADMAP.md](ROADMAP.md) | Development milestones and priorities |
| [CHANGELOG.md](CHANGELOG.md) | Release history |

## License

MIT
