-- This file should undo anything in `up.sql`

--remove hp from building_type
ALTER TABLE public.building_type
DROP COLUMN hp;

--remove cost name and level from emp
ALTER TABLE public.emp_type
DROP cost,
DROP "name",
DROP "level";

--remove name from attacker
ALTER TABLE public.attacker_type
DROP "name";

--remove name from defender and mine
ALTER TABLE public.defender_type
DROP "name";
ALTER TABLE public.mine_type
DROP "name";

--remove attacker and emp from available_blocks
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
