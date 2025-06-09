CREATE FUNCTION compute_leaderboard() RETURNS TABLE (team_id INT, score INT, rank INT) AS $$ WITH solves AS (
    SELECT team_id,
        challenge_id
    FROM submissions
    WHERE is_correct = true
),
last_solve AS (
    SELECT team_id,
        MAX(created_at) AS sub_time
    FROM submissions
    WHERE is_correct = true
    GROUP BY team_id
)
SELECT t.id,
    COALESCE(SUM(ch.c_points), 0)::int,
    ROW_NUMBER() OVER (
        ORDER BY COALESCE(SUM(ch.c_points), 0) DESC,
            ls.sub_time ASC NULLS LAST,
            t.id ASC
    )::int
FROM teams t
    LEFT JOIN solves ON t.id = solves.team_id
    LEFT JOIN challenges ch ON solves.challenge_id = ch.id
    LEFT JOIN last_solve ls ON t.id = ls.team_id
GROUP BY t.id,
    ls.sub_time $$ LANGUAGE SQL;