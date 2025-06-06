use axum::Router;
mod challenges;

pub fn router() -> Router<crate::State> {
    Router::new().nest("/challs", challenges::router())
}
