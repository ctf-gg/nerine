use axum::{
    extract::{State as StateE}, http::StatusCode, routing::{get, post}, Extension, Json, Router
};
use axum_extra::extract::{cookie::Cookie, CookieJar};
use chrono::{Duration, NaiveDateTime};
use nanoid::nanoid;
use serde::{Deserialize, Serialize};

use crate::{
    State, extractors::Auth, jwt::{decode_jwt, generate_jwt, Claims}, Result, DB
};

#[derive(Deserialize)]
struct TeamInfo {
    name: String,
    email: String,
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
}

// TODO also enforce email constraints here for workarounds like caps & a cleaner error message.
async fn register(
    StateE(cfg): StateE<State>,
    Extension(db): Extension<DB>,
    jar: CookieJar,
    Json(payload): Json<TeamInfo>,
) -> Result<(StatusCode, CookieJar, Json<Team>)> {
    let team = sqlx::query_as!(
        Team,
        "INSERT INTO teams (public_id, name, email) VALUES ($1, $2, $3) RETURNING *",
        nanoid!(),
        payload.name,
        payload.email
    )
    .fetch_one(&db)
    .await?;

    // TODO(aiden): if the duration is long, we'll need a way to revoke all jwts
    let jwt = generate_jwt(&cfg.jwt_keys, &team.public_id, Duration::days(30))?;

    let mut cookie = Cookie::new("token", jwt);
    cookie.set_path("/");

    Ok((StatusCode::CREATED, jar.add(cookie), Json(team)))
}

#[derive(Serialize, Deserialize)]
struct Token {
    token: String,
}

async fn gen_token(
    StateE(cfg): StateE<State>,
    Auth(Claims { team_id, .. }): Auth,
) -> Result<Json<Token>> {
    let jwt = generate_jwt(&cfg.jwt_keys, &team_id, Duration::days(30))?;

    return Ok(Json(Token { token: jwt }));
}

#[derive(Serialize)]
struct TeamId {
    id: String,
}

async fn login(
    StateE(cfg): StateE<State>,
    jar: CookieJar,
    Json(Token { token: jwt }): Json<Token>,
) -> Result<(CookieJar, Json<TeamId>)> {
    let claims = decode_jwt(&cfg.jwt_keys, &jwt)?;

    let mut cookie = Cookie::new("token", jwt);
    cookie.set_path("/");
    Ok((jar.add(cookie), Json(TeamId { id: claims.team_id })))
}

pub fn router() -> Router<State> {
    Router::new()
        .route("/register", post(register))
        .route("/login", post(login))
        .route("/gen_token", get(gen_token))
}
