-- This file should undo anything in `up.sql`

CREATE TABLE public.drone_usage(
    id serial NOT NULL,
    attacker_id INTEGER NOT NULL,
    map_id INTEGER NOT NULL,
    drone_x INTEGER NOT NULL,
    drone_y INTEGER NOT NULL,
    CONSTRAINT drone_usage_id_primary PRIMARY KEY (id),
    CONSTRAINT drone_usage_fk0 FOREIGN KEY (attacker_id) REFERENCES public.user(id),
    CONSTRAINT drone_usage_fk1 FOREIGN KEY (map_id) REFERENCES public.map_layout(id)
) WITH (
  OIDS=FALSE
);

CREATE TABLE public.building_weights (
    time INTEGER NOT NULL,
    building_id INTEGER NOT NULL,
    weight INTEGER NOT NULL,
    CONSTRAINT building_weights_pk PRIMARY KEY (time, building_id),
    CONSTRAINT building_weights_fk0 FOREIGN KEY (building_id) REFERENCES public.block_type(id)
) WITH (
    OIDS=FALSE
);

CREATE TABLE public.diffuser_type(
    id INTEGER NOT NULL,
    radius INTEGER NOT NULL,
    speed INTEGER NOT NULL,
    CONSTRAINT diffuser_type_id_primary PRIMARY KEY(id)
)WITH (
  OIDS=FALSE
);

ALTER TABLE public.levels_fixture ADD no_of_robots INTEGER NOT NULL DEFAULT 1000;

DROP TABLE IF EXISTS artifact;
DROP TABLE IF EXISTS available_blocks;

ALTER TABLE public.user
DROP artifacts,
DROP avatar_id,
DROP trophies,
DROP defenses_won,
DROP attacks_won,
ADD otps_sent INTEGER NOT NULL DEFAULT 0,
ADD avatar INTEGER NOT NULL DEFAULT 0,
ADD highest_rating INTEGER NOT NULL DEFAULT 0,
ADD is_verified BOOLEAN NOT NULL,
ADD password VARCHAR(255) NOT NULL,
ADD overall_rating INTEGER NOT NULL,
ADD phone VARCHAR(255) NOT NULL UNIQUE;

ALTER TABLE public.game
ADD  robots_destroyed INTEGER NOT NULL DEFAULT 0,
DROP  artifacts_collected;

ALTER TABLE public.mine_type
DROP COLUMN "level",
DROP COLUMN cost;

ALTER TABLE public.defender_type
DROP COLUMN "level",
DROP COLUMN cost;

ALTER TABLE public.attacker_type
DROP COLUMN "level",
DROP COLUMN cost;


CREATE TYPE building_category AS ENUM ('building', 'defender','diffuser','mine');
ALTER TABLE public.block_type
DROP category,
DROP building_type,
ADD diffuser_type INTEGER,
ADD building_category building_category NOT NULL,
ADD blk_type INTEGER NOT NULL,
ADD CONSTRAINT blk_type_fk FOREIGN KEY (blk_type) REFERENCES public.building_type(id);
DROP TYPE IF EXISTS block_category;

ALTER TABLE public.building_type RENAME TO building_type_temp;
ALTER TABLE public.block_type RENAME TO building_type;
ALTER TABLE public.building_type ADD CONSTRAINT diffuser_type_fk FOREIGN KEY (diffuser_type) REFERENCES public.diffuser_type(id);

ALTER TABLE public.building_type_temp
DROP "level",
DROP cost,
ADD entrance_x INTEGER NOT NULL DEFAULT 0,
ADD entrance_y INTEGER NOT NULL DEFAULT 0;
ALTER TABLE public.building_type_temp RENAME TO block_type;

ALTER TABLE public.map_spaces
ADD rotation INTEGER NOT NULL DEFAULT 0,
ADD building_type INTEGER NOT NULL,
DROP block_type_id,
DROP CONSTRAINT IF EXISTS map_spaces_fk1,
ADD CONSTRAINT building_type_fk FOREIGN KEY (building_type) REFERENCES public.building_type(id);

ALTER TABLE level_constraints
DROP CONSTRAINT IF EXISTS level_constraints_fk1,
ADD CONSTRAINT level_constraints_fk1 FOREIGN KEY (building_id) REFERENCES public.building_type(id);
