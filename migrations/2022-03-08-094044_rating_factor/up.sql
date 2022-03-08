-- Your SQL goes here

ALTER TABLE "levels_fixture"
ADD COLUMN rating_factor REAL NOT NULL DEFAULT 0.4;
