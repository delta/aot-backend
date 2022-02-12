-- Your SQL goes here

ALTER TABLE "user"
ADD COLUMN otp_session_id VARCHAR(255) NOT NULL DEFAULT '',
DROP CONSTRAINT IF EXISTS user_phone_key,
DROP CONSTRAINT IF EXISTS user_email_key;
