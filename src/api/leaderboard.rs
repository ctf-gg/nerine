use axum::{routing::get, Extension, Json, Router};
use chrono::Utc;
use sctf::{extractors::Auth, DB, EVENT};
use serde::Serialize;

// TODO: figure out whether we want pagnation
#[derive(Serialize)]
struct LeaderboardEntry {
    name: String,
    #[serde(rename = "id")]
    public_id: String,
    score: i32,
}

async fn leaderboard(db: DB) -> sctf::Result<Vec<LeaderboardEntry>> {
    let leaderboard_entries = sqlx::query_as!(
        LeaderboardEntry,
        r#"
        SELECT t.name, t.public_id, score as "score!"
            FROM teams t 
            LEFT JOIN compute_leaderboard() lb ON lb.team_id = t.id
            GROUP BY t.id, score, rank
            ORDER BY rank DESC
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
