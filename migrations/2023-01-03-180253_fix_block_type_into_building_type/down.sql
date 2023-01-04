-- This file should undo anything in `up.sql`

ALTER TABLE level_constraints
DROP COLUMN IF EXISTS building_id,
ADD COLUMN block_id INTEGER NOT NULL,
ADD CONSTRAINT level_constraints_pk PRIMARY KEY (level_id, block_id),
ADD CONSTRAINT level_constraints_fk1 FOREIGN KEY (block_id) REFERENCES public.block_type(id);


ALTER TABLE map_spaces
ADD COLUMN blk_type INTEGER NOT NULL,
ADD CONSTRAINT map_spaces_fk1 FOREIGN KEY (blk_type) REFERENCES public.block_type(id);

ALTER TABLE building_type
DROP COLUMN IF EXISTS blk_type;

ALTER TABLE building_type
DROP COLUMN IF EXISTS building_category;

DROP TYPE IF EXISTS building_category;
CREATE TYPE building_category AS ENUM ('building','road' ,'defender','diffuser','mine');

ALTER TABLE building_type
ADD COLUMN building_category building_category NOT NULL;

ALTER TABLE levels_fixture
ADD COLUMN no_of_defenders INTEGER NOT NULL,
ADD COLUMN no_of_mines INTEGER NOT NULL,
ADD COLUMN no_of_diffusers INTEGER NOT NULL;
