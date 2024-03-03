-- Your SQL goes here
ALTER TABLE public.game
ADD COLUMN date DATE NOT NULL;

ALTER TABLE public.game
RENAME COLUMN is_attacker_alive TO is_game_over;
