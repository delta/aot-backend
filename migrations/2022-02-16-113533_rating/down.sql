-- This file should undo anything in `up.sql`

ALTER TABLE "user"
ALTER COLUMN overall_rating TYPE INTEGER,
ALTER COLUMN highest_rating TYPE INTEGER;

CREATE TABLE IF NOT EXISTS public.attacker_path (
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
