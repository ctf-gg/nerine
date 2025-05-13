use crate::event::point_formula;
use serde::Serialize;
use sqlx::{Pool, Postgres};

pub type DB = Pool<Postgres>;

pub async fn update_chall_cache(db: &DB, chall_id: i32) -> crate::Result<()> {
    #[derive(Serialize)]
    struct ChallDetails {
        points_min: i32,
        points_max: i32,
        solves: i32,
    }

    let chall_details = sqlx::query_as!(
        ChallDetails,
        r#"WITH solves as (SELECT count(*)::int AS solves FROM submissions WHERE is_correct = true AND challenge_id = $1)
        SELECT points_min, points_max, solves AS "solves!" FROM challenges c, solves WHERE id = $1"#,
        chall_id
    ).fetch_one(db).await?;

    let points = point_formula(
        chall_details.points_min,
        chall_details.points_max,
        chall_details.solves,
    );

    sqlx::query!(
        "UPDATE challenges SET c_solves = $1, c_points = $2 WHERE id = $3",
        chall_details.solves,
        points,
        chall_id,
    )
    .execute(db)
    .await?;

    Ok(())
}
