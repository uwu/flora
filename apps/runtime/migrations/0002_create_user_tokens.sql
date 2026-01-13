-- Create API tokens table
CREATE TABLE IF NOT EXISTS user_tokens (
    token_id TEXT PRIMARY KEY,
    user_id TEXT NOT NULL,
    label TEXT,
    token_hash TEXT NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    last_used_at TIMESTAMPTZ
);
