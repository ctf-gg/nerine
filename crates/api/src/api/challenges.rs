use std::{collections::HashMap, sync::Arc, time::Duration};

use crate::{badges::award_badge, db::update_chall_cache, extractors::Auth, Error, Result, State};
use axum::{
    extract::{Path, State as StateE},
    routing::{get, post},
    Json, Router,
};
use bitstream_io::{BitRead, BitReader, LittleEndian};
use chrono::{NaiveDateTime, Utc};
use serde::{Deserialize, Serialize};
use tower_governor::{
    governor::GovernorConfigBuilder, key_extractor::SmartIpKeyExtractor, GovernorLayer,
};

#[derive(Deserialize, Serialize)]
pub struct PublicChallenge {
    #[serde(rename(serialize = "id"))]
    public_id: String,
    name: String,
    author: String,
    description: String,
    points: i32,
    solves: i32,
    attachments: serde_json::Value,
    category: String,
    #[serde(rename(serialize = "deploymentId"))]
    deployment_id: String,
    strategy: String,
    #[serde(rename(serialize = "selfSolved"))] // todo use built in camel case in future
    self_solved: bool,
}

// NOTE: All of the routes in this file are PUBLICALLY
// ACCESSABLE!! Do not leak any important information.
pub async fn list(
    StateE(state): StateE<State>,
    Auth(claims): Auth,
) -> Result<Json<Vec<PublicChallenge>>> {
    if Utc::now().naive_utc() < state.event.start_time {
        return Err(Error::EventNotStarted(state.event.start_time.clone()));
    }

    let solves = super::profile::get_solves(&state.db, &claims.team_id).await?;

    // TODO cd.public_id is an unexpected null apparently, this is a sqlx bug
    let mut challs = sqlx::query_as!(
        PublicChallenge,
        r#"SELECT 
            c.public_id,
            c.name,
            author,
            description,
            c_points AS points,
            c_solves AS solves,
            attachments,
            strategy::text AS "strategy!",
            COALESCE(cd.public_id, '') AS "deployment_id!",
            categories.name AS category,
            FALSE as "self_solved!"
        FROM challenges c JOIN categories ON categories.id = category_id
        LEFT JOIN challenge_deployments cd ON destroyed_at IS NULL AND challenge_id = c.id AND (team_id IS NULL or team_id = (SELECT id FROM teams WHERE public_id = $1))
        WHERE visible = true
        ORDER BY solves DESC"#,
        claims.team_id,
    )
    .fetch_all(&state.db)
    .await?;

    for c in &mut challs {
        if solves.iter().any(|s| s.public_id == c.public_id) {
            c.self_solved = true;
        }
    }

    Ok(Json(challs))
}

#[derive(Deserialize)]
pub struct Submission {
    flag: String,
    challenge_id: String,
}

fn leet<R>(flag: String, bits: BitReader<R, LittleEndian>) -> String {
    "".to_string()
}

pub async fn submit(
    StateE(state): StateE<State>,
    Auth(claims): Auth,
    Json(submission): Json<Submission>,
) -> Result<()> {
    let now = Utc::now().naive_utc();
    if now < state.event.start_time {
        return Err(Error::EventNotStarted(state.event.start_time.clone()));
    }
    if now > state.event.end_time {
        return Err(Error::EventEnded);
    }

    struct AnswerInfo {
        id: i32,
        flag: String,
        solves: i32,
    }

    let answer_info: AnswerInfo = sqlx::query_as!(
        AnswerInfo,
        "SELECT id, flag, c_solves AS solves FROM challenges WHERE public_id = $1",
        submission.challenge_id
    )
    .fetch_one(&state.db)
    .await?;

    let is_correct = answer_info.flag == submission.flag;

    sqlx::query!(
        r#"INSERT INTO submissions (submission, is_correct, team_id, challenge_id)
        VALUES ($1, $2, (SELECT id FROM teams WHERE public_id = $3), $4)"#,
        submission.flag,
        is_correct,
        claims.team_id,
        answer_info.id,
    )
    .execute(&state.db)
    .await?;

    if is_correct {
        update_chall_cache(&state.db, answer_info.id).await?;
        if answer_info.solves == 0 {
            award_badge(&state.db, answer_info.id, claims.team_id).await?;
        }
        Ok(())
    } else {
        Err(Error::WrongFlag)
    }
}

