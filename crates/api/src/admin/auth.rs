use axum::{Json, Router, extract::State as StateE, routing::post};
use chrono::Duration;
use serde::Deserialize;

use crate::{extractors::Admin, jwt::generate_jwt, Result, State};

#[derive(Deserialize)]
struct ResendToken {
    email: String,
}

async fn resend_token(
    StateE(state): StateE<State>,
    _: Admin,
    Json(payload): Json<ResendToken>,
) -> Result<()> {
    let team_partial = sqlx::query!(
        "SELECT public_id, name FROM teams WHERE email = $1",
        payload.email,
    )
    .fetch_one(&state.db)
    .await?;

    let jwt = generate_jwt(&state.config.jwt_keys, &team_partial.public_id, Duration::days(30))?;

    state.email.send_resend_token_email(&payload.email, &team_partial.name, &jwt).await?;

    Ok(())
}

pub fn router() -> Router<crate::State> {
    Router::new()
        .route("/resend_token", post(resend_token))
}
