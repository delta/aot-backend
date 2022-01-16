-- Your SQL goes here

ALTER TABLE public.map_layout
DROP CONSTRAINT map_layout_level_id_key,
DROP CONSTRAINT map_layout_player_key,
ADD CONSTRAINT map_layout_level_id_player_key UNIQUE (level_id, player);
