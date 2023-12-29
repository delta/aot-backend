-- This file should undo anything in `up.sql`

CREATE TABLE diffuser_type (
    id SERIAL PRIMARY KEY,
    radius INTEGER,
    speed INTEGER
);


CREATE INDEX idx_time ON building_weights (time);

CREATE INDEX idx_building_id ON building_weights (building_id);

CREATE TABLE building_weights (
    time INTEGER PRIMARY KEY,
    building_id INTEGER,
    weight INTEGER
);

ALTER TABLE block_type
DROP COLUMN category_id,
DROP COLUMN category,
ADD COLUMN entrance_y INTEGER,
ADD COLUMN entrance_x INTEGER,
ADD COLUMN capacity INTEGER,
ADD COLUMN height INTEGER,
ADD COLUMN width INTEGER,
ADD COLUMN name VARCHAR;

DROP TYPE block_category;

ALTER TABLE public.building_type
DROP COLUMN level,
DROP COLUMN cost,
ADD COLUMN defender_type INTEGER,
ADD COLUMN building_category VARCHAR,
ADD COLUMN mine_type INTEGER,
ADD COLUMN diffuser_type INTEGER,
DROP COLUMN name,
DROP COLUMN width,
DROP COLUMN height,
DROP COLUMN capacity;


ALTER TABLE public.mine_type
DROP COLUMN level,
DROP COLUMN cost;

ALTER TABLE public.defender_type
DROP COLUMN level,
DROP COLUMN cost;

ALTER TABLE public.attacker_type
DROP COLUMN level,
DROP COLUMN cost;

DROP TABLE IF EXISTS public.available_blocks;
DROP TABLE IF EXISTS public.artifact;


ALTER TABLE public.map_spaces
ADD rotation INTEGER,
ADD building_type INTEGER,
DROP block_type_id,
DROP CONSTRAINT IF EXISTS block_type_id_fk;

ALTER TABLE public.game
ADD  robots_destroyed INTEGER,
DROP  artifacts_collected ;

ALTER TABLE public.user
DROP COLUMN artifacts,
DROP COLUMN avatar_id,
DROP COLUMN trophies,
DROP COLUMN defenses_won,
DROP COLUMN attacks_won,
ADD COLUMN otps_sent INTEGER,
ADD COLUMN avatar VARCHAR,
ADD COLUMN highest_rating INTEGER,
ADD COLUMN is_verified BOOLEAN,
ADD COLUMN password VARCHAR,
ADD COLUMN is_pragyan BOOLEAN,
ADD COLUMN overall_rating INTEGER,
ADD COLUMN username VARCHAR,
ADD COLUMN phone VARCHAR,
DROP COLUMN oauth_token;
