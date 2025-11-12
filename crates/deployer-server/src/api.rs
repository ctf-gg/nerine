use std::collections::HashMap;

use axum::{
    Json, Router,
    extract::{Path, State as StateE},
    routing::{get, post},
};
use chrono::NaiveDateTime;
use deployer_common::challenge::Challenge;
use log::debug;
use nanoid::nanoid;
use serde::{Deserialize, Serialize};
use sqlx::types::JsonValue;

use crate::{
    Result, State,
    config::write_challenges_to_dir,
    deploy::{self, ChallengeDeployment},
};

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct ChallengeDeploymentRow {
    // meant to be try_into'd into a proper ChallengeDeployment
    // so don't care about serialize anyways
    pub id: i32,
    pub public_id: String,
    pub team_id: Option<i32>,
    pub challenge_id: i32,
    pub deployed: bool,
    pub data: Option<JsonValue>,
    pub created_at: NaiveDateTime,
    pub expired_at: Option<NaiveDateTime>,
    pub destroyed_at: Option<NaiveDateTime>,
}

impl TryInto<ChallengeDeployment> for ChallengeDeploymentRow {
    type Error = crate::error::Error;

    fn try_into(self) -> std::result::Result<ChallengeDeployment, Self::Error> {
        let data2 = self.data.map(serde_json::from_value).transpose()?;
        Ok(ChallengeDeployment {
            id: self.id,
            public_id: self.public_id,
            team_id: self.team_id,
            challenge_id: self.challenge_id,
            deployed: self.deployed,
            data: data2,
            created_at: self.created_at,
            expired_at: self.expired_at,
            destroyed_at: self.destroyed_at,
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
) -> Result<Json<ChallengeDeployment>> {
    let mut tx = state.db.begin().await?;

    if sqlx::query!(
        "SELECT id FROM challenge_deployments WHERE team_id IS NOT DISTINCT FROM $1 and challenge_id = $2 AND destroyed_at IS NULL",
        payload.team_id,
        payload.challenge_id,
    )
        .fetch_optional(&mut *tx)
        .await?
        .is_some() {
        return Err(crate::error::Error::AlreadyDeployed);
    }

    let deployment: ChallengeDeployment = sqlx::query_as!(
        ChallengeDeploymentRow,
        "INSERT INTO challenge_deployments (public_id, team_id, challenge_id) VALUES ($1, $2, $3) RETURNING *",
        nanoid!(),
        payload.team_id,
        payload.challenge_id,
    )
        .fetch_one(&mut *tx)
        .await?
        .try_into()?;

    tx.commit().await?;

    debug!("got back deployment {:?}", deployment);

    // start deploying the chall
    state.tasks.spawn(deploy::deploy_challenge_task(
        state.clone(),
        deployment.clone(),
    ));

    Ok(Json(deployment.sanitize()))
}

// NOTE(ani): the reason this doesn't take public_id is because we should only actually have
// zero/one non-destroyed challenge deployment, and thus this ought to be unique. it also saves us
// queries on the api side since we'd already have both of these ids.
async fn destroy_challenge(
    StateE(state): StateE<State>,
    Json(payload): Json<ChallengeDeploymentReq>,
) -> Result<()> {
    let deployment = match sqlx::query_as!(
        ChallengeDeploymentRow,
        "SELECT * FROM challenge_deployments WHERE team_id IS NOT DISTINCT FROM $1 AND challenge_id = $2 AND destroyed_at IS NULL",
        payload.team_id,
        payload.challenge_id,
    )
        .fetch_optional(&state.db)
        .await? {
        None => return Ok(()),
        Some(d) => d,
    };

    let deployment = deployment.try_into()?;
    state
        .tasks
        .spawn(deploy::destroy_challenge_task(state.clone(), deployment));

    Ok(())
}

async fn get_challenge(
    StateE(state): StateE<State>,
    Path(pub_id): Path<String>,
) -> Result<Json<ChallengeDeployment>> {
    let deployment: ChallengeDeployment = sqlx::query_as!(
        ChallengeDeploymentRow,
        "SELECT * FROM challenge_deployments WHERE public_id = $1",
        pub_id,
    )
    .fetch_one(&state.db)
    .await?
    .try_into()?;

    Ok(Json(deployment.sanitize()))
}

async fn reload_challenges(StateE(state): StateE<State>) -> Result<()> {
    debug!("Reloading challenges");
    let mut challs_new = crate::config::load_challenges_from_dir(&state.config.challenges_dir)?;

    let mut wg = state.challenge_data.write().await;
    std::mem::swap(&mut challs_new, &mut *wg);

    debug!("Reloaded challenges");

    Ok(())
}

async fn load_challenges(
    StateE(state): StateE<State>,
    Json(challs): Json<HashMap<String, Challenge>>,
) -> Result<()> {
    debug!("Loading challenges from api endpoint");
    let mut wg = state.challenge_data.write().await;
    std::mem::swap(&mut (challs.clone()), &mut *wg);
    write_challenges_to_dir(&state.config.challenges_dir, challs)?;

    debug!("Loaded challenges from api endpoint");
    Ok(())
}

pub fn router() -> Router<crate::State> {
    Router::new()
        .route("/challenges/reload", post(reload_challenges))
        .route("/challenges/load", post(load_challenges))
        .route("/challenge/deploy", post(deploy_challenge))
        .route("/challenge/destroy", post(destroy_challenge))
        .route("/deployment/{id}", get(get_challenge))
}
