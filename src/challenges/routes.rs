use axum::{Extension, Json};
use chrono::Utc;
use sctf::EVENT;
use serde::{Deserialize, Serialize};

use crate::{account::Auth, DB};

#[derive(Deserialize, Serialize)]
pub struct Challenge {
    id: i32,
    public_id: String,
    name: String,
    description: String,
    points_min: i32,
    points_max: i32,
    flag: String,
    attachments: serde_json::Value,
    visible: bool,

    category: Category,
    group: ChallengeGroup,
}

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

#[derive(Deserialize, Serialize)]
pub struct Category {
    id: i32,
    name: String,
}

#[derive(Deserialize, Serialize)]
pub struct ChallengeGroup {
    id: i32,
    name: String,
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
