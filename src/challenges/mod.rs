use axum::{routing::get, Router};

mod routes;

pub fn router() -> Router {
    Router::new()
        .route("/all", get(routes::all))
}