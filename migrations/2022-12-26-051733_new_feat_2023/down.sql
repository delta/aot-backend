-- This file should undo anything in `up.sql`
ALTER TABLE public.levels_fixture DROP COLUMN no_of_defenders;
ALTER TABLE public.levels_fixture DROP COLUMN no_of_attackers;
ALTER TABLE public.levels_fixture DROP COLUMN no_of_mines;
ALTER TABLE public.levels_fixture DROP COLUMN no_of_diffusers;
ALTER TABLE public.map_spaces DROP COLUMN building_type;
DROP TABLE public.building_type;
DROP TYPE IF EXISTS building_category;
DROP TABLE public.defender_type;
DROP TABLE public.diffuser_type;
DROP TABLE public.mine_type;
DROP TABLE public.emp_type;
DROP TABLE public.attacker_type;
