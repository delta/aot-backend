-- This file should undo anything in `up.sql`

ALTER TABLE "levels_fixture"
DROP COLUMN IF EXISTS rating_factor;
