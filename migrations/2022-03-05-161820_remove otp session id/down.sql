-- This file should undo anything in `up.sql`
ALTER TABLE "user"
ADD COLUMN otp_session_id VARCHAR(255) NOT NULL DEFAULT '';
