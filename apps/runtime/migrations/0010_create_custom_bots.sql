CREATE TABLE IF NOT EXISTS custom_bots (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    owner_user_id TEXT NOT NULL,
    label TEXT,
    bot_user_id TEXT NOT NULL UNIQUE,
    bot_username TEXT NOT NULL,
    application_id TEXT NOT NULL,
    token_ciphertext BYTEA NOT NULL,
    token_nonce BYTEA NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_custom_bots_owner_user_id
    ON custom_bots(owner_user_id, created_at DESC);
