use axum::Router;
use envconfig::Envconfig;
use eyre::Context;
use sqlx::postgres::PgPoolOptions;

mod api;
mod config;
mod deploy;
mod error;

use config::State;
use error::Result;

#[tokio::main]
async fn main() -> eyre::Result<()> {
    pretty_env_logger::init();
    dotenvy::dotenv().ok();

    let cfg = config::Config::init_from_env()
        .context("initialize config from environment")?;

    let challs = config::load_challenges_from_dir(&cfg.challenges_dir)?;

    let pool = PgPoolOptions::new()
        .max_connections(1)
        .connect(&cfg.database_url)
        .await?;

    sqlx::migrate!("../../migrations").run(&pool).await?;

    let app = Router::<State>::new()
        .nest("/api", api::router())
        .with_state(State::new(config::StateInner {
            config: cfg,
            db: pool,
            challenge_data: challs.into(),
        }));

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3001").await?;
    axum::serve(listener, app).await?;

    Ok(())
}
