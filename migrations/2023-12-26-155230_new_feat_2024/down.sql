-- This file should undo anything in `up.sql`

ALTER TABLE public.user
  DROP COLUMN oauth_token,
  ADD COLUMN phone VARCHAR,
  ADD COLUMN username VARCHAR,
  ADD COLUMN overall_rating INTEGER,
  ADD COLUMN is_pragyan BOOLEAN,
  ADD COLUMN password VARCHAR,
  ADD COLUMN is_verified BOOLEAN,
  ADD COLUMN highest_rating INTEGER,
  ADD COLUMN avatar VARCHAR,
  ADD COLUMN otps_sent INTEGER;
ALTER TABLE public.user
  DROP COLUMN attacks_won,
  DROP COLUMN defenses_won,
  DROP COLUMN trophies,
  DROP COLUMN avatar_id,
  DROP COLUMN artifacts;

ALTER TABLE public.game
  ADD COLUMN robots_destroyed INTEGER,
  DROP COLUMN artifacts_collected;

ALTER TABLE public.map_spaces
  ADD COLUMN blk_type VARCHAR,
  ADD COLUMN rotation INTEGER,
  ADD COLUMN building_type VARCHAR,
  DROP COLUMN block_type_id,
  DROP CONSTRAINT block_type_id_fk;

DROP TABLE IF EXISTS public.artifact;
DROP TABLE IF EXISTS public.available_blocks;

ALTER TABLE public.attacker_type
  DROP COLUMN level,
  DROP COLUMN cost;

ALTER TABLE public.defender_type
  DROP COLUMN level,
  DROP COLUMN cost;

ALTER TABLE public.mine_type
  DROP COLUMN level,
  DROP COLUMN cost;

ALTER TABLE public.building_type
  ADD COLUMN defender_type VARCHAR,
  ADD COLUMN building_category VARCHAR,
  ADD COLUMN mine_type VARCHAR,
  ADD COLUMN diffuser_type VARCHAR,
  DROP COLUMN name,
  DROP COLUMN width,
  DROP COLUMN height,
  DROP COLUMN capacity,
  DROP COLUMN level,
  DROP COLUMN cost;

DROP TYPE IF EXISTS block_category CASCADE;

ALTER TABLE public.block_type
  DROP COLUMN category,
  DROP COLUMN category_id,
  ADD COLUMN name VARCHAR,
  ADD COLUMN width INTEGER,
  ADD COLUMN height INTEGER,
  ADD COLUMN capacity INTEGER,
  ADD COLUMN entrance_x INTEGER,
  ADD COLUMN entrance_y INTEGER;
