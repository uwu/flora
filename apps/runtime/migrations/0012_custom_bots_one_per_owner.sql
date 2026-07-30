-- Enforce one User Bot per owner. Newer duplicates are removed along with
-- their persisted deployments, keeping the oldest bot for each owner.
WITH duplicate_bots AS (
    SELECT a.id
    FROM custom_bots a
    JOIN custom_bots b ON b.owner_user_id = a.owner_user_id
    WHERE (a.created_at, a.id) > (b.created_at, b.id)
), removed_bots AS (
    DELETE FROM custom_bots
    WHERE id IN (SELECT id FROM duplicate_bots)
    RETURNING id
)
DELETE FROM deployments
WHERE guild_id IN (
    SELECT '__flora_custom_bot__:' || id::text FROM removed_bots
);

DROP INDEX IF EXISTS idx_custom_bots_owner_user_id;

CREATE UNIQUE INDEX IF NOT EXISTS custom_bots_owner_user_id_unique
    ON custom_bots(owner_user_id);
