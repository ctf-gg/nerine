use chrono::Utc;
use serde::Serialize;
use serde_json::json;

use crate::db::DB;

pub async fn award_badge(db: &DB, chall_id: i32, team_id: String) -> crate::Result<()> {
    sqlx::query!(r#"UPDATE teams SET extra_data = '{ "badges": [] }' WHERE extra_data IS NULL"#)
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

    let client = reqwest::Client::new();

    #[derive(Serialize)]
    struct WebhookData {
        content: String,
        embeds: Option<()>,
        attachments: Vec<()>,
    }
    let msg = format!(
        "Congrats to `{}` for first blooding `{}`!",
        sqlx::query!("SELECT name FROM teams WHERE public_id = $1", team_id)
            .fetch_one(db)
            .await?
            .name,
        sqlx::query!("SELECT public_id FROM challenges WHERE id = $1", chall_id)
            .fetch_one(db)
            .await?
            .public_id
    );
    client.post("https://discord.com/api/webhooks/1383231404976115743/e5Cv4VMmzXqU6sxeq5LClzze3MJd0ilolR9Bc9hvS_1hcO4pXAhTnzfB5VwO9CmfeVoW").json(&WebhookData {
        content: msg,
        embeds: None,
        attachments: Vec::new(),
    }).send().await?;

    Ok(())
}
