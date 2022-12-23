-- This file should undo anything in `up.sql`

ALTER TABLE map_layout
DROP COLUMN IF EXISTS is_valid;
