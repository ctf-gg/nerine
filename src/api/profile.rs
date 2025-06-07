use axum::{
    extract::{State as StateE, Path},
    routing::{get, post},
    Json, Router,
};
use chrono::NaiveDateTime;
use crate::{extractors::Auth, jwt::Claims, DB, Result, State};
use serde::{Deserialize, Serialize};

use super::auth::Team;

#[derive(Deserialize)]
struct UpdateProfile {
    name: String,
    email: String,
}

async fn update(
    StateE(state): StateE<State>,
    Auth(Claims { team_id, .. }): Auth,
    Json(payload): Json<UpdateProfile>,
) -> Result<Json<Team>> {
    let team = sqlx::query_as!(
        Team,
        "UPDATE teams SET name = $1, email = $2 WHERE public_id = $3 RETURNING *",
        payload.name,
        payload.email,
        team_id
    )
    .fetch_one(&state.db)
    .await?;

    Ok(Json(team))
}

#[derive(Serialize)]
pub(crate) struct Solve {
    // FIXME(ani): shouldn't be here
    pub(crate) public_id: String,
    name: String,
    points: i32,
    #[serde(rename(serialize = "solvedAt"))]
    solved_at: NaiveDateTime,
}

pub(crate) async fn get_solves(db: &DB, pub_id: &str) -> Result<Vec<Solve>> {
    let solves = sqlx::query_as!(
        Solve,
        r#"WITH 
            team AS (SELECT id FROM teams WHERE public_id = $1),
            solved_challs AS (SELECT challenge_id AS id, created_at FROM submissions, team WHERE is_correct = true AND team_id = team.id)
        SELECT public_id, name, c_points AS points, created_at AS solved_at FROM challenges c JOIN solved_challs sc ON sc.id = c.id"#,
        pub_id
    ).fetch_all(db).await?;

    return Ok(solves);
}

// TODO we do want to put placement here
// but thats also gonna be some work (because it seems like we would probably want to cache user points)
#[derive(Serialize)]
#[serde(tag = "type")]
enum Profile {
    #[serde(rename = "private")]
    Private {
        name: String,
        email: String,
        score: i32,
        rank: i32,
        solves: Vec<Solve>,
    },
    #[serde(rename = "public")]
    Public {
        name: String,
        score: i32,
        rank: i32,
        solves: Vec<Solve>,
    },
}

async fn profile(
    StateE(state): StateE<State>,
    Auth(Claims { team_id, .. }): Auth,
    Path(pub_id): Path<String>,
) -> Result<Json<Profile>> {
    struct TeamDetails {
        name: String,
        email: String,
        rank: Option<i32>,
        score: Option<i32>,
    }

    let details = sqlx::query_as!(
        TeamDetails,
        r#"
        SELECT name, email, rank, score FROM teams t
            JOIN compute_leaderboard() lb ON lb.team_id = t.id 
            WHERE t.id = (SELECT id FROM teams WHERE public_id = $1)"#,
        pub_id
    )
    .fetch_one(&state.db)
    .await?;
    let solves = get_solves(&state.db, &pub_id).await?;

    return if team_id == pub_id {
        Ok(Json(Profile::Private {
            name: details.name,
            email: details.email,
            rank: details.rank.unwrap_or(-1),
            score: details.score.unwrap_or(-1),
            solves,
        }))
    } else {
        Ok(Json(Profile::Public {
            name: details.name,
            rank: details.rank.unwrap_or(-1),
            score: details.score.unwrap_or(-1),
            solves,
        }))
    };
}

pub fn router() -> Router<crate::State> {
    Router::new()
        .route("/update", post(update))
        .route("/{id}", get(profile))
}