#[derive(Serialize)]
struct ChallengeDeploymentReq {
    challenge_id: i32,
    team_id: Option<i32>,
}
// keep in sync with deployer-server/api
#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct ChallengeDeployment {
    pub id: String,
    pub deployed: bool,
    pub data: Option<DeploymentData>,
    pub created_at: NaiveDateTime,
    pub expired_at: Option<NaiveDateTime>,
    pub destroyed_at: Option<NaiveDateTime>,
}

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct DeploymentData {
    #[serde(skip_serializing)]
    pub container_id: String,
    pub ports: HashMap<u16, HostMapping>,
}

#[derive(Deserialize, Serialize, Debug, Clone)]
#[serde(rename_all = "lowercase", tag = "type")]
pub enum HostMapping {
    Tcp { port: u16 },
    // subdomain name
    Http { subdomain: String, base: String },
}

pub async fn deploy(
    StateE(state): StateE<State>,
    Auth(claims): Auth,
    Path(pub_id): Path<String>,
) -> Result<Json<ChallengeDeployment>> {
    let now = Utc::now().naive_utc();
    if now < state.event.start_time {
        return Err(Error::EventNotStarted(state.event.start_time.clone()));
    }

    let record = sqlx::query!(
        r#"SELECT teams.id AS team_id, challenges.id AS challenge_id, challenges.strategy::text AS "strategy!"
FROM teams, challenges 
WHERE teams.public_id = $1 AND challenges.public_id = $2;"#,
        claims.team_id,
        pub_id,
    )
    .fetch_one(&state.db)
    .await?;

    let client = reqwest::Client::new();

    if record.strategy == "static" {
        return Err(Error::GenericError);
    }

    // TODO unhardcode this later
    let deployment: ChallengeDeployment = client
        .post("http://deployer:3001/api/challenge/deploy")
        .json(&ChallengeDeploymentReq {
            challenge_id: record.challenge_id,
            team_id: Some(record.team_id),
        })
        .send()
        .await?
        .error_for_status()?
        .json()
        .await?;

    Ok(Json(deployment))
}

pub async fn get_deployment(
    Auth(_): Auth,
    Path(pub_id): Path<String>,
) -> Result<Json<ChallengeDeployment>> {
    let client = reqwest::Client::new();

    // TODO unhardcode this later
    let deployment: ChallengeDeployment = client
        .get(format!("http://deployer:3001/api/deployment/{pub_id}"))
        .send()
        .await?
        .error_for_status()?
        .json()
        .await?;

    Ok(Json(deployment))
}

pub fn router() -> Router<crate::State> {
    let governor_conf = Arc::new(
        GovernorConfigBuilder::default()
            .per_second(4)
            .burst_size(5)
            .key_extractor(SmartIpKeyExtractor)
            .finish()
            .unwrap(),
    );

    let governor_limiter = governor_conf.limiter().clone();
    let interval = Duration::from_secs(60);
    // a separate background task to clean up
    std::thread::spawn(move || loop {
        std::thread::sleep(interval);
        tracing::info!("rate limiting storage size: {}", governor_limiter.len());
        governor_limiter.retain_recent();
    });
    let ratelimited = Router::new()
        .route("/submit", post(submit))
        .route("/deploy/get/{deployment_id}", get(get_deployment))
        .route("/deploy/new/{chall_id}", post(deploy))
        .layer(GovernorLayer {
            config: governor_conf,
        });

    Router::new().merge(ratelimited).route("/", get(list))
}
