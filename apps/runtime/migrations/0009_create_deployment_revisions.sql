CREATE TABLE IF NOT EXISTS deployment_revisions (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    guild_id TEXT NOT NULL,
    entry TEXT NOT NULL,
    files JSONB,
    bundle TEXT NOT NULL,
    source_map JSONB,
    status TEXT NOT NULL CHECK (status IN ('success', 'failed')),
    deployed_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    deploy_source TEXT NOT NULL CHECK (deploy_source IN ('cli', 'webui', 'bootstrap', 'api', 'unknown')),
    actor_user_id TEXT,
    actor_username TEXT,
    actor_type TEXT NOT NULL CHECK (actor_type IN ('session', 'token', 'system')),
    error_message TEXT,
    build_id TEXT,
    base_revision_id UUID,
    change_summary JSONB,
    CONSTRAINT deployment_revisions_base_revision_fkey
        FOREIGN KEY (base_revision_id)
        REFERENCES deployment_revisions(id)
        ON DELETE SET NULL
);

CREATE INDEX IF NOT EXISTS idx_deployment_revisions_guild_deployed_at
    ON deployment_revisions(guild_id, deployed_at DESC, id DESC);

CREATE INDEX IF NOT EXISTS idx_deployment_revisions_guild_status_deployed_at
    ON deployment_revisions(guild_id, status, deployed_at DESC, id DESC);

INSERT INTO deployment_revisions (
    guild_id,
    entry,
    files,
    bundle,
    source_map,
    status,
    deployed_at,
    deploy_source,
    actor_type
)
SELECT
    d.guild_id,
    d.entry,
    d.files,
    d.bundle,
    d.source_map,
    'success',
    COALESCE(d.updated_at, d.created_at, NOW()),
    'unknown',
    'system'
FROM deployments d
WHERE NOT EXISTS (
    SELECT 1
    FROM deployment_revisions r
    WHERE r.guild_id = d.guild_id
);
