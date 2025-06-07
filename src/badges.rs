use chrono::Utc;
use serde_json::json;

use crate::db::DB;

pub async fn award_badge(db: &DB, chall_id: i32, team_id: String) -> crate::Result<()> {
    sqlx::query!(r#"UPDATE teams SET extra_data = '{ "badges": [] }' WHERE extra_data = NULL"#)
        .execute(db)
        .await?;

    let record = sqlx::query!(
        "WITH chall AS (SELECT name, category_id FROM challenges WHERE id = $1) 
        SELECT categories.name AS category_name, chall.name AS chall_name FROM categories, chall WHERE id = chall.category_id",
        chall_id
    )
    .fetch_one(db)
    .await?;

    let badge_json = json! ([{
        "type": record.category_name,
        "obtained": Utc::now(),
        "chall": record.chall_name
    }]);

    sqlx::query!(
        r#"UPDATE teams 
        SET extra_data = 
            jsonb_set(
                extra_data, 
                array['badges'],
                extra_data->'badges' || $1
            ) 
        WHERE public_id = $2"#,
        badge_json,
        team_id
    )
    .execute(db)
    .await?;

    Ok(())
}
