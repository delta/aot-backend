-- Your SQL goes here

CREATE TABLE public.user (
	id serial NOT NULL,
	name VARCHAR(255) NOT NULL,
	email VARCHAR(255) NOT NULL UNIQUE,
	phone VARCHAR(255) NOT NULL UNIQUE,
	username VARCHAR(255) NOT NULL UNIQUE,
	overall_rating INTEGER NOT NULL,
	is_pragyan BOOLEAN NOT NULL,
	password VARCHAR(255) NOT NULL,
	is_verified BOOLEAN NOT NULL,
	CONSTRAINT user_pk PRIMARY KEY (id)
) WITH (
  OIDS=FALSE
);

CREATE TABLE public.levels_fixture (
	id INTEGER NOT NULL,
	start_date DATE NOT NULL,
	end_date DATE NOT NULL,
	CONSTRAINT levels_fixture_pk PRIMARY KEY (id)
) WITH (
  OIDS=FALSE
);

CREATE TABLE public.map_layout (
	id serial NOT NULL UNIQUE,
    player serial NOT NULL UNIQUE,
	level_id serial NOT NULL UNIQUE,
	CONSTRAINT map_layout_pk PRIMARY KEY (id),
    CONSTRAINT map_layout_fk0 FOREIGN KEY (player) REFERENCES public.user(id),
    CONSTRAINT map_layout_fk1 FOREIGN KEY (level_id) REFERENCES public.levels_fixture(id)
) WITH (
  OIDS=FALSE
);

CREATE TABLE public.game (
	id serial NOT NULL,
	attack_id serial NOT NULL,
	defend_id serial NOT NULL,
	map_layout_id serial NOT NULL,
	attack_score INTEGER NOT NULL,
	defend_score INTEGER NOT NULL,
	CONSTRAINT game_pk PRIMARY KEY (id),
    CONSTRAINT game_fk0 FOREIGN KEY (attack_id) REFERENCES public.user(id),
    CONSTRAINT game_fk1 FOREIGN KEY (defend_id) REFERENCES public.user(id),
    CONSTRAINT game_fk2 FOREIGN KEY (map_layout_id) REFERENCES public.map_layout(id)
) WITH (
  OIDS=FALSE
);

CREATE TABLE public.block_type (
	id serial NOT NULL,
	name VARCHAR(255) NOT NULL,
	width serial NOT NULL,
	height serial NOT NULL,
	weight INTEGER NOT NULL,
	CONSTRAINT block_type_pk PRIMARY KEY (id)
) WITH (
  OIDS=FALSE
);

CREATE TABLE public.map_spaces (
	id serial NOT NULL,
	map_id serial NOT NULL,
	blk_type serial NOT NULL,
	x_coordinate INTEGER NOT NULL,
	y_coordinate INTEGER NOT NULL,
	CONSTRAINT map_spaces_pk PRIMARY KEY (id),
    CONSTRAINT map_spaces_fk0 FOREIGN KEY (map_id) REFERENCES public.map_layout(id),
    CONSTRAINT map_spaces_fk1 FOREIGN KEY (blk_type) REFERENCES public.block_type(id)
) WITH (
  OIDS=FALSE
);

CREATE TABLE public.attack_type (
	id serial NOT NULL,
	att_type serial NOT NULL,
	attack_radius INTEGER NOT NULL,
	attack_damage INTEGER NOT NULL,
	CONSTRAINT attack_type_pk PRIMARY KEY (id)
) WITH (
  OIDS=FALSE
);

CREATE TABLE public.attacker_path (
	id INTEGER NOT NULL,
	y_coord serial NOT NULL,
	x_coord serial NOT NULL,
	is_emp BOOLEAN NOT NULL,
	game_id INTEGER NOT NULL,
	emp_type INTEGER NOT NULL,
	emp_time INTEGER NOT NULL,
	CONSTRAINT attacker_path_pk PRIMARY KEY (id, game_id),
    CONSTRAINT attacker_path_fk0 FOREIGN KEY (game_id) REFERENCES public.game(id),
    CONSTRAINT attacker_path_fk1 FOREIGN KEY (emp_type) REFERENCES public.attack_type(id)
) WITH (
  OIDS=FALSE
);



CREATE TABLE public.shortest_path (
	base_id INTEGER NOT NULL,
	source_x INTEGER NOT NULL,
	source_y INTEGER NOT NULL,
	dest_x INTEGER NOT NULL,
	dest_y INTEGER NOT NULL,
	pathlist VARCHAR(1000) NOT NULL,
	CONSTRAINT ShortestPath_pk PRIMARY KEY (base_id,source_x,source_y,dest_x,dest_y),
    CONSTRAINT ShortestPath_fk0 FOREIGN KEY (base_id) REFERENCES public.map_layout(id)
) WITH (
  OIDS=FALSE
);
