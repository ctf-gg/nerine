-- Add up migration script here
ALTER TABLE challenge_deployments
    ALTER COLUMN expired_at DROP NOT NULL,
    ALTER COLUMN expired_at DROP DEFAULT;
