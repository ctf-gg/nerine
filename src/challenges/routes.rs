use axum::{Extension, Json};
use chrono::Utc;
use sctf::EVENT;
use serde::{Deserialize, Serialize};

use crate::{account::Auth, DB};

impl Challenge {
    fn into_public(self) -> PublicChallenge {
        PublicChallenge {
            public_id: self.public_id,
            name: self.name,
            description: self.description,
            points_min: self.points_min,
            points_max: self.points_max,
            attachments: self.attachments,
            category: self.category.name,
        }
    }
}

#[derive(Deserialize, Serialize)]
pub struct PublicChallenge {
    #[serde(rename(serialize = "id"))]
    public_id: String,
    name: String,
    description: String,
    points_min: i32,
    points_max: i32,
    attachments: serde_json::Value,
    category: String,
}

// NOTE: All of the routes in this file are PUBLICALLY
// ACCESSABLE!! Do not leak any important information.
pub async fn all(
    Extension(db): Extension<DB>,
    Auth(_): Auth,
) -> sctf::Result<Json<Vec<PublicChallenge>>> {
    if Utc::now().naive_utc() < EVENT.start_time {
        return Err(sctf::Error::EventNotStarted);
    }

    let challs = sqlx::query_as!(
        PublicChallenge,
        r#"SELECT 
            public_id,
            challenges.name,
            description,
            points_min,
            points_max,
            attachments, 
            categories.name AS category 
        FROM challenges JOIN categories ON categories.id = category_id"#
    )
    .fetch_all(&db)
    .await?;

    Ok(Json(challs))
}

#[derive(Deserialize)]
pub struct Submission {
    flag: String,
    challenge_id: String,
}

pub async fn submit(
    Extension(db): Extension<DB>,
    Auth(claims): Auth,
    Json(submission): Json<Submission>,
) -> sctf::Result<()> {
    let now = Utc::now().naive_utc();
    if now < EVENT.start_time {
        return Err(sctf::Error::EventNotStarted);
    }
    if now > EVENT.end_time {
        return Err(sctf::Error::EventEnded);
    }

    let is_correct = sqlx::query!(
        r#"WITH challenge_data AS (SELECT id, flag FROM challenges WHERE public_id = $2)
        INSERT INTO submissions (submission, is_correct, team_id, challenge_id)
        SELECT 
            $1,
            $1 = challenge_data.flag,
            (SELECT id FROM teams WHERE public_id = $3),
            challenge_data.id
        FROM challenge_data RETURNING is_correct"#,
        submission.flag,
        submission.challenge_id,
        claims.team_id,
    )
    .fetch_one(&db)
    .await?
    .is_correct;

    if is_correct {
        Ok(())
    } else {
        Err(sctf::Error::WrongFlag)
    }
}
