DROP INDEX challenge_points;

ALTER TABLE challenges 
DROP COLUMN c_solves,
DROP COULMN c_points;