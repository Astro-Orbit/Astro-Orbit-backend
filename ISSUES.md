# Astro Orbit — Engineering Issues

> Generated from full-source audit

## Contents

1. [Ship Blockers](#1-ship-blockers)
2. [Missing Implementations](#2-backend-missing-implementations)
3. [Auth & Security](#3-backend-auth--security)
4. [Database & Repositories](#4-backend-database--repositories)
5. [Middleware & Infrastructure](#5-backend-middleware--infrastructure)
6. [Code Quality & Dead Code](#6-backend-code-quality--dead-code)
7. [Cross-Cutting Testing](#7-cross-cutting-testing)
8. [Developer Experience](#8-developer-experience)

---

# 1. Ship Blockers

---

## Issue #1 — Empty `analytics/` directory causes compile failure

**Labels:** bug, backend, blocker

**Summary:**
`src/lib.rs:16` declares `pub mod analytics;` but `src/analytics/` is empty — no `mod.rs`. The Rust compiler will reject this. CI `cargo build` will fail.

**Acceptance Criteria:**
- [ ] `cargo build --features default` succeeds
- [ ] Either remove `pub mod analytics;` from `lib.rs` and delete `src/analytics/`, or create `src/analytics/mod.rs`

**Files:** `src/lib.rs:16`, `src/analytics/`
**Difficulty:** Beginner

---

## Issue #2 — Three database tables referenced by code have no migration

**Labels:** bug, backend, database, blocker

**Summary:**
`job_queue`, `activity_log`, and `repositories` tables are queried by `PgJobQueue`, `PgActivityRepository`, and `PgRepositoryRepository` respectively, but neither migration `001_initial.sql` nor `002_auth_identity.sql` creates them. Runtime queries fail with "relation does not exist".

**Acceptance Criteria:**
- [ ] Migration `003_missing_tables.sql` created with DDL for all three tables
- [ ] Column types match struct fields in `models/` and `jobs/types.rs`
- [ ] Appropriate indexes and foreign keys included

**Files:** `migrations/`, `src/jobs/queue.rs`, `src/repositories/activity_repo.rs`, `src/repositories/repository_repo.rs`
**Difficulty:** Intermediate

---

## Issue #3 — Docker HEALTHCHECK uses nonexistent subcommand

**Labels:** bug, backend, docker, blocker

**Summary:**
`Dockerfile:63-64` runs `/app/astro-orbit-backend health` but the binary serves HTTP on port 8080 with no `health` subcommand. Orchestrators mark container unhealthy.

**Acceptance Criteria:**
- [ ] HEALTHCHECK correctly probes `/v1/health`
- [ ] Either install curl in runner or add health subcommand to main.rs

**Files:** `Dockerfile:63-64`
**Difficulty:** Intermediate

---

## Issue #4 — Auth and org-context middleware both hardcode `Role::Viewer`

**Labels:** bug, backend, auth, blocker

**Summary:**
Two places hardcode `Role::Viewer` for every user: `middleware/auth.rs:43-44` maps any JWT org claim to Viewer permissions without querying the DB, and `middleware/org_context.rs:26` sets `role: Role::Viewer` without querying `organization_members`. RBAC is completely non-functional — every user gets Viewer permissions regardless of actual role.

**Acceptance Criteria:**
- [ ] Auth middleware extracts user's actual role from `organization_members` table
- [ ] Org context middleware queries `SELECT role FROM organization_members WHERE organization_id = $1 AND user_id = $2`
- [ ] If no membership row exists, permissions are empty (not Viewer)
- [ ] Hardcoded `Role::Viewer` removed from both files

**Files:** `src/middleware/auth.rs:43-44`, `src/middleware/org_context.rs:26`
**Difficulty:** Advanced

---

## Issue #5 — Audit middleware reads AuthContext from response extensions — always gets None

**Labels:** bug, backend, audit, blocker

**Summary:**
`middleware/audit.rs:23` calls `response.extensions().get::<AuthContext>()`. AuthContext is inserted into **request** extensions by the auth middleware, not response extensions. This always returns None, so audit logging never captures actor identity.

**Acceptance Criteria:**
- [ ] Read AuthContext from `request.extensions()` before `next.run(request).await`
- [ ] Write audit entry asynchronously after response is sent
- [ ] Verified that actor_id is populated in audit_logs table

**Files:** `src/middleware/audit.rs:23`
**Difficulty:** Advanced

---

## Issue #6 — Three middleware modules not declared in `middleware/mod.rs` — never compiled

**Labels:** bug, backend, blocker

**Summary:**
`src/middleware/mod.rs` only declares `auth`, `request_id`, `response_meta`. Files `audit.rs`, `org_context.rs`, and `authorization.rs` exist on disk but are **never compiled** — they are dead code. The router references `midware::auth::middleware` only. No org permission check, no audit, no authorization middleware ever runs.

**Acceptance Criteria:**
- [ ] Add `pub mod authorization;`, `pub mod org_context;`, `pub mod audit;` to `middleware/mod.rs`
- [ ] Wire each middleware into the appropriate route layer in `router/mod.rs`
- [ ] Verify all three are compiled and executed in request lifecycle

**Files:** `src/middleware/mod.rs`, `src/middleware/audit.rs`, `src/middleware/org_context.rs`, `src/middleware/authorization.rs`
**Difficulty:** Intermediate

---

## Issue #7 — Database migration failure silently ignored on startup

**Labels:** bug, backend, blocker

**Summary:**
`main.rs:49-51` — `if let Err(e) = run_migrations(&pool).await { tracing::error!(...); }` — if migrations fail, the error is logged but the server continues starting with an un-migrated database. Handlers querying new tables/columns will get runtime errors.

**Acceptance Criteria:**
- [ ] Server exits with non-zero exit code on migration failure
- [ ] Error includes the specific migration name and SQL error
- [ ] Graceful startup aborts before binding to port

**Files:** `src/main.rs:49-51`
**Difficulty:** Intermediate

---

# 2. Missing Implementations

---

## Issue #9 — Four service traits defined but never implemented

**Labels:** feature, backend

**Summary:**
`ProjectService`, `ContractService`, `DeploymentService`, and `NotificationService` traits exist in `src/services/` but have zero concrete implementations. All 20+ corresponding handler endpoints return stub data (`Uuid::new_v4()`, empty strings). Repository layer implementations already exist for all four domains.

**Acceptance Criteria:**
- [ ] `ProjectServiceImpl` struct created implementing `ProjectService`
- [ ] `ContractServiceImpl` struct created implementing `ContractService`
- [ ] `DeploymentServiceImpl` struct created implementing `DeploymentService`
- [ ] `NotificationServiceImpl` struct created implementing `NotificationService`
- [ ] Each method delegates to the corresponding repository
- [ ] Handlers are updated to use real service calls
- [ ] Integration tests for CRUD paths of each service

**Difficulty:** Advanced

---

## Issue #10 — DTO types duplicated between handlers and centralized `dto/` module

**Labels:** refactor, backend

**Summary:**
5 handler files define duplicate request/response structs that already exist in `src/dto/`:
- `handlers/projects.rs` — `CreateProjectRequest`, `ProjectResponse`, `UpdateProjectRequest`
- `handlers/contracts.rs` — 5 duplicate types
- `handlers/deployments.rs` — 3 duplicate types
- `handlers/notifications.rs` — 2 duplicate types
- `handlers/wallets.rs` — 3 duplicate types

Some fields differ between copies (e.g., `organization_id` presence).

**Acceptance Criteria:**
- [ ] All handlers import from `crate::dto::*` instead of defining locally
- [ ] Field discrepancies resolved (prefer `dto/` version as source of truth)
- [ ] `crate::dto/` module extended for any missing types (e.g., `LogEntry`, `ActivityFeedItem`)

**Difficulty:** Intermediate

---

## Issue #11 — Six doc-comment-only source files provide no implementation

**Labels:** refactor, backend

**Summary:**
`src/stellar/contract.rs`, `event.rs`, `transaction.rs` and `src/notifications/channels.rs`, `preferences.rs`, `template.rs` contain only doc comments (4 lines each). They compile but add dead weight.

**Acceptance Criteria:**
- [ ] Each file either contains a working implementation or is removed
- [ ] `mod` declarations removed from parent `mod.rs` if files removed

**Difficulty:** Beginner (removal) / Advanced (implementation)

---

## Issue #12 — Analytics overview and per-project analytics endpoints return zeroed data

**Labels:** feature, backend

**Summary:**
Only `dashboard_stats` and `dashboard_activity` use the real `AnalyticsServiceImpl`. The other 4 endpoints return hardcoded zeros: `overview`, `contract_calls`, `gas_usage`, `active_users` in `handlers/analytics.rs:35-55`.

**Acceptance Criteria:**
- [ ] `overview()` calls `AnalyticsService::org_stats()`
- [ ] `contract_calls()`, `gas_usage()`, `active_users()` call appropriate service methods
- [ ] `AnalyticsService` trait extended with new methods if needed

**Difficulty:** Intermediate

---

## Issue #13 — All 5 wallet endpoints return fake data

**Labels:** feature, backend

**Summary:**
`handlers/wallets.rs` — create, list, get, update, delete all return `Uuid::new_v4()` with empty `public_key`. `WalletService` and `PgWalletRepository` are fully implemented.

**Acceptance Criteria:**
- [ ] All 5 CRUD endpoints wired to real `WalletService`
- [ ] Wallets scoped to authenticated user
- [ ] Integration tests for each endpoint

**Difficulty:** Intermediate

---

## Issue #14 — Notification and admin endpoints all stub

**Labels:** feature, backend

**Summary:**
- `notifications.rs`: list returns empty vec, mark-read no-op, preferences returns `{}`
- `admin.rs`: users/orgs/stats all return empty/zeroed data. No admin role check exists.

**Acceptance Criteria:**
- [ ] Notifications persist to DB, respect per-user scoping
- [ ] Admin endpoints require `is_admin` flag or admin role
- [ ] Admin stats return real aggregate counts

**Difficulty:** Intermediate

---

## Issue #15 — All 5 explorer endpoints return fake data

**Labels:** feature, backend

**Summary:**
`handlers/explorer.rs` — contract details, transactions, events all return empty data. `SorobanClient` implements real RPC methods (`getContractData`, `getTransaction`, `getLedgerEntries`) but the explorer never uses them.

**Acceptance Criteria:**
- [ ] Contract details queries `getContractData` from Soroban RPC
- [ ] Transaction endpoints query `getTransaction` from Soroban RPC
- [ ] Events endpoint queries on-chain events with filtering
- [ ] Error handling for network failures, invalid IDs

**Difficulty:** Advanced

---

## Issue #16 — Org member management endpoints (6) are all stubs

**Labels:** feature, backend

**Summary:**
`handlers/orgs.rs` — `add_member`, `list_members`, `update_member`, `remove_member`, `list_roles`, `create_role` all return fake data. The `/v1/orgs/:id/members/*` and `/v1/orgs/:id/roles/*` routes are non-functional.

**Acceptance Criteria:**
- [ ] Member add/remove/update uses `OrganizationMemberRepository`
- [ ] Role management CRUD implemented
- [ ] Permission checks enforce admin-only for member management
- [ ] Integration tests

**Difficulty:** Advanced

---

## Issue #17 — Deployment pipeline stages are all no-ops

**Labels:** feature, backend

**Summary:**
`src/deployment/pipeline.rs`:
- `validate_contracts()` — empty function body
- `perform_security_scan()` — always returns `true`
- `deploy_to_network()` — only calls `get_network()` and returns `Ok`, never submits a transaction

**Acceptance Criteria:**
- [ ] `validate_contracts()` verifies contract IDs exist and are compilable
- [ ] `perform_security_scan()` calls an external scanner or validates WASM
- [ ] `deploy_to_network()` builds and submits Soroban deploy transaction via `SorobanClient`

**Difficulty:** Advanced

---

## Issue #18 — Notification delivery channels are all no-ops

**Labels:** feature, backend

**Summary:**
`src/jobs/notification_jobs.rs` — all notification handlers (`email`, `webhook`, `in-app`) only call `tracing::info!` and never deliver. The `notifications/` directory files are all doc-only.

**Acceptance Criteria:**
- [ ] Email channel sends via SMTP/SES
- [ ] Webhook channel sends HTTP POST to configured URL
- [ ] In-app channel writes to `notifications` table
- [ ] Retry logic for failed delivery

**Difficulty:** Advanced

---

## Issue #19 — Contract security scan endpoints return fake data

**Labels:** feature, backend

**Summary:**
`handlers/security.rs` — create_scan, list_scans, get_scan, findings all return fake UUIDs and empty arrays. No actual security analysis.

**Acceptance Criteria:**
- [ ] Scan creation triggers background job
- [ ] Findings persist to database
- [ ] At least one analysis stage implemented (e.g., WASM bytecode size check)

**Difficulty:** Intermediate

---

## Issue #20 — Repository sync endpoint doesn't actually sync

**Labels:** feature, backend

**Summary:**
`repository_service.rs:138-151` — `sync_repository()` only updates `last_synced_at` timestamp. Never fetches branches, commits, or files from the external VCS provider.

**Acceptance Criteria:**
- [ ] Imports branch list from the provider API
- [ ] Imports commit history or latest commit SHA
- [ ] Provider-agnostic interface (GitHub/GitLab adapter pattern)

**Difficulty:** Intermediate

---

## Issue #21 — No way to create a project from the frontend

**Labels:** feature, frontend, backend

**Summary:**
Both the "New Project" buttons on `dashboard/page.tsx:59` and `projects/page.tsx:38` link to `ROUTES.PROJECTS` (the projects list page). There is no `/projects/new` page or form. The backend `POST /orgs/:id/projects` returns stub data anyway.

**Acceptance Criteria:**
- [ ] `/projects/new` page created with form (name, slug, description, network)
- [ ] Backend `POST /orgs/:id/projects` creates real project via `ProjectServiceImpl`
- [ ] After creation, user is redirected to the new project detail page

**Difficulty:** Intermediate

---

## Issue #22 — Org update handler returns fake member_count and role

**Labels:** bug, backend

**Summary:**
`org_service.rs:96-97` — `OrgResult` after update returns `role: String::new()`, `member_count: 0` instead of querying the actual values.

**Acceptance Criteria:**
- [ ] After update, return current `role` from `organization_members`
- [ ] `member_count` queries `SELECT COUNT(*) FROM organization_members WHERE organization_id = $1`

**Difficulty:** Intermediate

---

# 3. Auth & Security

---

# 3. Auth & Security

---

## Issue #23 — Accept-invite doesn't validate invitee email matches authenticated user

**Labels:** bug, backend, security

**Summary:**
`org_service.rs:141-159` — `accept_invite()` checks the token hash but never compares `invitation.invitee_email` against the authenticated user's email. If an invite is sent to `alice@example.com` and Bob obtains the token, Bob can accept it.

**Acceptance Criteria:**
- [ ] If `invitation.invitee_email` is set, verify it matches the auth user's email
- [ ] If `invitation.invitee_user_id` is set, verify it matches the auth user's ID
- [ ] Return `Unauthorized` on mismatch

**Difficulty:** Intermediate

---

## Issue #24 — Reject-invite requires no authentication

**Labels:** bug, backend, security

**Summary:**
`org_service.rs:162-175` — `reject_invite()` takes only `token`, no `user_id`. Anyone with the invitation token (even unauthenticated) can reject it.

**Acceptance Criteria:**
- [ ] Require auth context
- [ ] Verify caller matches the invitee user or email

**Difficulty:** Intermediate

---

## Issue #25 — Auth session falls back to JWT-only validation when Redis is unavailable

**Labels:** bug, backend, security

**Summary:**
`middleware/auth.rs:41,67` — If Redis is down, `session_data` becomes `None` but auth still succeeds with `wallet_address: ""` and stale permissions. The session validation layer is silently bypassed.

**Acceptance Criteria:**
- [ ] When session validation is required, fail auth if Redis is unavailable (or use DB fallback)
- [ ] Log a warning when falling back to degraded auth mode

**Difficulty:** Intermediate

---

## Issue #26 — Token refresh cache update failure silently swallowed

**Labels:** bug, backend

**Summary:**
`auth_service.rs:164` — `store.store(...).await.ok();` — if Redis write fails during token refresh, the error is discarded. The DB session is updated but the cache is stale, meaning subsequent session reads may get old data.

**Acceptance Criteria:**
- [ ] Log cache update failures at `warn` level
- [ ] Optionally retry cache write once

**Difficulty:** Beginner

---

## Issue #27 — `verify_api_key` bypasses dependency injection

**Labels:** refactor, backend

**Summary:**
`auth_service.rs:211` — creates `PgApiKeyRepository::new(self.pool.clone())` directly instead of using the injected `Arc<dyn ApiKeyRepository>`. This makes unit testing harder and breaks the service layer abstraction.

**Acceptance Criteria:**
- [ ] Use the injected repository field instead of creating a new one
- [ ] Verify in tests that mocked repository is used

**Difficulty:** Beginner

---

## Issue #28 — CORS middleware panics on invalid origin URL

**Labels:** bug, backend

**Summary:**
`middleware/mod.rs:32` — `.allow_origin().parse().expect("...")` — if any origin in the CORS config is an invalid URL, the server panics at startup instead of logging an error and skipping.

**Acceptance Criteria:**
- [ ] Return an error or skip invalid origins instead of panicking
- [ ] Log warning for each invalid configured origin

**Difficulty:** Beginner

---

## Issue #29 — No CSRF, XSS, or security headers configured

**Labels:** feature, backend, security

**Summary:**
`middleware/mod.rs` lacks `Content-Security-Policy`, `X-Frame-Options`, `X-Content-Type-Options`, `Referrer-Policy` headers. HSTS config (`hsts_max_age`) is parsed but never applied.

**Acceptance Criteria:**
- [ ] Add `Content-Security-Policy` header middleware
- [ ] Add `X-Frame-Options: DENY`
- [ ] Add `X-Content-Type-Options: nosniff`
- [ ] Wire HSTS header using configured `hsts_max_age`

**Difficulty:** Intermediate

---

## Issue #30 — No brute-force protection on auth endpoints

**Labels:** feature, backend, security

**Summary:**
`POST /auth/challenge` and `POST /auth/login` have no rate limiting, no failed-attempt tracking, no account lockout. An attacker can attempt unlimited wallet signature challenges.

**Acceptance Criteria:**
- [ ] Rate limit `POST /auth/challenge` to 10 req/min per public key
- [ ] Rate limit `POST /auth/login` to 5 req/min per IP
- [ ] After 10 failed login attempts, lock account for 15 minutes

**Difficulty:** Intermediate

---

## Issue #31 — Audit IP address stored via SQL cast without validation

**Labels:** bug, backend

**Summary:**
`audit_repo.rs:49,51` — `$7::inet` — if `ip_address` is `Some("not-an-ip")`, PostgreSQL throws a cast error. No Rust-side IP address validation before storage.

**Acceptance Criteria:**
- [ ] Validate IP address format before building SQL query
- [ ] Return validation error for invalid IPs

**Difficulty:** Beginner

---

## Issue #32 — `api_key_repo` silently swallows JSON serialization errors

**Labels:** bug, backend

**Summary:**
`api_key_repo.rs:67` — `serde_json::to_value(scopes).unwrap_or_default()` — if scopes cannot be serialized (e.g., non-string types), silently stores empty JSON `{}` in the database instead of reporting the error.

**Acceptance Criteria:**
- [ ] Return error instead of `unwrap_or_default` when serialization fails

**Difficulty:** Beginner

---

# 4. Database & Repositories

---

# 4. Database & Repositories

---

## Issue #33 — `wallet_repo.set_primary` not wrapped in a database transaction

**Labels:** bug, backend

**Summary:**
`wallet_repo.rs:120-131` — two UPDATE statements (unset old primary, set new primary) are executed separately. If the first succeeds and the second fails, no wallet is primary for that user.

**Acceptance Criteria:**
- [ ] Both updates wrapped in `sqlx::Transaction`
- [ ] Rollback on failure

**Difficulty:** Intermediate

---

## Issue #34 — `deployment_repo` missing `update_status` method

**Labels:** bug, backend

**Summary:**
`deployment_repo.rs` has no method to update deployment status. The pipeline state machine in `deployment/pipeline.rs` has no way to persist state transitions (Pending → Building → Deploying → Completed/Failed).

**Acceptance Criteria:**
- [ ] Add `update_status(id: Uuid, status: DeploymentStatus) -> Result<(), AppError>`
- [ ] Validate state transitions in the repository or service layer
- [ ] Integration test verifies status changes persist

**Difficulty:** Intermediate

---

## Issue #35 — `activity_repo.find_by_user` potential duplicate rows from JOIN

**Labels:** bug, backend

**Summary:**
`activity_repo.rs:77-92` — JOIN between `organization_members` and `activity_log` on `organization_id` could produce duplicates if a user belongs to multiple orgs and activity entries exist for multiple orgs. No `SELECT DISTINCT`.

**Acceptance Criteria:**
- [ ] Add `SELECT DISTINCT` to the query
- [ ] Add integration test with multi-org user to verify no duplicates

**Difficulty:** Beginner

---

## Issue #36 — `analytics_service` makes N+1 database queries per org

**Labels:** performance, backend

**Summary:**
`analytics_service.rs:61-69` — for each project, makes 2 separate `COUNT` queries (contracts, deployments). For an org with N projects → 2N+1 queries. Should use `LEFT JOIN ... GROUP BY`.

**Acceptance Criteria:**
- [ ] Replace per-project queries with a single aggregated query
- [ ] Verify query count is O(1) not O(N) via query logging in test

**Difficulty:** Intermediate

---

## Issue #37 — `org_repo` slug uniqueness check doesn't filter soft-deletes

**Labels:** bug, backend

**Summary:**
Slug uniqueness queries during org creation/update may collide with soft-deleted orgs. Soft-delete is `deleted_at IS NOT NULL`, but the uniqueness check may not filter these.

**Acceptance Criteria:**
- [ ] Add `AND deleted_at IS NULL` to all slug uniqueness checks
- [ ] Integration test: create org, soft-delete, create with same slug → succeeds

**Difficulty:** Beginner

---

## Issue #38 — `invitation_repo.find_by_token_hash` doesn't filter by status

**Labels:** bug, backend

**Summary:**
`invitation_repo.rs:70-83` — returns invitations regardless of status (pending, accepted, rejected, expired). Callers must check `status` after retrieval, which is error-prone.

**Acceptance Criteria:**
- [ ] Add `AND status = 'pending'` to the query, or make status filterable via parameter

**Difficulty:** Beginner

---

## Issue #39 — `user_service` uses raw SQL instead of repository method

**Labels:** refactor, backend

**Summary:**
`user_service.rs:57-76` — calls `sqlx::query_as` directly on the pool to UPDATE users instead of calling `user_repo.update()`. Bypasses the repository layer.

**Acceptance Criteria:**
- [ ] Add `update()` method to `PgUserRepository`
- [ ] Replace raw SQL in `user_service` with `user_repo.update()`

**Difficulty:** Beginner

---

## Issue #40 — Missing contract repository `delete` and `update` methods

**Labels:** feature, backend

**Summary:**
`contract_repo.rs` implements 4 trait methods but `ContractRepository` trait has no `delete` or `update`. The handler `contracts.rs` has update and delete routes.

**Acceptance Criteria:**
- [ ] Add `delete` and `update` to `ContractRepository` trait and `PgContractRepository`

**Difficulty:** Beginner

---

# 5. Middleware & Infrastructure

---

# 5. Middleware & Infrastructure

---

## Issue #41 — Rate limiter exists (implemented) but is never wired

**Labels:** feature, backend

**Summary:**
`src/cache/rate_limiter.rs` fully implements a Redis-backed sliding window rate limiter. `RateLimitConfig` is parsed from env. But it is never applied as middleware or called from any route. The entire file is dead code.

**Acceptance Criteria:**
- [ ] Rate limiter middleware applied to protected route layer
- [ ] Stricter limits on public auth endpoints (10 req/min)
- [ ] Rate limit headers in responses

**Difficulty:** Intermediate

---

## Issue #42 — Authorization middleware defined but never compiled or applied

**Labels:** bug, backend

**Summary:**
`middleware/authorization.rs` implements a `require(permission)` middleware factory. It is not declared in `middleware/mod.rs` (Issue #6) and never applied in `router/mod.rs`. All permission checks are ad-hoc in handlers.

**Acceptance Criteria:**
- [ ] Authorization middleware declared in `mod.rs` and compiled
- [ ] Applied per-route-group with appropriate permission strings
- [ ] Ad-hoc `PolicyEngine::check` calls in handlers removed where middleware covers

**Difficulty:** Intermediate

---

## Issue #43 — Org context middleware not compiled or wired

**Labels:** bug, backend

**Summary:**
`middleware/org_context.rs` parses `X-Org-Id` or URL path to populate `OrganizationContext`. Not in `middleware/mod.rs` (Issue #6). Never wired in router. The auth middleware sets `organization: None` and leaves it.

**Acceptance Criteria:**
- [ ] Declared and compiled
- [ ] Wired in router after auth middleware
- [ ] Queries `organization_members` for real role (fixes Issue #4)

**Difficulty:** Intermediate

---

## Issue #44 — Audit middleware not compiled or wired

**Labels:** bug, backend

**Summary:**
`middleware/audit.rs` implements audit logging for mutating requests. Not in `middleware/mod.rs` (Issue #6). Never wired in router.

**Acceptance Criteria:**
- [ ] Declared and compiled
- [ ] Wired for all mutating routes (POST/PATCH/DELETE)
- [ ] Writes to `audit_logs` table via `PgAuditLogRepository` (fixes Issue #5)

**Difficulty:** Intermediate

---

## Issue #45 — Response `request_id` is always empty

**Labels:** bug, backend

**Summary:**
`response/mod.rs` creates `ApiResponse` with `request_id: String::new()`. The `response_meta` middleware either doesn't run or doesn't populate the request ID. Clients cannot correlate responses with server-side traces.

**Acceptance Criteria:**
- [ ] `RequestId` middleware generates UUID per request, stores in extensions
- [ ] Response includes `X-Request-Id` header
- [ ] `request_id` populated in every `ApiResponse`
- [ ] Request ID added to tracing spans

**Difficulty:** Intermediate

---

## Issue #46 — `RwLock<AppState>` adds unnecessary contention on every request

**Labels:** performance, backend

**Summary:**
Every handler calls `state.read().await` on `Arc<RwLock<AppState>>`. Since `Config` is immutable after startup and `db`/`cache` are set once, the RwLock adds synchronization overhead with zero benefit.

**Acceptance Criteria:**
- [ ] Use `Arc<AppState>` directly instead of `Arc<RwLock<AppState>>`
- [ ] If hot-reload is needed, use `arc_swap` for specific fields

**Difficulty:** Intermediate

---

## Issue #47 — Event bus has no persistence — events lost on restart

**Labels:** feature, backend

**Summary:**
`event bus.rs` uses `tokio::sync::broadcast::channel` (bounded, in-memory). If the app restarts, all unprocessed events are lost. Capacity overflow drops oldest events silently. 8 event types carry important domain state.

**Acceptance Criteria:**
- [ ] Transactional outbox pattern: events written to `event_outbox` table in same transaction as state change
- [ ] Background worker reads outbox and publishes to in-memory bus + subscribers
- [ ] At-least-once delivery guarantee
- [ ] Event deduplication by event ID

**Difficulty:** Advanced

---

## Issue #48 — Job runner has no graceful shutdown

**Labels:** bug, backend

**Summary:**
`runner.rs:32-41` — the main loop never checks for `SIGTERM`/`SIGINT`. During shutdown, jobs continue to be dequeued, processed, or lost. No drain-on-exit.

**Acceptance Criteria:**
- [ ] Listen for shutdown signal via `tokio::signal::ctrl_c` or `SIGTERM`
- [ ] On signal, finish current job and stop dequeuing
- [ ] Configurable drain timeout before forced exit

**Difficulty:** Intermediate

---

## Issue #49 — Event bus does not await handler completion or track failures

**Labels:** refactor, backend

**Summary:**
`bus.rs:38-40` — handlers are `tokio::spawn`ed and forgotten. If a handler panics or fails, there's no notification. No way to know if event processing succeeded.

**Acceptance Criteria:**
- [ ] Log handler completion and failures
- [ ] Option to make handler execution blocking (for critical events)

**Difficulty:** Beginner

---

## Issue #50 — OTLP telemetry config parsed but never initialized

**Labels:** bug, backend

**Summary:**
`telemetry/mod.rs` — `TelemetryConfig` has `otlp_endpoint`, `metrics_port`, `enable_otlp` fields. `init()` only sets up `tracing-subscriber fmt`. No OTLP exporter, no Prometheus metrics endpoint despite port 9090 being exposed in Dockerfile.

**Acceptance Criteria:**
- [ ] When `enable_otlp` is true, initialize `opentelemetry-otlp` exporter
- [ ] Start Prometheus metrics server on `metrics_port` (9090)
- [ ] Verify metrics appear at `GET /metrics`

**Difficulty:** Advanced

---

## Issue #51 — `health_check_interval` config parsed but never passed to connection pool

**Labels:** bug, backend

**Summary:**
`database/mod.rs:16-28` — `health_check_interval` is parsed from config but `PgPoolOptions::new()` never calls `.health_check_interval()`. The pool uses the default (30s) regardless of config.

**Acceptance Criteria:**
- [ ] Pass configured `health_check_interval` to `PgPoolOptions`
- [ ] Test that pool interval is configurable

**Difficulty:** Beginner

---

## Issue #52 — Config parsing panics on missing env vars instead of returning errors

**Labels:** bug, backend

**Summary:**
`config/mod.rs:329-345` — `env()` and `env_parse()` call `panic!()` with a message when required env vars are missing. This crashes the server with a raw panic message instead of a graceful startup error.

**Acceptance Criteria:**
- [ ] Return `ConfigError` instead of panicking
- [ ] Main startup prints all missing vars and exits cleanly

**Difficulty:** Beginner

---

## Issue #53 — `stub_handler` pattern in 6 files returns fake UUIDs with no discernible structure

**Labels:** refactor, backend

**Summary:**
`handlers/projects.rs`, `contracts.rs`, `deployments.rs`, `wallets.rs`, `explorer.rs`, `security.rs` all generate `Uuid::new_v4()` and empty strings for every field. This means:
1. No way to distinguish between two stub responses (different requests return different fake IDs)
2. Frontend cannot cache or correlate data
3. If a stub response is accidentally returned in production, the empty strings could cause downstream errors

**Acceptance Criteria:**
- [ ] All handlers return real data from real services
- [ ] No `Uuid::new_v4()` exists in handler response paths

**Difficulty:** Intermediate (per handler)

---

## Issue #54 — `deployment/validator.rs` duplicates `validation/environment.rs`

**Labels:** refactor, backend

**Summary:**
`deployment/validator.rs:6-8` and `validation/environment.rs` both implement `validate_environment()`. One returns `bool`, the other returns `bool`. Identical logic, different locations.

**Acceptance Criteria:**
- [ ] Remove duplicate from `deployment/validator.rs`
- [ ] All callers use `crate::validation::environment::validate_environment()`

**Difficulty:** Beginner

---

## Issue #55 — Both `/v1/organizations` and `/v1/orgs` route prefixes registered

**Labels:** refactor, backend

**Summary:**
`router/mod.rs` registers both `/organizations/*` and `/orgs/*` routes pointing to the same handlers. The frontend uses both inconsistently. This doubles maintenance.

**Acceptance Criteria:**
- [ ] Choose canonical path (`/organizations`) and deprecate `/orgs`
- [ ] Update frontend constants to use canonical paths
- [ ] Add `Deprecation: true` header to legacy route responses

**Difficulty:** Intermediate

---

# 6. Code Quality & Dead Code

---

# 6. Code Quality & Dead Code

---

## Issue #56 — `ENCRYPTION_KEY` config parsed and validated but never used

**Labels:** cleanup, backend

**Summary:**
`config/mod.rs:322-324` — validates `ENCRYPTION_KEY` length >= 16. No code in the entire codebase references this field. It is dead config.

**Acceptance Criteria:**
- [ ] Either implement encryption using the key (e.g., for API key storage)
- [ ] Or remove the config field

**Difficulty:** Beginner

---

## Issue #57 — `hsts_max_age` config parsed but no HSTS header is ever set

**Labels:** cleanup, backend

**Summary:**
`config/mod.rs:202,296` — `hsts_max_age` is parsed but no middleware ever sets `Strict-Transport-Security`. Dead config.

**Acceptance Criteria:**
- [ ] Wire HSTS middleware using the configured max-age
- [ ] Add to `middleware/mod.rs`

**Difficulty:** Beginner

---

## Issue #58 — Three orphaned feature flags in Cargo.toml

**Labels:** cleanup, backend

**Summary:**
`analytics = []`, `cli = []`, `production = []` — three features defined but:
- `analytics`: no `#[cfg(feature = "analytics")]` anywhere
- `cli`: no CLI subcommand implemented
- `production`: only referenced as `cfg!(feature = "production")` in one place

**Acceptance Criteria:**
- [ ] Remove `analytics` and `cli` from `[features]`
- [ ] Document or remove `production`

**Difficulty:** Beginner

---

## Issue #59 — Redundant `pool` field in service implementations

**Labels:** cleanup, backend

**Summary:**
`RepositoryServiceImpl`, `AnalyticsServiceImpl`, `OrgService`, `UserServiceImpl` all accept `Arc<PgPool>` in constructor but never use it — they delegate to repositories that hold their own pool copies. The field is dead storage.

**Acceptance Criteria:**
- [ ] Remove unused `pool` fields from struct and constructor
- [ ] Update all call sites

**Difficulty:** Beginner

---

## Issue #60 — Benchmark files are stubs with TODO comments

**Labels:** cleanup, backend

**Summary:**
`benches/repository.rs` and `benches/crypto.rs` contain only `// TODO: implement` comments. `cargo bench` produces no useful output.

**Acceptance Criteria:**
- [ ] Implement meaningful benchmarks or remove files and `criterion` dev-dependency

**Difficulty:** Intermediate

---

## Issue #61 — `TestAppBuilder` environment-dependent, panics on missing vars

**Labels:** testing, backend

**Summary:**
`tests/common/test_app.rs:20` — calls `Config::from_env().unwrap()` which panics if env vars are missing. No test configuration fallback. Tests cannot run without PostgreSQL and Redis.

**Acceptance Criteria:**
- [ ] `TestConfig::default()` provides safe defaults
- [ ] Integration tests use `testcontainers` (already in dev-deps) or CI service
- [ ] Unit tests don't require database

**Difficulty:** Intermediate

---

## Issue #62 — `LogoutAllRequest` DTO defined but never referenced

**Labels:** cleanup, backend

**Summary:**
`dto/auth.rs:48-50` — `LogoutAllRequest` is defined but no handler accepts it. The `auth::logout_all` handler takes `AuthContext`, not this DTO.

**Acceptance Criteria:**
- [ ] Remove dead DTO or wire to handler

**Difficulty:** Beginner

---

## Issue #63 — `AcceptInviteRequest` and `RejectInviteRequest` DTOs defined but never used

**Labels:** cleanup, backend

**Summary:**
`dto/org.rs:45-52` — both DTOs are defined but the handlers `accept_invite` and `reject_invite` take `Path(token)` not JSON bodies. Dead code.

**Acceptance Criteria:**
- [ ] Remove dead DTOs or update handlers to use them

**Difficulty:** Beginner

---

## Issue #64 — `LogEntry` DTO defined but no handler returns log entries

**Labels:** cleanup, backend

**Summary:**
`dto/deployment.rs:21-26` — `LogEntry` is defined but the deployment logs handler returns `ApiResponse<Vec<LogEntry>>` using its own locally-defined `LogEntry` struct, not the DTO.

**Acceptance Criteria:**
- [ ] Remove the DTO or replace handler's local type with DTO import

**Difficulty:** Beginner

---

## Issue #65 — `src/deployment/` directory orphaned (not in lib.rs)

**Labels:** cleanup, backend

**Summary:**
`src/deployment/` exists with `state.rs`, `validator.rs`, `pipeline.rs` but `lib.rs` does not declare `pub mod deployment;`. Pipeline state machine and validator are never compiled.

**Acceptance Criteria:**
- [ ] Add `pub mod deployment;` to `lib.rs` or move files to appropriate modules

**Difficulty:** Beginner

---

## Issue #66 — `Error::InvalidRequest` variant used nowhere

**Labels:** cleanup, backend

**Summary:**
`errors/mod.rs` — `InvalidRequest` variant is defined but never constructed anywhere in the codebase. Dead error variant.

**Acceptance Criteria:**
- [ ] Remove unused error variant or implement its usage

**Difficulty:** Beginner

---

## Issue #67 — `auth::logout_all` handler exists but frontend has no "log out all devices" button

**Labels:** feature, frontend, backend

**Summary:**
Backend implements `POST /auth/logout-all` that revokes all user sessions. Frontend has no UI for this. The web app logout button only clears local state without calling any endpoint.

**Acceptance Criteria:**
- [ ] Add "Log out all devices" option to user dropdown or settings
- [ ] Call `POST /auth/logout-all` when activated
- [ ] Show confirmation dialog before action

**Difficulty:** Intermediate

---

## Issue #68 — `WalletChallenge` model has no deleted_at but challenges are hard-deleted

**Labels:** cleanup, backend

**Summary:**
`models/challenge.rs` — `WalletChallenge` has no `deleted_at`. Expired challenges are hard-DELETEd by `challenge_repo::delete_expired()`. Inconsistent with soft-delete pattern used elsewhere.

**Acceptance Criteria:**
- [ ] Either add soft-delete for consistency, or document the intentional difference

**Difficulty:** Beginner

---

# 7. Frontend: Auth & State Management

---

# 7. Cross-Cutting Testing

---

## Issue #171 — Backend has zero integration or API tests

**Labels:** testing, backend

**Summary:**
`tests/integration/mod.rs`, `tests/api/mod.rs`, `tests/unit/mod.rs` are all doc-only. No integration test functions exist. The only tests are 22 inline unit tests in `auth/challenge.rs`, `auth/wallet.rs`, `permissions/role.rs`, `permissions/policy.rs`, and `config/tests.rs`.

**Acceptance Criteria:**
- [ ] Integration test for auth flow (challenge → login → refresh → logout)
- [ ] Integration test for org CRUD
- [ ] API contract tests for all implemented endpoints
- [ ] Repository tests with test database

**Difficulty:** Advanced

---

## Issue #172 — `tests/common/test_app.rs` won't compile due to wrong function signature

**Labels:** bug, backend, testing

**Summary:**
`test_app.rs:19-24` — `TestAppBuilder::build()` calls `router::build_router(config)` with one argument, but `build_router` requires TWO arguments (`config`, `state`). This will not compile. The test suite is completely non-functional.

**Acceptance Criteria:**
- [ ] Fix function signature to match `router::build_router`
- [ ] Verify `cargo test` compiles and runs

**Difficulty:** Intermediate

---

## Issue #173 — No E2E tests for authenticated flows in frontend

**Labels:** testing, frontend

**Summary:**
Single E2E test only covers landing page. No test verifies: middleware redirect, login flow, dashboard rendering, navigation between pages.

**Acceptance Criteria:**
- [ ] E2E test for authenticated dashboard (inject cookie)
- [ ] E2E test for middleware redirect when unauthenticated

**Difficulty:** Advanced

---

## Issue #174 — No property-based or fuzz tests for any component

**Labels:** testing

**Summary:**
`proptest` is in backend's dev-dependencies but never used. Contracts deal with numeric IDs, timestamps, and hash values that would benefit from property-based invariants. No fuzz testing exists.

**Acceptance Criteria:**
- [ ] Property-based test for deployment version monotonicity
- [ ] Property-based test for permission hierarchy consistency
- [ ] Property-based test for pagination param bounds

**Difficulty:** Advanced

---

## Issue #175 — No a11y tests or automated accessibility audit

**Labels:** testing, accessibility

**Summary:**
`@storybook/addon-a11y` is installed but no a11y stories exist. No `axe-core` integration in Playwright or Vitest.

**Acceptance Criteria:**
- [ ] Add `axe-playwright` as dev-dependency
- [ ] Add a11y test for landing page and dashboard page

**Difficulty:** Intermediate

---

## Issue #176 — No load or stress tests for backend or contracts

**Labels:** testing

**Summary:**
No benchmarks, load tests, or stress tests exist. No data on how the API performs under concurrent load, how the job queue handles backpressure, or how contracts handle high-throughput scenarios.

**Acceptance Criteria:**
- [ ] `k6` or `locust` test for top 5 API endpoints
- [ ] Contract stress test (max storage size, max event count)

**Difficulty:** Advanced

---

# 20. Developer Experience

---

# 8. Developer Experience

---

## Issue #177 — Frontend CI workflows may be empty or missing

**Labels:** devops, frontend

**Summary:**
`.github/workflows/` — CI/CD workflow files may exist but appear empty based on audit. No automated quality checks run on push/PR.

**Acceptance Criteria:**
- [ ] Verify `ci.yml` contains `typecheck`, `lint`, `test`, `build` jobs
- [ ] Verify `cd.yml` contains Docker build and push
- [ ] README badges reflect actual CI status

**Difficulty:** Beginner

---

## Issue #178 — No Rust/cargo caching in any CI workflow

**Labels:** devops

**Summary:**
Backend CI, contracts CI, and all CD/release workflows build from scratch with no cargo caching. Full build takes 15-30 minutes.

**Acceptance Criteria:**
- [ ] Add `Swatinem/rust-cache` action to all Rust CI workflows
- [ ] Verify cache hit reduces build time to under 5 minutes

**Difficulty:** Beginner

---

## Issue #179 — No Makefile, Justfile, or task runner for common commands

**Labels:** devops

**Summary:**
Backend and contracts have no task runner. Developers must remember exact commands: `cargo build --features default`, `cargo test --workspace`, `cargo clippy --all-targets --all-features`. The `scripts/` directory in contracts is empty.

**Acceptance Criteria:**
- [ ] Create Makefile with targets: `build`, `test`, `lint`, `check`, `clean`, `docker-build`
- [ ] Include WASM build shortcut for contracts

**Difficulty:** Beginner

---

## Issue #180 — `commitlint.config.mjs` uses `sentence-case` — incompatible with conventional commits

**Labels:** bug, devops

**Summary:**
`commitlint.config.mjs:5` — `'subject-case': [2, 'always', 'sentence-case']` — conventional commits typically use lowercase subjects (e.g., `feat: add user login`). This rule will cause CI failures for standard-format commits.

**Acceptance Criteria:**
- [ ] Change to `'lower-case'` or remove the rule

**Difficulty:** Beginner

---

## Issue #181 — Frontend ESLint missing React Hooks plugin rules

**Labels:** devops, frontend

**Summary:**
`eslint.config.mjs` — no `react-hooks/exhaustive-deps` rule. Missing dependency warnings in `useEffect`/`useMemo`/`useCallback` won't be caught. Combined with `--max-warnings 0`, this means code with missing deps can pass lint.

**Acceptance Criteria:**
- [ ] Add `eslint-plugin-react-hooks` rules
- [ ] Address any new warnings

**Difficulty:** Beginner

---

## Issue #182 — Frontend `tsconfig.json` doesn't catch unused variables

**Labels:** devops, frontend

**Summary:**
`tsconfig.json` — `strict: true` but no `noUnusedLocals` or `noUnusedParameters`. Dead code and unused params are not caught by `tsc --noEmit`.

**Acceptance Criteria:**
- [ ] Enable `noUnusedLocals` and `noUnusedParameters`
- [ ] Address any new errors

**Difficulty:** Intermediate

---

## Issue #183 — Storybook and e2e directories excluded from TypeScript checking

**Labels:** devops, frontend

**Summary:**
`tsconfig.json:41-42` — excludes `stories` and `e2e` directories. Type errors in story files and E2E tests go unnoticed.

**Acceptance Criteria:**
- [ ] Include stories and e2e in type-checking (they are small directories)

**Difficulty:** Beginner

---

## Issue #184 — `package.json` `check` script runs sequentially: typecheck → lint → test → build

**Labels:** devops, frontend

**Summary:**
`check` script takes 3x as long as necessary. These steps are independent and could run in parallel.

**Acceptance Criteria:**
- [ ] Use `concurrently` (or `npm-run-all`) to run steps in parallel

**Difficulty:** Beginner

---

## Issue #185 — No `.env.example` files for local development in backend/frontend

**Labels:** documentation

**Summary:**
Backend and frontend do not ship `.env.example` files. Contributors must guess or dig through config code to find required environment variables.

**Acceptance Criteria:**
- [ ] Create `.env.example` for backend with all config variables documented
- [ ] Create `.env.example` for frontend (one may exist, verify completeness)

**Difficulty:** Beginner

---

## Issue #186 — No ADR documents exist beyond template

**Labels:** documentation

**Summary:**
`docs/ADR/` contains a template and two ADRs (use-axum, use-sqlx) but many significant decisions are undocumented: why postgres over mysql, why event bus pattern, why push-based registry pattern, why separate trait/impl pattern.

**Acceptance Criteria:**
- [ ] Add ADRs for: event bus choice, registry architecture, service/repository pattern, deployment state machine design, wallet auth pattern

**Difficulty:** Intermediate
