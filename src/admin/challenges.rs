use axum::{
    routing::{delete, get, patch, post},
    Extension, Json, Router,
};
use nanoid::nanoid;
use crate::{db::update_chall_cache, extractors::Admin, DB, Result};
use serde::{Deserialize, Serialize};
use sqlx::{postgres::PgRow, FromRow, Row};

#[derive(Deserialize, Serialize)]
struct Challenge {
    id: i32,
    public_id: String,
    name: String,
    author: String,
    description: String,
    points_min: i32,
    points_max: i32,
    flag: String,
    attachments: serde_json::Value,
    visible: bool,

    category: Category,
    group: Option<ChallengeGroup>,
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
            visible: row.try_get("visible")?,
            category: Category {
                id: row.try_get("category_id")?,
                name: row.try_get("category_name")?,
            },
            group: match row.try_get("group_id") {
                Ok(gid) => Some(ChallengeGroup {
                    id: gid,
                    name: row.try_get("group_name")?,
                }),
                Err(sqlx::Error::ColumnNotFound(_)) => None,
                Err(e) => Err(e)?,
            },
        })
    }
}
#[derive(Deserialize, Serialize)]
struct Category {
    id: i32,
    name: String,
}

#[derive(Deserialize, Serialize)]
struct ChallengeGroup {
    id: i32,
    name: String,
}

async fn get_challenges(
    Extension(db): Extension<DB>,
    _: Admin,
) -> Result<Json<Vec<Challenge>>> {
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
    .fetch_all(&db)
    .await?;

    return Ok(Json(challs));
}

#[derive(Deserialize)]
struct UpsertChallenge {
    id: Option<String>,
    name: String,
    author: String,
    description: String,
    points_min: i32,
    points_max: i32,
    flag: String,
    attachments: serde_json::Value,
    visible: bool,

    category_id: i32,
}

async fn upsert_challenge(
    Extension(db): Extension<DB>,
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
                category_id
            ) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10) 
            ON CONFLICT(public_id) DO UPDATE 
            SET 
                name = $2,
                author = $3
                description = $4,
                points_min = $5,
                points_max = $6,
                flag = $7,
                attachments = $8,
                visible = $9,
                category_id = $10
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
    .fetch_one(&db)
    .await?;

    update_chall_cache(&db, chall.id).await?;

    Ok(Json(chall))
}

#[derive(Deserialize)]
struct DeleteChallenge {
    id: String,
}

async fn delete_challenge(
    Extension(db): Extension<DB>,
    _: Admin,
    Json(payload): Json<DeleteChallenge>,
) -> Result<()> {
    sqlx::query!("DELETE FROM challenges WHERE public_id = $1", payload.id)
        .execute(&db)
        .await?;

    Ok(())
}

#[derive(Deserialize)]
struct CreateCategory {
    name: String,
}

async fn create_category(
    Extension(db): Extension<DB>,
    _: Admin,
    Json(payload): Json<CreateCategory>,
) -> Result<Json<Category>> {
    Ok(Json(sqlx::query_as!(
        Category,
        "INSERT INTO categories (name) VALUES ($1) RETURNING *",
        payload.name
    )
    .fetch_one(&db)
    .await?))
}

pub fn router() -> Router<crate::State> {
    Router::new()
        .route("/", get(get_challenges))
        .route("/", delete(delete_challenge))
        .route("/", patch(upsert_challenge))
        .route("/category", post(create_category))
}
