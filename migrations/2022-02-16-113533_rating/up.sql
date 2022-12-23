-- Your SQL goes here

ALTER TABLE "user"
ALTER COLUMN overall_rating TYPE REAL,
ALTER COLUMN highest_rating TYPE REAL;

DROP TABLE IF EXISTS attacker_path;
