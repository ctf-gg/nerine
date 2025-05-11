use axum::{
    http::StatusCode,
    routing::{get, post},
    Extension, Json, Router,
};
use axum_extra::extract::{cookie::Cookie, CookieJar};
use chrono::{Duration, NaiveDateTime};
use nanoid::nanoid;
use serde::{Deserialize, Serialize};

use crate::DB;

use sctf::{
    extractors::Auth,
    jwt::{decode_jwt, generate_jwt, Claims},
};

#[derive(Deserialize)]
struct TeamInfo {
    name: String,
    email: String,
}

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

async fn register(
    Extension(db): Extension<DB>,
    jar: CookieJar,
    Json(payload): Json<TeamInfo>,
) -> sctf::Result<(StatusCode, CookieJar, Json<Team>)> {
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
    let jwt = generate_jwt(&team.public_id, Duration::days(30))?;

    Ok((
        StatusCode::CREATED,
        jar.add(Cookie::new("token", jwt)),
        Json(team),
    ))
}

async fn gen_token(
    Auth(Claims { team_id, .. }): Auth,
) -> sctf::Result<Json<String>> {
    let jwt = generate_jwt(&team_id, Duration::days(30))?;

    return Ok(Json(jwt));
}

#[derive(Deserialize)]
struct LoginTeam {
    token: String,
}

// TODO(aiden): i'm trying to avoid the extra query to get the team so im just returning a status code here
// but maybe worth reconsidering it.
async fn login(
    jar: CookieJar,
    Json(LoginTeam { token: jwt }): Json<LoginTeam>,
) -> sctf::Result<(StatusCode, CookieJar)> {
    decode_jwt(&jwt)?;

    Ok((StatusCode::OK, jar.add(Cookie::new("token", jwt))))
}


pub fn router() -> Router {
    Router::new()
        .route("/register", post(register))
        .route("/login", post(login))
        .route("/gen_token", get(gen_token))
}
