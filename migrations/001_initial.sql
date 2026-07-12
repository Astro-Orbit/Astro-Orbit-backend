-- ────────────────────────────────────────────────────────────
-- Astro Orbit — Initial Schema
-- Migration: 001_initial
-- ────────────────────────────────────────────────────────────

-- Enable UUID generation
CREATE EXTENSION IF NOT EXISTS "pgcrypto";

-- ── Users ─────────────────────────────────────────────────

CREATE TABLE users (
    id              UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    stellar_public  VARCHAR(56) NOT NULL UNIQUE,
    display_name    VARCHAR(100),
    avatar_url      TEXT,
    email           VARCHAR(255),
    email_verified  BOOLEAN NOT NULL DEFAULT FALSE,
    totp_enabled    BOOLEAN NOT NULL DEFAULT FALSE,
    created_at      TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at      TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    deleted_at      TIMESTAMPTZ
);

CREATE INDEX idx_users_stellar_public ON users(stellar_public) WHERE deleted_at IS NULL;

-- ── Organizations ─────────────────────────────────────────

CREATE TABLE organizations (
    id          UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    name        VARCHAR(100) NOT NULL,
    slug        VARCHAR(100) NOT NULL UNIQUE,
    description TEXT,
    avatar_url  TEXT,
    plan        VARCHAR(20) NOT NULL DEFAULT 'free',
    settings    JSONB NOT NULL DEFAULT '{}',
    created_at  TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at  TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    deleted_at  TIMESTAMPTZ
);

CREATE INDEX idx_organizations_slug ON organizations(slug) WHERE deleted_at IS NULL;

-- ── Organization Members ─────────────────────────────────

CREATE TABLE organization_members (
    organization_id  UUID NOT NULL REFERENCES organizations(id) ON DELETE CASCADE,
    user_id          UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    role             VARCHAR(20) NOT NULL DEFAULT 'member',
    joined_at        TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    PRIMARY KEY (organization_id, user_id)
);

CREATE INDEX idx_org_members_user ON organization_members(user_id);

-- ── Projects ─────────────────────────────────────────────

CREATE TABLE projects (
    id               UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    organization_id  UUID NOT NULL REFERENCES organizations(id) ON DELETE CASCADE,
    name             VARCHAR(100) NOT NULL,
    slug             VARCHAR(100) NOT NULL,
    description      TEXT,
    visibility       VARCHAR(20) NOT NULL DEFAULT 'private',
    network          VARCHAR(20) NOT NULL DEFAULT 'testnet',
    settings         JSONB NOT NULL DEFAULT '{}',
    created_at       TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at       TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    deleted_at       TIMESTAMPTZ,
    UNIQUE(organization_id, slug)
);

CREATE INDEX idx_projects_org ON projects(organization_id) WHERE deleted_at IS NULL;

-- ── Contracts ────────────────────────────────────────────

CREATE TABLE contracts (
    id              UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    project_id      UUID NOT NULL REFERENCES projects(id) ON DELETE CASCADE,
    name            VARCHAR(100) NOT NULL,
    contract_id     VARCHAR(56) NOT NULL,
    wasm_hash       VARCHAR(64),
    source_language VARCHAR(20) NOT NULL DEFAULT 'rust',
    abi             JSONB,
    verified        BOOLEAN NOT NULL DEFAULT FALSE,
    created_at      TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at      TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    deleted_at      TIMESTAMPTZ,
    UNIQUE(project_id, name)
);

CREATE INDEX idx_contracts_project ON contracts(project_id) WHERE deleted_at IS NULL;
CREATE INDEX idx_contracts_contract_id ON contracts(contract_id) WHERE deleted_at IS NULL;

-- ── Contract Versions ─────────────────────────────────────

