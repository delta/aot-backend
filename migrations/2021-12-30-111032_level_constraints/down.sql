-- This file should undo anything in `up.sql`

DROP TABLE public.level_constraints;

ALTER TABLE block_type
DROP COLUMN entrance_x,
DROP COLUMN entrance_y;

ALTER TABLE map_spaces
DROP COLUMN rotation;

ALTER TABLE attack_type
ALTER COLUMN att_type TYPE INTEGER USING (att_type::integer);

ALTER TABLE levels_fixture
DROP COLUMN no_of_bombs;

ALTER TABLE attacker_path
ALTER COLUMN emp_type SET NOT NULL,
ALTER COLUMN emp_time SET NOT NULL;

DROP TABLE public.building_weights;

ALTER TABLE block_type
ADD COLUMN weight INTEGER;
