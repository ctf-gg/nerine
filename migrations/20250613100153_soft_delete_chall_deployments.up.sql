-- Add up migration script here
ALTER TABLE challenge_deployments ADD COLUMN destroyed_at TIMESTAMP;
