use std::sync::Arc;

use axum::{http::HeaderValue, Router};
use std::time::Duration;
use envconfig::Envconfig;
use eyre::Context;
use sqlx::postgres::PgPoolOptions;
use tower_governor::{governor::GovernorConfigBuilder, GovernorLayer};
use tower_http::cors::{Any, CorsLayer};

mod admin;
mod api;
mod badges;
mod config;
mod db;
mod email;
mod error;
mod event;
mod extractors;
mod jwt;

use config::State;
use db::DB;
use error::{Error, Result};

#[tokio::main]
async fn main() -> eyre::Result<()> {
    pretty_env_logger::init();
    dotenvy::dotenv().ok();

    let cfg = config::Config::init_from_env().context("initialize config from environment")?;

    let event =
        event::Event::read_from_path(&cfg.event_path).context("read event from environment")?;

    let pool = PgPoolOptions::new()
        .max_connections(5)
        .connect(&cfg.database_url)
        .await?;

    let governor_conf = Arc::new(
        GovernorConfigBuilder::default()
            .per_second(5)
            .burst_size(12)
            .finish()
            .unwrap(),
    );

    let governor_limiter = governor_conf.limiter().clone();
    let interval = Duration::from_secs(60);
    // a separate background task to clean up
    std::thread::spawn(move || loop {
        std::thread::sleep(interval);
        tracing::info!("rate limiting storage size: {}", governor_limiter.len());
        governor_limiter.retain_recent();
    });

    sqlx::migrate!("../../migrations").run(&pool).await?;

    let cors = CorsLayer::new()
        .allow_methods(Any)
        .allow_origin([cfg.cors_origin.parse::<HeaderValue>().unwrap()])
        .allow_headers(Any);
    // .allow_credentials(true);

    let app = Router::<State>::new()
        .nest("/api", api::router())
        .with_state(State::new(config::StateInner {
            email: email::EmailService::new(&cfg),
            config: cfg,
            event,
            db: pool,
        }))
        .layer(cors);
        // .layer(GovernorLayer {
        //     config: governor_conf,
        // });

    // run our app with hyper, listening globally on port 3000
    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
    axum::serve(listener, app).await.unwrap();

    Ok(())
}
