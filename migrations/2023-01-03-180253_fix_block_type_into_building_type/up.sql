-- Your SQL goes here

ALTER TABLE level_constraints
DROP COLUMN IF EXISTS block_id,
ADD COLUMN building_id INTEGER NOT NULL,
ADD CONSTRAINT level_constraints_pk PRIMARY KEY (level_id, building_id),
ADD CONSTRAINT level_constraints_fk1 FOREIGN KEY (building_id) REFERENCES public.building_type(id);

ALTER TABLE map_spaces
DROP COLUMN IF EXISTS blk_type;

ALTER TABLE building_type
ADD COLUMN blk_type INTEGER NOT NULL,
ADD CONSTRAINT blk_type_fk FOREIGN KEY (blk_type) REFERENCES public.block_type(id);

ALTER TABLE building_type
DROP COLUMN IF EXISTS building_category;

DROP TYPE IF EXISTS building_category;
CREATE TYPE building_category AS ENUM ('building','defender','diffuser','mine');

ALTER TABLE building_type
ADD COLUMN building_category building_category NOT NULL;

ALTER TABLE levels_fixture
DROP COLUMN IF EXISTS no_of_defenders,
DROP COLUMN IF EXISTS no_of_mines,
DROP COLUMN IF EXISTS no_of_diffusers;
