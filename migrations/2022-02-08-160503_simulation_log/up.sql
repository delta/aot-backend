-- Your SQL goes here

CREATE TABLE public.simulation_log (
	game_id serial NOT NULL,
	log_text text NOT NULL,
	CONSTRAINT simulation_log_pk PRIMARY KEY (game_id),
	CONSTRAINT simulation_log_fk0 FOREIGN KEY (game_id) REFERENCES public.game(id)
) WITH (
    OIDS=FALSE
);
