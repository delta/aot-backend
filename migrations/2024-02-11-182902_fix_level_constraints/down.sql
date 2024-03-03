-- This file should undo anything in `up.sql`
ALTER TABLE public.level_constraints
    RENAME COLUMN no_of_blocks TO no_of_buildings;

ALTER TABLE public.level_constraints
    DROP CONSTRAINT IF EXISTS level_constraints_fk1;

ALTER TABLE public.level_constraints
    ADD CONSTRAINT level_constraints_fk1 FOREIGN KEY (block_id) REFERENCES public.building_type(id);

ALTER TABLE public.level_constraints
    RENAME COLUMN block_id TO building_id;
