use std::str::FromStr;

use crate::{db::{update_chall_cache, DeploymentStrategy}, extractors::Admin, Result, State};
use axum::{
    extract::State as StateE,
    routing::{delete, get, patch, post},
    Json, Router,
};
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
            ) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12) 
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
                strategy = $12
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
                m.strategy,
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

pub fn router() -> Router<crate::State> {
    Router::new()
        .route("/", get(get_challenges))
        .route("/", delete(delete_challenge))
        .route("/", patch(upsert_challenge))
        .route("/category", get(list_categories))
        .route("/category", post(create_category))
}
