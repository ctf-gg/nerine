use axum::{
    extract::Path,
    routing::{get, post},
    Extension, Json, Router,
};
use sctf::{extractors::Auth, DB, jwt::Claims};
use serde::{Deserialize, Serialize};


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

#[derive(Serialize)]
struct Solve {
    name: String,
    points: i32,
}

async fn get_score_solves(db: &DB, pub_id: &str) -> sctf::Result<(i32, Vec<Solve>)> {
    let solves = sqlx::query_as!(
        Solve,
        r#"WITH solved_challs AS (SELECT challenge_id AS id FROM submissions WHERE is_correct = true AND team_id = (SELECT id FROM teams WHERE public_id = $1))
        SELECT name, c_points AS points FROM challenges c JOIN solved_challs sc ON sc.id = c.id"#,
        pub_id
    ).fetch_all(db).await?;

    return Ok((solves.iter().map(|x| x.points).sum(), solves));
}

// TODO we do want to put placement here
// but thats also gonna be some work (because it seems like we would probably want to cache user points)
#[derive(Serialize)]
struct PublicProfile {
    name: String,
    score: i32,
    solves: Vec<Solve>,
}

async fn profile(
    Extension(db): Extension<DB>,
    Auth(_): Auth,
    Path(pub_id): Path<String>,
) -> sctf::Result<Json<PublicProfile>> {
    let name: String = sqlx::query!("SELECT name FROM teams WHERE public_id = $1", pub_id)
        .fetch_one(&db)
        .await?
        .name;
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
        .route("/{id}", get(profile))
}
