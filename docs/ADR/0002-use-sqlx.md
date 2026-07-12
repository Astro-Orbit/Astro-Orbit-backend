# ADR 0002: Use SQLx for Database Access

## Status

Accepted

## Context

We need a PostgreSQL database access library. Options include:

- **SQLx** (async, compile-time checked queries)
- **Diesel** (synchronous ORM, complex setup)
- **SeaORM** (async ORM, higher level)
- **tokio-postgres** (low-level driver)

Key requirements:
- Native async support with Tokio
- Compile-time query verification
- Migration support
- Minimal runtime overhead
- No ORM magic — we want explicit control over SQL

## Decision

We will use **SQLx** as our database access library.

Rationale:
1. **Compile-time checked SQL** — `query!` and `query_as!` macros
   verify SQL against the live database at compile time, catching
   schema mismatches before runtime.
2. **Migration support** — Built-in migration system with
   `sqlx::migrate!` that embeds SQL migrations in the binary.
3. **Async-native** — First-class Tokio support with connection
   pooling, prepared statements, and streaming.
4. **Explicit SQL** — No ORM abstraction layer. We write SQL and
   map results to Rust structs with `FromRow`. This gives us full
   control over query performance.
5. **JSONB support** — First-class JSON/JSONB support matching
   our data model requirements.
6. **PostgreSQL-specific features** — Full support for custom types,
   enums, arrays, and PostgreSQL-specific SQL.

## Consequences

- Compile-time checks require a running database during development.
- No ORM means more manual query writing, but better performance
  and control.
- Migration files must be kept in sync with the database schema.

## Compliance

Enforced by Cargo.toml dependency. CI runs migrations before tests
to enable compile-time verification.
