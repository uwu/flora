-- Migrate deployments from guild-only key to generic scope key.
ALTER TABLE deployments
    ADD COLUMN IF NOT EXISTS scope_type TEXT,
    ADD COLUMN IF NOT EXISTS scope_id TEXT;

UPDATE deployments
SET
    scope_type = COALESCE(scope_type, 'guild'),
    scope_id = COALESCE(scope_id, guild_id);

ALTER TABLE deployments
    ALTER COLUMN scope_type SET NOT NULL,
    ALTER COLUMN scope_id SET NOT NULL;

ALTER TABLE deployments
    DROP CONSTRAINT IF EXISTS deployments_pkey;

ALTER TABLE deployments
    ADD CONSTRAINT deployments_pkey PRIMARY KEY (scope_type, scope_id);

ALTER TABLE deployments
    DROP COLUMN IF EXISTS guild_id;
