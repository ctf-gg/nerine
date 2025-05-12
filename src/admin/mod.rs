use axum::Router;
mod challenges;

pub fn router() -> Router {
    Router::new().nest("/challs", challenges::router())
}
