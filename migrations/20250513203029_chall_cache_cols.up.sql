ALTER TABLE challenges 
ADD COLUMN c_solves INT NOT NULL DEFAULT 0,
ADD COLUMN c_points INT NOT NULL DEFAULT 0;

CREATE INDEX challenge_points ON challenges (id, c_points);