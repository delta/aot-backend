-- This file should undo anything in `up.sql`



ALTER TABLE public.available_blocks

DROP CONSTRAINT attacker_id_fk,
DROP attacker_type_id,
DROP category,
DROP CONSTRAINT available_blocks_id_primary,
DROP id,
ALTER COLUMN block_type_id SET NOT NULL,
ADD CONSTRAINT available_blocks_id_primary PRIMARY KEY(user_id, block_type_id);

DROP TYPE item_category;


-- ADD attacker_type_id INTEGER NOT NULL,
-- ADD CONSTRAINT attacker_id_fk FOREIGN KEY (attacker_type_id) REFERENCES public.attacker_type(id),

-- ADD category item_category NOT NULL,

-- ADD id INTEGER NOT NULL,

-- DROP CONSTRAINT available_blocks_id_primary,
-- ADD CONSTRAINT available_blocks_id_primary PRIMARY KEY(id)

-- ALTER block_type_id DROP NOT NULL;
