# syntax=docker/dockerfile:1
# ────────────────────────────────────────────────────────────
# Astro Orbit Backend — Multi-stage Docker Build
# ────────────────────────────────────────────────────────────

# ---------- Builder Stage ----------
FROM rust:1.84-slim-bookworm AS builder

RUN apt-get update && apt-get install -y --no-install-recommends \
    pkg-config \
    libssl-dev \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /app

RUN cargo new --bin astro-orbit-backend
COPY Cargo.toml Cargo.lock* ./
RUN cargo build --release --features db-postgres,cache-redis,telemetry 2>/dev/null || true

COPY src ./src
COPY migrations ./migrations
RUN cargo build --release --features db-postgres,cache-redis,telemetry

# ---------- Runner Stage ----------
FROM gcr.io/distroless/cc-debian12:nonroot

WORKDIR /app

COPY --from=builder /app/target/release/astro-orbit-backend /app/astro-orbit-backend
COPY --from=builder /app/migrations /app/migrations

EXPOSE 8080
EXPOSE 9090

USER nonroot:nonroot

ENTRYPOINT ["/app/astro-orbit-backend"]
