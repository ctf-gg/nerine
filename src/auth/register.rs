use axum::{http::StatusCode, Extension, Json};
use chrono::NaiveDateTime;
use nanoid::nanoid;
use serde::{Deserialize, Serialize};

use crate::DB;

#[derive(Deserialize)]
pub struct RegisterTeam {
    name: String,
    email: String,
}

#[derive(Deserialize, Serialize)]
pub struct Team {
    #[serde(skip_serializing)]
    id: i32,
    #[serde(rename(serialize = "id"))]
    public_id: String,
    name: String,
    email: String,
    created_at: NaiveDateTime,
}

pub async fn route(
    Extension(db): Extension<DB>,
    Json(payload): Json<RegisterTeam>,
) -> sctf::Result<(StatusCode, Json<Team>)> {
    let team = sqlx::query_as!(
        Team,
        "INSERT INTO teams (public_id, name, email) VALUES ($1, $2, $3) RETURNING *",
        nanoid!(),
        payload.name,
        payload.email
    )
    .fetch_one(&db)
    .await?;

    Ok((StatusCode::CREATED, Json(team)))
}
