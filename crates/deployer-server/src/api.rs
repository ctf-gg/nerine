use axum::{extract::State as StateE, routing::post, Json, Router};
use chrono::NaiveDateTime;
use log::debug;
use serde::{Deserialize, Serialize};
use sqlx::types::JsonValue;

use crate::{State, Result, deploy::{self, ChallengeDeployment}};

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct ChallengeDeploymentRow {
    pub id: i32,
    pub team_id: Option<i32>,
    pub challenge_id: i32,
    pub deployed: bool,
    pub data: Option<JsonValue>,
    pub created_at: NaiveDateTime,
    pub expired_at: Option<NaiveDateTime>,
}

impl TryInto<ChallengeDeployment> for ChallengeDeploymentRow {
    type Error = crate::error::Error;

    fn try_into(self) -> std::result::Result<ChallengeDeployment, Self::Error> {
        let data2 = self.data.map(serde_json::from_value).transpose()?;
        Ok(ChallengeDeployment {
            id: self.id,
            team_id: self.team_id,
            challenge_id: self.challenge_id,
            deployed: self.deployed,
            data: data2,
            created_at: self.created_at,
            expired_at: self.expired_at,
        })
    }
}

#[derive(Deserialize)]
struct ChallengeDeploymentReq {
    challenge_id: i32,
    team_id: Option<i32>,
}

async fn deploy_challenge(
    StateE(state): StateE<State>,
    Json(payload): Json<ChallengeDeploymentReq>,
) -> Result<()> {
    let deployment = sqlx::query_as!(
        ChallengeDeploymentRow,
        "INSERT INTO challenge_deployments (team_id, challenge_id) VALUES ($1, $2) RETURNING *",
        payload.team_id,
        payload.challenge_id,
    )
        .fetch_one(&state.db)
        .await?
        .try_into()?;

    debug!("got back deployment {:?}", deployment);

    // start deploying the chall
    tokio::spawn(deploy::deploy_challenge_task(state, deployment));

    //todo
    Ok(())
}

async fn destroy_challenge(
    StateE(state): StateE<State>,
    Json(payload): Json<ChallengeDeploymentReq>,
) -> Result<()> {
    let deployment = match sqlx::query_as!(
        ChallengeDeploymentRow,
        "SELECT * FROM challenge_deployments WHERE team_id IS NOT DISTINCT FROM $1 AND challenge_id = $2",
        payload.team_id,
        payload.challenge_id,
    )
        .fetch_optional(&state.db)
        .await? {
        None => return Ok(()),
        Some(d) => d,
    };

    let deployment = deployment.try_into()?;
    tokio::spawn(deploy::destroy_challenge_task(state, deployment));

    Ok(())
}

pub fn router() -> Router<crate::State> {
    Router::new()
        .route("/deploy_challenge", post(deploy_challenge))
        .route("/destroy_challenge", post(destroy_challenge))
}
