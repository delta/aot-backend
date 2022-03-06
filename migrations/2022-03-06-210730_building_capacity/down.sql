-- This file should undo anything in `up.sql`

ALTER TABLE "block_type"
DROP COLUMN IF EXISTS capacity;

ALTER TABLE "levels_fixture"
DROP COLUMN IF EXISTS no_of_robots;
