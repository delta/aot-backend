-- Your SQL goes here

ALTER TABLE map_layout
ADD COLUMN is_valid boolean NOT NULL DEFAULT false;
