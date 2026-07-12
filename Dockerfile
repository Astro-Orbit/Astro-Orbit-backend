# syntax=docker/dockerfile:1
# ────────────────────────────────────────────────────────────
# Astro Orbit Backend — Multi-stage Docker Build with Cache
# ────────────────────────────────────────────────────────────

# ---------- Planner Stage ----------
FROM rust:slim-bookworm AS planner

RUN rustup component add rustfmt clippy

RUN apt-get update && apt-get install -y --no-install-recommends \
    pkg-config \
    libssl-dev \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /app

# We just need the files for dependency resolution
COPY Cargo.toml Cargo.lock* ./
RUN mkdir src benches && echo "fn main() {}" > src/main.rs && echo "" > src/lib.rs
RUN touch benches/repository.rs

# ---------- Builder Stage ----------
FROM rust:slim-bookworm AS builder

RUN apt-get update && apt-get install -y --no-install-recommends \
    pkg-config \
    libssl-dev \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /app

# Copy cached dependencies from planner
COPY --from=planner /app/Cargo.toml /app/Cargo.lock* ./
COPY --from=planner /app/src ./src
COPY --from=planner /app/benches ./benches

# Build dependencies first (cached layer)
RUN cargo build --release --features default 2>/dev/null || true

# Copy real source code
COPY src ./src
COPY benches ./benches
COPY migrations ./migrations

# Force rebuild of the actual application
RUN touch src/main.rs src/lib.rs
RUN cargo build --release --features default

# ---------- Runner Stage ----------
FROM gcr.io/distroless/cc-debian12:nonroot

WORKDIR /app

COPY --from=builder /app/target/release/astro-orbit-backend /app/astro-orbit-backend
COPY --from=builder /app/migrations /app/migrations

EXPOSE 8080
EXPOSE 9090

USER nonroot:nonroot

HEALTHCHECK --interval=30s --timeout=10s --start-period=5s --retries=3 \
    CMD ["/app/astro-orbit-backend", "health"]

ENTRYPOINT ["/app/astro-orbit-backend"]
