use std::{collections::HashMap, str::FromStr};

use crate::{
    db::{update_chall_cache, DeploymentStrategy},
    extractors::Admin,
    Result, State,
};
use axum::{
    extract::State as StateE,
    routing::{delete, get, patch, post},
    Json, Router,
};
use chrono::NaiveDateTime;
use deployer_common::challenge::Challenge as DeployerChallenge;
use eyre::eyre;
use nanoid::nanoid;
use serde::{Deserialize, Serialize};
use sqlx::{postgres::PgRow, FromRow, Row};

impl FromStr for DeploymentStrategy {
    type Err = eyre::Error;
    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        match s {
            "static" => Ok(DeploymentStrategy::Static),
            "instanced" => Ok(DeploymentStrategy::Instanced),
            _ => Err(eyre!("{s} is not a valid deployment strategy")),
        }
    }
}

#[derive(Deserialize, Serialize)]
pub struct Challenge {
    pub id: i32,
    pub public_id: String,
    pub name: String,
    pub author: String,
    pub description: String,
    pub points_min: i32,
    pub points_max: i32,
    pub flag: String,
    pub attachments: serde_json::Value,
    pub strategy: DeploymentStrategy,
    pub visible: bool,

    pub category: Category,
    pub group: Option<ChallengeGroup>,
}

impl FromRow<'_, PgRow> for Challenge {
    fn from_row(row: &PgRow) -> sqlx::Result<Self> {
        Ok(Self {
            id: row.try_get("id")?,
            public_id: row.try_get("public_id")?,
            name: row.try_get("name")?,
            author: row.try_get("author")?,
            description: row.try_get("description")?,
            points_min: row.try_get("points_min")?,
            points_max: row.try_get("points_max")?,
            flag: row.try_get("flag")?,
            attachments: row.try_get("attachments")?,
            strategy: DeploymentStrategy::from_str(row.try_get("strategy")?)
                .unwrap_or(DeploymentStrategy::Static),
            visible: row.try_get("visible")?,
            category: Category {
                id: row.try_get("category_id")?,
                name: row.try_get("category_name")?,
            },
            group: match row.try_get("group_id") {
                Ok(Some(gid)) => Some(ChallengeGroup {
                    id: gid,
                    name: row.try_get("group_name")?,
                }),
                Ok(None) | Err(sqlx::Error::ColumnNotFound(_)) => None,
                Err(e) => Err(e)?,
            },
        })
    }
}
#[derive(Deserialize, Serialize)]
pub struct Category {
    pub id: i32,
    pub name: String,
}

#[derive(Deserialize, Serialize)]
pub struct ChallengeGroup {
    pub id: i32,
    pub name: String,
}

async fn get_challenges(StateE(state): StateE<State>, _: Admin) -> Result<Json<Vec<Challenge>>> {
    let challs: Vec<Challenge> = sqlx::query_as(
        "WITH chall AS (SELECT * FROM challenges) SELECT 
                m.id,
                m.public_id,
                m.name,
                m.author,
                m.description,
                m.points_min,
                m.points_max,
                m.flag,
                m.attachments,
                m.strategy,
                m.visible,
                c.id AS category_id,
                c.name AS category_name,
                g.id AS group_id,
                g.name AS group_name
            FROM 
                chall m
                JOIN categories c ON m.category_id = c.id
                LEFT JOIN challenge_groups g ON m.group_id = g.id",
    )
    .fetch_all(&state.db)
    .await?;

    return Ok(Json(challs));
}

#[derive(Deserialize)]
pub struct UpsertChallenge {
    pub id: Option<String>,
    pub name: String,
    pub author: String,
    pub description: String,
    pub points_min: i32,
    pub points_max: i32,
    pub flag: String,
    pub attachments: serde_json::Value,
    pub strategy: DeploymentStrategy,
    pub visible: bool,

    pub category_id: i32,
    pub group_id: Option<i32>,
}

