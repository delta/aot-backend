-- Your SQL goes here

--add hp to building_type
ALTER TABLE public.building_type
ADD COLUMN hp INTEGER NOT NULL;

--add cost name and level to emp
ALTER TABLE public.emp_type
ADD cost INTEGER NOT NULL,
ADD "name" VARCHAR(255) NOT NULL,
ADD "level" INTEGER NOT NULL;

--add name to attacker
ALTER TABLE public.attacker_type
ADD "name" VARCHAR(255) NOT NULL;

--add name to defender and mine
ALTER TABLE public.defender_type
ADD "name" VARCHAR(255) NOT NULL;
ALTER TABLE public.mine_type
ADD "name" VARCHAR(255) NOT NULL;

--add attacker and emp to available_blocks
CREATE TYPE item_category AS ENUM ('attacker', 'emp', 'block');


ALTER TABLE public.available_blocks

ADD attacker_type_id INTEGER,
ADD CONSTRAINT attacker_id_fk FOREIGN KEY (attacker_type_id) REFERENCES public.attacker_type(id),

Add emp_type_id INTEGER,
ADD CONSTRAINT emp_id_fk FOREIGN KEY (emp_type_id) REFERENCES public.emp_type(id),

ADD category item_category NOT NULL,

ADD id serial NOT NULL,

DROP CONSTRAINT available_blocks_id_primary,
ADD CONSTRAINT available_blocks_id_primary PRIMARY KEY(id),

ALTER COLUMN block_type_id DROP NOT NULL;
