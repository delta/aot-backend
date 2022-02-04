-- This file should undo anything in `up.sql`

ALTER TABLE game
DROP COLUMN IF EXISTS robots_destroyed,
DROP COLUMN IF EXISTS emps_used,
DROP COLUMN IF EXISTS damage_done,
DROP COLUMN IF EXISTS no_of_attacker_suicided;

ALTER TABLE "user"
DROP COLUMN IF EXISTS highest_rating;
