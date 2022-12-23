-- This file should undo anything in `up.sql`

ALTER TABLE map_spaces
ADD CONSTRAINT map_spaces_rotation_check CHECK (rotation >= 0 AND rotation <= 270 AND rotation % 90 = 0);
