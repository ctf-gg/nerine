use axum::{routing::get, Extension, Json, Router};
use chrono::Utc;
use sctf::{extractors::Auth, DB, EVENT};
use serde::Serialize;

// TODO: figure out whether we want pagnation
#[derive(Serialize)]
struct LeaderboardEntry {
    name: String,
    public_id: String,
    score: i32,
}

async fn leaderboard(db: DB) -> sctf::Result<Vec<LeaderboardEntry>> {
    let leaderboard_entries = sqlx::query_as!(
        LeaderboardEntry,
        r#"
        WITH solves AS (SELECT team_id, challenge_id FROM submissions WHERE is_correct = true)
        SELECT teams.name, teams.public_id, SUM(c_points)::int AS "score!"
            FROM challenges 
            JOIN solves ON challenge_id = challenges.id 
            JOIN teams ON team_id = teams.id GROUP BY teams.id
            ORDER BY "score!" DESC
        "#
    )
    .fetch_all(&db)
    .await?;

    Ok(leaderboard_entries)
}

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
    Router::new().route("/", get(get_lb))
}
