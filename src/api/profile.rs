use axum::{
    extract::Path,
    routing::{get, post},
    Extension, Json, Router,
};
use sctf::{extractors::Auth, jwt::Claims, DB};
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
        r#"WITH 
            team AS (SELECT id FROM teams WHERE public_id = $1),
            solved_challs AS (SELECT challenge_id AS id FROM submissions, team WHERE is_correct = true AND team_id = team.id)
        SELECT name, c_points AS points FROM challenges c JOIN solved_challs sc ON sc.id = c.id"#,
        pub_id
    ).fetch_all(db).await?;

    return Ok((solves.iter().map(|x| x.points).sum(), solves));
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
    Extension(db): Extension<DB>,
    Auth(Claims { team_id, .. }): Auth,
    Path(pub_id): Path<String>,
) -> sctf::Result<Json<Profile>> {
    struct TeamDetails {
        name: String,
        email: String,
        rank: Option<i32>,
    }

    let details = sqlx::query_as!(
        TeamDetails,
        r#"
        WITH
        team AS (SELECT id FROM teams WHERE public_id = $1),
        solves AS (SELECT team_id, challenge_id FROM submissions WHERE is_correct = true),
        last_solve AS (SELECT team_id, MAX(created_at) AS sub_time FROM submissions WHERE is_correct = true GROUP BY team_id),
        rank AS (SELECT 
                t.id, 
                ROW_NUMBER() OVER (
                    ORDER BY 
                        COALESCE(SUM(c_points), 0) DESC,
                        ls.sub_time ASC NULLS LAST,
                        t.id ASC
                )::int AS rank
            FROM teams t
            LEFT JOIN solves ON t.id = solves.team_id
            LEFT JOIN challenges ch ON solves.challenge_id = ch.id
            LEFT JOIN last_solve ls ON t.id = ls.team_id
            GROUP BY t.id, ls.sub_time)
        SELECT name, email, rank FROM teams JOIN rank ON rank.id = teams.id JOIN team ON teams.id = team.id"#,
        pub_id
    )
    .fetch_one(&db)
    .await?;
    let (score, solves) = get_score_solves(&db, &pub_id).await?;

    return if team_id == pub_id {
        Ok(Json(Profile::Private {
            name: details.name,
            email: details.email,
            rank: details.rank.unwrap_or(-1),
            score,
            solves,
        }))
    } else {
        Ok(Json(Profile::Public {
            name: details.name,
            rank: details.rank.unwrap_or(-1),
            score,
            solves,
        }))
    };
}

pub fn router() -> Router {
    Router::new()
        .route("/update", post(update))
        .route("/{id}", get(profile))
}
