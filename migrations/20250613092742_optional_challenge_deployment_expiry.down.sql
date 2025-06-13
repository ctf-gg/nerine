-- Add down migration script here
ALTER TABLE challenge_deployments
    ALTER COLUMN expired_at SET NOT NULL,
    ALTER COLUMN expired_at SET DEFAULT NOW() + make_interval(mins => 10);
