use axum::Json;
use serde::{Deserialize, Serialize};

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

struct UpsertChallenge {
    id: Option<String>,
    name: String,
    description: String,
    points_min: i32,
    points_max: i32,
    flag: String,
    attachments: String,
    visible: bool,

    category_id: i32
}

pub async fn create_challenge() -> sctf::Result<Json<Challenge>> {

}