-- Your SQL goes here
CREATE TABLE public.emp_type(
    id INTEGER NOT NULL ,
    att_type VARCHAR(255) NOT NULL,
    attack_radius INTEGER NOT NULL,
    attack_damage INTEGER NOT NULL,
    CONSTRAINT emp_type_id_primary PRIMARY KEY(id)
) WITH (
  OIDS=FALSE
);

CREATE TABLE public.attacker_type(
    id INTEGER  NOT NULL ,
    max_health INTEGER NOT NULL,
    speed INTEGER NOT NULL,
    amt_of_emps INTEGER NOT NULL,
    CONSTRAINT attacker_type_id_primary PRIMARY KEY(id)
)WITH (
  OIDS=FALSE
);




CREATE TABLE public.defender_type(
    id INTEGER  NOT NULL ,
    speed INTEGER NOT NULL,
    damage INTEGER NOT NULL,
    radius INTEGER NOT NULL,
    CONSTRAINT defender_type_id_primary PRIMARY KEY(id)
)WITH (
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

CREATE TABLE public.mine_type(
    id INTEGER NOT NULL ,
    radius INTEGER NOT NULL,
    damage INTEGER NOT NULL,
    CONSTRAINT mine_type_id_primary PRIMARY KEY(id)
)WITH (
  OIDS=FALSE
);




CREATE TYPE building_category AS ENUM ('building', 'road', 'defender','diffuser','mine');


CREATE TABLE public.building_type(
    id INTEGER NOT NULL,
    defender_type INTEGER,
    diffuser_type INTEGER,
    mine_type INTEGER,
    building_category building_category NOT NULL,
    CONSTRAINT building_type_id_primary PRIMARY KEY(id),
    CONSTRAINT defender_type_fk FOREIGN KEY (defender_type) REFERENCES public.defender_type(id),
    CONSTRAINT diffuser_type_fk FOREIGN KEY (diffuser_type) REFERENCES public.diffuser_type(id),
    CONSTRAINT mine_type_fk FOREIGN KEY (mine_type) REFERENCES public.mine_type(id)
)WITH(
    OIDS=FALSE
);



ALTER TABLE public.map_spaces ADD COLUMN building_type INTEGER NOT NULL;
ALTER TABLE public.map_spaces ADD CONSTRAINT building_type_fk FOREIGN KEY (building_type) REFERENCES public.building_type(id);



ALTER TABLE public.levels_fixture ADD no_of_defenders INTEGER NOT NULL;
ALTER TABLE public.levels_fixture ADD no_of_attackers INTEGER NOT NULL;
ALTER TABLE public.levels_fixture ADD no_of_mines INTEGER NOT NULL;
ALTER TABLE public.levels_fixture ADD no_of_diffusers INTEGER NOT NULL;
