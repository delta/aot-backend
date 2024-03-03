-- This file should undo anything in `up.sql`
ALTER TABLE public.shortest_path
DROP COLUMN next_hop_x;
ALTER TABLE public.shortest_path
DROP COLUMN next_hop_y;
ALTER TABLE public.shortest_path
ADD COLUMN pathlist VARCHAR(1000) NOT NULL;
