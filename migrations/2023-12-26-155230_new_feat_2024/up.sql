-- Your SQL goes here

ALTER TABLE public.user
DROP  phone,
DROP  overall_rating,
DROP  password,
DROP  is_verified,
DROP  highest_rating,
DROP  avatar,
DROP  otps_sent,
ADD  attacks_won INTEGER NOT NULL DEFAULT 0,
ADD  defenses_won INTEGER NOT NULL DEFAULT 0,
ADD  trophies INTEGER NOT NULL,
ADD  avatar_id INTEGER NOT NULL DEFAULT 0,
ADD  artifacts INTEGER NOT NULL DEFAULT 0;


ALTER TABLE public.game
DROP  robots_destroyed,
ADD  artifacts_collected INTEGER NOT NULL;


ALTER TABLE public.attacker_type
ADD "level" INTEGER NOT NULL,
ADD cost INTEGER NOT NULL;

ALTER TABLE public.defender_type
ADD "level" INTEGER NOT NULL,
ADD cost INTEGER NOT NULL;

ALTER TABLE public.mine_type
ADD "level" INTEGER NOT NULL,
ADD cost INTEGER NOT NULL;


ALTER TABLE public.block_type RENAME TO building_type_temp;
ALTER TABLE public.building_type_temp
DROP  entrance_x,
DROP  entrance_y,
ADD "level" INTEGER NOT NULL,
ADD cost INTEGER NOT NULL;

ALTER TABLE public.building_type DROP CONSTRAINT diffuser_type_fk;
ALTER TABLE public.building_type RENAME TO block_type;
ALTER TABLE public.building_type_temp RENAME TO building_type;

CREATE TYPE block_category AS ENUM ('defender', 'mine', 'building');

ALTER TABLE public.block_type
DROP diffuser_type,
DROP building_category,
DROP blk_type,
ADD category block_category NOT NULL,
ADD building_type INTEGER NOT NULL,
ADD CONSTRAINT building_type_fk FOREIGN KEY (building_type) REFERENCES public.building_type(id);
DROP TYPE building_category;

ALTER TABLE public.map_spaces
DROP  rotation,
DROP  building_type,
ADD block_type_id INTEGER NOT NULL,
ADD CONSTRAINT map_spaces_fk1 FOREIGN KEY (block_type_id) REFERENCES public.block_type(id);

CREATE TABLE public.available_blocks(
    block_type_id INTEGER NOT NULL,
    user_id INTEGER NOT NULL,
    CONSTRAINT available_blocks_id_primary PRIMARY KEY(user_id, block_type_id),
    CONSTRAINT user_id_fk FOREIGN KEY (user_id) REFERENCES public.user(id),
    CONSTRAINT block_type_id_fk FOREIGN KEY (block_type_id) REFERENCES public.block_type(id)
) WITH (
  OIDS=FALSE
);

CREATE TABLE public.artifact(
    map_space_id INTEGER NOT NULL,
    count INTEGER NOT NULL,
    CONSTRAINT artifact_id_primary PRIMARY KEY(map_space_id),
    CONSTRAINT map_space_id_fk FOREIGN KEY (map_space_id) REFERENCES public.map_spaces(id)
) WITH (
  OIDS=FALSE
);

ALTER TABLE public.level_constraints
DROP CONSTRAINT level_constraints_fk1,
ADD CONSTRAINT level_constraints_fk1 FOREIGN KEY (building_id) REFERENCES public.building_type(id);

ALTER TABLE public.levels_fixture DROP no_of_robots;

DROP TABLE diffuser_type;
DROP TABLE building_weights;
DROP TABLE drone_usage;
