use std::env;

use axum::{Extension, Router};
use eyre::Result;
use sqlx::{postgres::PgPoolOptions, Pool, Postgres};

mod account;
mod admin;
mod challenges;
mod deploy;

pub type DB = Pool<Postgres>;

#[tokio::main]
async fn main() -> Result<()> {
    dotenvy::dotenv()?;

    let pool = PgPoolOptions::new()
        .max_connections(5)
        .connect(env::var("DATABASE_URL")?.as_str())
        .await?;

    sqlx::migrate!().run(&pool).await?;

    let app = Router::new()
        .nest(
            "/api",
            Router::new()
                .nest("/account", account::router())
                .nest("/challenges", challenges::router()),
        )
        .layer(Extension(pool));

    // run our app with hyper, listening globally on port 3000
    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
    axum::serve(listener, app).await.unwrap();

    Ok(())
}
