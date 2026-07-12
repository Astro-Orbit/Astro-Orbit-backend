#!/usr/bin/env bash
set -euo pipefail

# ────────────────────────────────────────────────────────────
# Astro Orbit — Seed Data Loader
# ────────────────────────────────────────────────────────────
# Usage: ./scripts/seed.sh
#
# Inserts seed data for local development.
# ────────────────────────────────────────────────────────────

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_DIR="$(dirname "$SCRIPT_DIR")"

source "${PROJECT_DIR}/.env" 2>/dev/null || true

DB_URL="${DATABASE_URL:-postgres://astro_orbit:astro_orbit@localhost:5432/astro_orbit}"

echo "Loading seed data..."
psql "${DB_URL}" <<'EOF'
-- Seed: Demo Organization
INSERT INTO organizations (id, name, slug, description, plan)
VALUES (
    '00000000-0000-0000-0000-000000000001',
    'Astro Labs',
    'astro-labs',
    'Demo organization for development',
    'free'
) ON CONFLICT (slug) DO NOTHING;

-- Seed: Demo Project
INSERT INTO projects (id, organization_id, name, slug, description)
VALUES (
    '00000000-0000-0000-0000-000000000010',
    '00000000-0000-0000-0000-000000000001',
    'Marketplace',
    'marketplace',
    'Demo NFT marketplace contract'
) ON CONFLICT (organization_id, slug) DO NOTHING;
EOF

echo "Seed data loaded successfully."
