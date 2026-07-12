# Astro Orbit — Architecture

> **Single source of truth for the Astro Orbit backend architecture.**
>
> Every decision documented here has a rationale. Every module has a
> defined responsibility. If it's not in this document, it's not
> part of the architecture.

---

## Table of Contents

1. [Project Vision](#1-project-vision)
2. [Goals & Non-Goals](#2-goals--non-goals)
3. [Architecture Philosophy](#3-architecture-philosophy)
4. [Layered Architecture](#4-layered-architecture)
5. [Dependency Rules](#5-dependency-rules)
6. [Module Ownership](#6-module-ownership)
7. [Module Responsibilities](#7-module-responsibilities)
8. [Data Flow](#8-data-flow)
9. [Request Lifecycle](#9-request-lifecycle)
10. [Authentication Architecture](#10-authentication-architecture)
11. [Authorization Model](#11-authorization-model)
12. [Database Philosophy](#12-database-philosophy)
13. [Caching Philosophy](#13-caching-philosophy)
14. [Configuration Philosophy](#14-configuration-philosophy)
15. [Error Philosophy](#15-error-philosophy)
16. [Logging Philosophy](#16-logging-philosophy)
17. [Observability](#17-observability)
18. [Background Jobs](#18-background-jobs)
19. [Events](#19-events)
20. [Notifications](#20-notifications)
21. [Deployment Pipeline](#21-deployment-pipeline)
22. [CI/CD](#22-cicd)
23. [API Design](#23-api-design)
24. [API Versioning](#24-api-versioning)
25. [Security Model](#25-security-model)
26. [Testing Strategy](#26-testing-strategy)
27. [Performance Goals](#27-performance-goals)
28. [Release Strategy](#28-release-strategy)
29. [Repository Standards](#29-repository-standards)
30. [Contribution Workflow](#30-contribution-workflow)
31. [Architecture Decision Records](#31-architecture-decision-records)
32. [Future Expansion](#32-future-expansion)
33. [Risk Analysis](#33-risk-analysis)
34. [Success Criteria](#34-success-criteria)

---

## 1. Project Vision

Astro Orbit is an **open-source developer platform for the Stellar and
Soroban ecosystem**. Its goal is to provide everything a developer needs
to build, deploy, manage, monitor, and secure Soroban applications from
one place.

**Elevator pitch:** Astro Orbit is an open-source developer platform that
streamlines the entire lifecycle of building, deploying, securing, and
operating Soroban smart contracts on Stellar.

### The Problem

Today, building on Soroban means stitching together multiple tools:
Soroban CLI, transaction explorers, event monitoring services, wallet
management, custom CI/CD, and manual security analysis. The developer
experience is fragmented.

### The Solution

Astro Orbit brings those workflows into one platform with a unified
dashboard and API. It replaces the fragmented toolchain with a cohesive
developer experience.

---

## 2. Goals & Non-Goals

### Goals

- **Single platform** for the entire Soroban development lifecycle
- **Production-grade** from day one — security, observability, reliability
- **Self-hostable** — open source, Docker-based deployment
- **Extensible** — API-first design, plugin architecture (future)
- **Scalable** — horizontal scaling via stateless services
- **Developer-friendly** — comprehensive documentation, SDKs, CLI

### Non-Goals

- **Not a general-purpose blockchain platform** — Stellar/Soroban only
- **Not a replacement for Stellar Core** — we build on top of it
- **Not a no-code platform** — developers write contracts, we handle the rest
- **Not a hosted-only service** — self-hosting is a first-class goal

---

## 3. Architecture Philosophy

### Domain-Driven Modules

Every module owns a specific domain (organizations, projects, deployments,
contracts). Module boundaries mirror business boundaries. Shared logic
lives in `utils/`, not duplicated across modules.

### Dependency Inversion

Handlers depend on service traits, not implementations. Services depend
on repository traits, not concrete database implementations. This enables
testing each layer independently.

### CQRS-Inspired

Reads bypass domain logic where safe. Write operations flow through
services for validation, authorization, and event emission. Complex
queries use dedicated repository methods rather than service orchestration.

### Deterministic Migrations

Database migrations are sequential SQL files, versioned and immutable.
Every migration is tested both forward and backward. Schema changes are
reviewed as part of the PR process.

### Fail-Fast Validation

Input validation happens at the boundary (handler layer). Invalid requests
are rejected before any business logic executes. Validation is declarative
using the `validator` crate.

### Stateless Services

Services hold no mutable state. All state lives in the database or cache.
This enables horizontal scaling by simply adding more instances.

### Feature-Gated Optionality

Optional capabilities (telemetry, analytics, specific cache backends) are
feature-gated in Cargo.toml. The default build includes PostgreSQL, Redis,
and telemetry.

---

## 4. Layered Architecture

```
┌──────────────────────────────────────────────────────────┐
│                     HTTP (Axum Router)                     │
│                  Route definitions only                    │
├──────────────────────────────────────────────────────────┤
│                    Middleware Stack                         │
│  Request ID → Tracing → CORS → Compression → Timeout       │
│  → Auth → Rate Limit                                       │
├──────────────────────────────────────────────────────────┤
│                       Handlers                              │
│  Parse request → Validate input → Call service → Respond   │
├──────────────────────────────────────────────────────────┤
│                       Services                              │
│  Business logic → Orchestration → Transaction coordination │
├──────────────────────────────────────────────────────────┤
│                     Repositories                            │
│  SQL queries → Data mapping → Trait implementations        │
├──────────────────────────────────────────────────────────┤
│                    Database (PostgreSQL)                    │
│                                                             │
├──────────────────────────────────────────────────────────┤
│                    External Services                        │
│  Redis (cache) │ Stellar (Soroban RPC) │ Metrics/Tracing   │
└──────────────────────────────────────────────────────────┘
```

### Layer Responsibilities

| Layer | Responsibility | Can Import |
|-------|----------------|------------|
| Router | Define routes, mount handlers | handlers, middleware |
| Middleware | Cross-cutting concerns | config, telemetry |
| Handlers | Input parsing, validation, response | dto, services, errors |
| Services | Business logic, coordination | repositories, events, cache |
| Repositories | Data access, query execution | models, database |
| Models | Domain entities | none |

---

## 5. Dependency Rules

### Strict Layering

```
Router → Handlers → Services → Repositories → Database
```

- Handlers NEVER import repositories, models, or database directly
- Services NEVER import handlers, router, or middleware
- Repositories NEVER import services or handlers

### No Circular Dependencies

Cargo's module system enforces acyclic dependencies. If module A depends
on B, B cannot depend on A. Shared types live in `models/` or `dto/`.

### Trait-Based Abstraction

Cross-layer dependencies use traits, not concrete types:
- Services depend on `UserRepository` trait, not `PgUserRepository`
- Handlers depend on `AuthService` trait, not `AuthServiceImpl`

### Feature Gate External Dependencies

Heavy external dependencies (telemetry, analytics) are feature-gated to
keep compile times minimal for development.

---

## 6. Module Ownership

| Module | Owner | Description |
|--------|-------|-------------|
| `config/` | Platform | Environment configuration |
| `router/` | API | Route definitions |
| `handlers/` | API | Request handlers |
| `middleware/` | Platform | Cross-cutting middleware |
| `auth/` | Security | Authentication logic |
| `permissions/` | Security | RBAC engine |
| `services/` | Domain | Business logic |
| `repositories/` | Data | Data access |
| `models/` | Data | Domain entities |
| `dto/` | API | Data transfer objects |
| `errors/` | Platform | Error types |
| `response/` | API | Response envelope |
| `events/` | Platform | Domain events |
| `jobs/` | Platform | Background jobs |
| `stellar/` | Stellar | Soroban integration |
| `deployment/` | Deploy | Deployment pipeline |
| `analytics/` | Analytics | Analytics engine |
| `notifications/` | Notifications | Notification delivery |
| `cache/` | Platform | Redis caching |
| `telemetry/` | Platform | Observability |
| `validation/` | Platform | Input validation |
| `utils/` | Platform | Shared utilities |

---

## 7. Module Responsibilities

### config/

Loads environment variables at startup into a single immutable `Config`
struct. Every configuration value must be present or the application fails
to start. No module accesses environment variables directly.

### router/

The single place where the API surface is defined. Builds the Axum `Router`
with all route groups under `/v1/`. Route definitions are declarative and
contain no business logic.

### handlers/

Thin layer that parses HTTP requests, validates input, calls a service,
and formats the response. Handlers follow a strict pattern:

```
fn handler(State, Path/Query/Json) -> Result<ApiResponse<T>, AppError>
```

### services/

Contains all business logic. Services orchestrate between repositories,
emit events, manage transactions, and coordinate external calls. Services
are stateless and receive dependencies via constructor injection.

### repositories/

Abstract data access behind traits. Each domain has a repository trait
with a PostgreSQL implementation. Repositories own all SQL queries and
handle the mapping between database rows and domain models.

### models/

Pure data structures with SQLx `FromRow` derive. No methods, no logic.
Models represent the database schema directly and are the single source
of truth for the data shape.

### errors/

Defines `AppError` enum with variants for every error category. Each
variant maps to an HTTP status code and a machine-readable error code.
Domain modules define their own error enums that convert into `AppError`.

### middleware/

Implements cross-cutting concerns as Tower layers:
- **RequestId**: Unique identifier per request, propagated to logs and responses
- **ResponseMeta**: Response time, API version headers
- **Auth**: JWT validation, user extraction
- **Rate Limit**: Token bucket per user/endpoint

### telemetry/

Initializes the observability stack:
- Structured JSON logging via `tracing-subscriber`
- OpenTelemetry tracing export (feature-gated)
- Prometheus metrics

### cache/

Redis-backed cache abstraction. Provides session storage, rate limiter,
and general-purpose caching. Never used as a consistency layer — always
falls back to the database.

### stellar/

Typed client for Soroban RPC. Handles contract deployment, transaction
submission, event subscription, and account queries. Abstracts the
JSON-RPC protocol details behind a clean Rust API.

### deployment/

State machine that manages the deployment lifecycle. Each deployment
transitions through stages: pending → building → scanning → deploying
→ deployed (or failed/rolled_back/cancelled).

### events/

In-process event bus using `tokio::sync::broadcast`. Domain events are
published by services and consumed by subscribers for side effects
(notifications, analytics, audit logging).

### jobs/

Background job processing. Uses PostgreSQL as the job queue for
durability. Jobs are claimed atomically, executed with retry, and
marked as failed after max retries.

---

## 8. Data Flow

### Write Path

```
Client → HTTP → Middleware → Handler → Service → Repository → Database
                          ↓                          ↓
                     Authorization                Event Bus
```

1. Client sends HTTP request
2. Middleware extracts user, validates JWT, checks rate limit
3. Handler deserializes request body, validates input
4. Handler calls service method
5. Service applies business logic, validates authorization
6. Service calls repository to persist data
7. Service publishes domain event
8. Handler formats response

### Read Path

```
Client → HTTP → Middleware → Handler → Repository → Database
```

For reads, handlers may call repositories directly when no business
logic is required. This avoids unnecessary layering for simple queries.

### Event Flow

```
Service → EventBus → Subscribers
                      ├── NotificationService
                      ├── AuditLogService
                      └── AnalyticsService
```

Services publish events after successful writes. Subscribers handle
side effects asynchronously. Events are in-process only for Phase 0.

---

## 9. Request Lifecycle

```
1. HTTP Request arrives at Axum server
2. Tower service stack processes request:
   a. TraceLayer — start tracing span, log request
   b. CorsLayer — validate CORS origin
   c. RequestBodyLimitLayer — enforce 10MB limit
   d. RequestTimeoutLayer — enforce 60s timeout
   e. RequestId — generate and inject request ID
   f. ResponseMetaLayer — (response phase)
3. Router matches path to handler
4. Handler extracts parameters (Path, Query, Json)
5. Handler validates input (validator derive)
6. Handler extracts user context from auth middleware
7. Handler calls service
8. Service executes business logic
9. Response flows back through middleware stack
10. TraceLayer logs response status and duration
```

---

## 10. Authentication Architecture

### Wallet Authentication Flow

The primary authentication method is Stellar wallet keypair verification
using the SEP-10 (WebAuth) standard:

```
1. Client: POST /v1/auth/challenge { public_key }
   Server: Generate random challenge, store in Redis (TTL 5min)
           Return { challenge, session_id }

2. Client: Sign challenge with Stellar private key
           POST /v1/auth/verify { public_key, signature, session_id }
   Server: Verify Ed25519 signature
           Create user if new
           Issue JWT access token (15min TTL)
           Issue refresh token (30 day TTL)
           Return { access_token, refresh_token, user }

3. Client: Use access_token in Authorization header
           POST /v1/auth/refresh { refresh_token }
   Server: Rotate refresh token
           Issue new access token
           Return { access_token, refresh_token }

4. Client: POST /v1/auth/logout
   Server: Revoke session, invalidate tokens
```

### JWT Structure

```json
{
  "sub": "user_uuid",
  "sid": "session_id",
  "org": "active_org_id",
  "iat": 1700000000,
  "exp": 1700000900
}
```

### Session Management

- Sessions tracked in Redis (fast lookup) and PostgreSQL (durability)
- Concurrent session limits enforced per user
- Sessions revoked on logout, password change, or admin action
- Refresh tokens rotated on each use (prevents replay)

---

## 11. Authorization Model

### Role-Based Access Control (RBAC)

| Role | Level | Permissions |
|------|-------|-------------|
| `owner` | Organization | All operations |
| `admin` | Organization | All except billing/delete |
| `member` | Organization | CRUD projects, deploy, view |
| `viewer` | Organization | Read-only |

### Permission Check Flow

```
Request → Auth Middleware (JWT validated)
       → Permission Middleware (check user + role + resource)
       → Handler
```

Permissions are checked at the middleware layer before handlers execute.
The permission engine evaluates:
1. Is the user a member of the organization?
2. Does the user's role include the required permission?
3. Is the action allowed on the specific resource?

### Permission Definitions

```rust
enum Permission {
    OrgRead, OrgWrite, OrgDelete,
    ProjectCreate, ProjectRead, ProjectWrite, ProjectDelete,
    ContractDeploy, ContractRead, ContractWrite,
    DeploymentCreate, DeploymentRead, DeploymentRollback,
    ApiKeyCreate, ApiKeyRead, ApiKeyDelete,
    AnalyticsRead,
    MemberInvite, MemberRoleChange, MemberRemove,
}
```

---

## 12. Database Philosophy

### Schema Design Principles

- **UUID primary keys** — never expose sequential IDs
- **Soft deletes** — `deleted_at TIMESTAMPTZ` on all user-facing tables
- **Created/updated timestamps** — every table has both
- **JSONB for flexible data** — settings, metadata, configurations
- **Foreign keys with CASCADE** — clean deletion semantics
- **Indexed foreign keys** — `CREATE INDEX ON table(foreign_key_id)`
- **Partial indexes** — `WHERE deleted_at IS NULL` for active records
- **ENUM types** — for bounded state machines (deployment_status)

### Migration Strategy

- Sequential SQL files in `migrations/` directory
- Every migration is reversible (up/down in same file)
- Migrations embedded in binary via `sqlx::migrate!`
- Run at startup before accepting traffic
- Tested forward and backward in CI

### Soft Delete Pattern

```sql
-- All user-facing tables follow this pattern:
ALTER TABLE users ADD COLUMN deleted_at TIMESTAMPTZ;

-- Queries always filter:
SELECT * FROM users WHERE id = $1 AND deleted_at IS NULL;

-- Admin queries can include deleted records:
SELECT * FROM users WHERE id = $1;
```

### Audit Logging

Every write operation affecting user data is recorded in the `audit_logs`
table with: actor, action, resource type, resource ID, metadata, IP, and
timestamp. This provides a complete audit trail for compliance and debugging.

---

## 13. Caching Philosophy

### What We Cache

| Data | Cache | TTL | Invalidation |
|------|-------|-----|--------------|
| Sessions | Redis | Until logout | Manual revocation |
| Auth challenges | Redis | 5 min | Auto-expire |
| Rate limit counters | Redis | 1 min | Auto-expire |
| User profiles | Redis | 5 min | On profile update |
| Analytics aggregates | Redis | 1 hour | TTL-based |

### What We Don't Cache

- Write operations (always go to database)
- Cryptographic operations (signature verification)
- Authorization decisions (enforced per request)
- Deployment state (read from database for consistency)

### Cache Consistency

Cache is never the source of truth. If Redis is unavailable, the system
falls back to database queries. Cache entries have TTLs and are explicitly
invalidated on relevant mutations.

---

## 14. Configuration Philosophy

### Principles

1. **All configuration from environment** — no config files, no hardcoded values
2. **Fail fast on missing config** — startup fails if required variables are missing
3. **Single source of truth** — `Config` struct is the only way to access configuration
4. **Immutable after startup** — configuration is loaded once and shared via `Arc`
5. **No silent defaults for security-critical settings** — encryption keys, secrets

### Environment Files

- `.env` — local development (gitignored)
- `.env.example` — documented template (committed)
- Production configuration via actual environment variables

### Config Modules

```rust
pub struct Config {
    pub app: AppConfig,           // name, env, port, host, secret_key, base_url
    pub database: DatabaseConfig,  // url, pool size, timeouts
    pub redis: RedisConfig,       // url, pool size, TTL
    pub auth: AuthConfig,         // JWT TTL, issuer, audience
    pub stellar: StellarConfig,   // RPC URL, network, timeout
    pub telemetry: TelemetryConfig, // OTLP endpoint, service name
    pub rate_limit: RateLimitConfig, // requests per minute, burst
    pub logging: LoggingConfig,   // level, format, file path
    pub security: SecurityConfig,  // encryption key, CORS origins
}
```

---

## 15. Error Philosophy

### Error Types

```rust
pub enum AppError {
    NotFound(Cow<'static, str>),
    Validation(Cow<'static, str>),
    Unauthenticated,
    Forbidden(Cow<'static, str>),
    Conflict(Cow<'static, str>),
    RateLimited,
    Internal(Cow<'static, str>),
    Unavailable(Cow<'static, str>),
    BadRequest(Cow<'static, str>),
}
```

### Error Response Format

```json
{
  "success": false,
  "error": {
    "code": "VALIDATION_ERROR",
    "message": "Name is required"
  },
  "meta": {
    "request_id": "req_abc123",
    "timestamp": "2026-07-12T10:00:00Z"
  }
}
```

### Principles

1. **Never leak internals** — internal errors are logged, not returned to client
2. **Machine-readable error codes** — clients can reliably parse error types
3. **Human-readable messages** — useful for debugging without exposing internals
4. **Always structured** — never return plain text or HTML errors
5. **Domain errors convert to AppError** — each domain module defines its own
   error enum implementing `IntoAppError`

---

## 16. Logging Philosophy

### Structured JSON Logs

All logs are structured JSON in production for machine parsing. Development
mode uses human-readable formatting.

### Log Levels

| Level | Usage |
|-------|-------|
| ERROR | Unhandled errors, database failures, external service failures |
| WARN | Recoverable errors, rate limit approaching, degraded performance |
| INFO | Request lifecycle, business events (deployment started, org created) |
| DEBUG | Query execution, cache operations |
| TRACE | Detailed request/response body, cryptographic operations |

### Request Logging

Every request produces one INFO log on completion:

```json
{
  "timestamp": "...",
  "level": "INFO",
  "message": "request completed",
  "request_id": "req_abc123",
  "method": "POST",
  "path": "/v1/orgs",
  "status": 201,
  "duration_ms": 45,
  "user_id": "usr_xyz",
  "org_id": "org_abc"
}
```

---

## 17. Observability

### Three Pillars

| Pillar | Tool | Export |
|--------|------|--------|
| Logs | `tracing` | stdout (JSON), file (optional) |
| Metrics | `prometheus` | `/metrics` endpoint |
| Traces | `opentelemetry` | OTLP endpoint |

### Metrics

| Metric | Type | Labels |
|--------|------|--------|
| `http_requests_total` | Counter | method, path, status |
| `http_request_duration_seconds` | Histogram | method, path |
| `db_connections_active` | Gauge | pool |
| `db_connections_idle` | Gauge | pool |
| `cache_hit_ratio` | Gauge | cache |
| `jobs_completed_total` | Counter | job_type, status |
| `jobs_duration_seconds` | Histogram | job_type |

### Health Checks

```
GET /v1/health → {
  "status": "pass",
  "version": "0.1.0",
  "uptime": 3600,
  "checks": {
    "database": { "status": "pass", "latency_ms": 2 },
    "redis": { "status": "pass", "latency_ms": 1 },
    "stellar_rpc": { "status": "pass", "latency_ms": 150 }
  }
}
```

---

## 18. Background Jobs

### Job Architecture

- Jobs stored in a `job_queue` PostgreSQL table
- Workers poll for pending jobs, claim atomically (`SELECT ... FOR UPDATE SKIP LOCKED`)
- Max retries: 3, with exponential backoff (1s, 5s, 30s)
- Dead letter queue after max retries

### Job Types

| Job | Trigger | Description |
|-----|---------|-------------|
| DeploymentJob | API call | Execute deployment pipeline |
| SecurityScanJob | Pre-deploy | Scan contract code |
| NotificationJob | Domain event | Send notification |
| AnalyticsAggregationJob | Scheduled | Compute analytics |

### Worker Implementation

```rust
// Workers run in a separate tokio task:
tokio::spawn(async move {
    loop {
        if let Some(job) = claim_job(&pool).await? {
            execute_job(job).await;
        }
        tokio::time::sleep(Duration::from_millis(500)).await;
    }
});
```

---

## 19. Events

### Event Bus

In-process event bus using `tokio::sync::broadcast`. Events are published
synchronously (within the request lifecycle) and consumed asynchronously
by subscribers.

### Event Types

| Event | Payload | Subscribers |
|-------|---------|-------------|
| `UserCreated` | user_id | Analytics, Welcome Email |
| `OrgCreated` | org_id, owner_id | Audit Log |
| `ProjectCreated` | project_id, org_id | Audit Log |
| `DeploymentStarted` | deployment_id | Notifications, Analytics |
| `DeploymentCompleted` | deployment_id | Notifications |
| `DeploymentFailed` | deployment_id, error | Notifications, Alerts |
| `ContractScanned` | scan_id, findings | Security Dashboard |

### Cross-Process Events (Future)

PostgreSQL LISTEN/NOTIFY for cross-process event delivery when the
application scales to multiple instances.

---

## 20. Notifications

### Delivery Channels

| Channel | Implementation | Status |
|---------|----------------|--------|
| In-app | Database table, polled by client | Phase 9 |
| Email | SMTP/SendGrid integration | Phase 9 |
| Webhook | HTTP POST to configured URL | Phase 9 |

### Notification Preferences

Users configure per-channel opt-in/opt-out for each notification type:

```json
{
  "deployment_completed": { "email": true, "in_app": true, "webhook": false },
  "deployment_failed": { "email": true, "in_app": true, "webhook": true },
  "contract_scan_completed": { "email": false, "in_app": true, "webhook": false }
}
```

---

## 21. Deployment Pipeline

### State Machine

```
                  ┌──────────┐
                  │  Pending  │
                  └────┬─────┘
                       │
                  ┌────▼─────┐
                  │ Building  │
                  └────┬─────┘
                       │
                  ┌────▼─────┐
                  │ Scanning  │ ◄── Sentinel security scan
                  └────┬─────┘
                       │
                  ┌────▼──────┐
                  │ Deploying  │
                  └────┬──────┘
                       │
              ┌────────┼────────┐
              │        │        │
         ┌────▼──┐ ┌──▼───┐ ┌──▼────┐
         │Deployed│ │Failed│ │Rolled │
         └────┬───┘ └──────┘ │ Back  │
              │              └───────┘
         ┌────▼───┐
         │Cancelled│
         └────────┘
```

### Pipeline Stages

1. **Building**: Compile contract WASM
2. **Scanning**: Run Sentinel security analysis
3. **Deploying**: Submit contract to Soroban RPC
4. **Verifying**: Confirm contract deployment on-chain
5. **Completed**: Mark deployment as successful

### Rollback

Rollback creates a new deployment that deploys the previous version.
It does not modify the existing deployment's history — rollbacks are
auditable events.

---

## 22. CI/CD

### CI Pipeline (GitHub Actions)

| Job | Trigger | Steps |
|-----|---------|-------|
| Lint | push, PR | rustfmt, clippy |
| Test | push, PR | migrations, cargo test |
| Build | push, PR | cargo build --release |
| Security | push, PR | cargo audit |
| Weekly Audit | schedule | cargo audit + deny |

### CD Pipeline

| Job | Trigger | Steps |
|-----|---------|-------|
| Docker Build | tag v* | Build multi-stage, push to GHCR |

### Branch Protection

- `main`: protected, requires PR review, CI passing
- `develop`: protected, requires CI passing
- Feature branches: no protection

---

## 23. API Design

### Base URL

All endpoints are under `/v1/`. Example: `https://api.astro-orbit.dev/v1/orgs`

### Response Envelope

Every response follows a consistent format:

```json
{
  "success": true,
  "data": { ... },
  "pagination": { "page": 1, "per_page": 20, "total": 100, "total_pages": 5 },
  "meta": { "request_id": "req_abc123", "timestamp": "..." }
}
```

### Status Codes

| Code | Usage |
|------|-------|
| 200 | Successful read |
| 201 | Successful create |
| 204 | Successful delete |
| 400 | Bad request |
| 401 | Unauthenticated |
| 403 | Forbidden |
| 404 | Not found |
| 409 | Conflict |
| 422 | Validation error |
| 429 | Rate limited |
| 500 | Internal error |
| 503 | Service unavailable |

### Endpoint Groups

See [PHASE 4 — API Design](#) in the project plan for the complete
endpoint reference.

---

## 24. API Versioning

### Strategy: URL Prefix

- Version embedded in URL path: `/v1/`, `/v2/`
- Backward-compatible changes (adding fields, new endpoints) within a version
- Breaking changes require a new version
- Deprecated versions receive a `Sunset` header with migration URL
- Minimum version support: current + one previous

### Version Lifecycle

1. `v1` is current
2. Breaking changes go in `v2`
3. `v1` deprecated, `Sunset` header added
4. `v1` removed after deprecation period (90 days)

---

## 25. Security Model

### Authentication

- Wallet-based Ed25519 challenge-response (SEP-10)
- JWT access tokens (15min TTL) signed with HS256
- Refresh token rotation (prevents stolen token replay)
- Sessions revocable by user or admin
- Rate limiting on auth endpoints (10 req/min)

### Authorization

- Organization-level RBAC with 4 built-in roles
- Resource-level permission checks on every request
- API keys with scoped permissions (read-only, deploy, admin)
- Admin endpoints restricted to platform admins

### Data Protection

- Secrets and encryption keys from environment variables
- API keys hashed (SHA-256) before storage
- Encryption at rest for sensitive database fields
- TLS for all network communication
- PII minimized in logs

### Input Validation

- All inputs validated at handler boundary
- Strict Content-Type enforcement
- Request body size limit: 10MB
- No dynamic SQL (parameterized queries)
- No eval/user-code execution

### Rate Limiting

- 100 requests/minute per user (general)
- 10 requests/minute per user (auth endpoints)
- Token bucket algorithm, Redis-backed
- Burst allowance: 20 requests
- Headers: `X-RateLimit-Limit`, `X-RateLimit-Remaining`, `X-RateLimit-Reset`

### Infrastructure Security

- Docker containers run as non-root user
- Health checks on all services
- Graceful shutdown with timeout
- Connection pooling with timeouts
- No hardcoded secrets in code

---

## 26. Testing Strategy

### Test Pyramid

```
         ╱╲
        ╱  ╲
       ╱ E2E╲        5%
      ╱──────╲
     ╱        ╲
    ╱Integration╲    25%
   ╱──────────────╲
  ╱                ╲
 ╱    Unit Tests    ╲   70%
╱──────────────────────╲
```

### Unit Tests (70%)

- Domain logic, validation, utility functions
- Pure functions with no external dependencies
- Property-based testing for validation and serialization

### Integration Tests (25%)

- Repository methods against real PostgreSQL
- Service orchestration with mock repositories
- API endpoints against full middleware stack
- Each test in its own transaction (rollback on completion)

### E2E Tests (5%)

- Full deployment pipeline (mocked Stellar RPC)
- Authentication flow from challenge to verified session
- Permission enforcement across role hierarchy

### Test Infrastructure

```rust
// Test app builder creates a fresh instance for each test:
let app = TestAppBuilder::new()
    .with_database(pool)
    .with_clock(fixed_clock)
    .with_stellar(mock_rpc)
    .build()
    .await;
```

### Coverage Goals

| Type | Target |
|------|--------|
| Services | 90%+ |
| Repositories | 100% (all methods) |
| Handlers | 100% (all endpoints, happy + error) |
| Validation | 95%+ |
| Utils | 90%+ |

### Benchmarks

- Repository queries with Criterion
- Cryptographic operations (signature verification)
- JSON serialization/deserialization
- Request throughput under concurrency

---

## 27. Performance Goals

| Metric | Target |
|--------|--------|
| P50 response time | < 50ms |
| P95 response time | < 200ms |
| P99 response time | < 500ms |
| Requests/second | > 1000 (single instance) |
| DB query time (P95) | < 50ms |
| Migration time | < 5s per migration |
| Startup time | < 3s |
| Binary size | < 50MB (release, stripped) |
| Memory usage (idle) | < 100MB |
| Memory usage (loaded) | < 500MB |

---

## 28. Release Strategy

### Versioning

Semantic versioning: `MAJOR.MINOR.PATCH`
- MAJOR: breaking API changes
- MINOR: new features, backward-compatible
- PATCH: bug fixes

### Release Process

1. Branch `release/vX.Y.Z` from `develop`
2. Update version in `Cargo.toml`
3. Update `CHANGELOG.md`
4. Run full CI pipeline
5. Create GitHub Release (tag vX.Y.Z)
6. GitHub Actions builds and pushes Docker image
7. Deploy to staging, run E2E tests
8. Deploy to production
9. Merge release branch to `main` and `develop`

### Hotfix Process

1. Branch `hotfix/X.Y.Z+1` from `main`
2. Fix, test, merge to `main`
3. Cherry-pick to `develop`
4. Release as PATCH version

---

## 29. Repository Standards

### Code Style

- 120 character line width
- 4-space indentation
- Module-level imports (not granular)
- Doc comments on all public types and methods
- No `unwrap()` or `expect()` in production code (use `AppError`)
- No TODO comments (create issues instead)

### Naming Conventions

| Element | Convention | Example |
|---------|------------|---------|
| Modules | snake_case | `user_service` |
| Types | UpperCamelCase | `UserService` |
| Functions | snake_case | `create_user` |
| Variables | snake_case | `user_repo` |
| Constants | SCREAMING_SNAKE_CASE | `MAX_RETRIES` |
| Error variants | UpperCamelCase | `NotFound` |

### File Organization

- One concept per file
- Files under 500 lines (extract helpers if larger)
- Tests in adjacent `tests/` file (unit) or `tests/` directory (integration)
- `mod.rs` files contain only re-exports and doc comments

---

## 30. Contribution Workflow

See [CONTRIBUTING.md](CONTRIBUTING.md) for the complete contribution guide.

---

## 31. Architecture Decision Records

Architecture Decision Records (ADRs) are stored in `docs/ADR/`. Each ADR
documents a significant architectural decision with context, decision,
consequences, and compliance.

### Current ADRs

| ID | Title | Status |
|----|-------|--------|
| 0001 | Use Axum as the Web Framework | Accepted |
| 0002 | Use SQLx for Database Access | Accepted |

### When to Write an ADR

- Choosing a new framework or library
- Changing the architecture pattern
- Adding a significant new capability
- Modifying the database schema design
- Changing the API versioning strategy

---

## 32. Future Expansion

### CLI

A command-line interface using `clap` for common operations:
- `astro login` — wallet authentication
- `astro deploy` — deploy contracts
- `astro contracts list` — manage contracts
- `astro orgs` — organization management

### SDKs

- TypeScript SDK (autogenerated from OpenAPI spec)
- Rust SDK (direct library usage)
- Python SDK (community-driven)

### VS Code Extension

- Deploy contracts from the editor
- View deployment status
- Monitor events in real-time

### Plugin System

- Webhook integrations
- Custom security scanners
- Custom deployment validators
- Notification channel plugins

### Kubernetes Operator

- Native Kubernetes deployment
- Helm charts
- Horizontal pod autoscaling

### Contract Marketplace

- Publish verified contracts
- Version management
- Dependency resolution
- Verified source code

---

## 33. Risk Analysis

| Risk | Likelihood | Impact | Mitigation |
|------|------------|--------|------------|
| Stellar network changes | Medium | High | Abstract Stellar integration behind traits, versioned API clients |
| Database migration failures | Low | High | Test migrations in CI, reversible migrations, zero-downtime deploys |
| Redis outage | Low | Medium | Graceful fallback to database, circuit breaker |
| Security vulnerability | Medium | Critical | Regular audits, dependency scanning, responsible disclosure policy |
| Performance regression | Low | Medium | Benchmark suite, performance testing in CI |
| API breaking changes | Medium | High | Strict versioning, deprecation policy, client SDK versioning |
| Single developer dependency | Medium | Medium | Comprehensive documentation, ADRs, code review requirements |

---

## 34. Success Criteria

### Technical

- [ ] All endpoints have integration tests
- [ ] 90%+ test coverage on business logic
- [ ] P95 response time under 200ms
- [ ] Zero-downtime deployments
- [ ] All security checks pass in CI
- [ ] Docker image builds in under 5 minutes

### Product

- [ ] User can authenticate with Stellar wallet
- [ ] User can create organizations and projects
- [ ] User can deploy contracts to Soroban testnet
- [ ] User can view deployment history and logs
- [ ] User can manage team members and permissions
- [ ] User can monitor contract events

### Community

- [ ] Public roadmap maintained
- [ ] Issue templates and PR templates active
- [ ] CI status visible on README
- [ ] Documentation covers all public APIs
- [ ] Contribution guide is complete
- [ ] ADRs document all major decisions
