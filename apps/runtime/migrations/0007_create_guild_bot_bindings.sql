CREATE TABLE IF NOT EXISTS guild_bot_bindings (
    guild_id TEXT PRIMARY KEY,
    owner_user_id TEXT NOT NULL,
    bot_user_id TEXT NOT NULL,
    bot_username TEXT NOT NULL,
    application_id TEXT NOT NULL,
    token_ciphertext BYTEA NOT NULL,
    token_nonce BYTEA NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_guild_bot_bindings_bot_user_id
    ON guild_bot_bindings(bot_user_id);
