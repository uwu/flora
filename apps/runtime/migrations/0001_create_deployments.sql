-- Create per-guild deployment storage
CREATE TABLE IF NOT EXISTS deployments (
    guild_id TEXT PRIMARY KEY,
    language TEXT NOT NULL,
    script TEXT NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);
