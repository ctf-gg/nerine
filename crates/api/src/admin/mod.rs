use axum::Router;
mod auth;
mod challenges;
mod export;

pub fn router() -> Router<crate::State> {
    Router::new()
        .nest("/challs", challenges::router())
        .nest("/auth", auth::router())
        .nest("/export", export::router())
}
