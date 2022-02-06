-- This file should undo anything in `up.sql`

ALTER TABLE game
DROP COLUMN IF EXISTS robots_destroyed,
DROP COLUMN IF EXISTS emps_used,
DROP COLUMN IF EXISTS damage_done,
DROP COLUMN IF EXISTS is_attacker_alive;

ALTER TABLE "user"
DROP COLUMN IF EXISTS highest_rating;
