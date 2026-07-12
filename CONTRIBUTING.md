# Contributing to Astro Orbit

Thank you for considering contributing to Astro Orbit! This document
outlines the contribution workflow and conventions.

## Code of Conduct

All contributors must adhere to our [Code of Conduct](CODE_OF_CONDUCT.md).

## Getting Started

1. Fork the repository
2. Clone your fork: `git clone https://github.com/your-username/astro-orbit-backend.git`
3. Set up the development environment (see [README.md](README.md))
4. Create a feature branch: `git checkout -b feat/my-feature`

## Branch Strategy

| Branch | Purpose |
|--------|---------|
| `main` | Production-ready, protected |
| `develop` | Integration branch |
| `feat/*` | New features |
| `fix/*` | Bug fixes |
| `release/*` | Release candidates |

## Conventional Commits

We use [Conventional Commits](https://www.conventionalcommits.org/):

```
feat(orgs): add organization creation endpoint
fix(deploy): handle timeout in deployment pipeline
docs(api): document authentication flow
refactor(cache): extract cache abstraction
perf(db): add index on deployments.created_at
```

Allowed scopes: `api`, `auth`, `db`, `deploy`, `config`, `ci`, `docs`, `test`

## Development Workflow

1. **Write tests first** where practical
2. **Run linting**: `cargo fmt --all -- --check` and `cargo clippy -- -D warnings`
3. **Run tests**: `cargo test`
4. **Update documentation** if changing public API
5. **Add migration** if changing database schema
6. **Add ADR** if making architectural changes

## Pull Request Process

1. Create a PR against the `develop` branch
2. Fill out the PR template completely
3. Ensure CI passes (lint, test, build)
4. Request review from at least one maintainer
5. Address review feedback (if any)
6. Squash merge once approved

## Definition of Done

- [ ] Code compiles without warnings
- [ ] All tests pass
- [ ] Documentation updated (if public API changed)
- [ ] Migration tested forward + backward
- [ ] API contract verified
- [ ] Performance impact assessed

## Project Structure

```
src/
├── config/        Environment configuration
├── router/        Axum router definitions
├── handlers/      HTTP request handlers
├── middleware/     Request/response middleware
├── auth/          Authentication logic
├── permissions/   RBAC engine
├── services/      Business logic layer
├── repositories/  Data access layer
├── models/        Domain entities
├── dto/           Request/response types
├── errors/        Error handling
├── response/      Response envelope
├── events/        Domain events
├── jobs/          Background jobs
├── stellar/       Soroban integration
├── deployment/    Deployment pipeline
├── analytics/     Analytics engine
├── notifications/ Notification delivery
├── cache/         Redis caching
├── telemetry/     Observability stack
├── validation/    Input validation
└── utils/         Shared utilities
```
