#!/usr/bin/env bash
set -euo pipefail

# ────────────────────────────────────────────────────────────
# Astro Orbit — Development Environment Reset
# ────────────────────────────────────────────────────────────
# Usage: ./scripts/reset_dev.sh
#
# Drops and recreates the development database, runs all
# migrations, and loads seed data.
# ────────────────────────────────────────────────────────────

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_DIR="$(dirname "$SCRIPT_DIR")"

source "${PROJECT_DIR}/.env" 2>/dev/null || true

DB_URL="${DATABASE_URL:-postgres://astro_orbit:astro_orbit@localhost:5432/astro_orbit}"
DB_NAME="$(basename "${DB_URL}")"
ADMIN_URL="${DB_URL%/*}/postgres"

echo "WARNING: This will DROP the '${DB_NAME}' database and recreate it."
read -rp "Continue? [y/N] " confirm
[[ "${confirm}" =~ ^[Yy]$ ]] || exit 0

echo "Dropping database '${DB_NAME}'..."
psql "${ADMIN_URL}" -c "DROP DATABASE IF EXISTS ${DB_NAME}"

echo "Creating database '${DB_NAME}'..."
psql "${ADMIN_URL}" -c "CREATE DATABASE ${DB_NAME}"

echo "Running migrations..."
cargo sqlx migrate run --database-url "${DB_URL}"

echo "Loading seed data..."
"${SCRIPT_DIR}/seed.sh"

echo "Development environment reset complete."
