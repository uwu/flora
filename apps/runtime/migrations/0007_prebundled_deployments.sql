ALTER TABLE deployments
    ADD COLUMN IF NOT EXISTS source_map JSONB;

DO $$
BEGIN
    IF EXISTS (
        SELECT 1
        FROM information_schema.columns
        WHERE table_name = 'deployments'
          AND column_name = 'files'
    ) THEN
        EXECUTE $sql$
            UPDATE deployments
            SET source_map = (
                SELECT file
                FROM jsonb_array_elements(files) AS file
                WHERE (file ->> 'path') LIKE '%.map'
                LIMIT 1
            )
            WHERE source_map IS NULL
        $sql$;
    END IF;
END
$$;

ALTER TABLE deployments
    DROP COLUMN IF EXISTS files;
