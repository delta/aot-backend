-- Your SQL goes here
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
