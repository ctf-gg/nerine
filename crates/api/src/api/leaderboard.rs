use crate::{extractors::Auth, Error, Result, State, DB};
use axum::{extract::State as StateE, routing::get, Json, Router};
use chrono::{NaiveDateTime, Utc};
use serde::Serialize;

#[derive(Serialize, Clone)]
struct ScorePoint {
    date: NaiveDateTime,
    score: i32,
}

// TODO: figure out whether we want pagnation
#[derive(Serialize)]
struct LeaderboardEntry {
    #[serde(rename = "id")]
    public_id: String,
    name: String,
    score: i32,
    #[serde(rename(serialize = "extra"))]
    extra_data: serde_json::Value,
    #[serde(rename = "scoreHistory")]
    score_history: Vec<ScorePoint>,
}

struct DbLeaderboardEntry {
    public_id: String,
    name: String,
    score: i32,
    extra_data: serde_json::Value,
    rank: i32,
}

#[derive(sqlx::FromRow, Debug)]
struct TeamSolvePoint {
    created_at: NaiveDateTime,
    points: i32,
}

async fn calculate_score_history(
    db: &DB,
    team_public_id: &str,
    event_start_time: NaiveDateTime,
    all_event_timestamps: &[NaiveDateTime],
) -> Result<Vec<ScorePoint>> {
    let team_id_record = sqlx::query!("SELECT id FROM teams WHERE public_id = $1", team_public_id)
        .fetch_one(db)
        .await?;
    let team_id = team_id_record.id;

    let team_solve_points: Vec<TeamSolvePoint> = sqlx::query_as!(
        TeamSolvePoint,
        r#"
        SELECT s.created_at, ch.c_points as "points!"
        FROM submissions s
        JOIN challenges ch ON s.challenge_id = ch.id
        WHERE s.team_id = $1 AND s.is_correct = TRUE
        "#,
        team_id
    )
    .fetch_all(db)
    .await?;

    let mut history = Vec::new();

    if all_event_timestamps.is_empty() {
        history.push(ScorePoint {
            date: event_start_time,
            score: 0,
        });
        return Ok(history);
    }

    for &ts in all_event_timestamps {
        let score_at_ts: i32 = team_solve_points
            .iter()
            .filter(|solve| solve.created_at <= ts)
            .map(|solve| solve.points)
            .sum();

        if history.is_empty() || history.last().map_or(true, |p| p.score != score_at_ts) {
            history.push(ScorePoint {
                date: ts,
                score: score_at_ts,
            });
        }
    }

    if history.is_empty() {
        history.push(ScorePoint {
            date: event_start_time,
            score: 0,
        });
    }

    Ok(history)
}

async fn leaderboard(db: &DB, event_start_time: NaiveDateTime) -> Result<Vec<LeaderboardEntry>> {
    let db_entries = sqlx::query_as!(
        DbLeaderboardEntry,
        r#"
        SELECT t.public_id, t.name, lb.score as "score!", t.extra_data, lb.rank as "rank!"
            FROM teams t 
            JOIN compute_leaderboard() lb ON lb.team_id = t.id
            ORDER BY lb.rank ASC
        "#
    )
    .fetch_all(db)
    .await?;

    let all_event_timestamps_opt: Vec<Option<NaiveDateTime>> = sqlx::query_scalar!(
        r#"
        SELECT times.created_at FROM (
            SELECT DISTINCT s.created_at FROM submissions s WHERE s.is_correct = TRUE
            UNION
            SELECT $1 AS created_at
        ) AS times
        ORDER BY times.created_at ASC
        "#,
        event_start_time
    )
    .fetch_all(db)
    .await?;

    let all_event_timestamps: Vec<NaiveDateTime> = all_event_timestamps_opt
        .into_iter()
        .filter_map(|x| x)
        .collect();

    let mut leaderboard_entries = Vec::new();
    let default_history_point = ScorePoint {
        date: event_start_time,
        score: 0,
    };

    for db_entry in db_entries {
        let score_history = if db_entry.rank <= 10 {
            match calculate_score_history(
                db,
                &db_entry.public_id,
                event_start_time,
                &all_event_timestamps,
            )
            .await
            {
                Ok(history) => history,
                Err(_e) => {
                    vec![default_history_point.clone()]
                }
            }
        } else {
            Vec::new()
        };

        leaderboard_entries.push(LeaderboardEntry {
            public_id: db_entry.public_id,
            name: db_entry.name,
            score: db_entry.score,
            extra_data: db_entry.extra_data,
            score_history,
        });
    }

    Ok(leaderboard_entries)
}

async fn get_lb(
    StateE(state): StateE<State>,
    Auth(_): Auth,
) -> Result<Json<Vec<LeaderboardEntry>>> {
    if Utc::now().naive_utc() < state.event.start_time {
        return Err(Error::EventNotStarted(state.event.start_time.clone()));
    }

    return leaderboard(&state.db, state.event.start_time)
        .await
        .map(Json);
}

pub fn router() -> Router<crate::State> {
    Router::new().route("/", get(get_lb))
}
