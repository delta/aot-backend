-- Your SQL goes here

ALTER TABLE game
ADD COLUMN robots_destroyed INTEGER NOT NULL,
ADD COLUMN emps_used INTEGER NOT NULL,
ADD COLUMN damage_done INTEGER NOT NULL,
ADD COLUMN is_attacker_alive boolean NOT NULL DEFAULT false;


ALTER TABLE "user"
ADD COLUMN highest_rating INTEGER NOT NULL;
