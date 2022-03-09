-- This file should undo anything in `up.sql`

ALTER TABLE shortest_path ALTER COLUMN pathlist TYPE VARCHAR(1000);
