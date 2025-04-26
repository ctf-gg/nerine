use axum::{routing::{get, post}, Router};

mod routes;

pub fn router() -> Router {
    Router::new()
        .route("/all", get(routes::all))
        .route("/submit", post(routes::submit))
}