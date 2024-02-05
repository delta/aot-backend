-- Your SQL goes here


CREATE TYPE item_category AS ENUM ('attacker', 'block');


ALTER TABLE public.available_blocks

ADD attacker_type_id INTEGER,
ADD CONSTRAINT attacker_id_fk FOREIGN KEY (attacker_type_id) REFERENCES public.attacker_type(id),

ADD category item_category NOT NULL,

ADD id INTEGER NOT NULL,

DROP CONSTRAINT available_blocks_id_primary,
ADD CONSTRAINT available_blocks_id_primary PRIMARY KEY(id),

ALTER COLUMN block_type_id DROP NOT NULL;
