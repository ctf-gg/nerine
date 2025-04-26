use axum::{http::StatusCode, Extension, Json};
use axum_extra::extract::{cookie::Cookie, CookieJar};
use chrono::{Duration, NaiveDateTime};
use nanoid::nanoid;
use serde::{Deserialize, Serialize};

use crate::DB;

use super::{
    jwt::{decode_jwt, generate_jwt, Claims},
    Auth,
};

#[derive(Deserialize)]
pub struct TeamInfo {
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

pub async fn register(
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
    let jwt = generate_jwt(&team.public_id, Duration::days(7))?;

    Ok((
        StatusCode::CREATED,
        jar.add(Cookie::new("token", jwt)),
        Json(team),
    ))
}

#[derive(Deserialize)]
pub struct LoginTeam {
    token: String,
}

// TODO(aiden): i'm trying to avoid the extra query to get the team so im just returning a status code here
// but maybe worth reconsidering it.
pub async fn login(
    jar: CookieJar,
    Json(LoginTeam { token: jwt }): Json<LoginTeam>,
) -> sctf::Result<(StatusCode, CookieJar)> {
    decode_jwt(&jwt)?;

    Ok((StatusCode::OK, jar.add(Cookie::new("token", jwt))))
}

pub async fn update_profile(
    Extension(db): Extension<DB>,
    Auth(Claims { team_id, .. }): Auth,
    Json(payload): Json<TeamInfo>,
) -> sctf::Result<Json<Team>> {
    let team = sqlx::query_as!(
        Team,
        "UPDATE teams SET name = $1, email = $2 WHERE public_id = $3 RETURNING *",
        payload.name,
        payload.email,
        team_id
    )
    .fetch_one(&db)
    .await?;

    Ok(Json(team))
}
