-- This file should undo anything in `up.sql`

--remove hp to building_type
ALTER TABLE public.building_type
DROP COLUMN hp;

--add level and cost from mine and defender
ALTER TABLE public.mine_type
ADD "level" INTEGER NOT NULL,
ADD cost INTEGER NOT NULL;

ALTER TABLE public.defender_type
ADD "level" INTEGER NOT NULL,
ADD cost INTEGER NOT NULL;

--remove cost name and level to emp
ALTER TABLE public.emp_type
DROP cost,
DROP "name";

--remove attacker and emp to available_blocks
ALTER TABLE public.available_blocks

DROP CONSTRAINT attacker_id_fk,
DROP attacker_type_id,
DROP CONSTRAINT emp_id_fk,
DROP emp_type_id,
DROP category,
DROP CONSTRAINT available_blocks_id_primary,
DROP id,
ALTER COLUMN block_type_id SET NOT NULL,
ADD CONSTRAINT available_blocks_id_primary PRIMARY KEY(user_id, block_type_id);

DROP TYPE item_category;
---------------------
