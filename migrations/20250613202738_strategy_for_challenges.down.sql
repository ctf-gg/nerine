-- Add down migration script here
ALTER TABLE challenges DROP COLUMN author;
DROP TYPE deployment_strategy;