-- Your SQL goes here

CREATE TABLE public.level_constraints (
    level_id INTEGER NOT NULL,
    block_id INTEGER NOT NULL,
    no_of_buildings INTEGER NOT NULL,
    CONSTRAINT level_constraints_pk PRIMARY KEY (level_id, block_id),
    CONSTRAINT level_constraints_fk0 FOREIGN KEY (level_id) REFERENCES public.levels_fixture(id),
    CONSTRAINT level_constraints_fk1 FOREIGN KEY (block_id) REFERENCES public.block_type(id)
) WITH (
    OIDS=FALSE
);


ALTER TABLE block_type
ADD COLUMN entrance_x INTEGER,
ADD COLUMN entrance_y INTEGER;

UPDATE block_type
SET entrance_x = 0;

UPDATE block_type
SET entrance_y = 0;

ALTER TABLE block_type
ALTER COLUMN entrance_x SET NOT NULL,
ALTER COLUMN entrance_y SET NOT NULL;


ALTER TABLE map_spaces
ADD COLUMN rotation INTEGER CHECK (rotation >= 0 and rotation <= 270 and rotation % 90 = 0);

UPDATE map_spaces
SET rotation = 0;

ALTER TABLE map_spaces
ALTER COLUMN rotation SET NOT NULL;


ALTER TABLE attack_type
ALTER COLUMN att_type TYPE VARCHAR(255);


ALTER TABLE levels_fixture
ADD COLUMN no_of_bombs INTEGER;

UPDATE levels_fixture
SET no_of_bombs = 0;

ALTER TABLE levels_fixture
ALTER COLUMN no_of_bombs SET NOT NULL;

ALTER TABLE attacker_path
ALTER COLUMN emp_type DROP NOT NULL,
ALTER COLUMN emp_time DROP NOT NULL;


CREATE TABLE public.building_weights (
    time INTEGER NOT NULL,
    building_id INTEGER NOT NULL,
    weight INTEGER NOT NULL,
    CONSTRAINT building_weights_pk PRIMARY KEY (time, building_id),
    CONSTRAINT building_weights_fk0 FOREIGN KEY (building_id) REFERENCES public.block_type(id)
) WITH (
    OIDS=FALSE
);

ALTER TABLE block_type
DROP COLUMN weight;
