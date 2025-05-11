use axum::{
    extract::Path,
    routing::{get, post},
    Extension, Json, Router,
};
use sctf::{extractors::Auth, jwt::Claims};
use serde::{Deserialize, Serialize};
use sqlx::Row;

use crate::DB;

use super::auth::Team;

#[derive(Deserialize)]
struct UpdateProfile {
    name: String,
    email: String,
}

async fn update(
    Extension(db): Extension<DB>,
    Auth(Claims { team_id, .. }): Auth,
    Json(payload): Json<UpdateProfile>,
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

#[derive(Serialize, sqlx::FromRow)]
struct ChallDetails {
    name: String,
    points_min: i64,
    points_max: i64,
    // TODO figure out why this is possibly null
    // (probably that count can return null somehow?)
    solves: Option<i64>,
}

#[derive(Serialize)]
struct Solve {
    name: String,
    points: i64,
}

fn point_formula(
    ChallDetails {
        points_max,
        points_min,
        solves,
        ..
    }: &ChallDetails,
) -> i64 {
    return points_max - ((points_max - points_min) * solves.unwrap() / 20);
}

async fn get_score_solves(db: &DB, pub_id: &str) -> sctf::Result<(i64, Vec<Solve>)> {
    let chall_details = sqlx::query_as!(
        ChallDetails,
        "WITH
            team_id AS (SELECT id FROM teams WHERE public_id = $1 LIMIT 1),
            solved_challs AS (SELECT challenge_id AS id FROM submissions WHERE is_correct = true AND team_id = team_id),
            solves_per_chall AS (SELECT challenge_id AS id, count(*) AS solves FROM submissions JOIN solved_challs sc ON sc.id = challenge_id GROUP BY challenge_id)
        SELECT name, points_min, points_max, spc.solves FROM challenges c JOIN solves_per_chall spc ON spc.id = c.id",
        pub_id
    ).fetch_all(db).await?;

    let mut total_points = 0;
    let mut solves = vec![];
    for details in chall_details {
        let points = point_formula(&details);
        solves.push(Solve {
            name: details.name,
            points,
        });

        total_points += points;
    }

    return Ok((total_points, solves));
}

// TODO we do want to put placement here
// but thats also gonna be some work (because it seems like we would probably want to cache user points)
#[derive(Serialize)]
struct PublicProfile {
    name: String,
    score: i64,
    solves: Vec<Solve>,
}

async fn profile(
    Extension(db): Extension<DB>,
    Auth(_): Auth,
    Path(pub_id): Path<String>,
) -> sctf::Result<Json<PublicProfile>> {
    let name: String = sqlx::query("SELECT name FROM teams WHERE public_id = $1")
        .bind(&pub_id)
        .fetch_one(&db)
        .await?
        .try_get(0)?;
    let (score, solves) = get_score_solves(&db, &pub_id).await?;

    return Ok(Json(PublicProfile {
        name,
        score,
        solves,
    }));
}

pub fn router() -> Router {
    Router::new()
        .route("/update", post(update))
        .route("/:id", get(profile))
}
