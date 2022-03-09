-- Your SQL goes here

ALTER TABLE "block_type"
ADD COLUMN capacity INTEGER NOT NULL DEFAULT 100;


ALTER TABLE "levels_fixture"
ADD COLUMN no_of_robots INTEGER NOT NULL DEFAULT 1000;
