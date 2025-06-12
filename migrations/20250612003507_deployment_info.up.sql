-- Add up migration script here
CREATE TABLE challenge_deployments (
    id INT PRIMARY KEY GENERATED ALWAYS AS IDENTITY,

    team_id INT REFERENCES teams(id),
    challenge_id INT NOT NULL REFERENCES challenges(id),
    deployed BOOLEAN NOT NULL DEFAULT false,
    data JSONB,

    created_at TIMESTAMP NOT NULL DEFAULT NOW(),
    expired_at TIMESTAMP NOT NULL DEFAULT NOW() + make_interval(mins => 10),

    UNIQUE (team_id, challenge_id)
);
