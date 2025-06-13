-- Add up migration script here
CREATE TYPE deployment_strategy AS ENUM ('static', 'instanced');
ALTER TABLE challenges ADD COLUMN strategy deployment_strategy NOT NULL DEFAULT 'static';