-- Create KV stores metadata table
CREATE TABLE IF NOT EXISTS kv_stores (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    guild_id TEXT NOT NULL,
    store_name TEXT NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    UNIQUE (guild_id, store_name)
);

-- Index for faster lookups by guild
CREATE INDEX idx_kv_stores_guild_id ON kv_stores(guild_id);
