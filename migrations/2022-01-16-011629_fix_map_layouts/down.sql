-- This file should undo anything in `up.sql`

ALTER TABLE public.map_layout
DROP CONSTRAINT map_layout_level_id_player_key,
ADD CONSTRAINT map_layout_level_id_key UNIQUE (level_id),
ADD CONSTRAINT map_layout_player_key UNIQUE (player);
