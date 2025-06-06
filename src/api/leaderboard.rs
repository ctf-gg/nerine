use axum::{routing::get, Extension, Json, Router};
use chrono::Utc;
use crate::{extractors::Auth, DB, EVENT, Result, Error};
use serde::Serialize;

// TODO: figure out whether we want pagnation
#[derive(Serialize)]
struct LeaderboardEntry {
    #[serde(rename = "id")]
    public_id: String,
    name: String,
    score: i32,
}

async fn leaderboard(db: DB) -> Result<Vec<LeaderboardEntry>> {
    let leaderboard_entries = sqlx::query_as!(
        LeaderboardEntry,
        r#"
        SELECT t.name, t.public_id, score as "score!"
            FROM teams t 
            LEFT JOIN compute_leaderboard() lb ON lb.team_id = t.id
            GROUP BY t.id, score, rank
            ORDER BY rank ASC
        "#
    )
    .fetch_all(&db)
    .await?;

    Ok(leaderboard_entries)
}

async fn get_lb(
    Extension(db): Extension<DB>,
    Auth(_): Auth,
) -> Result<Json<Vec<LeaderboardEntry>>> {
    if Utc::now().naive_utc() < EVENT.start_time {
        return Err(Error::EventNotStarted);
    }

    return leaderboard(db).await.map(Json);
}

pub fn router() -> Router<crate::State> {
    Router::new().route("/", get(get_lb))
}
