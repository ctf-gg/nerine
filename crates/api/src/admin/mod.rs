use axum::Router;
mod challenges;
mod auth;

pub fn router() -> Router<crate::State> {
    Router::new()
        .nest("/challs", challenges::router())
        .nest("/auth", auth::router())
}
