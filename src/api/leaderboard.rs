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
        WITH solves AS (SELECT team_id, challenge_id, created_at FROM submissions WHERE is_correct = true),
        last_solve AS (SELECT team_id, MAX(created_at) AS sub_time FROM solves GROUP BY team_id)
        SELECT t.name, t.public_id, COALESCE(SUM(c_points), 0)::int AS "score!"
            FROM teams t 
            LEFT JOIN solves s ON t.id = s.team_id 
            LEFT JOIN challenges ch ON s.challenge_id = ch.id
            LEFT JOIN last_solve ls ON t.id = ls.team_id
            GROUP BY t.id, ls.sub_time
            ORDER BY 
                "score!" DESC,
                ls.sub_time ASC NULLS LAST,
                t.id ASC
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
