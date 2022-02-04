-- This file should undo anything in `up.sql`
ALTER TABLE public.user
ALTER COLUMN overall_rating TYPE INTEGER;
