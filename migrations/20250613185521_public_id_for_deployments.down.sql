-- Add down migration script here
ALTER TABLE challenge_deployments DROP COLUMN public_id;
