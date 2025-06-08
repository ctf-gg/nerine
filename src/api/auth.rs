use axum::{
    extract::State as StateE,
    http::StatusCode,
    routing::{get, post},
    Json, Router,
};
use axum_extra::extract::{cookie::Cookie, CookieJar};
use chrono::{Duration, NaiveDateTime};
use nanoid::nanoid;
use serde::{Deserialize, Serialize};
use validator::Validate;

use crate::{
    email::PendingTeamVerification,
    extractors::Auth,
    jwt::{decode_jwt, generate_jwt, Claims},
    Result, State,
};

#[derive(Deserialize, Validate)]
pub struct TeamInfo {
    #[validate(length(min = 1, max = 72))]
    pub name: String,
    #[validate(email)]
    pub email: String,
}
// TODO move this elsewhere and use teamid return only
#[derive(Deserialize, Serialize)]
pub struct Team {
    #[serde(skip_serializing)]
    pub id: i32,
    #[serde(rename(serialize = "id"))]
    pub public_id: String,
    pub name: String,
    pub email: String,
    pub created_at: NaiveDateTime,
    pub extra_data: serde_json::Value,
}

async fn register(
    StateE(state): StateE<State>,
    Json(payload): Json<TeamInfo>,
) -> Result<(StatusCode, Json<serde_json::Value>)> {
    payload.validate()?;

    state
        .email
        .send_verification_email(
            &payload.email,
            &payload.name,
            PendingTeamVerification {
                name: payload.name.clone(),
                email: payload.email.clone(),
            },
        )
        .await?;

    Ok((
        StatusCode::CREATED,
        Json(serde_json::json!({
            "email": payload.email
        })),
    ))
}

#[derive(Serialize, Deserialize)]
pub struct VerificationRequest {
    pub token: String,
}

async fn verify_email(
    StateE(state): StateE<State>,
    jar: CookieJar,
    Json(VerificationRequest {
        token: verification_token,
    }): Json<VerificationRequest>,
) -> Result<(CookieJar, Json<TeamId>)> {
    let team_details = state
        .email
        .consume_pending_verification(&verification_token)
        .await?;

    let team = sqlx::query_as!(
        Team,
        "INSERT INTO teams (public_id, name, email) VALUES ($1, $2, $3) RETURNING *",
        nanoid!(),
        team_details.name,
        team_details.email
    )
    .fetch_one(&state.db)
    .await?;

    // TODO(aiden): if the duration is long, we'll need a way to revoke all jwts
    let jwt = generate_jwt(&state.config.jwt_keys, &team.public_id, Duration::days(30))?;

    let mut cookie = Cookie::new("token", jwt);
    cookie.set_path("/");
    Ok((jar.add(cookie), Json(TeamId { id: team.public_id })))
}

#[derive(Serialize, Deserialize)]
struct VerificationDetailsRequest {
    token: String,
}

#[derive(Serialize)]
#[serde(untagged)]
pub enum VerificationDetailsResponse {
    TeamRegistration {
        name: String,
        email: String,
        verification_type: String,
    },
    EmailUpdate {
        name: String,
        new_email: String,
        verification_type: String,
    },
}

async fn get_verification_details(
    StateE(state): StateE<State>,
    Json(VerificationDetailsRequest { token }): Json<VerificationDetailsRequest>,
) -> Result<Json<VerificationDetailsResponse>> {
    match state.email.get_pending_verification_details(&token) {
        Some(crate::email::PendingVerification::Team(details)) => {
            Ok(Json(VerificationDetailsResponse::TeamRegistration {
                name: details.name,
                email: details.email,
                verification_type: "team_registration".to_string(),
            }))
        }

        Some(crate::email::PendingVerification::EmailUpdate(details)) => {
            let team_name_record = sqlx::query!(
                "SELECT name FROM teams WHERE public_id = $1",
                details.team_id
            )
            .fetch_one(&state.db)
            .await?;

            Ok(Json(VerificationDetailsResponse::EmailUpdate {
                name: team_name_record.name,
                new_email: details.new_email,
                verification_type: "email_update".to_string(),
            }))
        }
        None => Err(crate::error::Error::InvalidToken),
    }
}

#[derive(Serialize, Deserialize)]
struct Token {
    token: String,
}

async fn gen_token(
    StateE(state): StateE<State>,
    Auth(Claims { team_id, .. }): Auth,
) -> Result<Json<Token>> {
    let jwt = generate_jwt(&state.config.jwt_keys, &team_id, Duration::days(30))?;

    return Ok(Json(Token { token: jwt }));
}

#[derive(Serialize)]
struct TeamId {
    id: String,
}

async fn login(
    StateE(state): StateE<State>,
    jar: CookieJar,
    Json(Token { token: jwt }): Json<Token>,
) -> Result<(CookieJar, Json<TeamId>)> {
    let claims = decode_jwt(&state.config.jwt_keys, &jwt)?;

    let mut cookie = Cookie::new("token", jwt);
    cookie.set_path("/");
    Ok((jar.add(cookie), Json(TeamId { id: claims.team_id })))
}

pub fn router() -> Router<State> {
    Router::new()
        .route("/register", post(register))
        .route("/login", post(login))
        .route("/verify_email", post(verify_email))
        .route("/gen_token", get(gen_token))
        .route("/verification_details", post(get_verification_details))
}
