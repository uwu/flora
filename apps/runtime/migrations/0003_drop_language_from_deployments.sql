-- Drop language column; code is auto-detected at runtime
ALTER TABLE deployments
    DROP COLUMN IF EXISTS language;
