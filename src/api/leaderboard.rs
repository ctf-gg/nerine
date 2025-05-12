use std::collections::HashMap;

use axum::{routing::get, Extension, Json, Router};
use chrono::Utc;
use sctf::{event::point_formula, extractors::Auth, EVENT};
use serde::{Deserialize, Serialize};

use crate::DB;

// TODO: figure out whether we want pagnation
#[derive(Serialize)]
struct LeaderboardEntry {
    name: String,
    public_id: String,
    score: i64,
}

struct Solve {
    team_id: i64,
    public_id: String,
    challenge_id: i64,
}

#[derive(Serialize, sqlx::FromRow)]
struct ChallDetails {
    id: i64,
    points_min: i64,
    points_max: i64,
    solves: i64,
}

async fn leaderboard(db: DB) -> sctf::Result<Vec<LeaderboardEntry>> {
    let chall_details = sqlx::query_as!(
        ChallDetails,
        r#"WITH spc AS (
            SELECT challenge_id AS id, count(*) AS solves
            FROM submissions 
            WHERE is_correct = true GROUP BY challenge_id)
        SELECT c.id, points_min, points_max, spc.solves AS "solves!" FROM challenges c JOIN spc ON spc.id = c.id"#,
    ).fetch_all(&db).await?;

    let chall_values: HashMap<i64, i64> = chall_details
        .into_iter()
        .map(|details| {
            (
                details.id,
                point_formula(details.points_min, details.points_max, details.solves),
            )
        })
        .collect();

    let solves = sqlx::query_as!(
        Solve,
        "SELECT team_id, public_id, challenge_id FROM submissions 
        JOIN teams ON teams.id = team_id WHERE is_correct = true"
    )
    .fetch_all(&db)
    .await?;

    let mut team_scores: HashMap<i64, i64> = HashMap::new();

    // TODO reevaluate this for a faster way
    for solve in solves {
        team_scores.insert(
            solve.team_id,
            *team_scores.get(&solve.team_id).unwrap_or(&0)
                + chall_values.get(&solve.challenge_id).unwrap(),
        );
    }

    let mut teams: HashMap<i64, (String, String)> =
        sqlx::query_as("SELECT id, public_id, name FROM teams")
            .fetch_all(&db)
            .await?
            .into_iter()
            // TODO investigate this & why the cast is needed
            .map(|x: (i32, String, String)| (x.0 as i64, (x.1, x.2)))
            .collect();

    let mut leaderboard_entries: Vec<LeaderboardEntry> = team_scores
        .into_iter()
        .map(|(id, score)| {
            let (public_id, name) = teams.remove(&id).unwrap();
            LeaderboardEntry {
                name,
                public_id,
                score,
            }
        })
        .collect();
    // sort desc
    leaderboard_entries.sort_by_key(|x| -x.score);

    Ok(leaderboard_entries)
}

#[axum::debug_handler]
async fn get_lb(
    Extension(db): Extension<DB>,
    Auth(_): Auth,
) -> sctf::Result<Json<Vec<LeaderboardEntry>>> {
    if Utc::now().naive_utc() < EVENT.start_time {
        return Err(sctf::Error::EventNotStarted);
    }
    return leaderboard(db).await.map(Json);
}

pub fn router() -> Router {
    Router::new()
        .route("/", get(get_lb))
}
