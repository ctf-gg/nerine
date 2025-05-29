DROP INDEX challenge_points;

ALTER TABLE challenges 
DROP COLUMN c_solves, 
DROP COLUMN c_points;