CREATE TABLE contract_versions (
    id              UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    contract_id     UUID NOT NULL REFERENCES contracts(id) ON DELETE CASCADE,
    version         VARCHAR(20) NOT NULL,
    wasm_hash       VARCHAR(64) NOT NULL,
    metadata        JSONB NOT NULL DEFAULT '{}',
    created_at      TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_contract_versions_contract ON contract_versions(contract_id);

-- ── Deployments ──────────────────────────────────────────

CREATE TYPE deployment_status AS ENUM (
    'pending', 'building', 'scanning', 'deploying',
    'deployed', 'failed', 'rolled_back', 'cancelled'
);

CREATE TABLE deployments (
    id              UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    project_id      UUID NOT NULL REFERENCES projects(id) ON DELETE CASCADE,
    environment     VARCHAR(20) NOT NULL DEFAULT 'development',
    status          deployment_status NOT NULL DEFAULT 'pending',
    commit_sha      VARCHAR(40),
    commit_message  TEXT,
    branch          VARCHAR(100),
    version         VARCHAR(20),
    metadata        JSONB NOT NULL DEFAULT '{}',
    created_by      UUID NOT NULL REFERENCES users(id),
    started_at      TIMESTAMPTZ,
    completed_at    TIMESTAMPTZ,
    created_at      TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at      TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_deployments_project ON deployments(project_id);
CREATE INDEX idx_deployments_status ON deployments(status);
CREATE INDEX idx_deployments_created_at ON deployments(created_at DESC);

-- ── Deployment Contracts (join) ──────────────────────────

CREATE TABLE deployment_contracts (
    deployment_id UUID NOT NULL REFERENCES deployments(id) ON DELETE CASCADE,
    contract_id   UUID NOT NULL REFERENCES contracts(id) ON DELETE CASCADE,
    PRIMARY KEY (deployment_id, contract_id)
);

-- ── Deployment Logs ───────────────────────────────────────

CREATE TABLE deployment_logs (
    id            UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    deployment_id UUID NOT NULL REFERENCES deployments(id) ON DELETE CASCADE,
    level         VARCHAR(10) NOT NULL DEFAULT 'info',
    message       TEXT NOT NULL,
    created_at    TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_deployment_logs_deployment ON deployment_logs(deployment_id);

-- ── Wallets ──────────────────────────────────────────────

CREATE TABLE wallets (
    id          UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    public_key  VARCHAR(56) NOT NULL UNIQUE,
    name        VARCHAR(100),
    created_at  TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at  TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    deleted_at  TIMESTAMPTZ
);

CREATE INDEX idx_wallets_public_key ON wallets(public_key) WHERE deleted_at IS NULL;

-- ── API Keys ─────────────────────────────────────────────

CREATE TABLE api_keys (
    id              UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    organization_id UUID NOT NULL REFERENCES organizations(id) ON DELETE CASCADE,
    name            VARCHAR(100) NOT NULL,
    key_hash        VARCHAR(64) NOT NULL,
    key_prefix      VARCHAR(8) NOT NULL,
    permissions     JSONB NOT NULL DEFAULT '[]',
    last_used_at    TIMESTAMPTZ,
    expires_at      TIMESTAMPTZ,
    created_by      UUID NOT NULL REFERENCES users(id),
    created_at      TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    deleted_at      TIMESTAMPTZ
);

CREATE INDEX idx_api_keys_org ON api_keys(organization_id) WHERE deleted_at IS NULL;

-- ── Sessions ─────────────────────────────────────────────

CREATE TABLE sessions (
    id              UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id         UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    refresh_token_hash VARCHAR(64) NOT NULL,
    ip_address      INET,
    user_agent      TEXT,
    expires_at      TIMESTAMPTZ NOT NULL,
    created_at      TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    revoked_at      TIMESTAMPTZ
);

CREATE INDEX idx_sessions_user ON sessions(user_id);
CREATE INDEX idx_sessions_refresh_token ON sessions(refresh_token_hash);
CREATE INDEX idx_sessions_expires ON sessions(expires_at) WHERE revoked_at IS NULL;

-- ── Audit Log ────────────────────────────────────────────

CREATE TABLE audit_logs (
    id              UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    organization_id UUID REFERENCES organizations(id) ON DELETE SET NULL,
    actor_id        UUID REFERENCES users(id) ON DELETE SET NULL,
    action          VARCHAR(50) NOT NULL,
    resource_type   VARCHAR(50) NOT NULL,
    resource_id     UUID NOT NULL,
    metadata        JSONB NOT NULL DEFAULT '{}',
    ip_address      INET,
    created_at      TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_audit_logs_org ON audit_logs(organization_id);
CREATE INDEX idx_audit_logs_actor ON audit_logs(actor_id);
CREATE INDEX idx_audit_logs_created ON audit_logs(created_at DESC);

-- ── Updated At Trigger ────────────────────────────────────

CREATE OR REPLACE FUNCTION update_updated_at_column()
RETURNS TRIGGER AS $$
BEGIN
    NEW.updated_at = NOW();
    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

CREATE TRIGGER trg_users_updated_at
    BEFORE UPDATE ON users
    FOR EACH ROW EXECUTE FUNCTION update_updated_at_column();

CREATE TRIGGER trg_organizations_updated_at
    BEFORE UPDATE ON organizations
    FOR EACH ROW EXECUTE FUNCTION update_updated_at_column();

CREATE TRIGGER trg_projects_updated_at
    BEFORE UPDATE ON projects
    FOR EACH ROW EXECUTE FUNCTION update_updated_at_column();

CREATE TRIGGER trg_contracts_updated_at
    BEFORE UPDATE ON contracts
    FOR EACH ROW EXECUTE FUNCTION update_updated_at_column();

CREATE TRIGGER trg_deployments_updated_at
    BEFORE UPDATE ON deployments
    FOR EACH ROW EXECUTE FUNCTION update_updated_at_column();

CREATE TRIGGER trg_wallets_updated_at
    BEFORE UPDATE ON wallets
    FOR EACH ROW EXECUTE FUNCTION update_updated_at_column();
