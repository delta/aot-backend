-- This file should undo anything in `up.sql`

ALTER TABLE "user"
DROP COLUMN IF EXISTS otp_session_id;
