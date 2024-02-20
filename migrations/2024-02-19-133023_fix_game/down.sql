-- This file should undo anything in `up.sql`
ALTER TABLE public.game
DROP COLUMN date;

ALTER TABLE public.game
RENAME COLUMN is_game_over TO is_attacker_alive;
