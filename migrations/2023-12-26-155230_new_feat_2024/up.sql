-- Your SQL goes here

ALTER TABLE public.user
ADD COLUMN oauth_token VARCHAR,
DROP COLUMN phone,
DROP COLUMN username,
DROP COLUMN overall_rating,
DROP COLUMN is_pragyan,
DROP COLUMN password,
DROP COLUMN is_verified,
DROP COLUMN highest_rating,
DROP COLUMN avatar,
DROP COLUMN otps_sent;
ADD COLUMN attacks_won INTEGER,
ADD COLUMN defenses_won INTEGER,
ADD COLUMN trophies INTEGER,
ADD COLUMN avatar_id INTEGER,
ADD COLUMN artifacts INTEGER;


ALTER TABLE public.game
DROP COLUMN robots_destroyed,
ADD COLUMN artifacts_collected INTEGER;

ALTER TABLE public.map_spaces
DROP COLUMN blk_type,
DROP COLUMN rotation,
DROP COLUMN building_type,
ADD COLUMN block_type_id INTEGER NOT NULL,
CONSTRAINT block_type_id_fk FOREIGN KEY (block_type_id) REFERENCES public.block_type(id);


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


DROP TABLE diffuser_type;k
DROP TABLE building_weights;

ALTER TABLE public.attacker_type ADD COLUMN level INTEGER NOT NULL;
ALTER TABLE public.attacker_type ADD COLUMN cost INTEGER NOT NULL;

ALTER TABLE public.defender_type ADD COLUMN level INTEGER NOT NULL;
ALTER TABLE public.defender_type ADD COLUMN cost INTEGER NOT NULL;

ALTER TABLE public.mine_type ADD COLUMN level INTEGER NOT NULL;
ALTER TABLE public.mine_type ADD COLUMN cost INTEGER NOT NULL;

ALTER TABLE public.building_type
DROP COLUMN defender_type,
DROP COLUMN building_category,
DROP COLUMN mine_type,
DROP COLUMN diffuser_type,
ADD COLUMN name VARCHAR,
ADD COLUMN width INTEGER,
ADD COLUMN height INTEGER,
ADD COLUMN capacity INTEGER,
ADD COLUMN level INTEGER,
ADD COLUMN cost INTEGER;



CREATE TYPE block_category AS ENUM ('attacker', 'defender', 'mine', 'building', 'road');
ALTER TABLE block_type
ADD COLUMN category block_category,
ADD COLUMN category_id INTEGER;
DROP COLUMN name,
DROP COLUMN width,
DROP COLUMN height,
DROP COLUMN capacity,
DROP COLUMN entrance_x,
DROP COLUMN entrance_y,
CONSTRAINT category_id_fk FOREIGN KEY (category_id) REFERENCES public.attacker_type(id),
CONSTRAINT category_id_fk FOREIGN KEY (category_id) REFERENCES public.defender_type(id),
CONSTRAINT category_id_fk FOREIGN KEY (category_id) REFERENCES public.building_type(id),
CONSTRAINT category_id_fk FOREIGN KEY (category_id) REFERENCES public.mine_type(id);
