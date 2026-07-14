# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [0.4.0] - 2026-07-14

### Added

- Repository management: link/unlink git repos to projects with sync support
- Dashboard aggregation endpoints: org stats and user activity feed
- Soroban registry client for on-chain contract registry lookups
- Soroban RPC client with typed JSON-RPC calls
- Comprehensive deployment state machine with validated state transitions
- Deployment pipeline with build, scan, and deploy stages + Soroban integration
- Background job runner with priority queue, scheduling, and dead-letter queue
- Event bus with broadcast and handler registration
- Background workers for deployment and notification jobs
- Activity tracking model and repository for audit trails
- Deployment pre-validation (environment checks, contract requirements)
- Stellar-specific error types with AppError conversion

### Changed

- Fleshed out all stellar/ stubs into functional modules
- Fleshed out all deployment/ stubs into functional modules
- Fleshed out all jobs/ stubs (types, runner, queue, workers)
- Fleshed out events/ stubs (bus, types)
- Enhanced org repository with member_count query
- Updated analytics handler with service-backed dashboard endpoints

## [0.3.0] - 2026-07-13

### Added

- Weekly audit security advisories patched
- Enhanced permission policy engine
- Notification preferences

### Changed

- Audit findings remediated

## [0.2.0] - 2026-07-12

### Added

- Organization CRUD with member management and invitations
- Project management (create, list, update, delete)
- Contract registration and version tracking
- Deployment lifecycle (create, list, rollback, cancel, logs)
- API key management
- Wallet management
- Explorer endpoints (contracts, transactions, events)
- Analytics endpoints (overview, contract calls, gas usage, active users)
- Notification system (list, mark read, preferences)
- Security scanning (create scan, list scans, get findings)
- Admin endpoints (users, orgs, stats)
- User profile endpoints (get/update me, get by ID)
- Permission system with role-based access control
- Middleware auth layer

## [0.1.0] - 2026-07-12

### Added

- Repository foundation with layered architecture
- Axum web framework integration
- SQLx database access with PostgreSQL
- Redis caching layer
- Telemetry stack (tracing, OpenTelemetry, Prometheus)
- Unified error handling and response envelopes
- JWT-based authentication skeleton
- Health check and metrics endpoints
- Docker development and production configurations
- CI/CD pipeline with GitHub Actions
- Architecture documentation (ARCHITECTURE.md)
- Architecture Decision Records (ADRs)
