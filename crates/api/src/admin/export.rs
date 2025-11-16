// idk if this warrants an entire file/router but putting lb export in one of the existing ones felt weird
use axum::{extract::State as StateE, routing::get, Json, Router};
use serde::Serialize;

use crate::{db::update_chall_cache, extractors::Admin, Result, State};

#[derive(Serialize)]
struct CTFtimeStanding {
    pos: i32,
    team: String,
    score: i32,
}

#[derive(Serialize)]
struct CTFtimeLeaderboard {
    standings: Vec<CTFtimeStanding>,
}

#[derive(sqlx::FromRow)]
struct ExportLeaderboardEntry {
    name: String,
    score: i32,
    rank: i32,
}

async fn export_ctftime_leaderboard(
    StateE(state): StateE<State>,
    _: Admin,
) -> Result<Json<CTFtimeLeaderboard>> {
    let all_chall_ids: Vec<i32> = sqlx::query_scalar("SELECT id FROM challenges")
        .fetch_all(&state.db)
        .await?;
    for chall_id in all_chall_ids {
        update_chall_cache(&state.db, chall_id).await?;
    }

    let db_entries = sqlx::query_as!(
        ExportLeaderboardEntry,
        r#"
        SELECT t.name, lb.score as "score!", lb.rank as "rank!"
            FROM teams t
            JOIN compute_leaderboard(NULL) lb ON lb.team_id = t.id
            WHERE lb.score > 0
            ORDER BY lb.rank ASC
        "#
    )
    .fetch_all(&state.db)
    .await?;

    let standings: Vec<CTFtimeStanding> = db_entries
        .into_iter()
        .map(|entry| CTFtimeStanding {
            pos: entry.rank,
            team: entry.name,
            score: entry.score,
        })
        .collect();

    Ok(Json(CTFtimeLeaderboard { standings }))
}

pub fn router() -> Router<crate::State> {
    Router::new().route("/ctftime", get(export_ctftime_leaderboard))
}
