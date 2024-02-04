-- Your SQL goes here
ALTER TABLE public.shortest_path
DROP COLUMN pathlist;
ALTER TABLE public.shortest_path
ADD COLUMN next_hop_x INTEGER NOT NULL;
ALTER TABLE public.shortest_path
ADD COLUMN next_hop_y INTEGER NOT NULL;
