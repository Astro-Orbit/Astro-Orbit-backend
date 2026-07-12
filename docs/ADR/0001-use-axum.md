# ADR 0001: Use Axum as the Web Framework

## Status

Accepted

## Context

We need a Rust web framework for building the HTTP API layer. The main
contenders in the Rust ecosystem are:

- **Axum** (Tokio ecosystem, tower-based)
- **Actix-web** (actor-based, mature)
- **Rocket** (macro-heavy, stable)
- **Warp** (filter-based, less active)

Key requirements:
- Strong async support via Tokio (already selected as runtime)
- Tower middleware ecosystem for observability
- Type-safe extractors and routing
- Active maintenance and community
- Good ergonomics for our layered architecture

## Decision

We will use **Axum** as our web framework.

Rationale:
1. **Tokio-native** — Axum is built by the Tokio team and integrates
   seamlessly with the Tokio ecosystem (tracing, hyper, tower).
2. **Tower middleware** — Axum's tower-based middleware architecture
   allows us to compose reusable middleware layers for auth,
   logging, rate limiting, and request tracing.
3. **Type-safe extractors** — Axum's extractor pattern enables
   compile-time validation of request parameters and bodies.
4. **Modular routing** — Nested routers with `nest` allow clean
   API versioning and resource grouping.
5. **Ecosystem alignment** — Axum is the standard choice for new
   Tokio-based Rust web services and has the largest community.

## Consequences

- We benefit from the full Tower middleware ecosystem.
- Minor: need to stay current with Axum's active development cycle.
- Axum 0.7+ API is stable and well-documented.

## Compliance

Enforced by Cargo.toml dependency and CI build checks.
