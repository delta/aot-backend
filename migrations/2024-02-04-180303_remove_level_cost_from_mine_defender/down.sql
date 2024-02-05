-- This file should undo anything in `up.sql`


ALTER TABLE public.mine_type
ADD "level" INTEGER NOT NULL,
ADD cost INTEGER NOT NULL;

ALTER TABLE public.defender_type
ADD "level" INTEGER NOT NULL,
ADD cost INTEGER NOT NULL;
