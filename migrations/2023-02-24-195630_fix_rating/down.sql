-- This file should undo anything in `up.sql`

ALTER TABLE "user"
ALTER COLUMN overall_rating TYPE REAL,
ALTER COLUMN highest_rating TYPE REAL,
DROP COLUMN IF EXISTS avatar;
