-- This file should undo anything in `up.sql`
DELETE FROM actions
WHERE expiration IS NOT NULL;

ALTER TABLE actions
DROP COLUMN expiration;
