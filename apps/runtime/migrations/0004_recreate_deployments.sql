-- Recreate deployments table with multi-file bundle support
DROP TABLE IF EXISTS deployments;

CREATE TABLE IF NOT EXISTS deployments (
    guild_id TEXT PRIMARY KEY,
    entry TEXT NOT NULL,
    files JSONB NOT NULL,
    bundle TEXT NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);
