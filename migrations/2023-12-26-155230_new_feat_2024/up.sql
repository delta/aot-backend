-- Your SQL goes here

ALTER TABLE public.user
ADD  oauth_token VARCHAR,
DROP  phone,
DROP  username,
DROP  overall_rating,
DROP  is_pragyan,
DROP  password,
DROP  is_verified,
DROP  highest_rating,
DROP  avatar,
DROP  otps_sent,
ADD  attacks_won INTEGER,
ADD  defenses_won INTEGER,
ADD  trophies INTEGER,
ADD  avatar_id INTEGER,
ADD  artifacts INTEGER;


ALTER TABLE public.game
DROP  robots_destroyed,
ADD  artifacts_collected INTEGER;

ALTER TABLE public.map_spaces
DROP  rotation,
DROP  building_type,
ADD CONSTRAINT block_type_id_fk FOREIGN KEY (block_type_id) REFERENCES public.block_type(id);


CREATE TABLE public.artifact(
    id INTEGER NOT NULL ,
    map_space_id INTEGER NOT NULL,
    count INTEGER NOT NULL,
    CONSTRAINT artifact_id_primary PRIMARY KEY(id),
    CONSTRAINT map_space_id_fk FOREIGN KEY (map_space_id) REFERENCES public.map_spaces(id)
) WITH (
  OIDS=FALSE
);

CREATE TABLE public.available_blocks(
    id INTEGER NOT NULL ,
    block_type_id INTEGER NOT NULL,
    user_id INTEGER NOT NULL,
    CONSTRAINT available_blocks_id_primary PRIMARY KEY(id),
    CONSTRAINT user_id_fk FOREIGN KEY (user_id) REFERENCES public.user(id),
    CONSTRAINT block_type_id_fk FOREIGN KEY (block_type_id) REFERENCES public.block_type(id)
) WITH (
  OIDS=FALSE
);

ALTER TABLE public.attacker_type ADD level INTEGER;
ALTER TABLE public.attacker_type ADD cost INTEGER;

ALTER TABLE public.defender_type ADD level INTEGER;
ALTER TABLE public.defender_type ADD cost INTEGER;

ALTER TABLE public.mine_type ADD  level INTEGER;
ALTER TABLE public.mine_type ADD  cost INTEGER;

ALTER TABLE public.building_type
DROP  defender_type,
DROP  building_category,
DROP  mine_type,
DROP  diffuser_type,
ADD  name VARCHAR,
ADD  width INTEGER,
ADD  height INTEGER,
ADD  capacity INTEGER,
ADD  level INTEGER,
ADD  cost INTEGER;



CREATE TYPE block_category AS ENUM ('attacker', 'defender', 'mine', 'building', 'road');
ALTER TABLE block_type
ADD  category block_category,
ADD  category_id INTEGER,
DROP  name,
DROP  width,
DROP  height,
DROP  capacity,
DROP  entrance_x,
DROP  entrance_y,
ADD CONSTRAINT attacker_type_category_fk FOREIGN KEY (category_id) REFERENCES public.attacker_type(id),
ADD CONSTRAINT defender_type_category_fk FOREIGN KEY (category_id) REFERENCES public.defender_type(id),
ADD CONSTRAINT building_type_category_fk FOREIGN KEY (category_id) REFERENCES public.building_type(id),
ADD CONSTRAINT mine_type_category_fk FOREIGN KEY (category_id) REFERENCES public.mine_type(id);

DROP TABLE diffuser_type;
DROP TABLE building_weights;
