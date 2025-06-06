use axum::{
    extract::State as StateE, routing::{get, post}, Json, Router
};
use chrono::Utc;
use crate::{db::update_chall_cache, extractors::Auth, EVENT, Result, Error, State};
use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize)]
pub struct PublicChallenge {
    #[serde(rename(serialize = "id"))]
    public_id: String,
    name: String,
    author: String,
    description: String,
    points: i32,
    solves: i32,
    attachments: serde_json::Value,
    category: String,
}

// NOTE: All of the routes in this file are PUBLICALLY
// ACCESSABLE!! Do not leak any important information.
pub async fn list(
    StateE(state): StateE<State>,
    Auth(_): Auth,
) -> Result<Json<Vec<PublicChallenge>>> {
    if Utc::now().naive_utc() < EVENT.start_time {
        return Err(Error::EventNotStarted);
    }

    let challs = sqlx::query_as!(
        PublicChallenge,
        r#"SELECT 
            public_id,
            challenges.name,
            author,
            description,
            c_points AS points,
            c_solves AS solves,
            attachments, 
            categories.name AS category 
        FROM challenges JOIN categories ON categories.id = category_id
        ORDER BY solves DESC"#
    )
    .fetch_all(&state.db)
    .await?;

    Ok(Json(challs))
}

#[derive(Deserialize)]
pub struct Submission {
    flag: String,
    challenge_id: String,
}

pub async fn submit(
    StateE(state): StateE<State>,
    Auth(claims): Auth,
    Json(submission): Json<Submission>,
) -> Result<()> {
    let now = Utc::now().naive_utc();
    if now < EVENT.start_time {
        return Err(Error::EventNotStarted);
    }
    if now > EVENT.end_time {
        return Err(Error::EventEnded);
    }

    struct AnswerInfo {
        id: i32,
        flag: String,
    }

    let answer_info: AnswerInfo = sqlx::query_as!(
        AnswerInfo,
        "SELECT id, flag FROM challenges WHERE public_id = $1",
        submission.challenge_id
    )
    .fetch_one(&state.db)
    .await?;

    let is_correct = answer_info.flag == submission.flag;

    sqlx::query!(
        r#"INSERT INTO submissions (submission, is_correct, team_id, challenge_id)
        VALUES ($1, $2, (SELECT id FROM teams WHERE public_id = $3), $4)"#,
        submission.flag,
        is_correct,
        claims.team_id,
        answer_info.id,
    )
    .execute(&state.db)
    .await?;

    if is_correct {
        update_chall_cache(&state.db, answer_info.id).await?;
        Ok(())
    } else {
        Err(Error::WrongFlag)
    }
}

pub fn router() -> Router<crate::State> {
    Router::new()
        .route("/", get(list))
        .route("/submit", post(submit))
}