async fn upsert_challenge(
    StateE(state): StateE<State>,
    _: Admin,
    Json(payload): Json<UpsertChallenge>,
) -> Result<Json<Challenge>> {
    // sqlx query macro cannot understand the custom challenge fromRow
    let chall: Challenge = sqlx::query_as(
        "WITH merged AS (
            INSERT INTO challenges (
                public_id,
                name,
                author,
                description,
                points_min,
                points_max,
                flag,
                attachments,
                visible,
                category_id,
                group_id,
                strategy
            ) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12::deployment_strategy) 
            ON CONFLICT(public_id) DO UPDATE 
            SET 
                name = $2,
                author = $3,
                description = $4,
                points_min = $5,
                points_max = $6,
                flag = $7,
                attachments = $8,
                visible = $9,
                category_id = $10,
                group_id = $11,
                strategy = $12::deployment_strategy
                RETURNING *
            )
            SELECT 
                m.id,
                m.public_id,
                m.name,
                m.author,
                m.description,
                m.points_min,
                m.points_max,
                m.flag,
                m.attachments,
                m.strategy::text,
                m.visible,
                c.id AS category_id,
                c.name AS category_name,
                g.id AS group_id,
                g.name AS group_name
            FROM 
                merged m
                JOIN categories c ON m.category_id = c.id
                LEFT JOIN challenge_groups g ON m.group_id = g.id;",
    )
    .bind(payload.id.unwrap_or_else(|| nanoid!()))
    .bind(payload.name)
    .bind(payload.author)
    .bind(payload.description)
    .bind(payload.points_min)
    .bind(payload.points_max)
    .bind(payload.flag)
    .bind(payload.attachments)
    .bind(payload.visible)
    .bind(payload.category_id)
    .bind(payload.group_id)
    .bind(match payload.strategy {
        DeploymentStrategy::Static => "static",
        DeploymentStrategy::Instanced => "instanced",
    })
    .fetch_one(&state.db)
    .await?;

    update_chall_cache(&state.db, chall.id).await?;

    Ok(Json(chall))
}

#[derive(Deserialize)]
struct DeleteChallenge {
    id: String,
}

async fn delete_challenge(
    StateE(state): StateE<State>,
    _: Admin,
    Json(payload): Json<DeleteChallenge>,
) -> Result<()> {
    sqlx::query!("DELETE FROM challenges WHERE public_id = $1", payload.id)
        .execute(&state.db)
        .await?;

    Ok(())
}

#[derive(Deserialize)]
struct CreateCategory {
    name: String,
}

async fn create_category(
    StateE(state): StateE<State>,
    _: Admin,
    Json(payload): Json<CreateCategory>,
) -> Result<Json<Category>> {
    Ok(Json(
        sqlx::query_as!(
            Category,
            "INSERT INTO categories (name) VALUES ($1) RETURNING *",
            payload.name
        )
        .fetch_one(&state.db)
        .await?,
    ))
}

async fn list_categories(StateE(state): StateE<State>, _: Admin) -> Result<Json<Vec<Category>>> {
    Ok(Json(
        sqlx::query_as!(Category, "SELECT * FROM categories")
            .fetch_all(&state.db)
            .await?,
    ))
}

#[derive(Serialize)]
struct ChallengeDeploymentReq {
    challenge_id: i32,
    team_id: Option<i32>,
}
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
pub struct DeploymentDataS {
    #[serde(skip_serializing)]
    pub container_id: String,
    pub ports: HashMap<u16, HostMapping>,
}

pub type DeploymentData = HashMap<String, DeploymentDataS>;

#[derive(Deserialize, Serialize, Debug, Clone)]
#[serde(rename_all = "lowercase", tag = "type")]
pub enum HostMapping {
    Tcp { port: u16 },
    // subdomain name
    Http { subdomain: String, base: String },
}

