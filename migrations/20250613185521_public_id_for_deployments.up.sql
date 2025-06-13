-- Add up migration script here
ALTER TABLE challenge_deployments ADD COLUMN public_id TEXT NOT NULL UNIQUE;
