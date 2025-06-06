use axum::{http::HeaderValue, Router};
use envconfig::Envconfig;
use eyre::Context;
use sqlx::postgres::PgPoolOptions;
use tower_http::cors::{Any, CorsLayer};

mod admin;
mod api;
mod config;
mod db;
mod deploy;
mod error;
mod event;
mod extractors;
mod jwt;

use config::State;
use db::DB;
use error::{Error, Result};
use event::EVENT;

#[tokio::main]
async fn main() -> eyre::Result<()> {
    pretty_env_logger::init();
    dotenvy::dotenv().ok();

    let cfg = config::Config::init_from_env()
        .context("initialize config from environment")?;

    let pool = PgPoolOptions::new()
        .max_connections(5)
        .connect(&cfg.database_url)
        .await?;

    sqlx::migrate!().run(&pool).await?;

    let cors = CorsLayer::new()
        .allow_methods(Any)
        .allow_origin(["http://sctf.localhost".parse::<HeaderValue>().unwrap()])
        .allow_headers(Any);
        // .allow_credentials(true);

    let app = Router::<State>::new()
        .nest("/api", api::router())
        .with_state(State::new(config::StateInner {
            config: cfg,
            db: pool,
        }))
        .layer(cors);

    // run our app with hyper, listening globally on port 3000
    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
    axum::serve(listener, app).await.unwrap();

    Ok(())
}