async fn deploy_static(StateE(state): StateE<State>, _: Admin) -> Result<Json<serde_json::Value>> {
    let ids = sqlx::query!(r#"SELECT id FROM challenges WHERE strategy = 'static'"#)
        .fetch_all(&state.db)
        .await?;

    let client = reqwest::Client::new();

    let mut res = Vec::new();

    for id in ids {
        let deployment: serde_json::Value = client
            .post(&format!(
                "{}/api/challenge/deploy",
                state.config.deployer_base
            ))
            .json(&ChallengeDeploymentReq {
                challenge_id: id.id,
                team_id: None,
            })
            .send()
            .await?
            .json()
            .await?;

        println!("deployed: {}", deployment);
        res.push(deployment);
    }
    Ok(Json(serde_json::Value::Array(res)))
}

async fn destroy_static(StateE(state): StateE<State>, _: Admin) -> Result<()> {
    let ids = sqlx::query!(r#"SELECT id FROM challenges WHERE strategy = 'static'"#)
        .fetch_all(&state.db)
        .await?;

    let client = reqwest::Client::new();

    for id in ids {
        client
            .post(&format!(
                "{}/api/challenge/destroy",
                state.config.deployer_base
            ))
            .json(&ChallengeDeploymentReq {
                challenge_id: id.id,
                team_id: None,
            })
            .send()
            .await?;
    }
    Ok(())
}

async fn reload_deployer(StateE(state): StateE<State>, _: Admin) -> Result<()> {
    let client = reqwest::Client::new();

    client
        .post(&format!(
            "{}/api/challenges/reload",
            state.config.deployer_base
        ))
        .send()
        .await?;

    Ok(())
}

async fn load_deployer(
    StateE(state): StateE<State>,
    _: Admin,
    Json(challs): Json<HashMap<String, DeployerChallenge>>,
) -> Result<()> {
    let client = reqwest::Client::new();

    client
        .post(&format!(
            "{}/api/challenges/load",
            state.config.deployer_base
        ))
        .json(&challs)
        .send()
        .await?;

    Ok(())
}

async fn reap(StateE(state): StateE<State>, _: Admin) -> Result<Json<String>> {
    let containers = sqlx::query!("SELECT challenge_id, team_id FROM challenge_deployments WHERE NOW() > expired_at AND destroyed_at IS NULL").fetch_all(&state.db).await?;
    let client = reqwest::Client::new();
    for container in containers {
        client
            .post(format!(
                "{}/api/challenge/destroy",
                state.config.deployer_base
            ))
            .json(&ChallengeDeploymentReq {
                challenge_id: container.challenge_id,
                team_id: container.team_id,
            })
            .send()
            .await?
            .error_for_status()?;
    }

    Ok(Json("ok".to_string()))
}

#[derive(Deserialize)]
struct UpdateCachePayload {
    id: Option<i32>,
}

async fn update_cache_handler(
    StateE(state): StateE<State>,
    _: Admin,
    Json(payload): Json<UpdateCachePayload>,
) -> Result<Json<String>> {
    if let Some(chall_id) = payload.id {
        update_chall_cache(&state.db, chall_id).await?;
    } else {
        let all_chall_ids: Vec<(i32,)> = sqlx::query_as("SELECT id FROM challenges")
            .fetch_all(&state.db)
            .await?;
        for (chall_id,) in all_chall_ids {
            update_chall_cache(&state.db, chall_id).await?;
        }
    }
    Ok(Json("Cache updated".to_string()))
}

pub fn router() -> Router<crate::State> {
    Router::new()
        .route("/", get(get_challenges))
        .route("/", delete(delete_challenge))
        .route("/", patch(upsert_challenge))
        .route("/category", get(list_categories))
        .route("/category", post(create_category))
        .route("/deploy_static", post(deploy_static))
        .route("/destroy_static", post(destroy_static))
        .route("/reload_deployer", post(reload_deployer))
        .route("/load_deployer", post(load_deployer))
        .route("/reap", delete(reap))
        .route("/update_cache", post(update_cache_handler))
}
