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
use log::error;
use tokio_util::{sync::CancellationToken, task::TaskTracker};

#[tokio::main]
async fn main() -> eyre::Result<()> {
    pretty_env_logger::init();
    dotenvy::dotenv().ok();

    let cfg = config::Config::init_from_env().context("initialize config from environment")?;

    let challs = config::load_challenges_from_dir(&cfg.challenges_dir)?;

    let pool = PgPoolOptions::new()
        .max_connections(5)
        .connect(&cfg.database_url)
        .await?;

    sqlx::migrate!("../../migrations").run(&pool).await?;

    let tt = TaskTracker::new();
    let ct = CancellationToken::new();
    let ct_copy = ct.clone();

    ctrlc::set_handler(move || {
        ct_copy.cancel();
    })?;

    let state = State::new(config::StateInner {
        config: cfg,
        db: pool.clone(),
        challenge_data: challs.into(),
        tasks: tt.clone(),
    });

    let inherited_containers = sqlx::query_as!(
        api::ChallengeDeploymentRow,
        "SELECT * FROM challenge_deployments WHERE destroyed_at IS NULL AND expired_at IS NOT NULL"
    )
    .fetch_all(&pool)
    .await?;
    for container in inherited_containers {
        let container_id = container.id;
        if let Ok(container) = TryInto::<deploy::ChallengeDeployment>::try_into(container) {
            let expiration_time = container.expired_at.unwrap();
            let dur = (expiration_time - chrono::Utc::now().naive_utc())
                .max(chrono::TimeDelta::zero())
                .to_std()
                .unwrap();

            let state_clone = state.clone();
            tokio::spawn(async move {
                tokio::time::sleep(dur).await;
                deploy::destroy_challenge_task(state_clone, container).await;
            });
        } else {
            error!(
                "failed to start cleanup task for deployment {}",
                container_id
            );
        }
    }

    let app = Router::<State>::new()
        .nest("/api", api::router())
        .with_state(state);

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3001").await?;
    axum::serve(listener, app)
        .with_graceful_shutdown(ct.cancelled_owned())
        .await?;

    tt.close();
    tt.wait().await;

    Ok(())
}
