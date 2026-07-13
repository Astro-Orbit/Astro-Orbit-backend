-- ────────────────────────────────────────────────────────────
-- Astro Orbit — Auth & Identity Schema
-- Migration: 002_auth_identity
-- ────────────────────────────────────────────────────────────

-- ── Wallet Challenges (replay-protected nonce store) ──────

CREATE TABLE wallet_challenges (
    id              UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    public_key      VARCHAR(56) NOT NULL,
    challenge       TEXT NOT NULL,
    expires_at      TIMESTAMPTZ NOT NULL,
    used_at         TIMESTAMPTZ,
    created_at      TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_wallet_challenges_pk_expires ON wallet_challenges(public_key, expires_at);
CREATE INDEX idx_wallet_challenges_unused ON wallet_challenges(used_at) WHERE used_at IS NULL;

-- ── Link wallets to users (multi-wallet support) ─────────

ALTER TABLE wallets ADD COLUMN user_id UUID REFERENCES users(id) ON DELETE CASCADE;
ALTER TABLE wallets ADD COLUMN is_primary BOOLEAN NOT NULL DEFAULT FALSE;
CREATE INDEX idx_wallets_user ON wallets(user_id) WHERE deleted_at IS NULL;

-- ── Organization Invitations ─────────────────────────────

CREATE TABLE organization_invitations (
    id              UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    organization_id UUID NOT NULL REFERENCES organizations(id) ON DELETE CASCADE,
    invited_by      UUID NOT NULL REFERENCES users(id),
    invitee_email   VARCHAR(255),
    invitee_user_id UUID REFERENCES users(id),
    token_hash      VARCHAR(64) NOT NULL,
    role            VARCHAR(20) NOT NULL DEFAULT 'developer',
    status          VARCHAR(20) NOT NULL DEFAULT 'pending',
    expires_at      TIMESTAMPTZ NOT NULL,
    created_at      TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at      TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_invitations_org ON organization_invitations(organization_id);
CREATE INDEX idx_invitations_token ON organization_invitations(token_hash);

-- ── Extend sessions for device tracking ──────────────────

ALTER TABLE sessions ADD COLUMN device_type VARCHAR(50);
ALTER TABLE sessions ADD COLUMN device_name VARCHAR(100);
ALTER TABLE sessions ADD COLUMN last_used_at TIMESTAMPTZ;

-- ── Extend api_keys for user-level (personal) keys ──────

ALTER TABLE api_keys ADD COLUMN user_id UUID REFERENCES users(id) ON DELETE CASCADE;
ALTER TABLE api_keys ALTER COLUMN organization_id DROP NOT NULL;

-- ── Updated At Trigger for invitations ────────────────────

CREATE TRIGGER trg_invitations_updated_at
    BEFORE UPDATE ON organization_invitations
    FOR EACH ROW EXECUTE FUNCTION update_updated_at_column();
