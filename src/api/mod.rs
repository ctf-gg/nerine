use axum::Router;

mod auth;
mod challenges;
mod profile;


pub fn router() -> Router {
    Router::new()
        .nest("/admin", crate::admin::router())
        .nest("/auth", auth::router())
        .nest("/challs", challenges::router())
        .nest("/profile", profile::router())
}
