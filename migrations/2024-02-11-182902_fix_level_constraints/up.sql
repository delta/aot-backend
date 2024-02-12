-- Your SQL goes here
ALTER TABLE public.level_constraints
    RENAME COLUMN no_of_buildings TO no_of_blocks;

ALTER TABLE public.level_constraints
    DROP CONSTRAINT IF EXISTS level_constraints_fk1;

ALTER TABLE public.level_constraints
    ADD CONSTRAINT level_constraints_fk1 FOREIGN KEY (building_id) REFERENCES public.block_type(id);

ALTER TABLE public.level_constraints
    RENAME COLUMN building_id TO block_id;
