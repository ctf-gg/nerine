use axum::{http::StatusCode, Extension, Json};
use serde::Deserialize;

use crate::DB;

#[derive(Deserialize)]
struct RegisterTeam {
    name: String,
    email: String,
}

async fn register(Extension(db): Extension<DB>, Json(payload): Json<RegisterTeam>) -> Result<StatusCode> {
    let id = nanoid!()
    sqlx::query!(
        "INSERT INTO teams VALUES ($1, $2, $3)",
        nanoid!(),
        payload.name,
        payload.email
    )
    .execute(&db)
    .await?;

    StatusCode::CREATED
}
