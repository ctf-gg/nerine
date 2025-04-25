CREATE TABLE teams (
    id INT PRIMARY KEY GENERATED ALWAYS AS IDENTITY,
    public_id TEXT NOT NULL UNIQUE,

    name TEXT NOT NULL,
    email TEXT NOT NULL UNIQUE,

    created_at TIMESTAMP NOT NULL DEFAULT NOW()
);

CREATE INDEX teams_public_id_index ON teams (public_id);

CREATE TABLE categories (
    id INT PRIMARY KEY GENERATED ALWAYS AS IDENTITY,
    name TEXT NOT NULL UNIQUE
);

CREATE TABLE challenge_groups (
    id INT PRIMARY KEY GENERATED ALWAYS AS IDENTITY,
    name TEXT NOT NULL UNIQUE
);

CREATE TABLE challenges (
    id INT PRIMARY KEY GENERATED ALWAYS AS IDENTITY,
    public_id TEXT NOT NULL UNIQUE,

    name TEXT NOT NULL UNIQUE,
    description TEXT NOT NULL DEFAULT '',
    points_min INT NOT NULL,
    points_max INT NOT NULL,
    flag TEXT NOT NULL,
    attachments JSONB NOT NULL,
    visible BOOLEAN NOT NULL DEFAULT false,

    category_id INT NOT NULL REFERENCES categories (id),
    group_id INT REFERENCES challenge_groups (id),
    
    UNIQUE (name, category_id)
);

CREATE INDEX challenges_public_id_index ON challenges (public_id);

CREATE TABLE submissions (
    id BIGINT PRIMARY KEY GENERATED ALWAYS AS IDENTITY,

    submission TEXT NOT NULL,
    is_correct BOOLEAN NOT NULL,

    team_id INT NOT NULL REFERENCES teams(id),
    challenge_id INT NOT NULL REFERENCES challenges(id),
    
    created_at TIMESTAMP NOT NULL DEFAULT NOW()
);

CREATE INDEX submissions_by_own_team ON submissions (team_id, challenge_id, is_correct);
CREATE INDEX submissions_chall_solves ON submissions (challenge_id) WHERE is_correct;
CREATE INDEX submissions_team_solves ON submissions (team_id) WHERE is_correct;
CREATE UNIQUE INDEX submissions_only_one_solve ON submissions (team_id, challenge_id) WHERE is_correct;
