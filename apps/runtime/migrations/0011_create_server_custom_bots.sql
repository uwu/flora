CREATE TABLE IF NOT EXISTS server_custom_bots (
    guild_id TEXT PRIMARY KEY,
    configured_by_user_id TEXT NOT NULL,
    bot_user_id TEXT NOT NULL UNIQUE,
    bot_username TEXT NOT NULL,
    application_id TEXT NOT NULL,
    token_ciphertext BYTEA NOT NULL,
    token_nonce BYTEA NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);
