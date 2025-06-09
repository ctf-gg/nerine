ALTER TABLE challenges ADD COLUMN author TEXT;

UPDATE challenges SET author = 'unknown';

ALTER TABLE challenges
ALTER COLUMN author
SET NOT NULL;