-- This file should undo anything in `up.sql`

ALTER TABLE "levels_fixture"
ALTER COLUMN start_date TYPE DATE;

ALTER TABLE "levels_fixture"
ALTER COLUMN end_date TYPE DATE;
