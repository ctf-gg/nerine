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
use tokio_util::{sync::CancellationToken, task::TaskTracker};

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

    let tt = TaskTracker::new();
    let ct = CancellationToken::new();
    let ct_copy = ct.clone();

    ctrlc::set_handler(move || {
        ct_copy.cancel();
    })?;

    let app = Router::<State>::new()
        .nest("/api", api::router())
        .with_state(State::new(config::StateInner {
            config: cfg,
            db: pool,
            challenge_data: challs.into(),
            tasks: tt.clone(),
        }));

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3001").await?;
    axum::serve(listener, app)
        .with_graceful_shutdown(ct.cancelled_owned())
        .await?;

    tt.close();
    tt.wait().await;

    Ok(())
}
