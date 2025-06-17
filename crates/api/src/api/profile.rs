use crate::{extractors::Auth, jwt::Claims, Result, State, DB};
use axum::{
    extract::{Path, State as StateE},
    routing::{get, post},
    Json, Router,
};
use chrono::{NaiveDateTime, Utc};
use serde::Serialize;
use validator::Validate;

use super::auth::{Team, TeamInfo, VerificationRequest};

async fn update(
    StateE(state): StateE<State>,
    Auth(Claims { team_id, .. }): Auth,
    Json(payload): Json<TeamInfo>,
) -> Result<Json<serde_json::Value>> {
    payload.validate()?;

    let current_team = sqlx::query!(
        "SELECT email, name FROM teams WHERE public_id = $1",
        team_id
    )
    .fetch_one(&state.db)
    .await?;

    if current_team.email != payload.email {
        if current_team.name != payload.name {
            sqlx::query!(
                "UPDATE teams SET name = $1 WHERE public_id = $2",
                payload.name,
                team_id
            )
            .execute(&state.db)
            .await?;
        }

        state
            .email
            .send_email_change_verification_email(&team_id, &payload.name, &payload.email)
            .await?;

        Ok(Json(serde_json::json!({
            "message": "Verification email sent. Please check your inbox to confirm the new email address.",
            "name": payload.name
        })))
    } else {
        if current_team.name != payload.name {
            let team = sqlx::query_as!(
                Team,
                "UPDATE teams SET name = $1 WHERE public_id = $2 RETURNING *",
                payload.name,
                team_id
            )
            .fetch_one(&state.db)
            .await?;
            Ok(Json(serde_json::json!(team)))
        } else {
            let team = sqlx::query_as!(Team, "SELECT * FROM teams WHERE public_id = $1", team_id)
                .fetch_one(&state.db)
                .await?;
            Ok(Json(serde_json::json!(team)))
        }
    }
}

async fn verify_email_update(
    StateE(state): StateE<State>,
    Json(VerificationRequest { token }): Json<VerificationRequest>,
) -> Result<Json<Team>> {
    let pending_update = state.email.consume_pending_email_update(&token).await?;

    let team = sqlx::query_as!(
        Team,
        "UPDATE teams SET email = $1 WHERE public_id = $2 RETURNING *",
        pending_update.new_email,
        pending_update.team_id
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
        SELECT public_id, name, c_points AS points, created_at AS solved_at FROM challenges c JOIN solved_challs sc ON sc.id = c.id 
        ORDER BY solved_at DESC"#,
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

    let (rank, score) = if Utc::now().naive_utc() < state.event.start_time {
        (-1, -1)
    } else {
        (details.rank.unwrap_or(-1), details.score.unwrap_or(-1))
    };

    return if team_id == pub_id {
        Ok(Json(Profile::Private {
            name: details.name,
            email: details.email,
            rank,
            score,
            solves,
        }))
    } else {
        Ok(Json(Profile::Public {
            name: details.name,
            rank,
            score,
            solves,
        }))
    };
}

pub fn router() -> Router<crate::State> {
    Router::new()
        .route("/update", post(update))
        .route("/verify_email_update", post(verify_email_update))
        .route("/{id}", get(profile))
}
