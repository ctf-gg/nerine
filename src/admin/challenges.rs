use axum::{Extension, Json};
use nanoid::nanoid;
use serde::{Deserialize, Serialize};
use sqlx::{postgres::PgRow, FromRow, Row};

use crate::DB;

use super::Admin;

#[derive(Deserialize, Serialize)]
pub struct Challenge {
    id: i32,
    public_id: String,
    name: String,
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
pub struct Category {
    id: i32,
    name: String,
}

#[derive(Deserialize, Serialize)]
pub struct ChallengeGroup {
    id: i32,
    name: String,
}

pub struct UpsertChallenge {
    id: Option<String>,
    name: String,
    description: String,
    points_min: i32,
    points_max: i32,
    flag: String,
    attachments: serde_json::Value,
    visible: bool,

    category_id: i32,
}

pub async fn upsert_challenge(
    Extension(db): Extension<DB>,
    _: Admin,
    Json(payload): Json<UpsertChallenge>,
) -> sctf::Result<Json<Challenge>> {
    // sqlx query macro bug rejects this query
    let chall: Challenge = sqlx::query_as(
        "WITH merged AS (
            INSERT INTO challenges (
            public_id,
            name,
            description,
            points_min,
            points_max,
            flag,
            attachments,
            visible,
            category_id
        ) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9) 
        ON CONFLICT(public_id) DO UPDATE 
        SET 
            name = $2,
            description = $3,
            points_min = $4,
            points_max = $5,
            flag = $6,
            attachments = $7,
            visible = $8,
            category_id = $9
            RETURNING *
        )
        SELECT 
            m.id,
            m.public_id,
            m.name,
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
    .bind(payload.description)
    .bind(payload.points_min)
    .bind(payload.points_max)
    .bind(payload.flag)
    .bind(payload.attachments)
    .bind(payload.visible)
    .bind(payload.category_id)
    .fetch_one(&db)
    .await?;

    Ok(Json(chall))
}
