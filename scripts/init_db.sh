#!/usr/bin/env bash
set -euo pipefail

# ────────────────────────────────────────────────────────────
# Astro Orbit — Database Initialization
# ────────────────────────────────────────────────────────────
# Usage: ./scripts/init_db.sh
#
# Creates the database if it doesn't exist and runs all
# pending migrations.
# ────────────────────────────────────────────────────────────

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_DIR="$(dirname "$SCRIPT_DIR")"

source "${PROJECT_DIR}/.env" 2>/dev/null || true

DB_URL="${DATABASE_URL:-postgres://astro_orbit:astro_orbit@localhost:5432/astro_orbit}"
DB_NAME="$(basename "${DB_URL}")"
ADMIN_URL="${DB_URL%/*}/postgres"

echo "Creating database '${DB_NAME}' if it does not exist..."
psql "${ADMIN_URL}" -tc "SELECT 1 FROM pg_database WHERE datname = '${DB_NAME}'" | grep -q 1 \
    || psql "${ADMIN_URL}" -c "CREATE DATABASE ${DB_NAME}"

echo "Running migrations..."
cargo sqlx migrate run --database-url "${DB_URL}"

echo "Database initialized successfully."
