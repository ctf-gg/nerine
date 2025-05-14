use std::env;

use axum::{http::HeaderValue, Extension, Router};
use eyre::Result;
use sqlx::postgres::PgPoolOptions;
use tower_http::cors::{Any, CorsLayer};

mod admin;
mod api;
mod deploy;

#[tokio::main]
async fn main() -> Result<()> {
    dotenvy::dotenv()?;

    let pool = PgPoolOptions::new()
        .max_connections(5)
        .connect(env::var("DATABASE_URL")?.as_str())
        .await?;

    sqlx::migrate!().run(&pool).await?;

    let cors = CorsLayer::new()
        .allow_methods(Any)
        .allow_origin(["http://sctf.localhost".parse::<HeaderValue>().unwrap()])
        .allow_headers(Any);
        // .allow_credentials(true);

    let app = Router::new()
        .nest("/api", api::router())
        .layer(Extension(pool))
        .layer(cors);

    // run our app with hyper, listening globally on port 3000
    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
    axum::serve(listener, app).await.unwrap();

    Ok(())
}
