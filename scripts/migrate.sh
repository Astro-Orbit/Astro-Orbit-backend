#!/usr/bin/env bash
set -euo pipefail

# ────────────────────────────────────────────────────────────
# Astro Orbit — Database Migration Runner
# ────────────────────────────────────────────────────────────
# Usage: ./scripts/migrate.sh [up|down]
#
# Runs SQLx database migrations in the specified direction.
# Defaults to "up" (apply pending migrations).
# ────────────────────────────────────────────────────────────

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_DIR="$(dirname "$SCRIPT_DIR")"

source "${PROJECT_DIR}/.env" 2>/dev/null || true

DB_URL="${DATABASE_URL:-postgres://astro_orbit:astro_orbit@localhost:5432/astro_orbit}"
DIRECTION="${1:-up}"

case "${DIRECTION}" in
    up)
        echo "Applying pending migrations..."
        cargo sqlx migrate run --database-url "${DB_URL}"
        ;;
    down)
        echo "Reverting last migration..."
        cargo sqlx migrate revert --database-url "${DB_URL}"
        ;;
    *)
        echo "Usage: $0 [up|down]"
        exit 1
        ;;
esac

echo "Migration '${DIRECTION}' completed."
